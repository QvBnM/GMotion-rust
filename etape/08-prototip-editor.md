# Etapa 8 — prototipul editorului de diferente, nativ

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

## Scop

Decizia lui Mihai (2026-11-17): interfata se rescrie **nativ**, fara WebView2,
dar „afisajul sa fie oarecum la fel" ca v3.4; si se incepe acum, inaintea
modulului `upgrade`. Etapa asta face doar partea grea — editorul de
diferente — ca sa se afle costul real inainte de restul interfetei.

## Criterii de terminare

- [ ] toolkit ales, cu decizia in `JURNAL.md` (alternative, licenta,
      randare software pentru RDP)
- [ ] capturi de referinta ale v3.4 (`referinta/captura_v34/`), produse din
      codul vechi neatins, pentru conectare, lista si editor
- [ ] crate `rust/fereastra`: editorul side-by-side ca in v3.4 — blocuri
      aliniate pe aceeasi inaltime in ambele panouri, numere de linie reale,
      evidentiere de sintaxa (cuvinte cheie, variabile, numere, string-uri,
      comentarii), blocul modificat cu fundal si bara, bucatile schimbate din
      linie marcate, antetele SURSA / BAZA, bara de unelte, minimap
- [ ] un singur motor: modelul editorului vine din `gmotion_core`
      (`diff_aliniat`), fara a doua implementare
- [ ] captura editorului nativ (randat fara ecran, PNG) pe acelasi obiect ca
      referinta v3.4, comparata vizual si pusa langa ea in etapa
- [ ] navigare intre diferente (anterior / urmator) si derulare pe obiecte
      mari (virtualizare: un obiect de 20.000 de linii nu blocheaza)
- [ ] modelul editorului (aliniere, numerotare, segmente) are teste
- [ ] poarta verde, inclusiv `cargo check` pe windows-msvc
- [ ] `STARE.md`, `JURNAL.md`

In afara etapei (urmatoarea, interfata completa): ecranele de conectare si
lista, temele (Dracula, Nord), editarea textului final, modul ghidat,
deploy-ul din interfata, sesiunile.

## Pasi

1. Capturi v3.4 si decizia de toolkit.
2. Modelul editorului in Rust, cu teste.
3. Componenta Slint si captura fara ecran.
4. Comparatie, ajustari, navigare, virtualizare.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa. Capturile v3.4 se fac cu
  `referinta/captura_v34/` (Api-ul Python vechi pe fixture-ul
  `TESTSIMILITUDINE`, pagina reala `ui.PAGINA` in Chromium headless). Slint
  probat in scratchpad: compileaza si randeaza fara ecran in PNG.

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
