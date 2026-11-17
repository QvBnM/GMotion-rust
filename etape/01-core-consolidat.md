# Etapa 1 — `core` consolidat

**Stare:** NEPORNITA
**Branch:** —
**Deschisa:** — · **Incheiata:** —

## Scop

`gmotion-core` rămâne functional identic, dar devine depanabil: fiecare fisier
spune ce categorie de bug ii aparține, si zona cu istoric de bug-uri —
segmentarea — capata fuzzing pe invariantele ei.

**Nicio schimbare de semantica.** Daca testul de conformitate se mișca, ceva e
greșit. E o etapa ieftina, aleasa inaintea celor scumpe fiindca ridica
increderea exact acolo unde istoricul arata ca e nevoie.

## Criterii de terminare

- [ ] fiecare fisier din `rust/core/src/` are antet de 3-5 linii: rol +
      categoria de bug care ii aparține (regula 4 din `CLAUDE.md`)
- [ ] fuzz pe `split_statements`, care verifica invariantele 4 si 5 din
      `rust/core/CRATE.md`:
      nu taie in interiorul unui literal / comentariu bloc / paranteze /
      stiva `CASE`-`BEGIN`-`END`; concatenarea blocurilor acopera exact textul
      de intrare, fara pierdere si fara dublare
- [ ] fuzz pe idempotenta lui `norm` (invariantul 3) si pe
      `scor_similaritate(x, x) == 100.0` (invariantul 2)
- [ ] fuzz-ul ruleaza si in poarta, cu un buget de timp fix (cateva secunde),
      nu doar manual
- [ ] testul de conformitate **neschimbat** si verde — fixture-ul nu se atinge
- [ ] `rust/core/CRATE.md` actualizat: secțiunea `Testare` nu mai spune „nu
      exista inca fuzzing"
- [ ] poarta verde
- [ ] `STARE.md` actualizat

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

- (nepornita)

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
