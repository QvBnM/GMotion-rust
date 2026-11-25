# gmotion-core

<!-- api-hash: 47895b48d3d5380b -->

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
| `structura::verifica(text) -> Vec<Problema>` (`Problema`, `FelProblema`, `Bloc`) | ce nu se inchide in text: `BEGIN`/`END`, `TRY`/`CATCH`, `CASE`, paranteze, siruri / identificatori / comentarii, `GO`; cu linia; fara fals pozitiv pe obiectele reale din `testsimilitudine.json` (etapa 26) |

### `normalizare`
| Functie | Ce face |
|---|---|
| `norm(s)` | normalizare pentru comparare: spatii, majuscule, comentarii — **doar pe cod**; string-urile si identificatorii ne-simpli raman neatinsi |
| `normalizeaza(cod)` | forma canonica a unui obiect intreg |
| `pregateste_afisare(cod)` | forma pentru afisare, pastreaza structura |
| `unifica_linii(cod)` | normalizeaza terminatorii de linie |
| `calc_hash(normalizat)` | SHA-256 hex al formei normalizate |
| `hinturi(cod)` | hinturile de blocare din pozitia de hint, in ordine; normalizarea le sterge, asta le deosebeste (F-M3) |
| `amprenta_comentarii(cod)` | amprenta comentariilor; acelasi mecanism ca `hinturi`, pentru ce arunca normalizarea. Un obiect cu cod identic si comentarii schimbate e vizibil, fara sa se schimbe verdictul „identic" pe cod (etapa 30) |
| `scor_similaritate(ref, cur)` | procent 0-100 |
| `scor_pe_tokeni(a, b)` | acelasi scor, pe token-uri deja calculate |
| `tokeni_scor(text)` | token-izare folosita de scor |
| `round_py(x, zecimale)` | rotunjire bancara, identica cu `round()` din Python |
| `scor_maxim_posibil(n, m)` | marginea de sus a scorului din lungimi (`ratio <= 2*min/(n+m)`). **Exacta**, nu euristica: o pereche a carei margine e sub prag nu poate trece pragul, deci nu merita nicio comparatie. Pe matricile `n*m` din `diff` si `aliniere` sare peste cele mai multe perechi fara sa schimbe niciun rezultat |

### `segmentare`
| Functie / tip | Ce face |
|---|---|
| `segmenteaza(cod, uneste_cursor) -> Segmentare` | taie obiectul in blocuri, **cu liniile pe care stau** |
| `Segmentare` (`blocuri`, `instructiuni_retezate`, `normalizari`) | rezultatul: blocurile, cate instructiuni au fost taiate la plafon, si cate normalizari a cerut taierea (masura de cost deterministica) |
| `Bloc` (`text`, `linii`, `linie()`) | un bloc: textul lui si liniile (0-based) din original. `text` e **exact** aceste linii unite cu `\n` |
| `split_statements(cod, uneste_cursor)` | doar textele, infasuratoare peste `segmenteaza` |
| `linii_start(&Segmentare)` | linia de start a fiecarui bloc |
| `cheie_bloc(bloc)` | cheia de imperechere a unui bloc |

**Pozitia unui bloc nu se mai cauta** (din 2026-11-24, etapa 30). Pana atunci,
`linii_start_blocuri(cod, &[String])` cauta textul blocului inapoi in textul
original, cu o ramura de rezerva care punea linia blocului **precedent** cand
nu-l gasea — iar blocurile unite (`UNION`, ciclu de cursor) nu sunt subsir al
originalului, deci cadeau mereu pe ramura aceea: 8 din 109 obiecte reale
aveau cel putin un bloc asezat pe linia gresita, si „Aplica" scria peste alte
linii (B-1, CRITIC, din `cercetare/audit-qa-2026-11-23-clever-cerf.md`).
Functia nu mai exista: din texte **nu se poate** afla pozitia, de aia ghicea.
La fel a disparut `aliniere::harta_linii`, care reconstruia a doua oara harta
liniilor unui bloc prin potrivire in text.

### `diff`
| Functie / tip | Ce face |
|---|---|
| `compara(ref, cur, uneste_cursor) -> Comparare` | tot ce iese dintr-o comparare, calculat **o data**: randurile, blocurile amandoua parti cu pozitiile lor, si cate segmente au ramas pe imperechere pozitionala |
| `diff_aliniat(ref, cur) -> Vec<RandDiff>` | diferentele, cu aliniere prin programare dinamica |
| `rezumat_rapid(ref, cur)` | rezumatul pentru lista de obiecte |
| `rezumat_si_masura_rapide(ref, cur)` | rezumatul **si** masura, dintr-un singur plan: ce foloseste lista |
| `masoara(randuri) -> Masura` | cat s-a schimbat: blocuri pe feluri, si `schimbat` — procentul din obiect atins, ponderat pe marimea blocurilor. Langa scor, nu in locul lui: o instructiune schimbata din patruzeci da scor ~98 si `schimbat` ~2,5 |
| `rezumat_diff(randuri)` | rezumatul dintr-un diff deja calculat |
| `marcaje_pe_linii(segmente)` | bucatile schimbate, taiate pe liniile blocului — ca sa nu-si faca fiecare consumator propria taiere pe `\n`. **Pozitiile sunt in caractere**, nu in octeti ca la `lexer::Segment`: se indexeaza printr-un `Vec<char>`, nu cu `&text[a..b]` |
| `RandDiff` (`id`, `tip`, `mutat_cu`, `stanga`, `dreapta`, `seg_*`), `Segment`, `FelSchimbare`, `TipDiferenta`, `Masura`, `Comparare` | tipurile rezultatului |

**`TipDiferenta::Mutat`** (din 2026-11-24, etapa 30): acelasi bloc, in alt loc.
Vine pe **doua** randuri — unul cu text in martor, unul cu text in tinta — care
se arata reciproc prin `mutat_cu`; un singur rand cu text pe amandoua partile
ar strica ordinea pe care se sprijina tot restul (randurile cu text in dreapta
vin in ordinea blocurilor tintei, iar blocul mutat e prin definitie in alta
ordine). „Mutat" cere text normalizat **identic**: un bloc mutat si schimbat
ramane „sters" plus „adaugat". Pana la etapa 30 motorul n-avea notiunea, iar
reordonarea — refactorul cel mai frecvent — ieseau fie doua diferente fara
nicio legatura vizibila, fie, mai rau, **doua blocuri „modificate"** care nu
fusesera atinse.

**Imperecherea blocurilor are trei straturi**, in ordinea increderii:
ancorele lui `SequenceMatcher`; apoi blocurile identice ramase libere (aceeasi
cheie, deci acelasi text normalizat) — cele care se pot pastra in ordine devin
si ele ancore, cele incrucisate sunt mutari; si abia la urma scorul, pe ce a
ramas liber in fiecare gol. Stratul doi exista fiindca Ratcliff-Obershelp nu e
un LCS exhaustiv: un bloc care si-a schimbat locul peste potrivirea aleasa nu
mai e privit niciodata, si scorul il lua drept pereche a altui bloc.

### `aliniere`
| Functie / tip | Ce face |
|---|---|
| `aliniaza_cu(a, b, uneste_cursor)`, `diff_aliniat_cu`, `are_cursor(cod)` | varianta cu unirea ciclului de cursor (v3.4): blocul de cursor se ia intreg, de la DECLARE pana la DEALLOCATE |
| `aliniaza(stanga, dreapta) -> Aliniere` | diff-ul asezat pe linii, cu umplutura si cu liniile dintre blocuri; ce deseneaza editorul |
| `diff::e_bloc_banal(bloc)` | blocul e doar cuvinte de structura (`END`, `BEGIN`, `AS`, `GO`, `END TRY`...): nu poate ancora alinierea; se imperecheaza doar in golurile dintre ancore, de la capat (etapa 34) |
| `diff_aliniat_cu_raport(a, b, uneste_cursor)` | ca `diff_aliniat_cu`, plus cate segmente `replace` au ramas pe imperechere pozitionala |
| `compune(aliniere, stanga, dreapta, aplica)` | textul din dreapta cu blocurile alese luate din stanga: fiecare la pozitia lui din diff, fara plasare |
| `aplica_blocuri(stanga, dreapta, uneste_cursor, blocuri) -> String` | **„Aplica bloc"**: blocurile alese, aduse acolo unde spune plasarea (inlocuire sau inserare in gol, cu grupul). Singura cale prin care editorul, tragerea, modul ghidat si `cli` scriu un bloc |
| `aplica_blocuri_la(stanga, dreapta, uneste_cursor, blocuri, loc)`, `Loc`, `loc_la_linie(aliniere, linie)` | blocul lasat de om in alt loc decat cel sugerat (tragerea in editor, `gmotion editare --la`): peste un bloc din dreapta il inlocuieste, intre blocuri se insereaza (etapa 23) |
| `compune_plasat(aliniere, stanga, dreapta, plasari, aplica)` | ca `compune`, cu blocurile fara pereche duse unde spun `plasari` |
| `plasari_aliniere(aliniere, dreapta, uneste_cursor) -> Vec<Plasare>` | plasarea pe o aliniere, cu blocurile tintei taiate la fel; ce intreaba si editorul, ca ecranul sa arate locul in care scrie `aplica_blocuri` |
| `plasari_aliniere_cu_raport(...)` | la fel, plus `RaportPlasare`: cate comparatii s-au facut si cate plasari au fost puse pozitional la plafonul de cost |
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
| `plasari_cu_raport(...) -> (Vec<Plasare>, RaportPlasare)` | la fel, plus costul. Peste `MAX_COMPARATII_PLASARE` (200.000 de comparatii) blocurile fara pereche se asaza in golul lor pozitional, fara cautare, si faptul se **raporteaza** (`degradate`), nu se trece in tacere (restul lui B-5) |
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
1a. **Alinierea nu se ancoreaza pe blocuri banale** (`END`, `BEGIN`, `AS`...)
   si **antetul obiectului e perechea antetului** (primul `CREATE` / `ALTER`
   din fiecare text), oricat de diferite ar fi. Abatere deliberata fata de
   v3.4 (2026-11-25, `JURNAL.md`); obiectele reale care au aratat-o sunt in
   `tests/aliniere_reala.rs`.
1b. **Doua texte cu aceeasi forma normalizata sunt identice; orice alta
   diferenta e diferenta.** Decizia „identic" nu foloseste scorul (rotunjit).
   `cheie_bloc` egala inseamna text normalizat egal (128 de biti de SHA-256).
   Invarianta cere ca si **segmentarea** sa fie aceeasi pe doua texte cu
   aceeasi forma normalizata: o linie goala in antetul unui `CREATE FUNCTION`
   il rupea in doua blocuri, deci un obiect cu hash identic aparea „difera in
   cod" (auditul 2026-11-23). Antetul nu se desparte pana la `AS`-ul lui
   (`RE_ANTET_OBIECT` in `segmentare`).

1d. **Comentariile si hinturile sunt vizibile fara sa fie „cod".**
   Normalizarea le sterge, deci doua texte care difera doar prin ele au acelasi
   hash si invarianta 1b le declara identice. Ca sa nu dispara totusi din
   vedere, fiecare are o amprenta proprie (`hinturi`, `amprenta_comentarii`) pe
   care modulul o compara separat si o arata in „Difera in" ca diferenta
   ascunsa, langa setarile de sesiune. Verdictul pe **cod** nu se schimba.
   Consecinta pe care Mihai trebuie sa o stie: un obiect care difera doar
   intr-un comentariu intra acum in lista si in planul de deploy (etapa 30;
   pentru hinturi, asa e din F-M3).

1c. **Ce ramane fals, deliberat:** pe intrare **malformata** — un `[` sau un
   `'` care nu se inchide — invarianta 5 si idempotenta lui
   `pregateste_afisare` cad. Acolo nu exista un raspuns corect (`SELECT [a]];`
   chiar *este* un `[` neinchis in T-SQL), iar o regula care sa ghiceasca ar
   strica textul bine format. Zero aparitii pe cele 109 obiecte reale din
   `referinta/instantanee/`.
2. `scor_similaritate(x, x) == 100.0` pentru orice `x`.
3. `norm` e idempotenta: `norm(norm(s)) == norm(s)`.
4. `split_statements` nu taie niciodata in interiorul unui literal, al unui
   comentariu bloc, al unei paranteze deschise sau al unui `CASE`.
4d. **O instructiune neterminata nu inghite obiectul.** Peste
   `MAX_LINII_INSTRUCTIUNE` (500 de linii) se declara terminata cu forta, si
   `Segmentare::instructiuni_retezate` o spune. Exceptie deliberata la 4c: un
   `MERGE` fara `;` final sau un `UPDATE` al carui `SET` nu vine mai lipea tot
   restul obiectului intr-un singur bloc, deci diff-ul nu mai avea nicio
   valoare (un bloc „modificat" de 8.000 de linii). Cel mai lung bloc
   neterminat real din corpus are 39 de linii.

4e. **Un bloc e exact liniile lui din original, unite cu `\n`**
   (`Bloc::linii`). De aici pozitiile exacte, fara nicio cautare in text;
   verificat pe toate obiectele reale si pe fuzz.

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
| Un bloc reordonat iese „modificat" (sau „sters" + „adaugat") in loc de „mutat" | stratul de blocuri identice din `plan_blocuri`, sau `marcheaza_mutarile` | `src/diff.rs` |
| Segmentarea unui obiect mare dureaza secunde | o instructiune care nu se termina niciodata: `StareInstructiune`, plafonul `MAX_LINII_INSTRUCTIUNE` | `src/segmentare.rs` |
| „Difera in" spune „comentarii" pe un obiect cu hash identic | `amprenta_comentarii`; e intentionat, vezi invarianta 1d | `src/normalizare.rs` |

## Testare

```
cd rust && cargo test -p gmotion-core          # 136 teste, ~15 s (invariantele)
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
4. **`tests/cost.rs`** — garzi de cost, toate pe numere deterministe (pasi,
   normalizari, comparatii), nu pe timp: cautarea pe un obiect de 8.000 de
   linii (vazuta cazand cu automatul de sufixe oprit: 1,43 miliarde de pasi
   fata de plafonul de 5 milioane); segmentarea unei instructiuni neterminate,
   care trebuie sa ramana liniara (vazuta cazand cu starea incrementala si
   plafonul scoase: 125.751 -> 501.501 normalizari la dublarea liniilor);
   cresterea comparatiilor din `plasari`, si plafonul lor, vazut cazand cu
   plafonul ridicat la `usize::MAX`.
4b. **`tests/calitate.rs`** — cat de **bune** sunt diferentele, nu doar cat de
   stabile. Pe fiecare obiect real din `referinta/instantanee/` se aplica o
   mutatie cunoscuta (bloc sters, bloc adaugat, doua blocuri inversate, literal
   schimbat, reindentare, comentariu adaugat) si se cere motorului sa raporteze
   **exact** mutatia. Rata de recuperare are plafon de jos pe fiecare mutatie.
   Fara numarul asta, pragurile mostenite din v3.4 (44, 40, 55, 8, 40, 70, 80)
   nu se pot atinge: n-ai cum sa arati ca o schimbare e o imbunatatire. Primul
   lucru pe care l-a prins: doua blocuri inversate ieseau „2 modificate" pe 23
   din 25 de obiecte.
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
