"""
SQL Object Diff - fereastra nativa (pywebview), comparatie rapida + lazy diff.
v2.2: fara UAC/consola la pornire, fetch paralel, motor mai rapid,
      deployer corect (respecta editari manuale + excluderi), obiecte extra, export CSV.
"""
DEBUG_LIMIT = None
DEBUG_FILTER = None
#DEBUG_LIMIT = 10
#DEBUG_FILTER = 'TESTSIMILITUDINE'


import os, sys, re, csv, json, time, ctypes, tempfile, threading, difflib, socket

# ── FIX ARTEFACTE/GLITCH PE SERVER (RDP / masini fara GPU real) ───────────────
# WebView2 (motorul Chromium din pywebview) foloseste implicit accelerarea GPU.
# Pe un server accesat prin RDP nu exista GPU hardware, iar compozitorul GPU
# produce artefacte: benzi, zone care nu se redeseneaza, "glitch" de imagine.
# Fortam randarea software + dezactivam GPU-ul => randare corecta peste tot.
# (Local, cu placa video reala, diferenta de performanta e neglijabila.)
os.environ.setdefault(
    "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
    "--disable-gpu --disable-gpu-compositing --disable-software-rasterizer "
    "--disable-features=UseSkiaRenderer --disable-direct-composition "
    "--disable-gpu-rasterization --disable-accelerated-2d-canvas"
)
os.environ.setdefault("QTWEBENGINE_CHROMIUM_FLAGS", "--disable-gpu")  # fallback motor Qt

from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
import pyodbc, webview
import motor, ui

_RE_IDENT = re.compile(r'^[A-Za-z_][A-Za-z0-9_$#@. ]{0,127}$')
def nume_valid(n): return isinstance(n,str) and bool(_RE_IDENT.match(n))

def gaseste_driver_sql():
    """Cauta driverele ODBC SQL Server instalate si il alege pe cel mai nou.
    Preferinta: ODBC Driver 18 > 17 > ... > SQL Server Native Client > 'SQL Server' generic."""
    drv=list(pyodbc.drivers())
    candidati=[d for d in drv if 'ODBC Driver' in d and 'SQL Server' in d]
    def versiune(d):
        m=re.search(r'ODBC Driver (\d+)', d)
        return int(m.group(1)) if m else 0
    if candidati:
        return max(candidati, key=versiune), drv
    for d in drv:
        if 'SQL Server Native Client' in d:
            return d, drv
    if 'SQL Server' in drv:
        return 'SQL Server', drv
    return None, drv

def build_cs(server, database, auth, user=None, parola=None, port=None):
    adresa=f"{server},{port}" if port else server
    driver,_=gaseste_driver_sql()
    if not driver:
        raise RuntimeError("Niciun driver ODBC SQL Server nu a fost gasit pe acest calculator.")
    base=(f"DRIVER={{{driver}}};SERVER={adresa};"
          f"DATABASE={database};Encrypt=yes;TrustServerCertificate=yes;")
    return base+("Trusted_Connection=yes;" if auth in ('win','windows') else f"UID={user};PWD={parola};")

def verifica_driver():
    driver,drv=gaseste_driver_sql()
    return driver is not None, drv

# ── MOTOR DE RANDARE PROPRIU, INCLUS IN APLICATIE (Qt WebEngine / Chromium) ───
# Problema reala de pe servere vechi (Server 2012 si altele): pywebview
# foloseste implicit WebView2 (Chromium din Windows) - daca acel runtime
# lipseste de pe masina, Windows cade pe motorul vechi Internet Explorer
# (mshtml), care nu intelege CSS/JS modern (variabile CSS, printre altele) -
# de-aia interfata iese alb-negru sau stricata, indiferent cate patch-uri
# punem pe CSS.
#
# Solutia definitiva: aplicatia isi aduce PROPRIUL motor Chromium (Qt
# WebEngine, prin pachetul PyQtWebEngine), inclus direct in exe-ul compilat cu
# PyInstaller. Astfel nu mai conteaza deloc ce are serverul instalat - exact
# acelasi motor modern ruleaza peste tot, fara nicio instalare separata pe
# masina tinta.
#
# Cerinte O SINGURA DATA, la tine, inainte de compilare (nu pe server):
#   pip install PyQt5 PyQtWebEngine
# apoi compilezi cu PyInstaller ca de obicei. Recomandat --onedir (folder), nu
# --onefile: QtWebEngine porneste un proces auxiliar
# (QtWebEngineProcess.exe) care trebuie sa gaseasca niste fisiere langa exe;
# in modul --onefile (care se dez-arhiveaza intr-un folder temporar la fiecare
# pornire) acest lucru poate deveni nesigur. --onedir e mai robust si pentru
# un exe care oricum va fi de zeci-sute de MB din cauza Chromium-ului inclus.
def gui_disponibil():
    """Motorul Qt (Chromium propriu, bundled) e implicit si obligatoriu -
    elimina definitiv dependenta de WebView2/IE11 de pe masina tinta. Se poate
    suprascrie cu variabila de mediu GMOTION_GUI DOAR pentru dezvoltare locala
    (ex: testezi pe calculatorul tau, fara PyQtWebEngine instalat, si vrei
    temporar 'edgechromium') - pe orice build destinat unui server, foloseste
    'qt'."""
    return os.environ.get('GMOTION_GUI', 'qt')

# ── plasa de siguranta separata: bara de titlu CUSTOM vs fereastra nativa ─────
# Independent de motorul de randare (rezolvat mai sus cu Qt), bara de titlu
# custom (fara chenar Windows, cu colturi rotunjite si transparenta) e un
# efect de fereastra la nivel de Windows, care in general se compune corect
# doar cand DWM (Desktop Window Manager) ruleaza - pe editiile Server, DWM e
# oprit implicit daca nu e instalata optiunea "Desktop Experience". Verificam
# si aici, ca sa nu riscam o fereastra goala doar pentru un detaliu cosmetic;
# daca DWM nu ruleaza, folosim fereastra Windows standard (cu bara nativa) -
# restul aplicatiei (randata acum mereu cu Qt) nu e afectat cu nimic.
def _dwm_activ():
    if sys.platform!='win32':
        return True
    try:
        activ=ctypes.c_int(0)
        ctypes.windll.dwmapi.DwmIsCompositionEnabled(ctypes.byref(activ))
        return bool(activ.value)
    except Exception:
        return False

def cauta_servere_sql(timeout=2.0):
    """Broadcast SSRP (SQL Server Resolution Protocol) pe portul UDP 1434.
    Gaseste servere din reteaua locala cu serviciul SQL Browser activ.
    Necesita ca UDP 1434 sa nu fie blocat de firewall; nu traverseaza subnet-uri
    fara IP helper; instantele cu 'Hide Instance = Yes' nu raspund."""
    gasite=[]
    s=socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    s.settimeout(timeout)
    try:
        s.sendto(b'\x02', ('255.255.255.255', 1434))
        start=time.time()
        while time.time()-start < timeout:
            ramas=timeout-(time.time()-start)
            if ramas<=0: break
            s.settimeout(ramas)
            try:
                data, addr = s.recvfrom(4096)
            except socket.timeout:
                break
            txt=data[3:].decode('ascii', errors='ignore')
            for bloc in txt.split(';;'):
                info={}
                parti=bloc.split(';')
                for i in range(0, len(parti)-1, 2):
                    info[parti[i]]=parti[i+1]
                if 'ServerName' in info:
                    nume=info['ServerName']
                    if info.get('InstanceName') and info['InstanceName']!='MSSQLSERVER':
                        nume+='\\'+info['InstanceName']
                    gasite.append({
                        'server':nume,
                        'ip':addr[0],
                        'port':info.get('tcp'),  # None => named pipes / port ascuns
                        'raw':info
                    })
    finally:
        s.close()
    return gasite

# ── SESIUNE CRIPTATA (DPAPI) ──────────────────────────────────────────────────
# Salvam progresul (obiecte rezolvate/deployate + ce server/baze/referinta) intr-un
# fisier local, criptat cu Windows DPAPI (legat de userul si masina curenta -
# nimeni altcineva nu-l poate citi, nici macar daca copiaza fisierul pe alt PC).
# NU salvam parola SQL Auth niciodata - se cere mereu manual.

class _DATA_BLOB(ctypes.Structure):
    _fields_=[('cbData',ctypes.c_uint),('pbData',ctypes.POINTER(ctypes.c_char))]

def _dpapi_protect(data: bytes) -> bytes:
    buf=ctypes.create_string_buffer(data,len(data))
    blob_in=_DATA_BLOB(len(data),ctypes.cast(buf,ctypes.POINTER(ctypes.c_char)))
    blob_out=_DATA_BLOB()
    ok=ctypes.windll.crypt32.CryptProtectData(
        ctypes.byref(blob_in),None,None,None,None,0,ctypes.byref(blob_out))
    if not ok: raise OSError('CryptProtectData a esuat (DPAPI).')
    try:
        return ctypes.string_at(blob_out.pbData,blob_out.cbData)
    finally:
        ctypes.windll.kernel32.LocalFree(blob_out.pbData)

def _dpapi_unprotect(data: bytes) -> bytes:
    buf=ctypes.create_string_buffer(data,len(data))
    blob_in=_DATA_BLOB(len(data),ctypes.cast(buf,ctypes.POINTER(ctypes.c_char)))
    blob_out=_DATA_BLOB()
    ok=ctypes.windll.crypt32.CryptUnprotectData(
        ctypes.byref(blob_in),None,None,None,None,0,ctypes.byref(blob_out))
    if not ok: raise OSError('CryptUnprotectData a esuat (DPAPI) - fisier corupt sau alt user/masina.')
    try:
        return ctypes.string_at(blob_out.pbData,blob_out.cbData)
    finally:
        ctypes.windll.kernel32.LocalFree(blob_out.pbData)

def _slug_sesiune(s):
    s=(s or '').strip().lower()
    return re.sub(r'[^a-z0-9_-]+','_',s) or 'sesiune'

def _cale_sesiune(referinta=None):
    baza=os.environ.get('LOCALAPPDATA') or tempfile.gettempdir()
    folder=os.path.join(baza,'GMotionSQLDiff','sesiuni')
    os.makedirs(folder,exist_ok=True)
    if referinta is None:
        return folder
    return os.path.join(folder,f'{_slug_sesiune(referinta)}.dat')

class _Anulat(Exception):
    """Rularea a fost anulata (Deconecteaza) sau inlocuita de o rulare noua."""
    pass

_QUERY=f"""
SELECT s.name,o.name,o.type,m.definition
FROM sys.objects o
JOIN sys.schemas s ON s.schema_id=o.schema_id
JOIN sys.sql_modules m ON m.object_id=o.object_id
WHERE o.type IN ({','.join(f"'{t}'" for t in motor.TIPURI_OBIECTE)});"""

def aduce_obiecte(server,database,auth,user,parola):
    cs=build_cs(server,database,auth,user,parola)
    ob={}
    with pyodbc.connect(cs,timeout=30) as conn:
        for schema,nume,tip,definitie in conn.cursor().execute(_QUERY).fetchall():
            oid=f"{schema}.{nume}"
            if DEBUG_LIMIT and DEBUG_FILTER and DEBUG_FILTER not in oid: continue
            norm=motor.normalizeaza(definitie)
            ob[oid]={'afisare':motor.pregateste_afisare(definitie),
                     'normalizat':norm,'tok':norm.split(),
                     'hash':motor.calc_hash(norm),'tip':(tip or '').strip()}
    return ob


class Api:
    def __init__(self):
        self._prog={'mesaj':'','procent':0,'gata':False,'eroare':None,'log':[]}
        self._lista=[]        # [{id,obiect,baza,scor,rezumat,tip,cat,rezolvat,adaugat}]
        self._cache={}        # {baza: obiecte}   -- pentru diff lazy
        self._ref_cache={}    # obiectele din referinta
        self._referinta=''
        self._meta={}
        self._cancel=threading.Event()
        self._gen=0           # generatia rularii curente (invalideaza thread-urile vechi)

    # -------- descoperire servere in retea --------
    def cauta_servere(self):
        try:
            return {'ok':True,'servere':cauta_servere_sql()}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    # -------- sesiune (progres salvat, criptat DPAPI) --------
    def salveaza_sesiune(self, sesiune):
        """sesiune: dict trimis din JS - server/auth/user/baze/referinta/lista/meta.
        NU trebuie sa contina parola - se ignora explicit daca vine din greseala.
        Se salveaza intr-un fisier separat per baza de referinta."""
        try:
            sesiune=dict(sesiune or {})
            sesiune.pop('parola',None)
            ref=sesiune.get('referinta') or ''
            if not ref:
                return {'ok':False,'mesaj':'Referinta lipseste - nu se poate salva sesiunea.'}
            sesiune['salvat_la']=datetime.now().isoformat(timespec='seconds')
            raw=json.dumps(sesiune,ensure_ascii=False).encode('utf-8')
            enc=_dpapi_protect(raw)
            cale=_cale_sesiune(ref)
            tmp=cale+'.tmp'
            with open(tmp,'wb') as f: f.write(enc)
            os.replace(tmp,cale)
            return {'ok':True}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    def incarca_sesiune(self, referinta):
        try:
            if not referinta:
                return {'ok':True,'exista':False}
            cale=_cale_sesiune(referinta)
            if not os.path.exists(cale):
                return {'ok':True,'exista':False}
            with open(cale,'rb') as f: enc=f.read()
            raw=_dpapi_unprotect(enc)
            sesiune=json.loads(raw.decode('utf-8'))
            return {'ok':True,'exista':True,'sesiune':sesiune}
        except Exception as e:
            return {'ok':False,'exista':False,'mesaj':str(e)}

    def sterge_sesiune(self, referinta):
        try:
            if not referinta:
                return {'ok':True}
            cale=_cale_sesiune(referinta)
            if os.path.exists(cale): os.remove(cale)
            return {'ok':True}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    def lista_sesiuni(self):
        """Metadate pt toate sesiunile salvate, una per baza de referinta."""
        try:
            folder=_cale_sesiune()
            rezultat=[]
            if os.path.isdir(folder):
                for fis in os.listdir(folder):
                    if not fis.endswith('.dat'): continue
                    try:
                        with open(os.path.join(folder,fis),'rb') as f: enc=f.read()
                        raw=_dpapi_unprotect(enc)
                        s=json.loads(raw.decode('utf-8'))
                        lst=s.get('lista') or []
                        rezultat.append({
                            'referinta':s.get('referinta',''),
                            'server':s.get('server',''),
                            'salvat_la':s.get('salvat_la',''),
                            'total':len(lst),
                            'rezolvate':sum(1 for x in lst if x.get('rezolvat')),
                            'omise':sum(1 for x in lst if x.get('omis')),
                            'deploy':sum(1 for x in lst if x.get('deployat')),
                        })
                    except Exception:
                        continue
            rezultat.sort(key=lambda x:x.get('salvat_la',''), reverse=True)
            return {'ok':True,'sesiuni':rezultat}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    # -------- conexiune --------
    def test_conexiune(self,server,auth,user,parola,port=None):
        try:
            cs=build_cs(server,'master',auth,user,parola,port)
            with pyodbc.connect(cs,timeout=8) as conn:
                cur=conn.cursor()
                rows=cur.execute(
                    "SELECT name FROM sys.databases WHERE database_id>4 AND state=0 ORDER BY name"
                ).fetchall()
                info={}
                try:
                    r=cur.execute(
                        "SELECT SERVERPROPERTY('Edition'), "
                        "sys.dm_os_host_info.host_platform, "
                        "DATEDIFF(SECOND, sqlserver_start_time, SYSDATETIME()), "
                        "(SELECT COUNT(*) FROM sys.databases WHERE database_id>4) "
                        "FROM sys.dm_os_sys_info CROSS JOIN sys.dm_os_host_info"
                    ).fetchone()
                    if r:
                        info={'editie':r[0],'os':r[1],'uptime_sec':int(r[2]) if r[2] is not None else None,'nr_baze':r[3]}
                except Exception:
                    pass  # detaliile sunt un bonus - nu blocam conexiunea daca versiunea SQL nu le suporta
            return {'ok':True,'mesaj':'Conectat cu succes.','baze':[r[0] for r in rows],'info':info}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    # -------- porneste comparatia --------
    def ruleaza(self,server,auth,user,parola,baze,referinta):
        for b in list(baze)+[referinta]:
            if not nume_valid(b):
                return {'ok':False,'mesaj':f'Nume invalid: {b!r}'}
        self._gen+=1
        gen=self._gen
        self._cancel=threading.Event()
        self._prog={'mesaj':'Pornesc...','procent':0,'gata':False,'eroare':None,'log':[]}
        def lucru():
            try: self._comparatie(gen,server,auth,user,parola,list(baze),referinta)
            except _Anulat: pass
            except Exception as e:
                if gen==self._gen:
                    self._prog['eroare']=str(e); self._prog['gata']=True
        threading.Thread(target=lucru,daemon=True).start()
        return {'ok':True}

    def anuleaza(self):
        """Oprit din UI (Deconecteaza in timpul verificarii): invalideaza rularea
        curenta si semnaleaza thread-ului de comparatie sa se opreasca."""
        self._gen+=1
        self._cancel.set()
        self._prog={'mesaj':'Anulat.','procent':0,'gata':True,'eroare':None,'log':self._prog.get('log',[])+['[ANULAT] Operatiune anulata de utilizator.']}
        return {'ok':True}

    def progres(self): return self._prog

    def log_lines(self, de_la=0):
        """Returneaza liniile de log incepand de la indexul dat (pentru poll incremental)."""
        log = self._prog.get('log', [])
        return {'linii': log[de_la:], 'total': len(log)}

    def _set(self,gen,mesaj,procent):
        if gen!=self._gen: raise _Anulat()
        ts = datetime.now().strftime('%H:%M:%S')
        self._prog['mesaj']=mesaj
        self._prog['procent']=procent
        self._prog['log'].append(f'[{ts}] {mesaj}')

    def _check(self,gen):
        if gen!=self._gen or self._cancel.is_set(): raise _Anulat()

    def _comparatie(self,gen,server,auth,user,parola,baze,referinta):
        cancel=self._cancel
        alte=[b for b in baze if b!=referinta]
        # 1. aduc obiectele in PARALEL (max 4 conexiuni simultane)
        de_adus=[referinta]+alte
        gata=0; total_db=len(de_adus)
        self._set(gen,f"Conectare la {total_db} baze de date...",3)
        cache={}
        ex=ThreadPoolExecutor(max_workers=min(4,total_db))
        try:
            fut={ex.submit(aduce_obiecte,server,b,auth,user,parola):b for b in de_adus}
            for f in as_completed(fut):
                if cancel.is_set() or gen!=self._gen:
                    raise _Anulat()
                b=fut[f]
                result=f.result()
                cache[b]=result
                gata+=1
                n_ob=len(result)
                self._set(gen,f"[{gata}/{total_db}] {b} — {n_ob} obiecte incarcate",5+int(27*gata/total_db))
        finally:
            ex.shutdown(wait=False,cancel_futures=True)
        self._check(gen)
        ref=cache[referinta]
        n_ref=len(ref)
        self._set(gen,f"Referinta '{referinta}': {n_ref} obiecte. Incep comparatia...",33)
        # 2. compara prin hash + scor (fara diff complet - rapid!)
        toate=sorted(ref.keys()); lista=[]; ident=lipsa=extra=0; nid=0
        total=len(toate)
        raportat_la=0
        for idx,nume in enumerate(toate):
            if idx%1000==0:
                self._check(gen)
                proc=32+int(58*idx/max(1,total))
                self._set(gen,f"Compar obiectele: {idx}/{total} ({proc}%)...",proc)
                raportat_la=idx
            r=ref[nume]
            for b in alte:
                ob=cache[b]
                if nume not in ob:
                    lipsa+=1
                    lista.append({'id':nid,'obiect':nume,'baza':b,'scor':None,
                                  'rezumat':'lipseste complet','tip':r['tip'],
                                  'cat':'lipsa','rezolvat':False,'omis':False,'adaugat':{}})
                    nid+=1; continue
                if ob[nume]['hash']==r['hash']: ident+=1; continue
                scor=round(difflib.SequenceMatcher(
                    None,r['tok'],ob[nume]['tok'],autojunk=False).ratio()*100,2)
                if scor>=100.0: ident+=1; continue
                rez=motor.rezumat_rapid(r['afisare'],ob[nume]['afisare'])
                lista.append({'id':nid,'obiect':nume,'baza':b,'scor':scor,
                              'rezumat':rez,'tip':r['tip'],
                              'cat':'dif','rezolvat':False,'omis':False,'adaugat':{}})
                nid+=1
        # 3. obiecte EXTRA
        self._check(gen)
        self._set(gen,"Caut obiecte extra (exista in tinta, lipsesc din referinta)...",92)
        for b in alte:
            for nume,o in cache[b].items():
                if nume in ref: continue
                extra+=1
                lista.append({'id':nid,'obiect':nume,'baza':b,'scor':None,
                              'rezumat':'exista doar in aceasta baza','tip':o['tip'],
                              'cat':'extra','rezolvat':False,'omis':False,'adaugat':{}})
                nid+=1
        self._check(gen)
        self._cache=cache; self._ref_cache=ref; self._referinta=referinta
        self._lista=lista
        self._meta={'referinta':referinta,'baze':alte,'total':total,
                    'ident':ident,'lipsa':lipsa,'extra':extra}
        n_dif=len([x for x in lista if x['cat']=='dif'])
        self._set(gen,f"Gata! {total} obiecte: {ident} identice, {n_dif} diferite, {lipsa} lipsa, {extra} extra.",100)
        self._prog['gata']=True

    # -------- date pentru UI --------
    def rezultate(self):
        lite=[{'id':x['id'],'obiect':x['obiect'],'baza':x['baza'],'scor':x['scor'],
               'rezumat':x['rezumat'],'tip':x['tip'],'cat':x['cat'],
               'rezolvat':x['rezolvat'],'omis':x.get('omis',False),'adaugat':x['adaugat'],
               # hash-uri folosite doar la restaurarea sesiunii, ca sa detectam
               # daca obiectul s-a schimbat intre timp (referinta SAU tinta)
               'ref_hash':self._ref_cache.get(x['obiect'],{}).get('hash'),
               'cur_hash':self._cache.get(x['baza'],{}).get(x['obiect'],{}).get('hash')}
              for x in self._lista]
        return {'meta':self._meta,'lista':lite}

    # -------- diff LAZY (doar la deschidere) --------
    def elemente(self, obj_id):
        try:
            rec = next((x for x in self._lista if x['id'] == obj_id), None)
            if not rec: return []
            ref_afis = self._cache.get(self._referinta, {}).get(rec['obiect'], {}).get('afisare', '')
            cur_afis = self._cache.get(rec['baza'], {}).get(rec['obiect'], {}).get('afisare', '')
            rd = motor.diff_aliniat(ref_afis, cur_afis)
            result = motor.diff_to_elemente(rd)
            for el in result:
                if el.get('ss'): el['ss'] = [[x, y] for x, y in el['ss']]
                if el.get('ds'): el['ds'] = [[x, y] for x, y in el['ds']]
            #nt(f"elemente OK: {len(result)} elemente returnate")
            return result
        except Exception as e:
            import traceback
            #print(f"EROARE elemente: {e}")
            traceback.print_exc()
            return []
    # -------- export CSV --------
    def exporta_csv(self,randuri):
        try:
            if not randuri: return {'ok':False,'mesaj':'Nimic de exportat.'}
            nume_implicit=f"sqldiff_export_{datetime.now():%Y%m%d_%H%M%S}.csv"
            out=self._alege_fisier_salvare(nume_implicit,
                                           ('Fisier CSV (*.csv)','Toate fisierele (*.*)'))
            if out=='__ANULAT__':
                return {'ok':False,'anulat':True}
            if not out:
                # dialogul nu e disponibil -> revenim la folderul temp (comportament vechi)
                out=os.path.join(tempfile.gettempdir(),nume_implicit)
            if not out.lower().endswith('.csv'): out+='.csv'
            with open(out,'w',newline='',encoding='utf-8-sig') as f:
                w=csv.writer(f,delimiter=';')
                w.writerow(['Obiect','Baza','Tip','Categorie','Similaritate %','Diferente','Rezolvat','Omis'])
                for r in randuri:
                    w.writerow([r.get('obiect',''),r.get('baza',''),r.get('tip',''),
                                r.get('cat',''),
                                '' if r.get('scor') is None else r.get('scor'),
                                r.get('rezumat',''),
                                'DA' if r.get('rezolvat') else 'NU',
                                'DA' if r.get('omis') else 'NU'])
            try: os.startfile(out)
            except Exception: pass
            return {'ok':True,'cale':out}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    # -------- deployer --------
    def genereaza_deployer(self,bifate,mod='baza',n=10):
        """mod: 'unul' (tot intr-un fisier), 'baza' (unul per baza),
        'obiect' (unul per obiect), 'n' (cate n obiecte per fisier).
        Denumiri descriptive: index_baza_tip_obiect."""
        try:
            if not bifate: return {'ok':False,'mesaj':'Niciun obiect marcat.'}
            pe_baza={}; sarite=0
            for item in bifate:
                oid=item['id']; adaugat=item.get('adaugat') or {}
                rec=next((x for x in self._lista if x['id']==oid),None)
                if not rec: continue
                if rec.get('cat')=='extra':
                    sarite+=1; continue   # exista doar in tinta -> nimic de adus din referinta
                ref_afis=self._cache.get(self._referinta,{}).get(rec['obiect'],{}).get('afisare','')
                cur_afis=self._cache.get(rec['baza'],{}).get(rec['obiect'],{}).get('afisare','')
                # NOU: editorul live trimite textul final integral sub '__FINAL__'
                if isinstance(adaugat,dict) and '__FINAL__' in adaugat:
                    cod=adaugat['__FINAL__'] or ''
                    if not cod.strip():
                        continue
                    cod=self._aplica_create_or_alter(cod)
                    pe_baza.setdefault(rec['baza'],[]).append((rec['obiect'],cod,rec.get('tip','')))
                    continue
                if not cur_afis and not adaugat:
                    cod=ref_afis
                else:
                    rd=motor.diff_aliniat(ref_afis,cur_afis)
                    el=motor.diff_to_elemente(rd)
                    linii=[]
                    for e in el:
                        cheie=str(e['i'])
                        if cheie in adaugat:
                            v=adaugat[cheie]
                            if v=='__EXCLUS__': continue
                            if isinstance(v,str): linii.append(v)
                            elif e['dr'] is not None: linii.append(e['dr'])
                        elif e['dr'] is not None:
                            linii.append(e['dr'])
                    cod='\n'.join(linii)
                cod=self._aplica_create_or_alter(cod)
                pe_baza.setdefault(rec['baza'],[]).append((rec['obiect'],cod,rec.get('tip','')))
            if not pe_baza:
                return {'ok':False,'mesaj':'Obiectele marcate sunt doar "extra" - nu exista cod de adus din referinta.'}

            nume_sub=f"deploy_{re.sub(r'[^\w]','_',self._referinta or 'ref')}_{datetime.now():%Y%m%d_%H%M%S}"
            folder=self._alege_folder()
            if folder=='__ANULAT__':
                return {'ok':False,'anulat':True}
            out=os.path.join(folder,nume_sub) if folder else os.path.join(tempfile.gettempdir(),nume_sub)
            os.makedirs(out,exist_ok=True)

            fisiere=self._scrie_deployer(out,pe_baza,mod,n)
            nr=sum(len(v) for v in pe_baza.values())
            try: os.startfile(out)
            except Exception: pass
            return {'ok':True,'cale':out,'nr':nr,'fisiere':fisiere,'sarite':sarite}
        except Exception as e:
            return {'ok':False,'mesaj':str(e)}

    @staticmethod
    def _tip_nume(t):
        return {'P':'PROC','V':'VIEW','FN':'FUNC','TF':'FUNC','IF':'FUNC','TR':'TRIGGER'}.get((t or '').strip(),'OBJ')

    @staticmethod
    def _slug(s):
        return re.sub(r'_+','_',re.sub(r'[^\w.]','_',s or '')).strip('_')

    def _antet(self,baza):
        return (f"-- DEPLOYER {baza} | referinta: {self._referinta} | {datetime.now():%Y-%m-%d %H:%M}\n"
                f"USE [{baza}];\nGO\n\n")

    def _bloc_obiect(self,nome,cod):
        return f"-- ═══ {nome} ═══\n{cod}\nGO\n\n"

    def _scrie_deployer(self,out,pe_baza,mod,n):
        """Scrie fisierele conform modului ales. Intoarce numarul de fisiere create.
        Denumiri: <index>_<baza>_<TIP>_<obiect>.sql (per obiect) sau
        <index>_<baza>_partN.sql (grupate), <baza>.sql (per baza), deploy_complet.sql (tot)."""
        fisiere=0
        if mod=='obiect':
            # cate un fisier per obiect, cu nume descriptiv
            for baza,obiecte in pe_baza.items():
                idx=0
                for nome,cod,tip in obiecte:
                    idx+=1
                    nume=f"{idx:03d}_{self._slug(baza)}_{self._tip_nume(tip)}_{self._slug(nome)}.sql"
                    with open(os.path.join(out,nume),'w',encoding='utf-8') as f:
                        f.write(self._antet(baza)); f.write(self._bloc_obiect(nome,cod))
                    fisiere+=1
        elif mod=='n':
            # cate n obiecte per fisier
            n=max(1,int(n or 1))
            for baza,obiecte in pe_baza.items():
                part=0
                for i in range(0,len(obiecte),n):
                    part+=1
                    grup=obiecte[i:i+n]
                    nume=f"{self._slug(baza)}_part{part:02d}_obj{i+1}-{i+len(grup)}.sql"
                    with open(os.path.join(out,nume),'w',encoding='utf-8') as f:
                        f.write(self._antet(baza))
                        for nome,cod,tip in grup: f.write(self._bloc_obiect(nome,cod))
                    fisiere+=1
        elif mod=='unul':
            # tot intr-un singur fisier (toate bazele)
            nume="deploy_complet.sql"
            with open(os.path.join(out,nume),'w',encoding='utf-8') as f:
                for baza,obiecte in pe_baza.items():
                    f.write(self._antet(baza))
                    for nome,cod,tip in obiecte: f.write(self._bloc_obiect(nome,cod))
                    f.write("\n")
            fisiere=1
        else:  # 'baza' (implicit): un fisier per baza
            for baza,obiecte in pe_baza.items():
                nume=f"{self._slug(baza)}.sql"
                with open(os.path.join(out,nume),'w',encoding='utf-8') as f:
                    f.write(self._antet(baza))
                    for nome,cod,tip in obiecte: f.write(self._bloc_obiect(nome,cod))
                fisiere+=1
        return fisiere

    # ── DIALOGURI NATIVE DE SALVARE ──────────────────────────────────────────
    # Intorc: cale (string) daca userul a ales; '__ANULAT__' daca a apasat Cancel;
    # None daca dialogul nu e disponibil (=> apelantul revine la folderul temp).
    def _alege_fisier_salvare(self, nume_implicit, filtre=None):
        win=getattr(self,'_fereastra',None)
        if win is None: return None
        try:
            kw={'dialog_type':webview.SAVE_DIALOG,'save_filename':nume_implicit}
            if filtre: kw['file_types']=tuple(filtre)
            rez=win.create_file_dialog(**kw)
        except Exception:
            try:
                rez=win.create_file_dialog(webview.SAVE_DIALOG, save_filename=nume_implicit)
            except Exception:
                return None
        return self._dialog_rezultat(rez)

    def _alege_folder(self):
        win=getattr(self,'_fereastra',None)
        if win is None: return None
        try:
            rez=win.create_file_dialog(dialog_type=webview.FOLDER_DIALOG)
        except Exception:
            try:
                rez=win.create_file_dialog(webview.FOLDER_DIALOG)
            except Exception:
                return None
        return self._dialog_rezultat(rez)

    @staticmethod
    def _dialog_rezultat(rez):
        # pywebview intoarce None/() la Cancel, sau un tuple/lista/string cu calea
        if rez is None: return '__ANULAT__'
        if isinstance(rez,(list,tuple)):
            if len(rez)==0: return '__ANULAT__'
            return rez[0]
        return rez

    @staticmethod
    def _aplica_create_or_alter(c):
        """Transforma primul CREATE sau ALTER <tip obiect> in CREATE OR ALTER,
        indiferent de spatii/newline-uri/comentarii inainte. Nu atinge codul
        care e deja CREATE OR ALTER."""
        if re.search(r'\bCREATE\s+OR\s+ALTER\b', c, re.IGNORECASE):
            return c
        pat=re.compile(r'\b(CREATE|ALTER)\s+(PROC(?:EDURE)?|VIEW|FUNCTION|TRIGGER)\b',
                       re.IGNORECASE)
        def repl(m): return 'CREATE OR ALTER '+m.group(2).upper()
        result,n=pat.subn(repl,c,count=1)
        return result if n else c

    # ── CONTROALE FEREASTRA (bara de titlu custom) ───────────────────────────
    def win_minimize(self):
        try: self._fereastra.minimize()
        except Exception: pass
        return {'ok':True}

    @staticmethod
    def _get_hwnd(w):
        """Handle-ul nativ al ferestrei, compatibil cu diverse versiuni de pywebview."""
        try:
            if hasattr(w, 'hwnd'):
                return w.hwnd
            if hasattr(w, 'native') and hasattr(w.native, 'Handle'):
                return w.native.Handle.ToInt32()
        except Exception:
            pass
        return None

    @staticmethod
    def _work_area(hwnd=None):
        """Zona de lucru (ecran minus taskbar) a MONITORULUI pe care se afla fereastra.
        SPI_GETWORKAREA intoarce mereu monitorul principal, ceea ce facea ca
        maximizare/fullscreen sa mute fereastra pe monitorul 1 chiar daca era pe
        monitorul 2 - de asta folosim MonitorFromWindow + GetMonitorInfoW, care
        dau zona de lucru corecta a monitorului curent."""
        class RECT(ctypes.Structure):
            _fields_=[('left',ctypes.c_long),('top',ctypes.c_long),
                      ('right',ctypes.c_long),('bottom',ctypes.c_long)]
        class MONITORINFO(ctypes.Structure):
            _fields_=[('cbSize',ctypes.c_ulong),('rcMonitor',RECT),
                      ('rcWork',RECT),('dwFlags',ctypes.c_ulong)]
        user32=ctypes.windll.user32
        if hwnd:
            try:
                MONITOR_DEFAULTTONEAREST=2
                hmon=user32.MonitorFromWindow(ctypes.c_void_p(int(hwnd)), MONITOR_DEFAULTTONEAREST)
                if hmon:
                    mi=MONITORINFO()
                    mi.cbSize=ctypes.sizeof(MONITORINFO)
                    if user32.GetMonitorInfoW(hmon, ctypes.byref(mi)):
                        r=mi.rcWork
                        return r.left, r.top, r.right-r.left, r.bottom-r.top
            except Exception:
                pass
        # fallback: monitorul principal (doar daca n-am putut identifica hwnd-ul)
        r=RECT()
        user32.SystemParametersInfoW(0x0030, 0, ctypes.byref(r), 0)
        return r.left, r.top, r.right-r.left, r.bottom-r.top

    import ctypes

    def win_toggle_max(self):
        """Maximizare adaptată la scalarea DPI (lasă taskbar-ul complet liber),
        pe monitorul pe care se afla efectiv fereastra."""
        try:
            w = self._fereastra

            if getattr(self, '_fullscreen', False):
                w.toggle_fullscreen()
                self._fullscreen = False

            if getattr(self, '_maximizat', False):
                x, y, wd, ht = getattr(self, '_geo_anterior', (100, 100, 1200, 800))
                w.resize(wd, ht)
                w.move(x, y)
                self._maximizat = False
            else:
                self._geo_anterior = (w.x, w.y, w.width, w.height)

                # 1. Obținem DPI-ul ecranului curent pentru a compensa scalarea Windows (ex: 125%, 150%)
                user32 = ctypes.windll.user32
                user32.SetProcessDPIAware()  # Ne asigurăm că Python citește valorile reale de DPI

                hwnd = self._get_hwnd(w)
                if hwnd:
                    dpi = user32.GetDpiForWindow(hwnd)
                else:
                    # Fallback dacă nu găsim fereastra: folosim DPI-ul ecranului principal (96 = 100% scalare)
                    hdc = user32.GetDC(0)
                    dpi = ctypes.windll.gdi32.GetDeviceCaps(hdc, 88)  # LOGPIXELSX
                    user32.ReleaseDC(0, hdc)

                scale = dpi / 96.0  # Factorul de scalare (ex: 1.25 sau 1.5)

                # 2. Zona de lucru a monitorului CURENT (nu mereu cel principal)
                l, t, wd_phys, ht_phys = self._work_area(hwnd)

                # 3. Impartim coordonatele fizice la factorul de scalare pentru a le transforma in pixeli logici
                l = int(l / scale)
                t = int(t / scale)
                wd = int(wd_phys / scale)
                ht = int(ht_phys / scale)

                # Aplicăm noile dimensiuni corectate
                w.move(l, t)
                w.resize(wd, ht)
                self._maximizat = True

        except Exception as e:
            print(f"Eroare toggle_max cu DPI: {e}")

        return {'ok': True, 'maximizat': getattr(self, '_maximizat', False)}

    def win_fullscreen(self):
        """F11: fullscreen complet, peste taskbar. Toggle."""
        try:
            w = self._fereastra

            # Daca eram maximizati la nivel de zona de lucru, salvam asta
            era_maximizat = getattr(self, '_maximizat', False)

            w.toggle_fullscreen()
            self._fullscreen = not getattr(self, '_fullscreen', False)

            # Daca am intrat in fullscreen, anulam starea de maximizat (deoarece acum suntem peste tot ecranul)
            if self._fullscreen:
                self._maximizat = False
            else:
                # Daca am iesit din fullscreen si inainte eram maximizati, refacem maximizarea curat,
                # pe monitorul pe care se afla fereastra (nu mereu monitorul principal)
                if era_maximizat:
                    hwnd = self._get_hwnd(w)
                    l, t, wd, ht = self._work_area(hwnd)
                    w.move(l, t)
                    w.resize(wd, ht)
                    self._maximizat = True
        except Exception as e:
            print(f"Eroare fullscreen: {e}")
        return {'ok': True, 'fullscreen': getattr(self, '_fullscreen', False)}

    def win_resize_edge(self, w, h, edge):
        """Redimensionare din margini/colturi (grip-urile din bara custom).
        edge: n/s/e/w/ne/nw/se/sw. FixPoint tine fixa latura OPUSA celei trase,
        astfel tragerea din stanga/sus muta si fereastra, ca la o fereastra nativa."""
        try:
            if getattr(self,'_maximizat',False) or getattr(self,'_fullscreen',False):
                return {'ok':True}
            try:
                from webview.window import FixPoint
            except Exception:
                from webview import FixPoint
            fp_map={
                'e':  FixPoint.NORTH|FixPoint.WEST,
                's':  FixPoint.NORTH|FixPoint.WEST,
                'se': FixPoint.NORTH|FixPoint.WEST,
                'w':  FixPoint.NORTH|FixPoint.EAST,
                'sw': FixPoint.NORTH|FixPoint.EAST,
                'n':  FixPoint.SOUTH|FixPoint.WEST,
                'ne': FixPoint.SOUTH|FixPoint.WEST,
                'nw': FixPoint.SOUTH|FixPoint.EAST,
            }
            fp=fp_map.get(edge, FixPoint.NORTH|FixPoint.WEST)
            self._fereastra.resize(max(480,int(w)), max(600,int(h)), fix_point=fp)
        except Exception:
            pass
        return {'ok':True}

    def win_close(self):
        try: self._cancel.set()
        except Exception: pass
        try:
            self._fereastra.destroy()
        except Exception:
            os._exit(0)
        # plasa de siguranta: daca destroy() nu declanseaza inchiderea completa
        # a procesului (WebView2 agatat), fortam iesirea dupa un scurt delay.
        def _kill():
            time.sleep(1.2)
            os._exit(0)
        threading.Thread(target=_kill,daemon=True).start()
        return {'ok':True}

    # dimensiuni ferestrei in functie de ecran: mica la conectare, mare la lista/diff
    def win_resize(self, mod):
        """mod='mic' (conectare) sau 'mare' (lista/diff)."""
        try:
            # Blocam complet orice resize automat daca fereastra e maximizata sau fullscreen
            if getattr(self, '_maximizat', False) or getattr(self, '_fullscreen', False):
                return {'ok': True}

            if mod == 'mic':
                self._fereastra.resize(560, 800)
            else:
                self._fereastra.resize(1200, 800)
        except Exception as e:
            print(f"Eroare resize: {e}")
        return {'ok': True}


# ── BARA DE TITLU CUSTOM (injectata din backend, fara a atinge ui.py) ────────
_TITLEBAR = r"""
<style>
  :root{ --tb-h:38px; }
  /* Rezervam inaltimea barii SI corectam ecranele care folosesc 100vh, ca sa nu
     reapara taierea de sus. html primeste padding-ul, iar .screen (100vh) e
     recalculat sa incapa exact sub bara. */
  html{ box-sizing:border-box; padding-top:var(--tb-h); }
  .screen{ height:calc(100vh - var(--tb-h))!important; }
  #gm-titlebar{
    position:fixed; top:0; left:0; right:0; height:var(--tb-h); z-index:2147483000;
    display:flex; align-items:center; gap:10px; padding:0 6px 0 14px;
    background:var(--navy2); border-bottom:1px solid var(--border2);
    -webkit-user-select:none; user-select:none; font-family:system-ui,'Segoe UI',sans-serif;
  }
  /* zona care permite tragerea ferestrei. pywebview recunoaste nativ clasa
     .pywebview-drag-region (nu -webkit-app-region ca in Electron). */
  #gm-titlebar .tb-drag{ flex:1; height:100%; display:flex; align-items:center; gap:9px;
    cursor:default; }
  #gm-titlebar .tb-dot{ width:9px;height:9px;border-radius:50%;
    background:linear-gradient(135deg,var(--teal),var(--blue)); box-shadow:0 0 8px var(--teal2); }
  #gm-titlebar .tb-title{ font-size:12px; font-weight:700; letter-spacing:.4px; color:var(--txt); }
  #gm-titlebar .tb-title b{ color:var(--teal); }
  #gm-titlebar .tb-btns{ display:flex; height:100%; }
  #gm-titlebar .tb-btn{
    width:44px; height:var(--tb-h); border:none; background:transparent; color:var(--muted);
    display:flex; align-items:center; justify-content:center; cursor:pointer;
    font-size:15px; line-height:1; transition:background .15s,color .15s; padding:0;
  }
  #gm-titlebar .tb-btn:hover{ background:var(--border2); color:var(--txt); }
  #gm-titlebar .tb-btn.tb-close:hover{ background:var(--coral); color:#fff; }
  #gm-titlebar .tb-btn svg{ width:15px; height:15px; }
  /* GRIP-uri de redimensionare pe margini si colturi (fereastra frameless).
     Peste bara de titlu (z-index mai mare) ca sa poti trage si de sus. */
  .gm-grip{ position:fixed; z-index:2147483647; }
  .gm-grip[data-edge="n"]{ top:0;left:10px;right:10px;height:5px;cursor:ns-resize; }
  .gm-grip[data-edge="s"]{ bottom:0;left:10px;right:10px;height:6px;cursor:ns-resize; }
  .gm-grip[data-edge="e"]{ top:10px;bottom:10px;right:0;width:6px;cursor:ew-resize; }
  .gm-grip[data-edge="w"]{ top:10px;bottom:10px;left:0;width:6px;cursor:ew-resize; }
  .gm-grip[data-edge="ne"]{ top:0;right:0;width:12px;height:12px;cursor:nesw-resize; }
  .gm-grip[data-edge="nw"]{ top:0;left:0;width:12px;height:12px;cursor:nwse-resize; }
  .gm-grip[data-edge="se"]{ bottom:0;right:0;width:14px;height:14px;cursor:nwse-resize; }
  .gm-grip[data-edge="sw"]{ bottom:0;left:0;width:14px;height:14px;cursor:nesw-resize; }
</style>
<div id="gm-titlebar">
  <div class="tb-drag pywebview-drag-region">
    <span class="tb-dot"></span>
    <span class="tb-title">GMOTION <b>&bull;</b> SQL Diff &amp; Deploy</span>
  </div>
  <div class="tb-btns">
    <button class="tb-btn" id="tb-min" title="Minimizeaza" aria-label="Minimizeaza">
      <svg viewBox="0 0 16 16"><rect x="3" y="7.4" width="10" height="1.3" fill="currentColor"/></svg>
    </button>
    <button class="tb-btn" id="tb-max" title="Maximizeaza (F11 = fullscreen)" aria-label="Maximizeaza">
      <svg viewBox="0 0 16 16" id="tb-max-ico"><rect x="3.5" y="3.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.3"/></svg>
    </button>
    <button class="tb-btn tb-close" id="tb-close" title="Inchide" aria-label="Inchide">
      <svg viewBox="0 0 16 16"><path d="M4 4 L12 12 M12 4 L4 12" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/></svg>
    </button>
  </div>
</div>
<div class="gm-grip" data-edge="n"></div><div class="gm-grip" data-edge="s"></div>
<div class="gm-grip" data-edge="e"></div><div class="gm-grip" data-edge="w"></div>
<div class="gm-grip" data-edge="ne"></div><div class="gm-grip" data-edge="nw"></div>
<div class="gm-grip" data-edge="se"></div><div class="gm-grip" data-edge="sw"></div>
<script>
(function(){
  var ICO_MAX='<rect x="3.5" y="3.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.3"/>';
  var ICO_RESTORE='<rect x="5" y="3.5" width="7.5" height="7.5" fill="none" stroke="currentColor" stroke-width="1.2"/>'
                 +'<rect x="3.5" y="5" width="7.5" height="7.5" style="fill:var(--navy2)" stroke="currentColor" stroke-width="1.2"/>';
  function api(){ return (window.pywebview && window.pywebview.api) ? window.pywebview.api : null; }
  function setMaxIcon(maximizat){
    var ico=document.getElementById('tb-max-ico');
    var btn=document.getElementById('tb-max');
    if(maximizat){ ico.innerHTML=ICO_RESTORE; btn.title='Restaureaza (F11 = fullscreen)'; }
    else { ico.innerHTML=ICO_MAX; btn.title='Maximizeaza (F11 = fullscreen)'; }
  }
  function bind(){
    var a=api(); if(!a){ setTimeout(bind,120); return; }
    document.getElementById('tb-min').onclick=function(){ a.win_minimize && a.win_minimize(); };
    document.getElementById('tb-close').onclick=function(){ a.win_close && a.win_close(); };
    document.getElementById('tb-max').onclick=function(){
      if(!a.win_toggle_max) return;
      a.win_toggle_max().then(function(r){ setMaxIcon(r && r.maximizat); });
    };
    // dublu-click pe bara = toggle maximize la zona de lucru
    document.querySelector('#gm-titlebar .tb-drag').addEventListener('dblclick',function(){
      document.getElementById('tb-max').click();
    });
    // F11 = fullscreen complet (peste taskbar), toggle
    document.addEventListener('keydown',function(e){
      if(e.key==='F11'){
        e.preventDefault();
        if(a.win_fullscreen) a.win_fullscreen().then(function(){ setMaxIcon(false); });
      }
    });
    // GRIP-uri: tragere din margini/colturi -> resize prin backend
    document.querySelectorAll('.gm-grip').forEach(function(g){
      g.addEventListener('pointerdown',function(ev){
        if(!a.win_resize_edge) return;
        ev.preventDefault();
        var edge=g.getAttribute('data-edge');
        var sx=ev.screenX, sy=ev.screenY;
        var sw=window.innerWidth, sh=window.innerHeight;
        var raf=null, ultimW=sw, ultimH=sh;
        function mv(e2){
          var dx=e2.screenX-sx, dy=e2.screenY-sy;
          var w=sw, h=sh;
          if(edge.indexOf('e')>=0) w=sw+dx;
          if(edge.indexOf('w')>=0) w=sw-dx;
          if(edge.indexOf('s')>=0) h=sh+dy;
          if(edge.indexOf('n')>=0) h=sh-dy;
          ultimW=w; ultimH=h;
          if(raf) return;   // max un apel de resize pe frame
          raf=requestAnimationFrame(function(){ raf=null; a.win_resize_edge(ultimW,ultimH,edge); });
        }
        function up(){
          document.removeEventListener('pointermove',mv);
          document.removeEventListener('pointerup',up);
          try{ g.releasePointerCapture(ev.pointerId); }catch(x){}
        }
        document.addEventListener('pointermove',mv);
        document.addEventListener('pointerup',up);
        try{ g.setPointerCapture(ev.pointerId); }catch(x){}
      });
    });
  }
  if(document.readyState==='loading') document.addEventListener('DOMContentLoaded',bind);
  else bind();
})();
</script>
"""


def main():
    ok,drv=verifica_driver()
    if not ok:
        ctypes.windll.user32.MessageBoxW(None,
            "Lipseste 'ODBC Driver 17 for SQL Server'.\n"
            "https://go.microsoft.com/fwlink/?linkid=2249006\n\n"
            "Drivere: "+(', '.join(drv) if drv else '(niciunul)'),
            "SQL Object Diff",0x30)
        sys.exit(1)

    gui=gui_disponibil()
    dwm_ok=_dwm_activ()
    if not dwm_ok:
        ctypes.windll.user32.MessageBoxW(None,
            "Compositorul Windows (DWM) nu ruleaza pe acest server - probabil "
            "lipseste optiunea 'Desktop Experience' (frecvent pe Windows Server). "
            "Nu afecteaza functionarea aplicatiei (randarea foloseste Qt, inclus "
            "in aplicatie) - doar bara de titlu custom nu se poate desena corect "
            "fara DWM, asa ca aplicatia porneste cu fereastra Windows standard.",
            "GMotion - fereastra standard (fara DWM)",0x40)

    api=Api()
    pagina=ui.PAGINA
    if dwm_ok:
        # injectam bara de titlu custom la finalul HTML-ului (fara a modifica ui.py)
        if "</body>" in pagina:
            pagina=pagina.replace("</body>", _TITLEBAR+"</body>", 1)
        else:
            pagina=pagina+_TITLEBAR
        frameless=True
    else:
        # fara DWM: fereastra nativa Windows (cu bara ei) - nu mai depinde de
        # compositor, deci nu mai ramane alba. Fundalul transparent/custom nu
        # se mai aplica, dar continutul (SPA-ul) se randeaza normal.
        frameless=False

    try:
        fereastra=webview.create_window(
            "GMotion - SQL Diff & Deploy", html=pagina, js_api=api,
            width=560, height=800, min_size=(480,600),
            resizable=True,
            frameless=frameless,
            easy_drag=False,           # drag DOAR din bara custom (.pywebview-drag-region), nu din tot ecranul
            background_color="#0a0e17" if dwm_ok else "#ffffff"
        )
    except Exception as e:
        ctypes.windll.user32.MessageBoxW(None,
            "Nu am putut porni fereastra aplicatiei:\n"+str(e),
            "GMotion - eroare pornire",0x10)
        sys.exit(1)
    api._fereastra=fereastra
    api._maximizat=False
    api._fullscreen=False

    # ── INCHIDERE CURATA ──────────────────────────────────────────────────────
    # WebView2 (Chromium) lasa procese copil si handle-uri deschise pe .exe daca
    # procesul Python nu se termina explicit. Fara asta, dupa inchidere .exe-ul
    # ramane "in uz" si nu poate fi sters/suprascris de scriptul .bat.
    # La evenimentul 'closed' oprim tot fortat (thread-uri daemon + WebView2).
    def _la_inchidere():
        try: api._cancel.set()   # opreste orice comparatie in curs
        except Exception: pass
        # os._exit termina IMEDIAT procesul si toate thread-urile, eliberand
        # handle-ul pe .exe. Folosim un thread scurt ca destroy() sa apuce sa
        # inchida fereastra nativa inainte.
        def _kill():
            time.sleep(0.3)
            os._exit(0)
        threading.Thread(target=_kill,daemon=True).start()
    fereastra.events.closed += _la_inchidere

    #webview.start(debug=True, gui=gui)
    try:
        webview.start(gui=gui)
    except Exception as e:
        ctypes.windll.user32.MessageBoxW(None,
            "Interfata nu a putut porni:\n"+str(e)+
            "\n\nDaca ai vazut asta la compilare/testare locala, probabil lipsesc "
            "'PyQt5'/'PyQtWebEngine' (pip install PyQt5 PyQtWebEngine) sau nu au "
            "fost incluse corect in build-ul PyInstaller (foloseste --collect-all PyQt5).",
            "GMotion - eroare",0x10)
    # daca start() se termina normal (fereastra inchisa), fortam oprirea completa
    # ca sa nu ramana procese (Qt WebEngine porneste procese auxiliare proprii)
    # orfane care tin .exe-ul blocat.
    os._exit(0)

if __name__=='__main__': main()