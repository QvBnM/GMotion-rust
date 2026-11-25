# Etapa 30 — motorul de comparare: cost, pozitii, mutari, masura

**Stare:** INCHEIATA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-24 · **Incheiata:** 2026-11-25

## Scop

Analiza motorului din 2026-11-24 a gasit opt lucruri; etapa le remediaza pe
toate. La final: segmentarea nu mai e patratica si nu mai inghite obiectul
intreg, pozitiile blocurilor nu se mai ghicesc prin cautare in text, un bloc
reordonat se raporteaza ca **mutat**, exista o masura a calitatii alinierii
(nu doar a stabilitatii ei), iar lista are un scor care spune cat s-a
schimbat, nu cat seamana textul.

## Ce s-a masurat inainte (2026-11-24, release, corpus de 400 de perechi)

```
split_statements      59-80 ms   (16608 blocuri)       <- 60% din diff_aliniat
cheie_bloc            14-20 ms
diff_aliniat         105-115 ms
aliniaza             195-202 ms
aliniaza+plasari     216-324 ms
normalizeaza (tot corpusul, ambele parti)     13 ms
lexer::segmente                              0.7 ms
```

Patratic, reprodus:

```
UPDATE fara SET      500 linii:   94 ms  |  4000 linii: 7020 ms  -> 1 bloc
MERGE fara ; final  1000 linii:  351 ms  |  2000 linii: 1405 ms  -> 1 bloc
```

## Criterii de terminare

- [x] **1. Segmentarea nu mai renormalizeaza instructiunea curenta.**
      `UPDATE` fara `SET` si `MERGE` fara `;` la 4000 de linii stau sub 100 ms
      (de la 7020 ms), si costul creste liniar: ×2 la fiecare dublare, nu ×4.
      Test de cost cu plafon pe pasi, nu pe timp, **vazut cazand** cu starea
      incrementala scoasa.
- [x] **2. O instructiune neterminata nu mai inghite restul obiectului.**
      Plafon explicit, si cand se atinge se **spune** (nu in tacere).
- [x] **3. `split_statements` intoarce si pozitiile.** `linii_start_blocuri`
      nu mai cauta textul blocului inapoi in original: pozitia vine din
      segmentare. Ramura de rezerva (si clasa de bug B-1) dispare. Invarianta
      nouaa: linia raportata e linia reala, pe toate cele 109 obiecte din
      `referinta/instantanee/` si pe fuzz.
- [x] **4. `TipDiferenta::Mutat`.** Un bloc al carui text normalizat exista in
      tinta pe alta pozitie se raporteaza mutat, nu „sters" + „adaugat".
      Rezumatul, clasificarea, `cli` si fereastra il arata; plasarea il duce
      unde il cere martorul.
- [x] **5. Harness de mutatii** (`rust/core/tests/calitate.rs`): pe obiectele
      reale, mutatii cunoscute (bloc inserat, bloc sters, doua blocuri
      inversate, variabila redenumita, literal schimbat, reindentare), si
      motorul trebuie sa raporteze **exact** mutatia. Plafon pe rata de
      recuperare, ca sa cada daca scade. Vazut cazand cu un prag stricat.
- [x] **6. Scor pe blocuri**, camp nou langa cel existent (`scor_blocuri`):
      cat din obiect s-a schimbat, ponderat pe marimea blocurilor. Fixture-ul
      de referinta ramane neatins.
- [x] **7. Amprenta comentariilor** (`hash_comentarii`), ca la `hinturi`: un
      obiect cu cod identic si comentarii schimbate e vizibil, fara sa se
      schimbe verdictul „identic" pe cod.
- [x] **8. Retezare exacta in matricile de scor** (marginea
      `ratio <= 2*min/(n+m)`), plafoanele de 4096 ridicate, si garda de cost
      proprie pentru `plasari`.
- [x] fixture-urile de referinta **neatinse** (`referinta/rezultate_js.json`,
      `rezultate_python.json`, `plasare_js.json`); orice divergenta noua intra
      in `DIVERGENTE_DELIBERATE` cu motivul, nu in fixture
- [x] `rust/core/CRATE.md` la zi, cu api-hash-ul nou
- [x] fiecare verificare noua din poarta / test nou de garda **vazut cazand** o
      data (CLAUDE.md §10)
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat cu deciziile (Mutat, scor pe blocuri, amprenta
      comentariilor, pragurile atinse)

## Pasi

1. Masura de pornire, pusa in acest fisier. **Facut.**
2. Punctul 1 si 2: stare incrementala in `bloc_este_continuat`, plafon pe
   instructiunea neterminata, test de cost.
3. Punctul 5 inainte de punctele care ating alinierea: fara masura nu se poate
   spune daca un prag schimbat e o imbunatatire.
4. Punctul 3: `Bloc { text, linie, octeti }` din segmentare, consumatorii mutati.
5. Punctul 4: `Mutat` in plan, apoi in rezumat, clasificare, plasare, interfata.
6. Punctele 8, 6, 7.
7. Poarta, documente, `STARE.md`, `JURNAL.md`.

## Jurnal de lucru

- **2026-11-24** Analiza si masurarea motorului. Etapa deschisa cu masuratorile
  de pornire in fisier, ca sa se poata compara la final.
- **2026-11-25** Punctele 1-2 (starea incrementala si plafonul), apoi 5
  (harness-ul de calitate) inaintea celor care ating alinierea, apoi 3
  (pozitiile), 4 (`Mutat`), 8, 6, 7. Poarta, documente, stare.

## Ce s-a masurat la final (release, acelasi corpus)

```
segmenteaza        84,8 ms (16608 blocuri, cu pozitii)  <- inlocuieste
                                                           split_statements (59-80)
                                                           + linii_start_blocuri
diff_aliniat      113,8 ms   (era 105-115: straturile noi de imperechere
                              costa cat scoate retezarea pe lungimi)
aliniaza          148,6 ms   (era 195,2: nu mai segmenteaza de patru ori)
aliniaza+plasari  216,4 ms   (era 216-324)

UPDATE fara SET  4000 linii:  7020 ms -> 9,2 ms   (500 normalizari, retezate 1)
MERGE fara ;     2000 linii:  1405 ms -> 4,6 ms   (500 normalizari, retezate 1)
```

Rate de recuperare la final (`tests/calitate.rs`): bloc_sters 100%,
bloc_adaugat 100%, blocuri_inversate 100%, literal_schimbat 100%,
reindentare 100%, comentariu_adaugat 100%. Inainte de reparatia imperecherii,
blocuri_inversate era **8%**.

## Ce a iesit altfel decat planul

**Harness-ul de calitate a prins un bug mai grav decat lipsa lui „mutat".**
Planul il voia ca masura, ca sa se poata atinge pragurile. Prima rulare a
aratat ca doua blocuri inversate ies drept **„2 modificate"** pe 23 din 25 de
obiecte reale: motorul spunea „instructiunile astea doua s-au schimbat" despre
doua instructiuni neatinse. Asta nu e o nuanta de raportare, e un raspuns
gresit, si nicio proba de stabilitate nu-l putea prinde — fixture-ul,
invariantele si garda de cost erau toate verzi. Reparatia a cerut al doilea
strat de imperechere (blocurile identice, inaintea scorului), nu doar o
eticheta noua.

**Doua numere din teste au fost gresite, nu codul.** (CLAUDE.md §6, a patra
oara.) Prima: am cerut „exact 1 mutare" la inversarea a doua blocuri —
inversarea muta **doua** blocuri, si cate raporteaza motorul depinde de care
dintre ele rama ancorat; corect e „numai mutari, niciun modificat/sters/
adaugat". A doua: blocul unit de `UNION` pe care l-am pus in test era contiguu,
deci nu proba nimic.

**Plafonul si starea incrementala se acopera unul pe altul la masurare.**
Plafonul de 500 de linii singur margineste costul (nicio instructiune nu mai
creste nelimitat), iar starea incrementala singura il face liniar. Deci testul
de cost trece si cu starea veche, atata vreme cat plafonul e pus — a trebuit
stricate **amandoua** ca sa se vada rosu. Amandoua rama: plafonul apara
calitatea diff-ului (un obiect intreg intr-un bloc nu spune nimic), starea
apara costul chiar si sub plafon (500 de linii renormalizate ar fi 125.000 de
normalizari).

**Retezarea pe lungimi n-a adus timp, a adus margine.** Marginea
`ratio <= 2*min/(n+m)` sare peste cele mai multe perechi, dar straturile noi de
imperechere consuma exact ce economiseste ea: `diff_aliniat` a rama pe loc.
Castigul real e ca plafoanele de 4096 au putut urca la 16384, deci sunt mai
putine blocuri imperecheate pozitional — adica mai putine „modificat" care nu
inseamna nimic.

**Un bug vechi reparat ca efect secundar.** Cautarea pozitiei blocului in text
(B-1) avea deja doua ramuri; a treia ar fi fost urmatorul pas. Odata ce
segmentarea intoarce pozitiile, au disparut amandoua, plus `harta_linii` din
`aliniere`, care facea a treia data aceeasi socoteala. 60 de linii de cod de
ghicit, inlocuite cu un `Vec<usize>` pe care segmentarea il avea deja in mana.

**Ce n-am facut, si de ce.** In fereastra nu exista coloana „Schimbat": o
coloana noua in lista cere macheta si „da"-ul lui Mihai (CLAUDE.md §10, etapa
16). Numarul exista in `cli --detalii`, in `--json` si in CSV. La fel, un bloc
mutat pastreaza culorile blocului fara pereche si schimba doar vorba din
pastila — un al cincilea cod de stare ar fi alt desen.
