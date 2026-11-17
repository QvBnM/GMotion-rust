# Etapa 1 — `core` consolidat

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

`gmotion-core` rămâne functional identic, dar devine depanabil: fiecare fisier
spune ce categorie de bug ii aparține, si zona cu istoric de bug-uri —
segmentarea — capata fuzzing pe invariantele ei.

**Nicio schimbare de semantica.** Daca testul de conformitate se mișca, ceva e
greșit. E o etapa ieftina, aleasa inaintea celor scumpe fiindca ridica
increderea exact acolo unde istoricul arata ca e nevoie.

## Criterii de terminare

- [x] fiecare fisier din `rust/core/src/` are antet de 3-5 linii: rol +
      categoria de bug care ii aparține (regula 4 din `CLAUDE.md`)
- [x] fuzz pe `split_statements`, care verifica invariantele 4 si 5 din
      `rust/core/CRATE.md`:
      nu taie in interiorul unui literal / comentariu bloc / paranteze /
      stiva `CASE`-`BEGIN`-`END`; concatenarea blocurilor acopera exact textul
      de intrare, fara pierdere si fara dublare
- [x] fuzz pe idempotenta lui `norm` (invariantul 3) si pe
      `scor_similaritate(x, x) == 100.0` (invariantul 2)
- [x] fuzz-ul ruleaza si in poarta, cu un buget de timp fix (cateva secunde),
      nu doar manual
- [x] testul de conformitate **neschimbat** si verde — fixture-ul nu se atinge
- [x] `rust/core/CRATE.md` actualizat: secțiunea `Testare` nu mai spune „nu
      exista inca fuzzing"
- [x] poarta verde
- [x] `STARE.md` actualizat

## Pasi

1. antetele de fisier — `normalizare.rs`, `segmentare.rs`, `diff.rs`,
   `seqmatch.rs`. Scurt: ce face, si cum se vede cand se strica.
2. generator de T-SQL pentru fuzz. Nu SQL aleator: forme care **pun la
   incercare** invariantele — literale cu ghilimele duplicate, comentariu bloc
   neterminat, `CASE` imbricate, paranteze adanci, `UNION`, cursor `FOR`,
   `INSERT ... SELECT`. `referinta/genereaza_corpus.py` are deja un generator;
   se poate porni de la formele lui.
3. testele de invariante.
4. integrare in poarta, cu buget de timp.
5. actualizat `CRATE.md` + `STARE.md`.

## Jurnal de lucru

- **2026-11-17** Antete puse in toate cele patru module, fiecare cu categoria
  de bug care ii aparține. Reparate si caile `rust/referinta/` rămase in
  comentarii dupa mutarea din etapa 0.
- **2026-11-17** Fuzz fara dependinta noua: xorshift64 scris in test.
  `proptest` ar fi insemnat o dependinta intr-un crate care trebuie sa rămână
  minim, fiindca merge si pe wasm. 400 de seed-uri per test, ~1.5 s in total.
- **2026-11-17** Fuzz-ul pus la proba cu trei bug-uri introduse deliberat in
  `segmentare.rs`. Vezi mai jos — primele doua nu erau prinse.

## Ce a ieșit altfel decat planul

**Invariantul 5, scris de mine in `CRATE.md` la etapa 0, era fals.** Spunea ca
„concatenarea blocurilor acopera tot codul de intrare". Nu e adevarat, si nici
nu trebuie sa fie: segmentarea arunca intentionat liniile goale separatoare,
antetul de dinaintea primei instructiuni si blocurile care sunt doar comentariu,
iar blocurile unite se lipesc cu un singur `\n`. Invariantul adevarat e
`norm(blocuri.join("\n")) == norm(cod)` — se pierde doar ce normalizarea
oricum sterge.

Are o consecinta pe care n-o vazusem: **un bloc nu e garantat subsir al textului
original.** De-aia `linii_start_blocuri` are o ramura de rezerva pentru cazul in
care nu gaseste blocul — ancorele de scroll degradeaza silentios la blocurile
unite. Notat in `CRATE.md`; de verificat la etapa de editor, fiindca se vede
direct in sincronizarea panourilor.

**Prima versiune a fuzz-ului era decor pentru invariantul 4.** Trecea si cu
urmarirea literalelor si a comentariilor scoasa complet din `segmentare.rs`.
Cauza: decizia de split se ia per linie si doar cand linia **incepe** cu un
cuvant cheie, iar niciun fragment din generator nu avea o linie care incepe cu
un cuvant cheie in interiorul unui literal sau comentariu multilinie. Adaugate
sase fragmente exact de forma asta; abia apoi bug-ul a fost prins.

Lecția, pentru fuzz in general: nu e de ajuns ca generatorul sa produca forme
„complicate". Trebuie sa produca forma pe care **decizia testata** o
inspecteaza. Aici decizia se uita la inceputul liniei, deci conta unde cade
granita de linie, nu cat de incurcat e SQL-ul.

**Alinierea din `diff.rs` a rams fara invariante.** Nu era in criterii si nu am
adaugat-o: nu am gasit o proprietate ieftina si neredundanta fata de
conformitate. Candidat pentru mai tarziu — de exemplu, ca o permutare a
blocurilor nu schimba setul de perechi gasite.
