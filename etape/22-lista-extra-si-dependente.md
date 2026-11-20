# Etapa 22 — lista: Extra, dependentele si impactul

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-20 · **Incheiata:** —

## Scop

In lista se vede, fara sa deschizi nimic, ce obiecte sunt „Extra" (doar in
Modificata, nu se deployeaza, ca in v3.4), cine foloseste fiecare obiect si
cat de mare e impactul deploy-ului; in panoul de detalii, **care** obiecte il
folosesc si pe care le foloseste el, cu clic spre ele.

Cerut de Mihai, 2026-11-20. Alegerile lui (intrebate inainte):
jetoanele raman, „Doar in modificata" devine „Extra" si randul spune
„Extra — nu se deployeaza"; dependentele in panou **si** coloana „Folosit
de"; impactul ca coloana sortabila **si** filtru.

## Criterii de terminare

Fiecare: logica in afara ferestrei, cu test; ce se apasa, cu test care il
apasa; accesibil din `cli` (CLAUDE.md §3).

- [x] jetonul „Doar in modificata" se numeste „Extra"; pe randurile Extra,
      coloana Deploy spune „Extra · fara deploy" (in panou, „Extra — nu se
      deployeaza"; ambele limbi). Textul scurt in coloana: vezi jurnalul
- [x] coloana „Folosit de" (numarul de obiecte din Modificata care il
      folosesc), sortabila in ambele sensuri
- [x] coloana „Impact" (Scazut / Mediu / Ridicat), sortabila; aceeasi
      estimare ca in panou (`modul::impact::estimeaza`), nu a doua
- [x] filtrul pe impact (toate / Ridicat / Mediu / Scazut), combinat cu
      celelalte filtre si cu contoarele
- [x] panoul de detalii: listele „Folosit de" si „Foloseste", cu numele
      obiectelor; clic pe un nume il selecteaza in lista (daca e in lista)
- [x] „Foloseste" vine din acelasi lexer ca ordinea de deploy
      (`deploy::referinte`), nu dintr-o a doua cautare de nume
- [x] `gmotion impact` arata si ce foloseste obiectul (text si `--json`),
      probat rulandu-l
- [x] costul: calculul dependentelor pe tot instantaneul de previzualizare
      masurat si scris in jurnalul etapei
- [ ] captura listei in ambele teme, cu panoul deschis, si „da"-ul lui Mihai
      — capturile: `cercetare/capturi/lista-dependente-*.png`; **asteapta „da"-ul**
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. motorul: `impact::foloseste`, apoi `Obiect` din lista primeste
   `folosit_de` si `impact`; sortarea si filtrul in `model_lista`;
2. ecranul: Extra, cele doua coloane, filtrul, panoul cu cele doua liste;
3. `cli`, capturi, documente.

## Jurnal de lucru

- **2026-11-20** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-20** Facut tot, in afara de „da"-ul pe capturi. Ce a iesit pe
  drum:
  - impactul se calcula lenes, la deschiderea panoului; o coloana in lista
    il cere pe fiecare rand, deci dependentele se calculeaza acum o data,
    la construirea listei. Masurat pe 5.000 de proceduri de ~60 de linii:
    `gmotion impact` 10,97 s fata de `gmotion compara` 10,57 s, deci ~0,4 s
    pe baza (aplicatia face Martor + fiecare baza comparata);
  - „Folosit de" in panou venea prin impact, iar un obiect identic n-are
    impact: panoul arata „niciun obiect" desi era folosit. Prins de test;
  - „Extra — nu se deployeaza" nu incapea in coloana Deploy cu panoul
    deschis, iar coloana Obiect ramanea fara loc; in coloana e acum
    „Extra · fara deploy", in panou textul intreg;
  - cu panoul deschis, numele lungi intrau peste coloana Tip (vechi, dar
    agravat de cele doua coloane noi): acum se taie cu „…".

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
