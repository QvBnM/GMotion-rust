# Etapa 0 — Restructurare + reguli

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Un repo in care doi oameni — unul dintre ei un asistent care porneste de la zero
la fiecare sesiune — pot construi, citi si depana fara sa-si strice munca
reciproc. Fara cod nou de produs: doar structura, reguli si verificare.

## Criterii de terminare

- [x] Python-ul v3.4 mutat in `mostenire/`, inghetat, documentat
- [x] Harness-ul de conformitate scos din `rust/` in `referinta/`
- [x] Caile rupte de mutare reparate, fixture-urile ies identice
- [x] `CLAUDE.md`, `ARHITECTURA.md`, `HARTA.md`, `JURNAL.md` scrise
- [x] `rust/core/CRATE.md` — sablonul de contract per crate
- [x] `unelte/verifica.sh` ruleaza si raporteaza tot intr-o trecere
- [x] poarta verifica tinta wasm32, directia dependentelor, sincronizarea `CRATE.md`
- [x] **verificat ca poarta prinde incalcari**, nu doar ca trece
- [x] `README.md` rescris: structura noua, plan in zece etape
- [x] poarta verde
- [x] `JURNAL.md` completat cu cele patru decizii de arhitectura

## Pasi

1. mutari cu `git mv`, intr-un commit separat de orice logica
2. reparat caile: `conformitate.rs`, `bench.rs`, `genereaza.py`, `bench.py`, `.gitignore`
3. documentele de regula
4. poarta + `api-hash.sh` + `regenereaza-fixture.sh` + `verifica.bat`
5. testat ca poarta cade pe incalcari reale
6. `README.md`

## Jurnal de lucru

- **2026-11-17** Baseline inainte de orice mutare: 31 teste verzi. Important,
  fiindca altfel n-ai cum sa stii daca o regresie vine din mutare.
- **2026-11-17** Decis: CLI headless inainte de interfata, Python inghetat in
  `mostenire/`, `SursaObiecte` ca trait. Motivele in `JURNAL.md`.
- **2026-11-17** Instalat `wasm32-unknown-unknown`, ca verificarea de wasm din
  poarta sa ruleze real in loc sa fie raportata ca sarita. `core` compileaza.
- **2026-11-17** Fixture-urile regenerate dupa mutare au iesit identice — apar
  ca simple redenumiri in git. Dovada ca harness-ul se comporta la fel.

## Ce a iesit altfel decat planul

**Prima versiune a verificarii `mostenire/` nu prindea nimic.** Folosea
`git diff HEAD -- mostenire/`, care raporta drept „modificare" si simpla mutare
a fisierelor — deci commit-ul de restructurare cadea pe sine. Restrans la
`--diff-filter=M` (doar continut modificat). Dar asta inseamna ca verificarea nu
putea fi testata inainte de commit, fiindca fisierele erau inca la vechea cale
in `HEAD`; a trebuit retestata dupa.

Lectia, generalizabila: **o verificare care n-a fost vazuta cazand nu e o
verificare.** De-aia a intrat in criteriile de terminare un punct separat
pentru „poarta prinde incalcari", nu doar „poarta trece". Ambele verificari
netriviale (`api-hash` si `mostenire/`) au fost puse la proba cu o incalcare
deliberata.

**Poarta nu e completa pe `cmd`.** Verificarile de directie a dependentelor si
de sincronizare `CRATE.md` sunt scrise in bash si Python, deci pe Windows cer
Git Bash sau WSL. `verifica.bat` ruleaza doar fmt, clippy, teste si harness-ul.
Notat ca intrebare deschisa in `STARE.md`.
