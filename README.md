# GMotion — SQL Diff & Deploy

Aplicatie desktop (Windows) care compara obiectele de cod dintre baze SQL
Server. Alegi o baza de **REFERINTA** si una sau mai multe baze **TINTA**,
aplicatia arata diferentele intr-un editor side-by-side editabil, iar la final
genereaza scripturi `.sql` de deploy (`CREATE OR ALTER`) pentru obiectele pe
care le-ai rezolvat.

Obiecte acoperite: proceduri (`P`), view-uri (`V`), functii (`FN`, `IF`, `TF`),
triggere (`TR`) — citite din `sys.sql_modules`.

Versiunea curenta: **3.4** (tag `v3.4`).

> Rescrierea in Rust se face separat, in repo-ul `GMotion-rust`, care porneste
> din aceleasi commit-uri de baza. Acest repo rămâne versiunea Python, in
> productie.

---

## Structura

| Fisier | Linii | Rol |
|---|---:|---|
| `app.py` | 1102 | backend: pyodbc, sesiuni DPAPI, deployer, controale fereastra, `main()` |
| `motor.py` | 280 | motorul de comparare (normalizare, segmentare, aliniere) |
| `ui.py` | 5224 | interfata completa: HTML + CSS (6 teme) + ~3500 linii JS |
| `GMotion.spec` | — | specificatia PyInstaller |
| `BUILD.bat` | — | scriptul de build |
| `docs/README-original.md` | — | documentatia care insotea versiunea 3.4 |

Stack: Python + pywebview, interfata randata de WebView2 (motorul Edge).

---

## Rulare

```
pip install -r requirements.txt
python app.py
```

`GMOTION_GUI` suprascrie motorul de randare. Implicit `qt` — vezi problema 1.

## Build

Dublu-click pe `BUILD.bat`, sau direct:

```
pyinstaller --clean GMotion.spec
```

Rezultat: `dist\GMotion.exe`, un singur fisier, de copiat la client.

Build-ul **nu** se ruleaza ca Administrator; `BUILD.bat` refuza explicit.
Executabilul final nu cere UAC.

## Cerinte pe masina clientului

- Windows
- Un driver ODBC pentru SQL Server (18, 17, Native Client sau generic).
  `gaseste_driver_sql()` alege automat cel mai nou instalat.
- **WebView2 Runtime** (motorul Edge). Prezent pe Windows 10/11 modern; pe
  Windows vechi se instaleaza separat de la Microsoft.

---

## Probleme cunoscute

### 1. Codul cere Qt, executabilul livreaza WebView2

`app.py` porneste implicit pe `gui='qt'` (`gui_disponibil()` returneaza `'qt'`
daca `GMOTION_GUI` nu e setat), iar comentariile din el explica de ce: pe
servere fara WebView2 pywebview cadea pe motorul Internet Explorer, care nu
suporta variabile CSS, si interfata ieșea alb-negru. Solutia descrisa e
includerea Qt WebEngine in executabil.

**Solutia nu a fost construita.** Verificat in log-ul de build al versiunii 3.4:

- `GMotion.spec` declara `hiddenimports=[..., 'webview.platforms.edgechromium', ...]`
  — WebView2, nu Qt
- `Analysis-00.toc` nu conține **niciun** modul `PyQt5.Qt*` colectat
- `QtWebEngineProcess.exe`, procesul auxiliar de care Qt WebEngine are nevoie,
  lipseste complet
- build-ul a raportat `missing module named 'qtpy.QtWebEngineWidgets'`,
  `missing module named 'qtpy.QtWebEngineCore'` si
  `missing module named 'PyQt5.QtWebKitWidgets'`
- `BUILD.bat` instaleaza `pyinstaller pyodbc pywebview` — fara `PyQt5` si
  `PyQtWebEngine`
- `BUILD.bat` opreste `msedgewebview2.exe`, proces fara rol dupa trecerea pe Qt

Cronologia confirma: `GMotion.spec` si `BUILD.bat` sunt din 21 iulie, `app.py`
din 29 iulie. Codul a fost schimbat, pipeline-ul de build nu.

Consecinta practica: pe o masina fara WebView2 Runtime aplicatia nu are cu ce sa
randeze, desi codul crede ca motorul vine inclus.

**Decizie deschisa.** Doua variante, fiecare cu un cost real:

| | Independent de masina tinta | Dimensiune exe | `--onefile` |
|---|---|---|---|
| Qt impachetat, cum cere `app.py` | da | 46 MB -> ~250 MB | nu, obligatoriu `--onedir` |
| WebView2, cum e acum | nu, cere Runtime instalat | 46 MB | da |

Pana la decizie, `GMotion.spec` si `BUILD.bat` rămân neatinse, ca sa nu se
schimbe in liniste comportamentul livrat.

Daca se merge pe Qt, de corectat si: instalarea sa vina din
`requirements.txt`, o verificare de import a `PyQt5.QtWebEngineWidgets` inainte
de compilare, si `taskkill` pe `QtWebEngineProcess.exe` in loc de
`msedgewebview2.exe`.

### 2. Motorul de comparare e implementat de doua ori, si a divergat

`motor.py` (Python, folosit de backend pentru lista si rezumate) si portul lui
din `ui.py` (JavaScript, folosit de editorul live) ar trebui sa dea rezultate
identice. Portul JS include o reimplementare de mana a lui
`difflib.SequenceMatcher`, un MD5 scris manual si `_round_py()` — o imitatie a
rotunjirii bancare din Python, fiindca `Math.round` din JS ar da alte scoruri.

Masurat pe 24 de cazuri T-SQL: **8 diverg**. Normalizarea si scorul de
similaritate sunt identice; toata divergenta vine din segmentarea in blocuri si
de acolo se propaga in tipurile de diferenta. Doua ajung in fata
utilizatorului:

| Caz | Lista (`motor.py`) | Editor (JS) |
|---|---|---|
| `DECLARE` sters din antet | `1 modificate` | `1 sterse` |
| comentariu bloc pe mai multe linii | `2 sterse` | `diferente minore` |

In al doilea, un comentariu bloc multiline face `motor.py` sa raporteze doua
blocuri sterse **care nu exista** — fals pozitiv in lista, care dispare cand
deschizi obiectul.

Cauza: portul JS a fost dezvoltat mai departe si nu a fost adus inapoi in
Python. Are, in plus:

- splitare si pe `CREATE` si `DECLARE`
- urmarirea literalelor, a comentariilor bloc, a adancimii parantezelor si a
  stivei `CASE`/`BEGIN`/`END`, deci nu mai taie in mijlocul lor
- `_blocEsteContinuat` (cursor `FOR`, `INSERT ... SELECT`)
- `_unesteBlocuriCompuse`, `_unesteCicluriCursor`
- `_perechiReplace` — aliniere prin programare dinamica pe similaritate, in loc
  de imperechere pozitionala

Daca se face o corectie in Python fara migrare, portul JS e varianta corecta de
copiat. Harness-ul care masoara divergenta se afla in repo-ul `GMotion-rust`,
sub `rust/referinta/`.

### 3. Consum de memorie la incarcare

In `aduce_obiecte()`, pentru fiecare obiect din fiecare baza se pastreaza
simultan `afisare`, `normalizat`, `hash` **si `tok`** — o lista Python de
token-uri. Lista de string-uri e partea scumpa: fiecare token e un obiect
Python separat, cu antetul lui. Inmultit cu numarul de cuvinte, de obiecte si
de baze. Fetch-ul ruleaza pe 4 fire in paralel, deci varful e simultan.

Practic, cea mai mare problema de rulare — nu GPU-ul si nu viteza CPU.

**Fix tintit:** `tok` se calculeaza la cerere, doar pentru obiectele la care
hash-ul difera. Pentru cele identice, hash-ul e suficient si token-urile nu sunt
niciodata folosite.

### 4. Performanta CPU

`difflib.SequenceMatcher` e Python pur. Per obiect diferit se ruleaza un diff pe
token-uri (`scor_similaritate`) plus unul pe blocuri (`rezumat_rapid`), pe un
singur fir. Masurat pe 400 perechi de proceduri (~20.500 linii): 887 ms pentru
scor + rezumat.

### 5. GPU

Accelerarea GPU produce artefacte pe RDP si pe masini fara GPU real, si e deja
dezactivata explicit in `app.py` prin
`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`. **Nu** e cauza incetinelii — vezi
punctele 3 si 4.

### 6. Inchiderea procesului

Trei locuri din `app.py` folosesc `os._exit(0)`, plus fire de kill cu
`time.sleep`, fiindca motorul de randare lasa procese copil si handle-uri
deschise pe `.exe`, iar fara ele executabilul rămâne „in uz" si nu poate fi
suprascris de scriptul de build.

### 7. Minor

Mesajul de eroare la pornire cere „ODBC Driver 17", dar `gaseste_driver_sql()`
prefera versiunea cea mai noua (18 > 17 > ... > Native Client > generic).
Mesajul ar trebui sa reflecte logica reala.

---

## Securitate

- Nu cere drepturi de Administrator.
- Parola SQL nu se scrie niciodata pe disc; se cere manual la fiecare
  conectare si rămâne doar in memorie.
- Sesiunile (progresul: ce e rezolvat, ce a fost trimis la deploy) se salveaza
  criptate cu Windows DPAPI, legate de userul si masina curenta.
- Numele bazelor sunt validate strict inainte de a intra in interogari.
  Conexiunile folosesc `Encrypt=yes`.
- Deployer-ul e doar **generat** ca fisier `.sql`; nu se aplica automat.
- Fara server HTTP, fara port deschis, fara internet, fara telemetrie.
