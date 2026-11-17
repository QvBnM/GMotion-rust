"""
motor.py - Motor diff SQL cu strategie Gemini:
Split DOAR pe statements cu efect (SELECT, INSERT, UPDATE, DELETE, SET @var=, EXEC, IF, WHILE, RETURN)
CREATE/AS/BEGIN/END/DECLARE raman unite cu vecinii lor - formeaza 'headerul' obiectului
"""
import re, difflib, hashlib
from functools import lru_cache

TIPURI_OBIECTE = ('P', 'V', 'FN', 'IF', 'TF', 'TR')
_HINTS = (r'NOLOCK|READPAST|ROWLOCK|TABLOCK|TABLOCKX|UPDLOCK|XLOCK|HOLDLOCK|'
          r'READUNCOMMITTED|READCOMMITTED|REPEATABLEREAD|SERIALIZABLE|PAGLOCK|NOWAIT')


# ── Normalizare ────────────────────────────────────────────────────────────────
def _norm(s):
    if not s: return ''
    s = re.sub(r'/\*.*?\*/', ' ', s, flags=re.DOTALL)
    s = re.sub(r'--[^\n]*', ' ', s)
    s = re.sub(rf'\bWITH\s*\(\s*(?:{_HINTS})(?:\s*,\s*(?:{_HINTS}))*\s*\)', ' ', s, flags=re.IGNORECASE)
    s = re.sub(rf'\b(?:{_HINTS})\b', ' ', s, flags=re.IGNORECASE)
    s = re.sub(r'\[dbo\]\.', '', s, flags=re.IGNORECASE)
    s = re.sub(r'\bdbo\.', '', s, flags=re.IGNORECASE)
    s = re.sub(r'\bCOLLATE\s+database_default\b', '', s, flags=re.IGNORECASE)
    s = re.sub(r'\[([^\[\]]+)\]', r'\1', s)
    s = re.sub(r'\s+', ' ', s)
    # dupa eliminarea hint-urilor de lock / COLLATE database_default poate ramane
    # un spatiu orfan inainte de virgula/paranteza inchisa (ex: "char(24) ,")
    # care ar fi produs o normalizare diferita fata de originalul fara acel token
    # -> blocuri identice semantic erau marcate gresit ca MODIFICAT in editor.
    s = re.sub(r'\s+([,;)])', r'\1', s)
    return s.strip().lower()

def normalizeaza(cod):
    if not cod: return ''
    return _norm(cod.replace('\r\n', '\n').replace('\r', '\n'))

def pregateste_afisare(cod):
    if cod is None: return ''
    cod = cod.replace('\r\n', '\n').replace('\r', '\n')
    linii = [l.rstrip() for l in cod.split('\n')]
    while linii and not linii[0].strip(): linii.pop(0)
    while linii and not linii[-1].strip(): linii.pop()
    return '\n'.join(linii)

def calc_hash(n): return hashlib.sha256(n.encode()).hexdigest()

def scor_similaritate(ref, cur):
    return round(difflib.SequenceMatcher(
        None, normalizeaza(ref).split(), normalizeaza(cur).split(),
        autojunk=False).ratio() * 100, 2)

# ── Segmentare - DOAR pe statements cu efect ──────────────────────────────────
# BEGIN/END/DECLARE/CREATE/ALTER/AS = structura, nu split
# SELECT/INSERT/UPDATE/DELETE/SET/EXEC/IF/WHILE/RETURN/MERGE = split
_STMT_START = re.compile(
    r'^\s*(?:'
    r'SELECT\b|INSERT\b|UPDATE\b|DELETE\b|MERGE\b|'
    r'SET\s+@|'           # SET @var = ... (assignment, nu SET option)
    r'EXEC(?:UTE)?\b|'
    r'IF\b|ELSE\b|WHILE\b|'
    r'RETURN\b|PRINT\b|RAISERROR\b|THROW\b|'
    r'OPEN\b|CLOSE\b|FETCH\b|DEALLOCATE\b|'
    r'TRUNCATE\b|DROP\b|GRANT\b|DENY\b|REVOKE\b|'
    r'UNION\b'
    r')',
    re.IGNORECASE
)

def _split_statements(cod):
    """
    Imparte codul SQL in blocuri logice (statements cu efect).
    Strategia Gemini:
    - Tot ce e inainte de primul statement cu efect = bloc "header"
      (CREATE, AS, BEGIN, DECLARE, parametri etc.)
    - Split la: SELECT, INSERT, UPDATE, DELETE, SET @var=, EXEC, IF, WHILE, RETURN
    - IF/WHILE/ELSE nu se separa de corpul lor (un SELECT dupa IF = acelasi bloc)
    - END singur = bloc propriu
    - Liniile goale separa doar daca am trecut de header
    """
    if not cod: return []
    linii = cod.split('\n')
    blocuri = []
    curent = []
    in_header = True   # suntem in headerul obiectului (inainte de primul stmt cu efect)

    prev_strip = ''
    for linie in linii:
        s = linie.strip()

        if not s:
            # linie goala
            if not in_header and curent:
                blocuri.append('\n'.join(curent))
                curent = []
            elif in_header and curent:
                # in header: pastram linia goala ca parte din bloc
                curent.append(linie)
            prev_strip = ''
            continue

        # END singur => bloc propriu intotdeauna
        if re.match(r'^end\s*$', s, re.IGNORECASE):
            if curent:
                blocuri.append('\n'.join(curent))
            curent = [linie]
            in_header = False
            prev_strip = s
            continue

        # NU split daca linia precedenta era IF/WHILE/ELSE (corpul lor)
        corp_control = bool(re.match(r'^(if|else|while)\b', prev_strip, re.IGNORECASE))

        if _STMT_START.match(s) and not corp_control:
            # statement cu efect - marcam ca am iesit din header
            if in_header:
                in_header = False
            if curent:
                blocuri.append('\n'.join(curent))
            curent = [linie]
        else:
            curent.append(linie)

        prev_strip = s

    if curent:
        blocuri.append('\n'.join(curent))

    # uneste UNION / UNION ALL cu statement-ul urmator
    rezultat = []
    i = 0
    while i < len(blocuri):
        n = _norm(blocuri[i])
        if re.search(r'\bunion(?:\s+all)?\s*$', n) and i + 1 < len(blocuri):
            rezultat.append(blocuri[i] + '\n' + blocuri[i+1])
            i += 2
        else:
            rezultat.append(blocuri[i])
            i += 1

    return [b for b in rezultat if _norm(b)]

@lru_cache(maxsize=1024)
def _split_cached(cod):
    """Acelasi split, dar cache-uit: elemente() + deployer + rezumat refolosesc
    aceleasi texte, deci evitam re-segmentarea repetata a acelorasi obiecte."""
    return tuple(_split_statements(cod))

@lru_cache(maxsize=4096)
def _cheie_bloc(s):
    n = _norm(s)
    tip = 'other'
    for kw in ('select', 'insert', 'update', 'delete', 'set',
               'exec', 'if', 'while', 'return', 'create',
               'begin', 'end', 'declare', 'union'):
        if n.startswith(kw): tip = kw; break
    h = hashlib.md5(n.encode()).hexdigest()[:8]
    return tip + ':' + n[:40] + ':' + h

def rezumat_rapid(text_ref, text_cur):
    """Rezumat pentru lista: diff DOAR la nivel de blocuri (fara diff intra-bloc
    caracter-cu-caracter, care e partea scumpa). De ~10-50x mai rapid decat
    diff_aliniat() pe obiecte mari, cu acelasi rezultat numeric."""
    sv = _split_cached(text_ref or '')
    sn = _split_cached(text_cur or '')
    kv = [_cheie_bloc(s) for s in sv]
    kn = [_cheie_bloc(s) for s in sn]
    m = difflib.SequenceMatcher(None, kv, kn, autojunk=False)
    mod = add = st = 0
    for tag, i1, i2, j1, j2 in m.get_opcodes():
        if tag == 'replace':
            comun = min(i2 - i1, j2 - j1)
            # blocurile "replace" cu norm identic ar fi 'egal' in diff_aliniat
            for k in range(comun):
                if _norm(sv[i1 + k]) != _norm(sn[j1 + k]):
                    mod += 1
            st  += (i2 - i1) - comun
            add += (j2 - j1) - comun
        elif tag == 'delete':
            st += i2 - i1
        elif tag == 'insert':
            add += j2 - j1
    p = []
    if mod: p.append(f"{mod} modificate")
    if add: p.append(f"{add} adaugate")
    if st:  p.append(f"{st} sterse")
    return ', '.join(p) if p else 'diferente minore'

# ── Segmente cosmetice (ignorate la evidentiere, desi apar diferite caracter-cu-caracter) ──
# _norm() sterge [dbo]., dbo., COLLATE database_default si hint-urile de lock la
# CLASIFICARE (egal/modificat). Dar cand blocul e "modificat" din alt motiv (o
# diferenta reala oriunde in acelasi bloc unit), _diff_intra lucreaza pe textul
# BRUT si re-scoate la iveala exact aceste segmente cosmetice ca "diferente",
# desi userul le vede deja ca zgomot (nu sunt diferente reale). Le recunoastem
# explicit si le tratam ca 'neschimbat' la evidentiere.
_COSMETIC_RE = re.compile(
    rf'^(?:\[dbo\]\.?|dbo\.|\[|\]|collate\s+database_default|'
    rf'with\s*\(\s*(?:{_HINTS})(?:\s*,\s*(?:{_HINTS}))*\s*\)|(?:{_HINTS})|\s)+$',
    re.IGNORECASE)

def _e_cosmetic(s):
    return bool(s) and bool(_COSMETIC_RE.match(s))

# ── Diff intra-bloc ────────────────────────────────────────────────────────────
def _diff_intra(a, b):
    m = difflib.SequenceMatcher(None, a, b, autojunk=False)
    sr, sc = [], []
    for tag, i1, i2, j1, j2 in m.get_opcodes():
        va = a[i1:i2]; vb = b[j1:j2]; ch = (tag != 'equal')
        if ch and (va == '' or _e_cosmetic(va)) and (vb == '' or _e_cosmetic(vb)):
            ch = False   # diferenta e doar cosmetica (paranteze/dbo./COLLATE/hint) - nu o evidentiem
        if va: sr.append((va, ch))
        if vb: sc.append((vb, ch))
    return sr, sc

# ── Diff principal ─────────────────────────────────────────────────────────────
def diff_aliniat(text_ref, text_cur):
    sv = list(_split_cached(text_ref or ''))
    sn = list(_split_cached(text_cur or ''))

    kv = [_cheie_bloc(s) for s in sv]
    kn = [_cheie_bloc(s) for s in sn]

    m = difflib.SequenceMatcher(None, kv, kn, autojunk=False)
    randuri = []; idx = 0

    for tag, i1, i2, j1, j2 in m.get_opcodes():
        if tag == 'equal':
            for k in range(i2 - i1):
                randuri.append({'i': idx, 'tip': 'egal',
                    'st': sv[i1+k], 'dr': sn[j1+k],
                    'ss': None, 'ds': None}); idx += 1
        elif tag == 'replace':
            bv, bn = sv[i1:i2], sn[j1:j2]
            comun = min(len(bv), len(bn))
            for k in range(comun):
                nv = _norm(bv[k]); nn = _norm(bn[k])
                if nv == nn:
                    randuri.append({'i': idx, 'tip': 'egal',
                        'st': bv[k], 'dr': bn[k],
                        'ss': None, 'ds': None})
                else:
                    ss, ds = _diff_intra(bv[k], bn[k])
                    randuri.append({'i': idx, 'tip': 'modificat',
                        'st': bv[k], 'dr': bn[k],
                        'ss': ss, 'ds': ds})
                idx += 1
            for k in range(comun, len(bv)):
                randuri.append({'i': idx, 'tip': 'sters',
                    'st': bv[k], 'dr': None,
                    'ss': None, 'ds': None}); idx += 1
            for k in range(comun, len(bn)):
                randuri.append({'i': idx, 'tip': 'adaugat',
                    'st': None, 'dr': bn[k],
                    'ss': None, 'ds': None}); idx += 1
        elif tag == 'delete':
            for k in range(i1, i2):
                randuri.append({'i': idx, 'tip': 'sters',
                    'st': sv[k], 'dr': None,
                    'ss': None, 'ds': None}); idx += 1
        elif tag == 'insert':
            for k in range(j1, j2):
                randuri.append({'i': idx, 'tip': 'adaugat',
                    'st': None, 'dr': sn[k],
                    'ss': None, 'ds': None}); idx += 1

    return randuri

def rezumat_diff(randuri):
    mod = sum(1 for r in randuri if r['tip'] == 'modificat')
    add = sum(1 for r in randuri if r['tip'] == 'adaugat')
    st  = sum(1 for r in randuri if r['tip'] == 'sters')
    p = []
    if mod: p.append(f"{mod} modificate")
    if add: p.append(f"{add} adaugate")
    if st:  p.append(f"{st} sterse")
    return ', '.join(p) if p else 'diferente minore'

def diff_to_elemente(randuri):
    return [{'i': r['i'], 'tip': r['tip'],
             'st': r['st'], 'dr': r['dr'],
             'ss': r['ss'], 'ds': r['ds']} for r in randuri]