# Etapa 13 — aspectul nou, in doua moduri

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-18 · **Incheiata:** —

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
- [ ] modul se alege la pornire (`gmotion-ui --tema intunecat|luminos`) si se
      comuta din fereastra; alegerea se tine minte intre porniri
- [ ] conectarea, dupa schita: jurnalul ca banda jos, cautarea ca iconita;
      captura in ambele moduri
- [ ] lista, dupa schita: harta pictata in Rust, legata de tabel, cu
      coloanele existente; captura in ambele moduri
- [ ] editorul, dupa schita: doua panouri (sursa doar citire, baza), canalul
      dintre ele; captura in ambele moduri
- [ ] **niciun buton nou fara actiune**: ce din schita nu are inca logica
      (functiile din `lipsuri-portare.md`) nu se deseneaza pana nu o are
- [ ] testele de flux din `app` trec neschimbate in ce verifica
- [ ] `CRATE.md` din `fereastra` actualizat (API, invariante)
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

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

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
