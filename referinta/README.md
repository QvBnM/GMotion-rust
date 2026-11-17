# Referinta de conformitate

Harness care ruleaza acelasi set de cazuri prin ambele motoare existente si
raporteaza divergentele. Serveste la doua lucruri:

1. **dovada** ca `mostenire/motor.py` si portul JS din `mostenire/ui.py` nu mai sunt echivalente;
2. **referinta** contra careia se valideaza portul in Rust.

## Rulare

```
python3 -I referinta/genereaza.py     # -> rezultate_python.json
node     referinta/genereaza.mjs      # -> rezultate_js.json
python3 -I referinta/compara.py       # raport de divergente
```

`motor_js.mjs` e extras MECANIC din `mostenire/ui.py` (liniile 2593-3106, plus `_tokenize`
si `_normLinie`), ca sa nu existe risc de a-l rescrie din greseala. Daca `mostenire/ui.py`
se schimba, se re-extrage.

## Rezultat la `v0-baseline`

24 cazuri, 8 cu cel putin o divergenta.

| Camp | Divergente |
|---|---|
| `norm_ref` / `norm_cur` | 0/24 |
| `scor` | 0/24 |
| `blocuri_ref` | 8/24 |
| `blocuri_cur` | 7/24 |
| `rezumat_rapid` | 2/24 |
| `rezumat_diff` | 2/24 |
| `tipuri` | 8/24 |

Normalizarea si scorul de similaritate sunt identice. Toata divergenta vine din
**segmentarea in blocuri**, si de acolo se propaga in tipurile de diferenta.

Cauza: portul JS a fost dezvoltat mai departe decat `mostenire/motor.py` si a primit
lucruri care nu au fost aduse niciodata inapoi:

- splituieste si pe `CREATE` si `DECLARE`
- urmareste string-uri, comentarii bloc, adancimea parantezelor si stiva
  `CASE`/`BEGIN`/`END`, deci nu mai taie in mijlocul lor
- `_blocEsteContinuat` (cursor `FOR`, `INSERT ... SELECT`)
- `_unesteBlocuriCompuse`, `_unesteCicluriCursor`
- `_perechiReplace` - aliniere prin programare dinamica pe similaritate, in loc
  de imperechere pozitionala

### Doua divergente vizibile direct utilizatorului

| Caz | Lista (`mostenire/motor.py`) | Editor (JS) |
|---|---|---|
| `declare_in_header` | `1 modificate` | `1 sterse` |
| `comentariu_bloc_multiline` | `2 sterse` | `diferente minore` |

In al doilea caz, un comentariu bloc pe mai multe linii face `mostenire/motor.py` sa
raporteze doua blocuri sterse care **nu exista** - un fals pozitiv in lista,
care dispare cand deschizi obiectul. Exact genul de bug greu de urmarit, fiindca
simptomul si cauza sunt in fisiere diferite.

## Consecinta pentru portul Rust

Se porteaza **semantica JS**, nu `mostenire/motor.py`. Motorul JS e cel mai nou si cel
corect; `mostenire/motor.py` a rams in urma. Fixture-ul de referinta e
`rezultate_js.json`.

---

## Paritate pe un instantaneu

`paritate.py` ruleaza `Api._comparatie` din `mostenire/app.py` **neatins** pe
obiectele unui instantaneu (`pyodbc` e inlocuit, in procesul scriptului, cu un
modul care serveste randurile din fisier) si compara lista cu
`gmotion compara --json` pe acelasi instantaneu.

```
python3 -I referinta/paritate.py instantaneu.json --referinta Ref --gmotion rust/target/release/gmotion
python3 -I referinta/paritate.py --din-cazuri --gmotion rust/target/debug/gmotion   # ce ruleaza poarta
```

Optional `--csv export.csv` (exportul aplicatiei Python pe aceleasi baze) si
`--json raport.json`. Un rezumat diferit pe un obiect `diferit` se arbitreaza
prin motorul JS (`rezumat_js.mjs`): daca JS da ce da Rust, e `corectura`;
orice altceva e `regresie`. Cod de ieșire 0 / 1 (regresii) / 2 (intrare).

Un instantaneu mare se micsoreaza la obiectele de interes cu
`extrage.py instantaneu.json --filtru TEXT -o mic.json` (pastreaza obiectul din
toate bazele, ca lipsa/extra sa ramana lipsa/extra).

Pe instantaneul din `cazuri.json`: 0 regresii, o corectura
(`declare_in_header`). `comentariu_bloc_multiline` nu apare: normalizat e
identic, deci in lista e identic si nu primeste rezumat — falsul pozitiv din
v3.4 se vede doar in editor.

---

## Masuratori de performanta

Corpus sintetic: 400 perechi de proceduri, ~51 linii fiecare (~20.500 linii in
total). Generat cu `genereaza_corpus.py`, rulat cu `bench.py` si
`cargo run --release --example bench`.

| Operatie | `mostenire/motor.py` | `gmotion-core` |
|---|---:|---:|
| `scor_similaritate` + `rezumat_rapid` | 887 ms | **210 ms** |
| `diff_aliniat` | 63 ms | 90 ms |

**Cum se citesc cifrele, onest:**

Pe munca direct comparabila (scor + rezumat) Rust e de ~4x mai rapid. Nu de
20-50x, cum s-ar putea presupune. Motivul e ca `norm()` face zece treceri
succesive de regex pe acelasi text, iar asta domina timpul; o normalizare
intr-o singura trecere ar mai aduce mult, dar e o rescriere separata, cu risc
propriu de divergenta.

La `diff_aliniat` cifrele **nu sunt comparabile**: cele doua implementari fac
munca diferita. Se vede direct in numarul de randuri produse - 8424 pentru
Python, 6934 pentru Rust. Varianta Rust ruleaza alinierea prin programare
dinamica (`perechi_replace`), care compara fiecare bloc din referinta cu
fiecare bloc din tinta; `mostenire/motor.py` imperecheaza pozitional, mai ieftin si mai
greșit. Rust face deci substantial mai mult, in timp apropiat.

Castigul real al migrarii nu e deci viteza bruta, ci:

1. **un singur motor** in loc de doua divergente;
2. **memoria** - cea mai mare problema practica, vezi punctul 3 din README-ul
   principal: `tok` ca lista de obiecte Python per obiect per baza. In Rust
   token-urile sunt felii din acelasi tampon, nu obiecte separate cu antet
   propriu.

### Optimizari deja aplicate

- `bloc_este_continuat` se evalueaza ultima in conditie, nu la fiecare linie.
  Normalizeaza tot blocul acumulat, deci evaluarea eager facea segmentarea
  patratica in dimensiunea blocului. (`diff_aliniat`: 199 ms -> 90 ms)
- Forma normalizata a fiecarui bloc se calculeaza o singura data in bucla de
  `UNION` plus filtrare, nu de doua ori.
- `perechi_replace` precalculeaza token-urile o data per bloc, in loc sa
  renormalizeze la fiecare dintre cele `n * m` perechi.
