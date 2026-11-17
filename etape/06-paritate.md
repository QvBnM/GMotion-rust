# Etapa 6 — paritate pe baze reale

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

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

- [ ] `cargo test --all` trece pe Windows (singura proba reala pentru DPAPI:
      testele din `platforma/src/protejare.rs` fac ciclul cu implementarea
      tintei; `diagnostic` doar raporteaza daca e activ)
- [ ] `gmotion diagnostic`, `baze` si `instantaneu` merg pe masina lui Mihai
      (TLS SChannel, autentificare SQL si integrata)
- [ ] pe instantaneul real: zero diferente clasificate `regresie`
- [ ] lista exportata de aplicatia Python pe aceleasi baze (CSV) are aceleasi
      perechi (obiect, baza, categorie) ca Python-ul rulat pe instantaneu —
      dovada ca `tiberius` aduce aceleasi obiecte ca `pyodbc`
- [ ] timpul lui `gmotion instantaneu` pe bazele reale e masurat si comparat
      cu incarcarea din aplicatia Python (citirea Rust e secventiala, Python
      foloseste 4 conexiuni — vezi `JURNAL.md`, „Citirea bazelor rămâne
      secventiala"); daca e inacceptabil, paralelismul intra in `db`
- [ ] obiectele relevante din instantaneul real (extrase, anonimizate daca
      trebuie) sunt fixture de regresie in repo — instantaneul intreg are 202 MB
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

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

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
