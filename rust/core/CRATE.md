# gmotion-core

<!-- api-hash: 14af2056368f4067 -->

## Rol

Motorul de comparare a obiectelor SQL: normalizare, segmentare in blocuri, diff
aliniat, scoruri si rezumate. Singura implementare a acestei logici in tot
proiectul.

## Nu face

- **Fara I/O.** Niciun `std::fs`, nicio retea, nicio baza de date. Trebuie sa
  compileze pe `wasm32-unknown-unknown`, ca sa poată fi folosit si de backend si
  de editorul live — exact asta impiedica reapariția celor doua motoare
  divergente din v3.4.
- Fara nimic specific unei interfete: fara culori, fara HTML, fara formatare de
  afisat.
- Nu citeste obiecte din baza — asta e treaba `db`.
- Nu decide ce se compara cu ce la nivel de obiect — asta e treaba modulului.

## API public

### `normalizare`
| Functie | Ce face |
|---|---|
| `norm(s)` | normalizare pentru comparare: spatii, majuscule, comentarii |
| `normalizeaza(cod)` | forma canonica a unui obiect intreg |
| `pregateste_afisare(cod)` | forma pentru afisare, pastreaza structura |
| `unifica_linii(cod)` | normalizeaza terminatorii de linie |
| `calc_hash(normalizat)` | SHA-256 hex al formei normalizate |
| `scor_similaritate(ref, cur)` | procent 0-100 |
| `scor_pe_tokeni(a, b)` | acelasi scor, pe token-uri deja calculate |
| `tokeni_scor(text)` | token-izare folosita de scor |
| `round_py(x, zecimale)` | rotunjire bancara, identica cu `round()` din Python |

### `segmentare`
| Functie | Ce face |
|---|---|
| `split_statements(cod, uneste_cursor)` | taie obiectul in blocuri |
| `cheie_bloc(bloc)` | cheia de imperechere a unui bloc |
| `linii_start_blocuri(cod, blocuri)` | linia de start a fiecarui bloc, pentru ancore |

### `diff`
| Functie / tip | Ce face |
|---|---|
| `diff_aliniat(ref, cur) -> Vec<RandDiff>` | diferentele, cu aliniere prin programare dinamica |
| `rezumat_rapid(ref, cur)` | rezumatul pentru lista de obiecte |
| `rezumat_diff(randuri)` | rezumatul dintr-un diff deja calculat |
| `RandDiff`, `Segment`, `FelSchimbare`, `TipDiferenta` | tipurile rezultatului |

### `seqmatch`
`SequenceMatcher` — port fidel al `difflib.SequenceMatcher` din Python, cu
`autojunk=False`. `ratio`, `opcodes`, `matching_blocks`, `find_longest_match`.

### rădăcină
`TIPURI_OBIECTE` — tipurile comparate, ca in `sys.objects.type`:
`P` procedura, `V` view, `FN`/`IF`/`TF` functii, `TR` trigger.

## Invariante

1. **Semantica portata e cea a motorului JS** din `mostenire/ui.py`, nu cea din
   `mostenire/motor.py`. Diferentele fata de `motor.py` sunt intentionate: sunt
   corecturi de bug. Vezi `referinta/README.md`.
2. `scor_similaritate(x, x) == 100.0` pentru orice `x`.
3. `norm` e idempotenta: `norm(norm(s)) == norm(s)`.
4. `split_statements` nu taie niciodata in interiorul unui literal, al unui
   comentariu bloc, al unei paranteze deschise sau al unei stive
   `CASE` / `BEGIN` / `END`.
5. Concatenarea blocurilor intoarse de `split_statements` acopera tot codul de
   intrare — nu se pierde si nu se dubleaza text.
6. Functiile sunt totale pe intrari UTF-8 valide: nu intorc `Result`, nu dau
   panica.

## Erori tipice

| Simptom | Cauza probabila | Fisier |
|---|---|---|
| Obiecte identice raportate ca „modificate" | normalizarea nu curata ceva | `src/normalizare.rs` |
| Scor absurd | `scor_similaritate` sau `round_py` | `src/normalizare.rs` |
| Bloc taiat in mijlocul unui string / comentariu / `CASE` | urmarirea stivei si a literalelor | `src/segmentare.rs` |
| `UNION`, cursor `FOR`, `INSERT ... SELECT` rupt in doua | `bloc_este_continuat` si uniunile de blocuri | `src/segmentare.rs` |
| Numar de blocuri diferit de ce arata editorul | segmentare — clasa de bug care a produs toate divergentele din v3.4 | `src/segmentare.rs` |
| „sters + adaugat" in loc de „modificat" | alinierea `perechi_replace` | `src/diff.rs` |
| Rezumat care raporteaza blocuri inexistente | `rezumat_rapid` / `rezumat_diff`, sau segmentarea de dedesubt | `src/diff.rs` |
| Rezultate diferite de `difflib` | portul `SequenceMatcher`, atentie la `autojunk=False` | `src/seqmatch.rs` |

## Testare

```
cd rust && cargo test -p gmotion-core          # 31 teste
```

Acopera: teste unitare per modul, plus `tests/conformitate.rs` — toate cele 24
de cazuri din `referinta/cazuri.json`, pe 8 campuri, contra fixture-ului produs
de motorul JS (`referinta/rezultate_js.json`).

**Daca testul de conformitate cade, presupunerea implicita e ca portul Rust e
greșit.** Fixture-ul nu se regenereaza pentru a face un test sa treaca — asa au
divergat cele doua motoare din v3.4.

Nu acopera: performanta (vezi `examples/bench.rs`), si nu exista inca fuzzing pe
segmentare.

## Depanare

```
cd rust && cargo test -p gmotion-core <nume_test> -- --nocapture
cargo run --release --example bench      # cere referinta/corpus_bench.json
```

Un caz nou se adauga in `referinta/cazuri.json`, se regenereaza fixture-ul prin
`unelte/regenereaza-fixture.sh` (cere `node`), si abia apoi se repara codul.

Pentru a vedea unde difera de motorul vechi:
`python3 -I referinta/compara.py`.
