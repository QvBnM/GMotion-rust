# gmotion-core

<!-- api-hash: 33d8c15cc8db58cb -->

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
| `linii_go(text)` | liniile pe care `sqlcmd` le-ar lua drept separator `GO`, inclusiv `GO -- comentariu` |
| `Segment`, `Fel` | tipurile rezultatului |

### `normalizare`
| Functie | Ce face |
|---|---|
| `norm(s)` | normalizare pentru comparare: spatii, majuscule, comentarii — **doar pe cod**; string-urile si identificatorii ne-simpli raman neatinsi |
| `normalizeaza(cod)` | forma canonica a unui obiect intreg |
| `pregateste_afisare(cod)` | forma pentru afisare, pastreaza structura |
| `unifica_linii(cod)` | normalizeaza terminatorii de linie |
| `calc_hash(normalizat)` | SHA-256 hex al formei normalizate |
| `hinturi(cod)` | hinturile de blocare din pozitia de hint, in ordine; normalizarea le sterge, asta le deosebeste (F-M3) |
| `scor_similaritate(ref, cur)` | procent 0-100 |
| `scor_pe_tokeni(a, b)` | acelasi scor, pe token-uri deja calculate |
| `tokeni_scor(text)` | token-izare folosita de scor |
| `round_py(x, zecimale)` | rotunjire bancara, identica cu `round()` din Python |

### `segmentare`
| Functie | Ce face |
|---|---|
| `split_statements(cod, uneste_cursor)` | taie obiectul in blocuri |
| `cheie_bloc(bloc)` | cheia de imperechere a unui bloc |
| `linii_start_blocuri(cod, blocuri)` | linia de start a fiecarui bloc, pentru ancore; potrivirea se cauta doar la inceput de linie si in afara comentariilor / literalelor (prin `lexer`) |

### `diff`
| Functie / tip | Ce face |
|---|---|
| `diff_aliniat(ref, cur) -> Vec<RandDiff>` | diferentele, cu aliniere prin programare dinamica |
| `rezumat_rapid(ref, cur)` | rezumatul pentru lista de obiecte |
| `rezumat_diff(randuri)` | rezumatul dintr-un diff deja calculat |
| `marcaje_pe_linii(segmente)` | bucatile schimbate, taiate pe liniile blocului — ca sa nu-si faca fiecare consumator propria taiere pe `\n` |
| `RandDiff`, `Segment`, `FelSchimbare`, `TipDiferenta` | tipurile rezultatului |

### `aliniere`
| Functie / tip | Ce face |
|---|---|
| `aliniaza_cu(a, b, uneste_cursor)`, `diff_aliniat_cu`, `are_cursor(cod)` | varianta cu unirea ciclului de cursor (v3.4): blocul de cursor se ia intreg, de la DECLARE pana la DEALLOCATE |
| `aliniaza(stanga, dreapta) -> Aliniere` | diff-ul asezat pe linii, cu umplutura si cu liniile dintre blocuri; ce deseneaza editorul |
| `diff_aliniat_cu_raport(a, b, uneste_cursor)` | ca `diff_aliniat_cu`, plus cate segmente `replace` au ramas pe imperechere pozitionala |
| `compune(aliniere, stanga, dreapta, aplica)` | textul din dreapta cu blocurile alese luate din stanga: fiecare la pozitia lui din diff, fara plasare |
| `aplica_blocuri(stanga, dreapta, uneste_cursor, blocuri) -> String` | **„Aplica bloc"**: blocurile alese, aduse acolo unde spune plasarea (inlocuire sau inserare in gol, cu grupul). Singura cale prin care editorul, tragerea, modul ghidat si `cli` scriu un bloc |
| `aplica_blocuri_la(stanga, dreapta, uneste_cursor, blocuri, loc)`, `Loc`, `loc_la_linie(aliniere, linie)` | blocul lasat de om in alt loc decat cel sugerat (tragerea in editor, `gmotion editare --la`): peste un bloc din dreapta il inlocuieste, intre blocuri se insereaza (etapa 23) |
| `compune_plasat(aliniere, stanga, dreapta, plasari, aplica)` | ca `compune`, cu blocurile fara pereche duse unde spun `plasari` |
| `plasari_aliniere(aliniere, dreapta, uneste_cursor) -> Vec<Plasare>` | plasarea pe o aliniere, cu blocurile tintei taiate la fel; ce intreaba si editorul, ca ecranul sa arate locul in care scrie `aplica_blocuri` |
| `Aliniere` (`randuri`, `diff`, `diferente()`, `blocuri_nealiniate`), `RandAliniat` (`linie_in_bloc_stanga` / `_dreapta`, `primul_din_bloc`, `ultimul_din_bloc`, `ignorata`), `Linie` | tipurile rezultatului |

**Liniile a doua blocuri imperecheate se potrivesc intre ele, nu pe pozitie**
(din 2026-11-19, etapa 20): cheia e forma normalizata a liniei, iar in
segmentele `replace` imperecherea e pe scor, cu prag 40. Liniile goale si cele
care sunt numai comentariu nu intra in imperechere — normalizarea le ignora
oricum — si se reaseaza dupa, la locul lor. De aia `RandAliniat` are **doua**
pozitii in bloc, una pe parte: linia a treia din stanga poate sta langa a
cincea din dreapta. Peste `MAX_PRODUS_LINII` (4096 de perechi) imperecherea
cade pe pozitie, iar blocul apare in `Aliniere::blocuri_nealiniate`, ca sa nu
degradeze in tacere.

### `plasare`
| Functie / tip | Ce face |
|---|---|
| `grupuri_diferente(randuri, plasari) -> Vec<Vec<usize>>` | diferentele cum le vede omul: instructiunile consecutive fara pereche care se insereaza in acelasi gol sunt una singura (etapa 23); editorul aplica tot grupul |
| `plasari(randuri, blocuri_tinta) -> Vec<Plasare>` | unde ajunge **fiecare** bloc care difera, dupa ce cele doua straturi s-au impacat: perechea aleasa de diff bate, `sugereaza` raspunde doar unde nu exista pereche |
| `sugereaza(randuri, pozitie, blocuri_tinta) -> Option<Sugestie>` | unde ar ajunge blocul din martor: inlocuieste blocul k, sau se insereaza in golul g |
| `cel_mai_apropiat(text, blocuri, indice_preferat)` | blocul care seamana cel mai bine, cu regula de ambiguitate |
| `tokeni_potrivire(text)` | token-izarea folosita la potrivire (alta decat cea de la scorul din lista) |
| `Sugestie` (`randuri`, `text`, `tinta`), `Tinta::{Bloc, Gol}` | tipurile rezultatului |

Port 1:1 al stratului de plasare din v3.4 (`mostenire/ui.py`: `_sugereaza`,
`_celMaiApropiat`, `_gapSugerat`, `_grupPlasare`), cu pragurile lui: **55**
pentru inlocuire, **8 puncte** diferenta sub care scorul nu mai decide (atunci
conteaza pozitia, cu vecin la +-1 si scor minim **40**), **70** prag absolut
sub care se renunta, **6** blocuri plafon la grupare. Antetul (primul rand) se
potriveste direct cu primul bloc din tinta, fara scor — altfel parametrii noi
ii scad scorul sub prag si sugestia devine „insereaza la inceput", desi
antetul exista.

### `seqmatch`
`SequenceMatcher` — port fidel al `difflib.SequenceMatcher` din Python, cu
`autojunk=False`. `ratio`, `opcodes`, `matching_blocks`, `find_longest_match`,
`operatii`.

Pe intervale mari (peste 250.000 de perechi), cea mai lunga potrivire se cauta
cu un **automat de sufixe**, nu cu bucla din difflib: fara `isjunk` si cu
`autojunk` oprit, „cea mai lunga potrivire" e exact cel mai lung subsir comun,
iar difflib alege dintre ele pe cel cu `i` minim, apoi `j` minim — ce tine si
automatul, prin cea mai mica pozitie de final a fiecarei stari. Rezultatul e
**identic**, verificat element cu element in `teste_automat`; costul scade de
la cubic la aproape liniar (16.000 de linii: 38 s -> 0,53 s).

`operatii()` numara pasii elementari ai cautarilor facute pe acel matcher.
Deterministic, deci pe el se poate pune un plafon intr-un test (`tests/cost.rs`)
care nu devine instabil pe o masina incarcata.

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
`Limba` (`alege`, `numar` cu pluralul romanesc), `nume_tip_in(tip, limba)`,
`clasificare::{eticheta_in, propozitie_in, traduce_eticheta}` — aceleasi
texte in romana sau engleza, dupa comutatorul din fereastra.

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
   comentariu bloc, al unei paranteze deschise sau al unui `CASE`.
4c. Un bloc e o instructiune intreaga si nimic in plus: `BEGIN` si `END` au
   blocuri proprii (in afara de `BEGIN`-ul care e corpul unui `IF` / `WHILE` /
   `ELSE`), iar o instructiune neterminata (`IF` fara corp, `INSERT` fara
   sursa, `UPDATE` fara `SET`, `MERGE` fara `;`, CTE fara instructiunea lui)
   nu se rupe. De aici: o diferenta aplicata singura nu dezechilibreaza
   `BEGIN` / `END` (`tests/invariante.rs`, `aplicarea_unui_bloc_nu_strica_structura`).
4b. **Granitele le da `lexer`, si aici.** Pana la auditul QA segmentarea avea
   un al doilea lexer, care nu stia de `[...]`, `"..."` si `@`: un `[Case]`
   deschidea un `CASE` care nu se mai inchidea, deci tot obiectul devenea un
   singur bloc (P3). Linia „curatata" (fara siruri si comentarii) vine din
   `lexer::segmente`, iar `@case`, `#case` si `a.case` nu sunt cuvinte de
   stiva. Divergenta fata de v3.4 si de JS e deliberata: acolo bug-ul exista.
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
5b. **`compune` cu nimic aplicat da exact textul din dreapta, octet cu
   octet; cu toate diferentele aplicate, forma normalizata e a stangii.**
   Lucreaza pe liniile originale, nu pe textul blocurilor (care nu e garantat
   subsir, vezi 5). Liniile dintre blocuri (goale, comentarii) raman din
   dreapta. Verificat de fuzz in `tests/invariante.rs`.
   `aplica_blocuri` (cu plasare) pastreaza prima jumatate; a doua nu se cere,
   fiindca un bloc fara pereche poate inlocui un bloc din dreapta. Se cere in
   schimb, tot prin fuzz: blocul aplicat ajunge intreg, iar din dreapta nu se
   pierde mai mult decat blocul inlocuit.
5c. **Alinierea pe linii nu schimba ce linii apartin unui bloc**, doar cum se
   aseaza fata in fata. Fiecare linie a blocului apare exact o data, pe partea
   ei: de aia `compune` ramane corect (invarianta 5b) fara sa stie nimic
   despre imperecherea liniilor.
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
| Bloc „modificat" doar printr-un `BEGIN` / `END` lipit; „Aplica" pierde sau dubleaza un `END` | `RE_INCEPUT_INSTRUCTIUNE`, `asteapta_corp` | `src/segmentare.rs` |
| Blocul desenat la alta linie decat cea reala; „Aplica" schimba alte linii | `linii_start_blocuri` sau `aliniaza` | `src/segmentare.rs`, `src/aliniere.rs` |
| Numar de blocuri diferit de ce arata editorul | segmentare — clasa de bug care a produs toate divergentele din v3.4 | `src/segmentare.rs` |
| „sters + adaugat" in loc de „modificat" | alinierea `perechi_replace` (un bloc fata de un singur bloc se imperecheaza intotdeauna) | `src/diff.rs` |
| Doua linii fara legatura, una langa alta, intr-un bloc modificat | `perechi_linii` / `PRAG_LINIE` | `src/aliniere.rs` |
| Rezumat care raporteaza blocuri inexistente | `rezumat_rapid` / `rezumat_diff`, sau segmentarea de dedesubt | `src/diff.rs` |
| Rezultate diferite de `difflib` | portul `SequenceMatcher`, atentie la `autojunk=False` | `src/seqmatch.rs` |

## Testare

```
cd rust && cargo test -p gmotion-core          # 129 teste, ~15 s (invariantele)
```

Trei straturi:

1. **teste unitare** per modul — cazuri numite, pentru comportamente anume;
2. **`tests/conformitate.rs`** — toate cele 24 de cazuri din
   `referinta/cazuri.json`, pe 8 campuri, contra fixture-ului produs de motorul
   JS (`referinta/rezultate_js.json`);
3. **`tests/plasare.rs`** — paritate cu v3.4 pe plasare: fixture-ul
   `referinta/plasare_js.json` e scos din pagina v3.4 adevarata, rulata intr-un
   browser fara cap (`unelte/regenereaza-fixture-plasare.sh`). Plasarea nu se
   poate extrage mecanic din `ui.py` ca motorul de diff, fiindca tine de starea
   paginii.
4. **`tests/cost.rs`** — garda de cost: plafon pe numarul de pasi (nu pe
   timp) al cautarii, pe un obiect de 8.000 de linii. Vazuta cazand cu
   automatul de sufixe oprit: 1,43 miliarde de pasi fata de plafonul de 5
   milioane.
5. **`tests/invariante.rs`** — fuzz dirijat, 400 de intrari generate
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
