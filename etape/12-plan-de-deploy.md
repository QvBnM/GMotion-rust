# Etapa 12 — planul de deploy

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

## Scop

Ce ai pus „in plan" devine un deploy pe care il poti citi inainte sa-l faci:
ordinea pasilor, verificarile, si confirmarea — dupa macheta
`cercetare/machete/Deploy.dc.html`. Scriptul exista deja (`diff-obiecte`);
etapa asta adauga ce ii lipseste ca sa fie sigur de apasat.

## Criterii de terminare

- [ ] **verificarea „baza n-a mai fost atinsa"**: inainte de deploy se recitesc
      obiectele din plan si se compara amprentele cu cele de la comparare;
      ce s-a schimbat intre timp se spune pe nume si opreste pasul
- [ ] **scriptul de revenire**, din definitiile actuale ale bazei tinta,
      salvat inainte de deploy
- [ ] amandoua accesibile din `cli` (CLAUDE.md §3), cu test pe binar
- [ ] ecranul in Slint, fidel machetei: taburile Ordinea / Atentionari /
      Script, propozitia de rezumat, pasii numerotati in ordinea
      dependentelor, verificarile (✓ / !), „salvează întâi versiunea actuală",
      si — doar pentru PROD cu DROP — confirmarea prin tastarea numelui bazei
- [ ] planul (ce intra, in ce ordine, ce atentionari) are teste fara fereastra
- [ ] captura PNG langa machetă
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

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
