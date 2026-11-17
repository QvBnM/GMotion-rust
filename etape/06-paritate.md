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
- [ ] instantaneul real (sau o forma anonimizata) e fixture de regresie in repo
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

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
