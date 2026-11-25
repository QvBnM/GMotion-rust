# Harta de depanare: simptom -> unde caut

Scopul: de la ce vede utilizatorul la fisierul vinovat, intr-un pas. Se
completeaza pe masura ce apar crate-uri noi. Fiecare `CRATE.md` are propria
sectiune `Erori tipice`, mai detaliata; asta e indexul de deasupra lor.

**Regula:** cand depanezi un simptom care nu e in tabel, adaugi randul dupa ce-l
rezolvi. Daca a durat mai mult de cateva minute sa gasesti fisierul, randul
lipsea.

---

## Motorul de comparare

| Simptom | Cauza probabila | Fisier |
|---|---|---|
| Derularea sacadeaza / monitorul `--fps` arata cadre lungi | ceva desenat pe tot ecranul la fiecare cadru (fundalul hartii), sau randuri refacute in masa | `rust/fereastra/src/lib.rs` (`incarca_fundal_harta`, `incarca_lista`); masura: `rust/app/examples/masura_ui.rs` |
| Cifrele monitorului de cadre par absurde | statistica pe ferestre de o secunda; momentele vin din filtrul winit | `rust/app/src/cadre.rs`, `porneste_monitorul_cadre` in `rust/app/src/main.rs` |
| Cadre desenate in repaus | ceva cu `animation-tick()` sau un `Timer` care schimba o proprietate cat se vede | `rust/fereastra/ui/*.slint` (`Rotitor`, cursorul campului de editare) |
| Clic in editor lent (`redesenare lenta`) | coloana / arborele refacute in masa, sau ecranul listei refacut din editor | `incarca_coloana_lista` in `rust/fereastra/src/lib.rs`, redesenarea din `rust/app/src/main.rs` (`redesenare pe bucati` la `debug`) |
| Un obiect ales din Object Explorer inlocuieste tabul deschis | clicul deschide in previzualizare in loc sa aleaga randul | `rust/fereastra/ui/editor.slint` (`ales-explorer`), `Comanda::DeschideInTabNou` in `rust/app/src/rutare.rs` |
| Side-by-side cu goluri: un bloc doar intr-o parte, iar perechea lui mai jos, doar in cealalta | ancora proasta in alinierea blocurilor (`END` cu `END`), imperechere peste goluri | `plan_blocuri` in `rust/core/src/diff.rs`; reproducere: `cargo run -p gmotion-fereastra --example arata_blocuri -- before.sql after.sql` |
| Vederea sare dupa transpunere / Aplica / Ctrl+Z | `incarca_editor` reface randurile si `mergi_la` deruleaza | `incarca_editor` / `incarca_editarea` (`pastreaza-derularea`) in `rust/fereastra/src/lib.rs` |
| Cadru lung la salvarea sesiunii | unirea cu discul pusa inapoi in aplicatie (arunca modelul editorului) | `primeste_salvarea` in `rust/app/src/stare.rs` |
| `sesiune salvata` de multe ori pe secunda | salvarea nu mai trece prin `SalvareAmanata` | `rust/app/src/main.rs`, `rust/app/src/stare.rs` |
| Verificarea pe server spune „bine" desi serverul a dat eroare | `rezumat_verificare` / `text_verificare` | `rust/app/src/stare.rs` |
| O modificare reala apare „identic" | o regula de normalizare a lovit intr-un string sau comentariu: granitele din lexer | `rust/core/src/lexer.rs`, `rust/core/src/normalizare.rs` |
| Doua obiecte identice apar ca „modificate" | normalizarea nu curata ceva (spatii, comentarii, paranteze, majuscule) | `rust/core/src/normalizare.rs` |
| Scorul de similaritate pare absurd | `scor_similaritate` sau rotunjirea bancara | `rust/core/src/normalizare.rs` |
| Blocuri taiate in mijlocul unui string / comentariu / `CASE` | urmarirea literalelor si a stivei in segmentare | `rust/core/src/segmentare.rs` |
| Un `UNION`, cursor `FOR` sau `INSERT ... SELECT` apare rupt in doua | `bloc_este_continuat`, `uneste_blocuri_compuse`, `uneste_cicluri_cursor` | `rust/core/src/segmentare.rs` |
| Bloc „modificat" doar printr-un `BEGIN` / `END` lipit; „Aplica" pierde sau dubleaza un `END` | `RE_INCEPUT_INSTRUCTIUNE`, `asteapta_corp` | `rust/core/src/segmentare.rs` |
| Numar de blocuri diferit de ce arata editorul | segmentare — e exact clasa de bug care a produs toate divergentele din v3.4 | `rust/core/src/segmentare.rs` |
| Blocuri imperecheate gresit (se vede ca „sters + adaugat" in loc de „modificat") | alinierea prin programare dinamica, `perechi_replace` | `rust/core/src/diff.rs` |
| Un bloc reordonat iese „modificat", sau „sters + adaugat", in loc de „mutat" | stratul blocurilor identice din `plan_blocuri`, `marcheaza_mutarile` | `rust/core/src/diff.rs` |
| Segmentarea unui obiect mare dureaza secunde; un obiect intreg intr-un singur bloc | o instructiune care nu se termina (`MERGE` fara `;`): `StareInstructiune`, `MAX_LINII_INSTRUCTIUNE` | `rust/core/src/segmentare.rs` |
| Un bloc desenat pe alta linie decat e in text; „Aplica" scrie peste alte linii | `Bloc::linii` din segmentare (nu se mai cauta in text) | `rust/core/src/segmentare.rs` |
| „Difera in" spune „comentarii" pe un obiect cu hash identic | `amprenta_comentarii` + `diferente_ascunse`; e intentionat | `rust/core/src/normalizare.rs`, `rust/module/diff-obiecte/src/comparare.rs` |
| Rezumat gresit („2 sterse" care nu exista) | `rezumat_rapid` / `rezumat_diff`, sau segmentarea de dedesubt | `rust/core/src/diff.rs` |
| Diferente in interiorul liniei marcate aiurea | `diff_intra` | `rust/core/src/diff.rs` |
| Rezultate diferite de `difflib` din Python | portul `SequenceMatcher` (atentie: `autojunk=False`) | `rust/core/src/seqmatch.rs` |
| Testul de conformitate cade | **presupui intai ca portul Rust e gresit**, nu fixture-ul | `rust/core/tests/conformitate.rs` |

## Harness si fixture

| Simptom | Unde |
|---|---|
| Vreau sa vad divergentele Python vs JS | `python3 -I referinta/compara.py` |
| Vreau sa regenerez fixture-ul de referinta | `unelte/regenereaza-fixture.sh` — explicit, cu motiv in `JURNAL.md` |
| `corpus_bench.json` lipseste | `python3 -I referinta/genereaza_corpus.py` (negitat, se genereaza local) |
| Masuratori de performanta | `referinta/README.md`, sectiunea de la final |
| Paritate Python v3.4 / Rust pe un instantaneu | `python3 -I referinta/paritate.py <instantaneu> --referinta <baza> --gmotion <binar>` |
| Paritatea raporteaza „de verificat" | obiect cu string-uri / identificatori / comentarii sau foarte mare, tratat deliberat altfel decat in v3.4; `gmotion diferente`, apoi in `*.asteptari.json` cu motiv | `referinta/paritate.py` |
| Paritatea raporteaza o regresie | intai `rust/module/diff-obiecte/src/comparare.rs`, apoi motorul din `core`; o `regresie` pe `rezumat` arata si ce zice JS |
| Exportul CSV nu se potriveste cu instantaneul | export facut cu filtru activ in interfata, sau driverele aduc obiecte diferite (`rust/db/src/sqlserver.rs`) |

## Conectare

| Simptom | Cauza probabila | Fisier |
|---|---|---|
| Scriptul de deploy e refuzat cu `GM-MOD-008/009/010/011` | nume cu caractere de control / `GO` in definitie / director ne-gol / obiect criptat | `rust/module/diff-obiecte/src/deploy.rs` |
| Deploy-ul cu `--plan` se opreste cu `GM-MOD-012` | baza (sau referinta) s-a schimbat de la plan; daca nu s-a schimbat, amprenta e gresita | `rust/module/diff-obiecte/src/plan.rs` |
| Scriptul de revenire reface/sterge gresit sau in ordine gresita | `PasRevenire`, `text_revenire` | `rust/module/diff-obiecte/src/deploy.rs` |
| `GM-DB-001` „No such host is known” pe `SRV\INSTANTA` | instanta numita nerezolvata prin SQL Browser | `rust/db/src/sqlserver.rs`, `interpreteaza_server` |
| `GM-DB-001` „SQL browser timeout” | SQL Browser oprit / UDP 1434 blocat; ocolire cu `--port` | `rust/db/src/sqlserver.rs`, `deschide_tcp` |

## Platforma

| Simptom | Cauza probabila | Fisier |
|---|---|---|
| Jurnalul afisat nu mai primeste linii noi dupa o vreme | indexul trebuie sa fie absolut, nu pozitie in coada (tamponul se roteste la `MAX_LINII`) | `rust/platforma/src/jurnal.rs` |
| Un test cade o data la cateva zeci de rulari | stare globala partajata intre teste paralele: mediu, tamponul de jurnal, abonatul `tracing` | testul respectiv; vezi `CLAUDE.md` sectiunea 6 |

## Poarta

| Simptom | Unde |
|---|---|
| Poarta cade pe `api-hash` | ai schimbat API-ul public fara sa actualizezi `CRATE.md`; hash-ul nou e in iesire |
| Poarta cade pe directia dependentelor | un crate depinde la dreapta in lant, sau un modul de alt modul — vezi `ARHITECTURA.md` |
| Poarta sare verificari | citeste iesirea: lipseste `node`, `python3`, sau tinta `wasm32` |
| Poarta cade pe `stare si etape` | doua etape marcate ACTIVA, o stare necunoscuta, sau `STARE.md` nu numeste etapa activa |
| Poarta noteaza ca `STARE.md` e vechi | ruleaza `unelte/stare.sh` si actualizeaza-l — vezi sectiunea 0 din `CLAUDE.md` |

## Reluarea lucrului

| Intrebare | Unde |
|---|---|
| Unde am rams? Ce urmeaza? | `STARE.md` |
| Ce ne-am propus la etapa asta, cand e gata? | `etape/NN-*.md`, cea cu `Stare: ACTIVA` |
| De ce arata codul asa? | `JURNAL.md` |
| Ce s-a facut concret? | `git log --oneline` |

## Versiunea Python (mostenire)

Se consulta doar ca referinta de paritate. **Nu se modifica.** Problemele
cunoscute sunt in `mostenire/MOSTENIRE.md`.

---

## De scris (crate-uri care nu exista inca)

Randurile de mai jos sunt locuri rezervate, ca sa fie clar unde vor ajunge
simptomele. Se completeaza la etapa care creeaza crate-ul.

| Simptom | Fisier |
|---|---|
| Conexiunea cade / driver negasit | `rust/db/` |
| Un server nu apare la descoperirea instantelor | `rust/db/` |
| Obiecte lipsa sau trunchiate din baza | `rust/db/` |
| Sesiunea salvata se suprascrie intre module | `rust/platforma/` — cheia trebuie `<modul>/<slug>.dat` |
| Sesiunea nu se poate decripta | `rust/platforma/` (DPAPI) |
| O metoda de modul apare la nivelul de sus al interfetei | `rust/platforma/` — rutare, nu expunere directa |
| Scriptul de deploy generat e gresit | `rust/module/diff-obiecte/` |
| Fereastra: DPI, monitoare, maximizare, DWM | `rust/fereastra/` (un singur fisier Win32 — aceeasi categorie de bug) |
