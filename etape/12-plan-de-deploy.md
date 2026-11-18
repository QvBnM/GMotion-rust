# Etapa 12 — planul de deploy

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-18

## Scop

Ce ai pus „in plan" devine un deploy pe care il poti citi inainte sa-l faci:
ordinea pasilor, verificarile, si confirmarea — dupa macheta
`cercetare/machete/Deploy.dc.html`. Scriptul exista deja (`diff-obiecte`);
etapa asta adauga ce ii lipseste ca sa fie sigur de apasat.

## Criterii de terminare

- [x] **verificarea „baza n-a mai fost atinsa"**: inainte de deploy se recitesc
      obiectele din plan si se compara amprentele cu cele de la comparare;
      ce s-a schimbat intre timp se spune pe nume si opreste pasul
- [x] **scriptul de revenire**, din definitiile actuale ale bazei tinta,
      salvat inainte de deploy
- [x] amandoua accesibile din `cli` (CLAUDE.md §3), cu test pe binar
- [ ] ecranul in Slint, fidel machetei: taburile Ordinea / Atentionari /
      Script, propozitia de rezumat, pasii numerotati in ordinea
      dependentelor, verificarile (✓ / !), „salveaza intai versiunea actuala",
      si — doar pentru PROD cu DROP — confirmarea prin tastarea numelui bazei
- [ ] planul (ce intra, in ce ordine, ce atentionari) are teste fara fereastra
- [ ] captura PNG langa macheta
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Verificarea amprentelor inainte de deploy, in `diff-obiecte`, cu teste.
2. Scriptul de revenire, tot acolo.
3. Amandoua in `cli`.
4. Modelul planului in `fereastra`, apoi ecranul si captura.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod. Scriptul
  de deploy si ordinea pe dependente exista din etapa 4 (`deploy.rs`); aici se
  adauga garda si revenirea.
- **2026-11-18** Pasii 1-3 facuti. `diff-obiecte/src/plan.rs`: `Plan` cu
  amprenta exacta a fiecarei parti (nu hash-ul normalizat — decizia in
  `JURNAL.md`), garda `verifica` / `GM-MOD-012`, si selectia „ce intra"
  mutata din `cli` in modul, ca fereastra s-o refoloseasca. Revenirea:
  `PasRevenire` + `scrie_cu_revenire`, in `<director>/revenire/`, verificata
  inainte de primul fisier. `cli`: `gmotion plan` si `deploy --plan`, cu trei
  teste pe binar. Ambele teste noi de ordine/amprenta vazute cazand cu codul
  stricat deliberat. Doua teste vechi din `cli` presupuneau ca directorul de
  deploy contine doar fisiere; ajustate sa numere fisierele.
  Ramas pentru pasul 4: **atentionarile** planului (DROP, `QUOTED_IDENTIFIER
  OFF`, cicluri) inca nu exista ca model — criteriul „planul are teste fara
  fereastra" e doar pe jumatate.

- **2026-11-18** Etapa pusa pe pauza dupa pasul 3 (decizia lui Mihai, in
  `JURNAL.md`): intai paritatea functionala cu v3.4. Ecranul de deploy intra
  in ordinea lipsurilor din `cercetare/lipsuri-portare.md`.

## Ce a iesit altfel decat planul

- **Ecranul nu s-a facut in etapa asta.** Dupa pasul 3, Mihai a schimbat
  ordinea (intai paritatea cu v3.4) si apoi designul intreg. Criteriile
  neterminate (ecranul, atentionarile planului ca model, captura) nu sunt
  bifate: ecranul de deploy se face in noul aspect, dupa etapa 13; atentionarile
  raman in `cercetare/lipsuri-portare.md`, sectiunea 5.
- Ce s-a livrat (garda, revenirea, `gmotion plan`, `deploy --plan`) e complet
  si testat pe binar.
