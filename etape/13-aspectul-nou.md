# Etapa 13 — aspectul nou, in doua moduri

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-18 · **Incheiata:** 2026-11-18

## Scop

Ferestrele din `rust/fereastra` arata ca schita aprobata
(`cercetare/schita-slint/`): conectarea, lista (harta + tabelul din v3.4) si
editorul (sursa si baza in doua panouri), cu **aceeasi paleta pe toate
ecranele, in mod intunecat si luminos**, comutabile. Functiile care merg azi
merg si dupa; nicio functie nu dispare.

## Criterii de terminare

- [x] jetoanele au doua palete (`INTUNECAT`, `LUMINOS`) cu **aceleasi nume**;
      testul de contrast trece pe amandoua (4,5:1, abaterile notate)
- [x] fonturile Geist si Geist Mono, incluse in binar, cu licenta OFL alaturi
- [x] modul se alege la pornire (`gmotion-ui --tema intunecat|luminos`) si se
      comuta din fereastra; alegerea se tine minte intre porniri
- [x] conectarea, dupa schita: jurnalul ca banda jos, cautarea ca iconita;
      captura in ambele moduri
- [x] lista, dupa schita: harta pictata in Rust, legata de tabel, cu
      coloanele existente; captura in ambele moduri
- [x] editorul, dupa schita: doua panouri (sursa doar citire, baza), canalul
      dintre ele; captura in ambele moduri
- [x] **niciun buton nou fara actiune**: ce din schita nu are inca logica
      (functiile din `lipsuri-portare.md`) nu se deseneaza pana nu o are
- [x] testele de flux din `app` trec neschimbate in ce verifica
- [x] `CRATE.md` din `fereastra` actualizat (API, invariante)
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Jetoanele in doua palete, testul de contrast pe amandoua, fonturile Geist,
   `aplica_tema(ui, Mod)`. Modul implicit ramane luminos pana sunt rescrise
   toate ecranele (cele vechi n-au fost gandite pe fond inchis).
2. Conectarea.
3. Lista.
4. Editorul.
5. Comutatorul de mod in fereastra si tinerea lui minte; implicitul devine
   cel ales de Mihai.
6. Capturile in ambele moduri, langa schita.

## Jurnal de lucru

- **2026-11-18** Etapa deschisa la cererea lui Mihai („poti incepe sa rescrii
  designul"), dupa ce a aprobat schita si a cerut aceeasi paleta peste tot, ca
  mod intunecat si luminos. Etapa 12 s-a inchis cu ecranul de deploy mutat
  aici, ca ecran viitor in acelasi stil.

- **2026-11-18** Pasul 1 facut. `jetoane::Paleta` (45 de jetoane) cu
  `INTUNECAT` / `LUMINOS`; `aplica_tema(ui, Mod)`, `mod_curent`,
  `tigla_hasura(Mod)` (hasura urmeaza modul); `gmotion-ui --tema`. Lista
  jetoanelor din `lib.rs` e legata de `Paleta` prin destructurare fara `..`:
  vazuta cazand cu o linie scoasa. Testul de contrast vazut cazand pe
  `INTUNECAT` cu `text_tert` mai deschis (4,38:1 pe strat). Un test a prins
  un bug real: `din_nume(" Intunecat ")` nu recunostea „I" (`to_ascii_lowercase`).
  IBM Plex scos din binar.

- **2026-11-18** Pasii 2-6 facuti, plus ecranul de deploy. Cerute de Mihai pe
  drum: buton dedicat pentru tema (pe toate cele patru ecrane), fara
  diacritice nicaieri (poarta verifica), etapa dusa pana la capat cu deploy-ul
  ales de mine. Conectarea: carduri sursa / baza, cautarea serverelor ca
  iconita (legata la `db::cauta_instante`), banda de jurnal din
  `platforma::jurnal`. Lista: harta pictata in Rust si legata de tabel, filtrele
  pe stari in antet. Editorul: doua panouri, insule. Deploy: pasii in ordinea
  din script, garda, revenirea, impartirea, scrierea. Butoanele moarte din
  `lipsuri-portare.md` (sectiunea 0) au acum actiune. Testele au prins trei
  bug-uri reale: prefixul `[GM-...]` luat drept nivel de jurnal, I-caciula majuscul nerecunoscut
  la `--tema`, si (la diacritice) doua teste care ar fi incetat sa verifice
  ceva. Calculul „ce patrat e sub mouse" era scris de doua ori (Rust si Slint);
  mutat doar in `harta::indice_la`.

## Ce a iesit altfel decat planul

- **Etapa a crescut cu un ecran intreg.** Planul era rescrierea a trei ecrane;
  Mihai a cerut si deploy-ul, ramas din etapa 12. A cerut logica pe care o
  aveau deja `diff-obiecte` si `cli` (plan, garda, revenire), deci s-a facut
  in `app::deploy` peste aceleasi functii, nu de la zero.
- **Doua cereri noi la mijloc: butonul de tema si fara diacritice.** Prima a
  adus tema in configurare (`Configurare::tema`). A doua a atins 87 de
  fisiere si a cerut o exceptie marcata pentru datele de test care verifica
  nume cu diacritice ale utilizatorului: fara ea, doua teste ar fi trecut
  verificand altceva.
- **Regula „niciun buton fara actiune" a scos butoane din schita.** Aplica
  toate, Reset, Mod ghidat, Editeaza textul final, Ascunde diferentele,
  Re-sincronizeaza, Uneste cursorul nu se deseneaza: logica lor e inca lipsa
  (`lipsuri-portare.md`, sectiunea 4). Schita le arata; aplicatia le va arata
  cand fac ceva.
- **Recompara si cautarea serverelor au cerut logica, nu doar desen.** Erau
  butoane moarte sau lipsa; s-au legat la ce exista deja (`pornire::compara`,
  `db::cauta_instante`).
