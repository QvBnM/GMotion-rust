# GMotion v3.4 — Python, inghetat

**Acest director nu se modifica.** Nici formatare, nici „curatenie", nici
corecturi. E dovada contra careia se valideaza versiunea Rust.

Se sterge abia dupa ce Rust-ul trece paritatea pe baze reale (vezi planul din
`README.md`).

---

## Ce e aici

| Fisier | Linii | Rol |
|---|---:|---|
| `app.py` | 1102 | Backend: descoperire drivere ODBC, conexiune, descoperire instante SQL prin broadcast UDP 1434, sesiuni criptate DPAPI, clasa `Api` expusa catre interfata, control de fereastra Win32, export CSV, generator de deploy |
| `ui.py` | 5224 | Interfata completa: HTML + CSS + JS intr-un singur fisier, servita in WebView. Fara CDN-uri. Conține si **al doilea motor de comparare**, in JS |
| `motor.py` | 280 | Motorul de comparare folosit de backend |
| `GMotion.spec`, `BUILD.bat` | | Impachetare PyInstaller |
| `requirements.txt` | | `pyodbc`, `pywebview`, PyQt5, PyInstaller |
| `resurse/`, `GMotion.ico` | | Iconite si fundal |
| `README-original.md` | | Documentatia versiunii Python |

Arhitectura: fereastra WebView locala, cu clasa `Api` ca punte Python <-> JS.

## Cum se rula

```
pip install -r mostenire/requirements.txt
python mostenire/app.py
```

---

## Problemele cunoscute — de ce exista rescrierea

**1. Motorul de comparare exista in doua copii, care au divergat.**
`motor.py` (lista, scoruri, rezumate) si portul lui in JS din `ui.py` (editorul
live) ar trebui sa dea rezultate identice. Masurat pe 24 de cazuri: **8
diverg**. In doua, diferenta ajunge in fata utilizatorului — un comentariu bloc
pe mai multe linii face `motor.py` sa raporteze doua blocuri sterse care nu
exista, fals pozitiv care dispare cand deschizi obiectul. Simptomul si cauza
sunt in fisiere diferite, deci e genul de bug greu de urmarit.
Toata divergenta provine din **segmentarea in blocuri**. Detalii si tabel in
`referinta/README.md`.

**2. Dependente pe masina tinta.** Cere WebView2 Runtime si un driver ODBC
instalate pe server.

**3. Memoria.** `aduce_obiecte()` tine, pentru fiecare obiect din fiecare baza,
o lista Python de token-uri pe langa text. Fiecare token e un obiect separat,
cu antetul lui.

**4. Inconsistenta de impachetare.** `app.py` porneste implicit pe `gui='qt'`,
dar `GMotion.spec` nu impacheteaza Qt. Liniile de PyQt5 din `requirements.txt`
descriu ce cere codul, nu ce ajunge in executabil.

**5. Coliziune de sesiuni.** Cheia de sesiune e doar numele bazei de referinta
(`_cale_sesiune(referinta)` -> `<slug>.dat`). Un al doilea modul care salveaza
progres pe aceeasi baza ar suprascrie sesiunea primului. In Rust cheia devine
`<modul>/<slug>.dat` din start.

**6. Expunere in loc de rutare.** Clasa `Api` expune metodele modulului de diff
direct la nivelul de sus. Cu patru module, metodele comune (`test_conexiune`,
`cauta_servere`, `progres`, `log_lines`, controalele de fereastra) trebuie sa
stea la nivelul platformei, scrise o singura data, iar cele specifice sa se
inregistreze cu prefix.

---

## Ce se citeste de aici, si de ce

- `motor.py` si `ui.py` sunt intrarile harness-ului din `referinta/`. Daca se
  schimba (nu ar trebui), fixture-urile trebuie re-extrase.
- `motor_js.mjs` din `referinta/` e extras **mecanic** din `ui.py`
  (liniile 2593-3106, plus `_tokenize` si `_normLinie`), ca sa nu existe risc
  de a-l rescrie din greseala.
