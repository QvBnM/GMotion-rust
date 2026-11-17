# Etapa 8 — prototipul editorului de diferente, nativ

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Decizia lui Mihai (2026-11-17): interfata se rescrie **nativ**, fara WebView2,
dar „afisajul sa fie oarecum la fel" ca v3.4; si se incepe acum, inaintea
modulului `upgrade`. Etapa asta face doar partea grea — editorul de
diferente — ca sa se afle costul real inainte de restul interfetei.

## Criterii de terminare

- [x] toolkit ales, cu decizia in `JURNAL.md` (alternative, licenta,
      randare software pentru RDP)
- [x] capturi de referinta ale v3.4 (`referinta/captura_v34/`), produse din
      codul vechi neatins, pentru conectare, lista si editor
- [x] crate `rust/fereastra`: editorul side-by-side ca in v3.4 — blocuri
      aliniate pe aceeasi inaltime in ambele panouri, numere de linie reale,
      evidentiere de sintaxa (cuvinte cheie, variabile, numere, string-uri,
      comentarii), blocul modificat cu fundal si bara, bucatile schimbate din
      linie marcate, antetele SURSA / BAZA, bara de unelte, minimap
- [x] un singur motor: modelul editorului vine din `gmotion_core`
      (`diff_aliniat`), fara a doua implementare
- [x] captura editorului nativ (randat fara ecran, PNG) pe acelasi obiect ca
      referinta v3.4, comparata vizual si pusa langa ea in etapa
- [x] navigare intre diferente (anterior / urmator) si derulare pe obiecte
      mari (virtualizare: un obiect de 20.000 de linii nu blocheaza)
- [x] modelul editorului (aliniere, numerotare, segmente) are teste
- [x] poarta verde, inclusiv `cargo check` pe windows-msvc
- [x] `STARE.md`, `JURNAL.md`
- [x] jetoanele si fonturile aprobate in `editor.slint`, cu contrastul
      verificat ca test (≥ 4,5:1), nu ca impresie

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
- **2026-11-17** Cercetare de design si simulare in doua runde; Mihai a
  aprobat varianta a doua (`cercetare/design-aprobat.md`). Etapa se termina pe
  designul aprobat, nu pe aspectul v3.4: jetoanele si fonturile din
  `design-aprobat.md` inlocuiesc culorile v3.4 din `editor.slint`; captura v3.4
  ramane referinta de continut (aceleasi informatii), nu de aspect.

- **2026-11-17** Etapa inchisa pe designul aprobat. Ce s-a facut in ultima
  sesiune: jetoanele si fonturile IBM Plex in locul culorilor v3.4 (jetoanele
  stau in `src/jetoane.rs`, nu in `.slint`), plierea sirurilor identice, modul
  unificat, navigarea de la tastatura (`n p j k u esc`), harta diferentelor,
  hasura pe partea fara linii, antetul si bara de jos din macheta.
  Capturi: `cercetare/machete/capturi/`.

  **Masurat** (`--example masura`, Linux, randare software, release):

  | randuri | model | pliat | afisate | incarca | cadru 1 | cadru derulat |
  |---|---|---|---|---|---|---|
  | 304 | 13 ms | da | 9 | 0,0 ms | 10 ms | 3 ms |
  | 304 | 6 ms | nu | 304 | 0,4 ms | 13 ms | 6 ms |
  | 3.004 | 72 ms | nu | 3.004 | 1,8 ms | 11 ms | 10 ms |
  | 30.004 | 3.140 ms | da | 161 | 1,3 ms | 27 ms | 16 ms |
  | 30.004 | 3.264 ms | nu | 30.004 | 25,8 ms | 12 ms | 10 ms |

  Deci: un obiect de 20.000 de linii (30.004 randuri) se deruleaza la ~10 ms
  pe cadru, cat unul de 300. Ce creste e **construirea modelului**: ~3,1 s,
  o singura data, la deschiderea obiectului (performanta patratica pe obiecte
  uriase, deja notata ca ramasa deschisa).

  **Verificarea a fost vazuta cazand:** scos `ListView`-ul (inlocuit cu
  `Flickable` + `VerticalLayout`), acelasi caz de 30.004 randuri nepliate sare
  la **711 ms** primul cadru si **233 ms** pe cadru derulat. Cu plierea
  pornita insa nu se vede nimic (161 de randuri afisate) — de-aia masuratoarea
  ruleaza si nepliat. La fel pentru fonturi: cu `--font-mono UnNumeInexistent`
  iese alt PNG, deci Plex-ul incorporat chiar se foloseste.

## Ce a ieșit altfel decat planul

1. **Etapa s-a terminat pe alt aspect decat cel cu care a inceput.** Planul
   initial era „ca in v3.4"; pe parcurs Mihai a cerut cercetare de design si a
   aprobat alt aspect. Captura v3.4 a ramas referinta de **continut** (aceleasi
   informatii despre aceleasi diferente), nu de aspect. Comparatia:
   `cercetare/machete/capturi/`.

2. **Criteriul de contrast a schimbat designul, nu l-a confirmat.** Scris ca
   test peste fiecare pereche text/fundal, a cazut pe doua jetoane aprobate.
   Asta e exact ce trebuia sa faca; valorile noi si abaterea acceptata sunt in
   `JURNAL.md`. Lectie: un criteriu „se verifica contrastul" nu e o formalitate
   de bifat la final — se scrie ca test, si poate cere o decizie.

3. **Prima masuratoare de virtualizare nu masura nimic.** Plierea era pornita,
   deci in lista ajungeau 161 de randuri in loc de 30.004, si timpul pe cadru
   iesea plat si cu `ListView`, si fara el. Doar dupa ce a rulat si nepliat
   s-a vazut diferenta de la 12 ms la 711 ms. O masuratoare care nu cade cand
   strici lucrul masurat nu masoara lucrul acela.

4. **Macheta si motorul numesc invers aceleasi blocuri** (verde = doar in
   sursa, pentru utilizator; `Sters` = doar in referinta, pentru motor).
   Incrucisarea sta intr-un singur loc, cu test. Vezi `JURNAL.md`.

5. **Au ramas doua lucruri de intrebat**, fiindca nu sunt in machete:
   cum se marcheaza in modul **unificat** care linie e din sursa si care din
   baza (macheta are tabul, nu si randarea), si daca „47 linii" din macheta
   ramane asa sau devine „47 de linii" (in cod e forma corecta gramatical,
   ca la „22 de linii identice" din aceeasi macheta).
