# Etapa 9 — lista de diferente si previzualizarea

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

## Scop

Ecranul principal din `cercetare/machete/Main.dc.html`: lista obiectelor care
difera, cu bara laterala de filtre si previzualizarea obiectului selectat.
La final se poate parcurge o comparare intreaga fara linie de comanda — se
vede ce difera, in ce anume difera, si cum arata diferenta — pe o lista de
50.000 de obiecte, fara sa se blocheze.

## Criterii de terminare

- [ ] **„Difera in" vine din motor, nu din interfata:** clasificarea
      diferentelor unui obiect pe `cod` / `sir` / `comentariu` / `setari SET`,
      facuta prin `core::lexer` (granitele se decid doar acolo), cu teste pe
      cazurile care au pacalit v3.4 (majuscula intr-un sir, comentariu
      schimbat, `QUOTED_IDENTIFIER`)
- [ ] clasificarea e accesibila si din `cli` (CLAUDE.md §3), cu test pe binar
- [ ] modelul listei in Rust, fara fereastra, cu teste: filtrare pe categorie
      si tip, cautare, sortare pe linii, contoarele din bara laterala,
      progresul revizuirii, selectia
- [ ] ecranul in Slint, fidel machetei: antet (sursa → baza, mediul, cautare,
      Recompara, Deploy cu numarul din plan), bara laterala de 200 px,
      tabelul cu litera de stare / obiect / tip / `+N −M` / „Difera in" / bifa,
      randul selectat plin `selectie`, previzualizarea dedesubt
- [ ] fara comutatoare de zgomot, nicaieri (decizia lui Mihai)
- [ ] jetoanele noi (daca apar) trec testul de contrast ≥ 4,5:1
- [ ] **lista virtualizata, masurata pe 50.000 de obiecte**: incarcare, cadru,
      derulare, filtrare — si **verificarea vazuta cazand** (fara
      virtualizare, cifrele trebuie sa sara)
- [ ] captura PNG langa macheta, in `cercetare/machete/capturi/`
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Clasificarea „difera in" in `core` (prin `lexer`), cu teste; expusa in
   `diff-obiecte` si in `cli`.
2. Modelul listei (filtre, contoare, selectie) in `fereastra`, cu teste.
3. Ecranul in Slint, pe jetoanele existente.
4. Previzualizarea: aceeasi constructie de randuri ca editorul (un singur
   motor), cu identicul pliat.
5. Masuratoarea pe 50.000 de obiecte si capturile.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod. Vine
  dupa etapa 8 (editorul), de unde se refolosesc: jetoanele, fonturile,
  plierea, constructia randurilor de diferente.

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
