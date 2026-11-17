# Etapa 6 — paritate pe baze reale

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Pe baze SQL Server reale, lista de obiecte data de `gmotion` coincide cu cea a
versiunii Python v3.4, iar fiecare diferenta ramasa e explicata ca o corectura
de bug (motorul JS din editor da acelasi raspuns ca Rust-ul). Dupa asta,
versiunea Python poate fi retrasa.

## Criterii de terminare

Partea care se poate face fara Windows:

- [x] `referinta/paritate.py` ruleaza **codul Python vechi, neatins**
      (`Api._comparatie` din `mostenire/app.py`) pe un instantaneu, fara server
- [x] acelasi script compara rezultatul cu `gmotion compara --json` si
      clasifica fiecare diferenta: `regresie` sau `corectura` (JS == Rust)
- [x] un instantaneu sintetic din `referinta/cazuri.json` trece prin tot
      lantul, iar poarta il ruleaza
- [x] verificarea din poarta a fost vazuta cazand (regresie introdusa deliberat)

Partea care are nevoie de rulari pe Windows:

- [x] `cargo test --all` trece pe Windows (singura proba reala pentru DPAPI:
      testele din `platforma/src/protejare.rs` fac ciclul cu implementarea
      tintei; `diagnostic` doar raporteaza daca e activ)
- [x] `gmotion diagnostic`, `baze` si `instantaneu` merg pe masina lui Mihai
      (TLS SChannel, autentificare SQL si integrata)
- [x] pe instantaneul real: zero diferente clasificate `regresie`
- [x] ~~lista exportata de aplicatia Python (CSV)~~ — inlocuit, cu ceva mai
      strict: `paritate_pyodbc.py` compara obiect cu obiect (hash normalizat,
      text de afisare, tip) ce aduce `pyodbc` cu ce aduce `tiberius`. Dovada ca
      driverele aduc aceleasi obiecte
- [x] timpul lui `gmotion instantaneu` pe bazele reale e masurat si comparat
      cu incarcarea din aplicatia Python (citirea Rust e secventiala, Python
      foloseste 4 conexiuni — vezi `JURNAL.md`, „Citirea bazelor rămâne
      secventiala"); daca e inacceptabil, paralelismul intra in `db`
- [x] obiectele relevante din instantaneul real (extrase, anonimizate daca
      trebuie) sunt fixture de regresie in repo — instantaneul intreg are 202 MB
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. `referinta/paritate.py`: Python vechi pe instantaneu, comparat cu Rust.
2. Instantaneu sintetic din cazurile de conformitate, rulat de poarta.
3. Mihai ruleaza `diagnostic` si `instantaneu` pe Windows, plus exportul CSV
   din aplicatia Python.
4. `paritate.py` pe instantaneul real; fiecare `regresie` se repara in Rust.
5. Fixture de regresie din instantaneul real.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa. Observatie care schimba planul din `STARE.md`:
  `Api._comparatie` din `mostenire/app.py` nu are nevoie de server daca
  primeste obiectele gata aduse. Deci lista Python se poate produce **aici**,
  din instantaneu, ruland codul vechi neatins (`pyodbc` si `webview` inlocuite
  cu module goale la import). Exportul CSV de pe Windows ramane necesar doar
  ca dovada ca cele doua drivere aduc aceleasi obiecte.
- **2026-11-17** `referinta/paritate.py` + `rezumat_js.mjs`, pas nou in poarta.
  Pe instantaneul din cazuri: 0 regresii, 1 corectura (`declare_in_header`).
  Vazut cazand de doua ori, deliberat: rezumatul `lipsa` schimbat in
  `comparare.rs`, si „sterse" -> „eliminate" in `core/src/diff.rs`. **Prima
  incalcare a gasit un bug in script**: crapa cu `KeyError` in loc sa
  raporteze, fiindca incerca sa arbitreze prin JS un rand `lipsa`, care nu are
  definitie in ambele baze. Corectat: la `lipsa`/`extra` un rezumat diferit e
  direct regresie.
- **2026-11-17** Asteptat doua corecturi de rezumat (README-ul de referinta
  numeste doua divergente vizibile), a ieșit una. Recalculat: la
  `comentariu_bloc_multiline` textul normalizat e identic, deci in lista e
  identic in ambele versiuni si nu primeste rezumat deloc. Cifra era corecta.

- **2026-11-17** Prima rulare pe Windows (Mihai): `cargo test --all` verde,
  166 teste (168 pe Linux, minus doua care exista doar pe non-Windows — numarat
  pe crate-uri, se potriveste). `diagnostic --json`: `protejare_criptata` si
  `auth_integrata_disponibila` `true`. Compilarea in release merge, deci si
  legarea de `crypt32`. **Gaura gasita:** `dus_intors` din `protejare.rs` ar
  trece si cu o protejare care intoarce textul neschimbat, deci nu dovedea
  DPAPI. Adaugat `pe_windows_datele_nu_apar_in_clar`, vazut cazand pe
  implementarea de Linux (cfg inversat temporar). Trebuie rulat pe Windows.
- **2026-11-17** Prima comanda data lui Mihai avea `<repo>` ca loc de
  completat; PowerShell il citeste ca redirectare si nu ruleaza nimic. Si
  `$env:GMOTION_PAROLA = "..."` ar fi pus parola in istoricul PSReadLine.
  Ambele corectate in `STARE.md`.

- **2026-11-17** Poarta a iesit o data rosie fara nicio modificare de cod.
  Cauta: `jurnal::teste::linii_cere_doar_ce_e_nou` lucra pe tamponul global,
  in care scriau in paralel alte teste — 4 caderi din 400 de rulari.
  Investigand, **bug real in productie**: indexul dat de `jurnal::linii` era
  pozitia in coada, iar dupa `MAX_LINII` coada se roteste la lungime constanta,
  deci „ce e nou de la N" intorcea gol pentru totdeauna — jurnalul din
  interfata ar fi inghetat dupa 2000 de linii. Corectat cu index absolut;
  testele pe `Tampon` local, vazute cazand pe logica veche; 0 caderi din 600
  dupa. Tot de aici: poarta numara testele din randurile `test ... ok`, care
  se amesteca uneori cu jurnalul (se vede si in ieșirea de pe Windows), deci
  numarul afisat varia (167/168). Numara acum din `test result:`.

- **2026-11-17** Pe Windows: `pe_windows_datele_nu_apar_in_clar` trece —
  DPAPI cripteaza de-adevaratelea. Prima conectare reala:
  `instantaneu --server TS2409\MIHAIG` -> `GM-DB-001 No such host is known`.
  **Regresie fata de Python**: numele intreg ajungea la DNS; ODBC rezolva
  singur instantele numite prin SQL Browser, iar feature-ul
  `sql-browser-tokio` era activat in `Cargo.toml` dar nefolosit. Corectat
  (`interpreteaza_server` + `connect_named`), acceptand si `GAZDA,PORT` ca
  Python-ul. Testat pe Linux cu un SQL Browser fals pe UDP; vazut cazand pe
  codul vechi cu exact simptomul de pe Windows (1433 in loc de Browser).
  Lectia: niciun test nu folosea un nume de server cu `\`, deși e forma cea
  mai comuna in productie — si chiar `servere` o produce.

- **2026-11-17** Instantaneul real merge: `TS2409\MIHAIG`, doua baze,
  42.427 obiecte, 26,5 s (~13 s/baza, secvential). **202 MB** — planul de a-l
  aduce in repo nu tine (limita GitHub de 100 MB, si e cod CharismaERP).
  Schimbat: `paritate.py` ruleaza pe masina lui Mihai, vine doar raportul.
  Pentru asta: raportul se scurteaza la 40 de cazuri pe categorie (complet in
  `--json`) si ieșirea e fortata UTF-8 (consola Windows). Probat pe un
  instantaneu sintetic de 45.000 obiecte: 2,6 s. Tot de aici: `tiberius`
  scria ~8 linii INFO la fiecare conexiune si acoperea progresul; filtrul
  implicit e acum `info,tiberius=warn`.

- **2026-11-17** **Paritate pe instantaneul real: 0 regresii.** Referinta
  `CharismaERP_GetsBet` (21.193 obiecte), tinta `_Copy`: 194 randuri in
  ambele, 21.042 identice, 149 diferite, 2 lipsa, 43 extra — categorie, tip si
  scor identice pe toate. 70 din cele 149 de rezumate difera de v3.4, toate 70
  confirmate de motorul JS: in v3.4 lista si editorul se contrazic pe ~47% din
  obiectele modificate; in Rust spun acelasi lucru. Inca nedovedit: ca
  `tiberius` aduce aceleasi obiecte ca `pyodbc` (ambele parti au rulat pe
  obiectele aduse de Rust) — se inchide cu linia finala a aplicatiei Python pe
  aceleasi baze, care da si timpul. De verificat si ca `motor_js.mjs` extras
  e acelasi cu ce ruleaza in aplicatie: doua obiecte deschise in editorul v3.4.
  Adaugat `referinta/extrage.py` pentru fixture-ul mic (`TESTSIMILITUDINE_*`).

- **2026-11-17** Mihai a cerut o singura comanda. `unelte/etapa6.ps1` ruleaza
  tot si scrie `etapa6-rezultat.txt`. Pasul nou: `referinta/paritate_pyodbc.py`
  ruleaza `Api._comparatie` cu **pyodbc real** — da timpul Python (4
  conexiuni) si compara obiect cu obiect (hash normalizat, text de afisare,
  tip) ce aduce pyodbc cu ce a adus tiberius. Mai puternic decat linia de
  stare din interfata, si fara interfata. Probat local cu pyodbc fals: trece pe
  date identice, prinde un obiect lipsa si un text alterat. **Scriptul .ps1
  NU a fost rulat niciodata**: aici nu exista PowerShell (instalarea lui a
  fost refuzata de permisiunile sesiunii). Prima rulare la Mihai e si proba.

- **2026-11-17** `etapa6.ps1` a mers din prima pe Windows. Rezultate: auth SQL
  si integrata Windows merg (`baze`); descoperirea gaseste 15 instante;
  **pyodbc si tiberius aduc aceleasi 42.427 obiecte**, identice ca hash, text
  si tip; paritatea pe instantaneul proaspat si pe fixture-ul
  `TESTSIMILITUDINE` (18+18 obiecte, 48,5 KB): 0 regresii. Timp: Rust citeste
  cele doua baze in 19 s secvential; Python 29,6 s cu 4 conexiuni (citire +
  comparatie, deci nu strict comparabil). Rust nu e mai lent, deci
  paralelismul amanat nu e necesar. Defect gasit: jurnalul punea coduri ANSI
  si in ieșire redirectata (fisierul de rezultat era plin de `[2m`); corectat
  cu `IsTerminal`, test pe binar vazut cazand.

- **2026-11-17** Fixture-ul `TESTSIMILITUDINE` (18+18 obiecte) e in repo,
  commitat de Mihai, si poarta ruleaza `paritate.py` pe el (18 randuri, 10
  corecturi, 0 regresii, la fel ca pe Windows); vazut cazand pe o regresie
  introdusa in `core`. **In fixture, campul `server` continea lungimea
  parolei** (`parola (N caractere, ascuns)`): `instantaneu` scria acolo
  descrierea pentru jurnal. Corectat in cod (`eticheta_server`), in
  `Redactat` (nu mai spune lungimea), si in fixture (doar acel camp).
- **2026-11-17** La cererea lui Mihai, istoricul branch-ului rescris de la
  `58fbc06`: commit-ul cu fixture-ul refacut cu campul curat (autor si data
  pastrate), mesajele si notele fara cifra, commit-ul de curatare disparut;
  continutul final identic. Impins cu `--force-with-lease`.

## Ce a ieșit altfel decat planul

**Motorul nu a avut nicio regresie; toate defectele au fost la granite.** 0
regresii pe 21.193 de obiecte reale. In schimb, la prima atingere a tintei
reale au iesit: instantele numite nu se puteau conecta deloc (`SRV\INST` mergea
la DNS), jurnalul afisat ar fi inghetat dupa 2000 de linii, coduri ANSI in
fisiere, lungimea parolei intr-un fisier commitat. Niciunul nu era in logica
testata cu conformitate si fuzz; toate erau in I/O, in mediul Windows, sau in
ce ajunge in fisiere. Testele unitare si de conformitate acopera o clasa de
bug; tinta reala gaseste alta, si nu se substituie una cu alta.

**Lista Python nu a trebuit exportata din interfata.** Planul cerea un CSV.
`Api._comparatie` s-a putut rula neatins, intai pe instantaneu (pyodbc fals),
apoi cu pyodbc real — mai strict decat CSV-ul (obiect cu obiect, nu doar
perechi), si fara interfata.

**Instantaneul real nu a putut veni incoace.** 202 MB si cod CharismaERP. Deci
uneltele au trebuit sa ruleze la Mihai (`etapa6.ps1`, nerulat de mine
niciodata, a mers din prima), iar in repo a intrat doar un extras.

**Doua teste treceau fara sa dovedeasca nimic.** `dus_intors` din DPAPI ar fi
trecut si fara criptare; numaratoarea de teste din poarta varia cu
intercalarea jurnalului. „Trece" nu e „dovedeste"; de aici regula de a vedea
fiecare verificare cazand, aplicata de opt ori in etapa asta.

**Instructiunile mele pentru Windows au fost primul lucru stricat.** `<repo>`
ca loc de completat (PowerShell il citeste ca redirectare), parola scrisa in
comanda (ar intra in istoric). O comanda data utilizatorului e cod si se
verifica la fel.

**Rust e mai rapid decat se presupunea**: 19 s fata de 29,6 s, fara
paralelism. Decizia de a-l amana a ramas valabila.
