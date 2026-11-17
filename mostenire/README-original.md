# SQL Object Diff — comparator + deployer de obiecte SQL

Aplicație desktop (fereastră nativă) care compară proceduri, view-uri, funcții și triggere
între baze de date SQL Server, față de o bază de **referință**. Reconstruiești obiectele
aducând ce le lipsește, marchezi ce ai rezolvat, și generezi la final un **deployer .sql**
cu toate modificările în lanț (`CREATE OR ALTER`), gata de rulat manual.

---

## Cum îl transformi în .exe (o singură dată, pe Windows)

Mașină Windows cu **Python 3.9–3.12** (recomandat; vezi nota despre 3.14 mai jos).

1. Copiază folderul (cu `app.py`, `motor.py`, `ui.py`, `SQLObjectDiff.spec`, `BUILD.bat`) pe Windows.
2. **Dublu-click normal** pe `BUILD.bat`.
   (sau manual: deschide folderul în Explorer, scrie `cmd` în bara de adresă, Enter, apoi
   `pip install pyinstaller pyodbc pywebview` și `pyinstaller SQLObjectDiff.spec`)
3. Rezultatul: **`dist\SQLObjectDiff.exe`** — un singur fișier, de copiat la orice client.

> **Important:** compilarea NU se face ca administrator (PyInstaller o refuză).

---

## Cum se folosește la client

1. Dublu-click pe `SQLObjectDiff.exe` -> se deschide **direct** fereastra aplicației
   (fără prompt UAC, fără fereastră de consolă).
2. În fereastra aplicației:
   - Alegi **SQL** (user+parolă) sau **Windows** (cont curent).
   - Completezi **Server**, apeși **Testează conexiunea**.
   - Bifezi bazele de comparat, marchezi una ca **referință**.
   - **Pornește verificarea** (vezi procent + timp scurs live).
3. Apare **lista** diferențelor:
   - Filtre: căutare (în nume **și** în rezumatul diferențelor), bază, **tip obiect**
     (proceduri / view-uri / funcții / triggere), **categorie** (diferite / lipsă / extra),
     stare (rezolvat / nerezolvat) și **interval de similaritate** (min–max %).
   - Sortare pe Obiect, Bază, Tip, Similaritate sau Stare (cu indicator ▲▼).
   - **Export CSV** al listei filtrate (se deschide direct în Excel).
   - Coloana **Stare**: bulină plină (verde) = rezolvat, bulină goală = nerezolvat.
   - Categoria **EXTRA** = obiecte care există în baza țintă dar NU în referință.
4. **Deschide** un obiect -> vizualizarea pe **3 coloane** (Sursă | gutter cu ->/<- | Bază | Preview).
   Aduci blocurile lipsă, vezi rezultatul reconstruit live. **Back** te întoarce la listă.
5. Bifezi **"Am ajuns la o concluzie"** pentru obiectele rezolvate.
6. **Generează deployer .sql** -> un fișier per bază țintă (cu `USE [baza]` + `CREATE OR ALTER` +
   `GO`), salvat într-un folder temporar care se deschide automat. Îl rulezi manual în SSMS.

---

## Cerințe pe mașina clientului

- **ODBC Driver 17 for SQL Server** (component de sistem; de obicei prezent cu SSMS).
  Dacă lipsește, aplicația afișează link de descărcare.
- **WebView2 Runtime** (motorul Edge; prezent pe Windows 10/11 modern). Dacă lipsește pe un
  Windows vechi, se instalează gratuit de la Microsoft (Evergreen Runtime).

---

## Securitate

- Nu necesită drepturi de Administrator (conexiunile SQL nu au nevoie de elevare).
- Datele de conectare **manual de fiecare dată**, **nimic pe disc** (parola doar în RAM).
- Fereastră nativă pywebview — **fără server HTTP**, fără port deschis.
- Numele bazelor validate strict (anti-injecție). Conexiuni cu `Encrypt=yes`.
- Deployer-ul e **doar generat** (fișier .sql), **nu se aplică automat** — tu îl rulezi.
- Fără internet, fără telemetrie.

---

## Notă despre Python 3.14

Ai raportat Python 3.14. E foarte nou și `pywebview`/`pyodbc`/PyInstaller pot avea probleme la
împachetare pe el. Dacă `.exe`-ul crapă la pornire, instalează **Python 3.12** și recompilează.

---

## Fișiere

- `app.py` — backend: conexiuni (fetch paralel), comparație, deployer, export CSV, fereastra pywebview.
- `motor.py` — normalizare, diff aliniat, scor (logică pură).
- `ui.py` — interfața (SPA: conectare -> listă cu filtre -> detaliu 3 coloane -> deployer).
- `SQLObjectDiff.spec` — build (fără consolă, fără UAC, hiddenimports pentru pywebview).
- `BUILD.bat` — build one-click (refuză rularea ca admin).

---

## Noutăți v2.3

- **Autentificarea Windows funcționează** (era stricată de la început: interfața trimitea
  `win`, backend-ul aștepta `windows`, deci pica mereu pe autentificare SQL cu user gol).
- **Baza de referință este evidențiată** în listă la selectare (fundal + nume amber +
  bară laterală), nu doar steluța.
- **„Deconectează" în timpul verificării** nu mai blochează aplicația: rularea din fundal
  este anulată curat, ecranul revine la conectare, iar o verificare nouă pornește imediat.
  Rulările vechi nu mai pot suprascrie rezultatele unei rulări noi (protecție pe generații).

---

## Noutăți v2.2

- **Pornire directă**: fără prompt UAC și fără fereastră de consolă la deschidere.
- **Motor mai rapid**: obiectele se aduc din toate bazele **în paralel** (până la 4 conexiuni),
  iar rezumatul diferențelor se calculează doar la nivel de blocuri (~16x mai rapid pe
  obiecte mari), cu rezultat identic. Segmentarea SQL este cache-uită.
- **Deployer corectat**: acum respectă blocurile *modificate* aduse din sursă, blocurile
  **excluse** manual și **editările din Edit Preview** (înainte erau ignorate silențios).
  Obiectele *lipsă complet* marcate ca rezolvate se aduc integral din referință (înainte
  generau cod gol dacă nu apăsai „Aplică toate"). `ALTER` devine și el `CREATE OR ALTER`.
- **Obiecte EXTRA**: se detectează și obiectele care există doar în baza țintă.
- **UI**: dropdown-ul de teme apare deasupra listei (toate temele selectabile), starea
  „rezolvat" e afișată ca **bulină plină**, coloanele listei sunt aliniate cu antetul,
  filtre noi (tip, categorie, interval de similaritate), export CSV, ETA la progres,
  scurtături: `Esc` (înapoi/închide), `Ctrl+F` (căutare), `Alt+↑/↓` (navigare diferențe).

> Notă: interfața încarcă fonturile, iconițele și editorul (CodeMirror) de pe CDN-uri;
> pe o mașină complet fără internet acestea nu se vor afișa (aplicația rămâne funcțională).
