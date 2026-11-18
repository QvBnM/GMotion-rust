# Etapa 5 — `cli`

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

**O aplicatie completa si utilizabila**, fara interfata grafica. Si modul in
care orice bug devine reproductibil pe orice masina.

## Criterii de terminare

- [x] opt comenzi: `diagnostic`, `servere`, `baze`, `instantaneu`, `compara`,
      `diferente`, `deploy`, `export`
- [x] fiecare comanda care citeste obiecte merge si cu `--sursa <instantaneu>`
- [x] parola **nu** se accepta ca argument; test care cade daca cineva o adauga
- [x] `--json` scoate numai JSON la iesirea standard
- [x] esecul da cod de iesire nenul, cu codul original in mesaj
- [x] teste pe binarul adevarat, nu pe functii interne
- [x] `CRATE.md`
- [x] poarta verde

## Pasi

1. `acreditari` — de unde vine parola
2. `sursa` — server sau instantaneu, intr-un singur loc
3. `afisare` — text si JSON
4. `comenzi` — implementarea
5. `main` — argumente si coduri de iesire
6. probe manuale, apoi aceleasi probe ca teste

## Jurnal de lucru

- **2026-11-17** Binarul probat de mana pe un instantaneu construit pentru
  proba: baze, compara, diferente, deploy (toate patru modurile), export,
  coduri de iesire. Abia apoi scrise ca teste.
- **2026-11-17** 27 de teste in `cli`, din care 10 pe binarul adevarat.

## Ce a iesit altfel decat planul

**`--json` era rupt, si am aflat doar fiindca am rulat binarul.** Jurnalul
`tracing` scria la iesirea **standard** (implicitul lui `fmt::layer()`), deci
`gmotion compara --json` producea o iesire care nu se putea parsa. Toate testele
de unitate treceau, fiindca niciunul nu se uita la iesirea unui proces.
Corectat in `platforma/src/jurnal.rs` (`with_writer(stderr)`) si acoperit acum
de un test care parseaza iesirea binarului.

Lectia: **un binar se probeaza rulandu-l.** Compilarea si testele de unitate nu
spun nimic despre ce ajunge pe iesirea standard.

**Traitul `SursaObiecte` nu era compatibil cu `dyn`, si s-a vazut abia aici.**
`obiecte_multe` avea `progres: impl FnMut`, iar un parametru generic intr-o
metoda face traitul inutilizabil ca `Box<dyn SursaObiecte>` — exact cum CLI-ul
alege intre server si instantaneu la rulare. Schimbat in `&mut dyn FnMut` **in
`db`**, nu ocolit in CLI, plus un test in `db` care verifica proprietatea.

Asta arata ceva despre ordinea etapelor: defectul era in `db` de la etapa 2 si
nu putea fi observat pana la primul consumator care avea nevoie de el la
rulare. Un argument in plus pentru CLI inainte de interfata — daca primul
consumator ar fi fost interfata, s-ar fi descoperit mult mai tarziu.

**A treia oara cu aceeasi greseala: numere gresite in expectativele mele de
test.** La etapa 4 numaratoarea de obiecte identice, aici de doua ori (o data
numarul, o data locul in care iese progresul). De fiecare data codul era corect
si testul gresit. Tipar clar: **intr-un test cu date construite de mana, cifra
asteptata e cel putin la fel de usor de gresit ca si codul.** Cand un test cade
pe un numar, prima ipoteza e ca numarul din test e gresit — nu codul. Pus in
`CLAUDE.md`.