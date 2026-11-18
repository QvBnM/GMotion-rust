# gmotion-core

<!-- api-hash: dd6699882594c467 -->

## Rol

Motorul de comparare a obiectelor SQL: normalizare, segmentare in blocuri, diff
aliniat, scoruri si rezumate. Singura implementare a acestei logici in tot
proiectul.

## Nu face

- **Fara I/O.** Niciun `std::fs`, nicio retea, nicio baza de date. Trebuie sa
  compileze pe `wasm32-unknown-unknown`, ca sa poata fi folosit si de backend si
  de editorul live — exact asta impiedica reaparitia celor doua motoare
  divergente din v3.4.
- Fara nimic specific unei interfete: fara culori, fara HTML, fara formatare de
  afisat.
- Nu citeste obiecte din baza — asta e treaba `db`.
- Nu decide ce se compara cu ce la nivel de obiect — asta e treaba modulului.

## API public

### `lexer`
| Functie / tip | Ce face |
|---|---|
| `segmente(text) -> Vec<Segment>` | granitele cod / string / identificator / comentariu, in octeti |
| `linii_go(text)` | liniile pe care `sqlcmd` le-ar lua drept separator `GO` |
| `Segment`, `Fel` | tipurile rezultatului |

### `normalizare`
| Functie | Ce face |
|---|---|
| `norm(s)` | normalizare pentru comparare: spatii, majuscule, comentarii — **doar pe cod**; string-urile si identificatorii ne-simpli raman neatinsi |
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

### `clasificare`
`clasifica_diferente(ref, tinta) -> Clasificare` (sau `clasifica_randuri` pe randuri deja aliniate) — in ce difera doua versiuni:
`cod` / `siruri` / `comentarii` (bucati schimbate, nu caractere), prin `lexer`.
`propozitie(ref, tinta, setari)` — propozitia din previzualizare:
„un singur caracter, intr-un sir: Nu → nu", altfel „difera in ...".
`eticheta(c, setari)` — textul din coloana „Difera in" al machetelor:
„cod, siruri", „un sir", „comentariu", „QUOTED_IDENTIFIER", „—".

**Comentariul singur nu ajunge aici:** normalizarea sterge comentariile (ca in
v3.4), deci un obiect care difera doar intr-un comentariu e „identic" si nu
intra in lista. Decizie confirmata 2026-11-17; vezi testul
`comentariul_singur_ramane_zgomot_filtrat`.

### radacina
`TIPURI_OBIECTE` — tipurile comparate, ca in `sys.objects.type`:
`P` procedura, `V` view, `FN`/`IF`/`TF` functii, `TR` trigger.
`nume_tip(tip)` — numele tipului pentru utilizator („procedura", „view",
„functie", „trigger"), acelasi text in `cli` si in fereastra.

## Invariante

1. **Semantica portata e cea a motorului JS** din `mostenire/ui.py`, nu cea din
   `mostenire/motor.py`. Diferentele fata de `motor.py` sunt intentionate: sunt
   corecturi de bug. Vezi `referinta/README.md`. **Exceptie, din 2026-11-17:**
   normalizarea nu mai atinge string-urile, identificatorii ne-simpli si
   comentariile din string-uri (JS si v3.4 aveau bug-ul). Divergentele sunt
   listate in `tests/conformitate.rs` (`DIVERGENTE_DELIBERATE`), nu ascunse
   prin regenerarea fixture-ului.
1b. **Doua texte cu aceeasi forma normalizata sunt identice; orice alta
   diferenta e diferenta.** Decizia „identic" nu foloseste scorul (rotunjit).
   `cheie_bloc` egala inseamna text normalizat egal (128 de biti de SHA-256).
2. `scor_similaritate(x, x) == 100.0` pentru orice `x`.
3. `norm` e idempotenta: `norm(norm(s)) == norm(s)`.
4. `split_statements` nu taie niciodata in interiorul unui literal, al unui
   comentariu bloc, al unei paranteze deschise sau al unei stive
   `CASE` / `BEGIN` / `END`.
5. `split_statements` nu pierde si nu dubleaza **cod**:
   `norm(blocuri.join("\n")) == norm(cod)`.
   Atentie, forma mai tare — concatenare egala cu textul de intrare — e
   **falsa**, si asta e intentionat: segmentarea arunca liniile goale
   separatoare, antetul de dinaintea primei instructiuni si blocurile care sunt
   doar comentariu, iar blocurile unite (`UNION`, cursor) se lipesc cu un singur
   `\n`. Tot ce se arunca normalizeaza la gol, deci egalitatea se cere pe forma
   normalizata. Consecinta practica: un bloc **nu** e garantat subsir al textului
   original, de-aia `linii_start_blocuri` are o ramura de rezerva cand nu
   gaseste blocul.
6. Functiile sunt totale pe intrari UTF-8 valide: nu intorc `Result`, nu dau
   panica.

## Erori tipice

| Simptom | Cauza probabila | Fisier |
|---|---|---|
| Obiecte identice raportate ca „modificate" | normalizarea nu curata ceva | `src/normalizare.rs` |
| O modificare reala raportata „identic" | o regula de cod a lovit intr-un string / comentariu: granitele din `lexer` | `src/lexer.rs`, `src/normalizare.rs` |
| Scor absurd | `scor_similaritate` sau `round_py` | `src/normalizare.rs` |
| Bloc taiat in mijlocul unui string / comentariu / `CASE` | urmarirea stivei si a literalelor | `src/segmentare.rs` |
| `UNION`, cursor `FOR`, `INSERT ... SELECT` rupt in doua | `bloc_este_continuat` si uniunile de blocuri | `src/segmentare.rs` |
| Numar de blocuri diferit de ce arata editorul | segmentare — clasa de bug care a produs toate divergentele din v3.4 | `src/segmentare.rs` |
| „sters + adaugat" in loc de „modificat" | alinierea `perechi_replace` | `src/diff.rs` |
| Rezumat care raporteaza blocuri inexistente | `rezumat_rapid` / `rezumat_diff`, sau segmentarea de dedesubt | `src/diff.rs` |
| Rezultate diferite de `difflib` | portul `SequenceMatcher`, atentie la `autojunk=False` | `src/seqmatch.rs` |

## Testare

```
cd rust && cargo test -p gmotion-core          # 37 teste, ~1.5 s
```

Trei straturi:

1. **teste unitare** per modul — cazuri numite, pentru comportamente anume;
2. **`tests/conformitate.rs`** — toate cele 24 de cazuri din
   `referinta/cazuri.json`, pe 8 campuri, contra fixture-ului produs de motorul
   JS (`referinta/rezultate_js.json`);
3. **`tests/invariante.rs`** — fuzz dirijat, 400 de intrari generate
   determinist per test, pe invariantele 2-6. Generatorul nu produce SQL
   aleator, ci formele care pun la incercare invariantele: literale si
   comentarii pe mai multe linii in care o linie **incepe** cu un cuvant cheie,
   `CASE` imbricate, paranteze pe mai multe linii, `UNION` la final, cursor
   `FOR`, `INSERT ... SELECT`. Un esec raporteaza seed-ul si textul.

**Daca testul de conformitate cade, presupunerea implicita e ca portul Rust e
gresit.** Fixture-ul nu se regenereaza pentru a face un test sa treaca — asa au
divergat cele doua motoare din v3.4.

Fuzz-ul a fost pus la proba cu trei bug-uri introduse deliberat in
`segmentare.rs` (starea de literal ignorata la split, adancimea parantezelor
ignorata, ultimul bloc aruncat) si le prinde pe toate trei. Prima versiune a
generatorului **nu** prindea primele doua — vezi `etape/01-core-consolidat.md`.

Nu acopera: performanta (vezi `examples/bench.rs`), si nici alinierea din
`diff.rs` la nivel de invariante — acolo exista doar cazuri numite si
conformitatea.

## Depanare

```
cd rust && cargo test -p gmotion-core <nume_test> -- --nocapture
cargo run --release --example bench      # cere referinta/corpus_bench.json
```

Un caz nou se adauga in `referinta/cazuri.json`, se regenereaza fixture-ul prin
`unelte/regenereaza-fixture.sh` (cere `node`), si abia apoi se repara codul.

Un esec de fuzz se reproduce cu seed-ul din mesaj. Generatorul e determinist:
`SEED_URI` si listele de fragmente sunt in capul lui `tests/invariante.rs`, si
un fragment nou se adauga acolo cand apare o forma de T-SQL pe care nu o
acopera.

Pentru a vedea unde difera de motorul vechi:
`python3 -I referinta/compara.py`.
