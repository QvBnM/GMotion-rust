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
| O modificare reala apare „identic" | o regula de normalizare a lovit intr-un string sau comentariu: granitele din lexer | `rust/core/src/lexer.rs`, `rust/core/src/normalizare.rs` |
| Doua obiecte identice apar ca „modificate" | normalizarea nu curata ceva (spatii, comentarii, paranteze, majuscule) | `rust/core/src/normalizare.rs` |
| Scorul de similaritate pare absurd | `scor_similaritate` sau rotunjirea bancara | `rust/core/src/normalizare.rs` |
| Blocuri taiate in mijlocul unui string / comentariu / `CASE` | urmarirea literalelor si a stivei in segmentare | `rust/core/src/segmentare.rs` |
| Un `UNION`, cursor `FOR` sau `INSERT ... SELECT` apare rupt in doua | `bloc_este_continuat`, `unește_blocuri_compuse`, `unește_cicluri_cursor` | `rust/core/src/segmentare.rs` |
| Numar de blocuri diferit de ce arata editorul | segmentare — e exact clasa de bug care a produs toate divergentele din v3.4 | `rust/core/src/segmentare.rs` |
| Blocuri imperecheate greșit (se vede ca „sters + adaugat" in loc de „modificat") | alinierea prin programare dinamica, `perechi_replace` | `rust/core/src/diff.rs` |
| Rezumat greșit („2 sterse" care nu exista) | `rezumat_rapid` / `rezumat_diff`, sau segmentarea de dedesubt | `rust/core/src/diff.rs` |
| Diferente in interiorul liniei marcate aiurea | `diff_intra` | `rust/core/src/diff.rs` |
| Rezultate diferite de `difflib` din Python | portul `SequenceMatcher` (atentie: `autojunk=False`) | `rust/core/src/seqmatch.rs` |
| Testul de conformitate cade | **presupui intai ca portul Rust e greșit**, nu fixture-ul | `rust/core/tests/conformitate.rs` |

## Harness si fixture

| Simptom | Unde |
|---|---|
| Vreau sa vad divergentele Python vs JS | `python3 -I referinta/compara.py` |
| Vreau sa regenerez fixture-ul de referinta | `unelte/regenereaza-fixture.sh` — explicit, cu motiv in `JURNAL.md` |
| `corpus_bench.json` lipseste | `python3 -I referinta/genereaza_corpus.py` (negitat, se genereaza local) |
| Masuratori de performanta | `referinta/README.md`, secțiunea de la final |
| Paritate Python v3.4 / Rust pe un instantaneu | `python3 -I referinta/paritate.py <instantaneu> --referinta <baza> --gmotion <binar>` |
| Paritatea raporteaza „de verificat" | obiect cu string-uri / identificatori / comentarii sau foarte mare, tratat deliberat altfel decat in v3.4; `gmotion diferente`, apoi in `*.asteptari.json` cu motiv | `referinta/paritate.py` |
| Paritatea raporteaza o regresie | intai `rust/module/diff-obiecte/src/comparare.rs`, apoi motorul din `core`; o `regresie` pe `rezumat` arata si ce zice JS |
| Exportul CSV nu se potriveste cu instantaneul | export facut cu filtru activ in interfata, sau driverele aduc obiecte diferite (`rust/db/src/sqlserver.rs`) |

## Conectare

| Simptom | Cauza probabila | Fisier |
|---|---|---|
| Scriptul de deploy e refuzat cu `GM-MOD-008/009/010/011` | nume cu caractere de control / `GO` in definitie / director ne-gol / obiect criptat | `rust/module/diff-obiecte/src/deploy.rs` |
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
| Poarta cade pe `api-hash` | ai schimbat API-ul public fara sa actualizezi `CRATE.md`; hash-ul nou e in ieșire |
| Poarta cade pe direcția dependentelor | un crate depinde la dreapta in lanț, sau un modul de alt modul — vezi `ARHITECTURA.md` |
| Poarta sare verificari | citeste ieșirea: lipseste `node`, `python3`, sau tinta `wasm32` |
| Poarta cade pe `stare si etape` | doua etape marcate ACTIVA, o stare necunoscuta, sau `STARE.md` nu numeste etapa activa |
| Poarta noteaza ca `STARE.md` e vechi | ruleaza `unelte/stare.sh` si actualizeaza-l — vezi secțiunea 0 din `CLAUDE.md` |

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
| Scriptul de deploy generat e greșit | `rust/module/diff-obiecte/` |
| Fereastra: DPI, monitoare, maximizare, DWM | `rust/fereastra/` (un singur fisier Win32 — aceeasi categorie de bug) |
