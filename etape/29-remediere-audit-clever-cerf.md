# Etapa 29 — remedierea auditului de pe clever-cerf

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-23 · **Incheiata:** —

## Scop

Auditul din `cercetare/audit-qa-2026-11-23-clever-cerf.md` a dat 5/10 pe
`b815d0e`. La final: remedierile din `claude/dreamy-ritchie-jdnfct` sunt unite
aici, iar constatarile care raman dupa unire sunt reparate, pana la cosmetic.

## Criterii de terminare

- [x] `claude/dreamy-ritchie-jdnfct` unit, cu poarta verde si cu coliziunile
      rezolvate explicit (`GM-DB-012`, B4 vs F-A9)
- [ ] CRITIC: C-1 (coloana „Obiect"), C-2 (panoul de detalii), D-1 (poarta isi
      rescrie fixture-ul)
- [ ] MARE: B-4 (fals „modificat"), B-5 (garda de cost pe `plasari`), C-M2
      (etichetele blocurilor netraduse), D-2 („core nu are I/O")
- [ ] MEDIU: A-3, A-4, A-6, A-7, A-8, B-6, B-7, C-D2, C-D5, C-D6, C-D7, D-3,
      D-4, D-5, D-8, D-10
- [ ] MIC si COSMETIC: restul listei din raport
- [ ] fiecare verificare noua din poarta a fost vazuta **cazand**, o data
      (CLAUDE.md §10)
- [ ] capturi noi la 1024x768 si 1366x768, in ambele teme, pentru lista si
      panoul de detalii; puse langa macheta
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat cu deciziile luate la unire si la remediere

## Pasi

1. Unirea cu `dreamy-ritchie`, conflict cu conflict. **Facut.**
2. Poarta: fixture-ul de referinta verificat, nu rescris; „core fara I/O"
   verificat pe text; „exact o etapa ACTIVA"; callback cu `_`.
3. Motorul: `RE_POATE_CONTINUA` cu `create`/`alter`; garda de cost pe
   `plasari`; unitatile din `marcaje_pe_linii`; ordinea plasarilor.
4. Deploy si sesiune: amprenta exacta la bifa si la textul final; textul final
   egal cu tinta nu intra in script; oprirea intre fisiere; `salvata` scrisa si
   din fereastra.
5. Interfata: latimile coloanelor si panoul de detalii, cu capturi; etichetele
   blocurilor prin `limba.alege`; jurnal la „Aplica diferenta".
6. `cli`: `--json` da JSON si pe eroare; `export` / `instantaneu` scriu ceva;
   `verifica` accesibila din linia de comanda.
7. Mici si cosmetice, in ordinea din raport.

## Jurnal de lucru

- **2026-11-23** Unire cu `dreamy-ritchie` (27 de fisiere cu conflicte). Doua
  coliziuni reale, nu de text: `GM-DB-012` folosit in paralel pentru doua erori
  diferite (a doua a trecut pe `GM-DB-014`), si B4 vs F-A9, care se contraziceau
  pe fond — castiga F-A9, garda B4 ramane pentru obiectele fara niciun text.
  Poarta verde: 749 de teste.
- **2026-11-23** Poarta: harness-ul nu mai rescrie fixture-ul, il compara;
  „core fara I/O" verificat pe text, fiindca `std::fs` compileaza pe wasm32;
  „exact o etapa ACTIVA" cade si la zero. Fiecare, vazuta cazand.

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
