# GMotion v4 — rescriere in Rust

Unealta pentru compararea obiectelor SQL Server (proceduri, view-uri, functii,
triggere) din mai multe baze de date modificate fata de o baza martor, si
pentru generarea scripturilor de deploy.

Versiunea in productie e GMotion v3.4, in Python. Codul ei e **inghetat** in
`mostenire/`, ca referinta de paritate; vezi `mostenire/MOSTENIRE.md`.

**Reia lucrul de unde a rams:** `STARE.md`, apoi etapa activa din `etape/`,
apoi `unelte/verifica.sh`. Ritualul complet e in `CLAUDE.md`, sectiunea 0.

| Document | Ce contine |
|---|---|
| `STARE.md` | unde suntem acum, urmatorul pas concret, ce e blocat, capcane active |
| `etape/NN-*.md` | scopul si criteriile de terminare ale fiecarei etape, plus jurnalul ei de lucru |
| `CLAUDE.md` | regulile de lucru. Contractul — se citeste inainte de orice schimbare |
| `ARHITECTURA.md` | harta crate-urilor, directia dependentelor, si de ce e asa |
| `HARTA.md` | simptom -> fisierul vinovat. Indexul de depanare |
| `JURNAL.md` | deciziile luate, cu alternativele si motivul. Append-only |
| `<crate>/CRATE.md` | contractul fiecarui crate: rol, API, invariante, depanare |

---

## De ce rescriere, si nu reparatii

Trei motive, in ordinea greutatii.

**1. Motorul de comparare exista in doua copii care au divergat deja.**
`motor.py` (backend: lista, scoruri, rezumate) si portul lui din `ui.py`
(editorul live) ar trebui sa dea rezultate identice. Masurat pe 24 de cazuri:
8 diverg. In doua, diferenta ajunge in fata utilizatorului — un comentariu bloc
pe mai multe linii face `motor.py` sa raporteze doua blocuri sterse care nu
exista. Vezi `referinta/README.md`.

Rust rezolva asta structural: acelasi cod compileaza in binar nativ si in
WebAssembly, deci backendul si editorul live folosesc fizic acelasi motor. Nu
mai pot diverge.

**2. Dependentele de pe masina tinta.** Versiunea Python cere WebView2 Runtime
si un driver ODBC instalate pe server. O fereastra nativa cu renderer software
nu cere niciunul, iar un driver TDS nativ (`tiberius`) ar scoate si ODBC din
ecuatie. Rezultatul e un binar mic, autonom, care merge si pe un Server 2012
curat.

**3. Memoria.** `aduce_obiecte()` tine, pentru fiecare obiect din fiecare baza,
o lista Python de token-uri pe langa text. Fiecare token e un obiect separat cu
antetul lui. In Rust token-urile sunt felii din acelasi tampon.

Viteza bruta **nu** e un motiv puternic: masurat, castigul pe munca
comparabila e ~4x, nu ordinele de magnitudine la care te-ai astepta. Cifrele si
explicatia sunt in `referinta/README.md`.

---

## Stare

| Componenta | Stare |
|---|---|
| `mostenire/` — GMotion v3.4 in Python | inghetat, referinta de paritate |
| `referinta/` — harness de conformitate + bench | **gata** |
| `unelte/` — poarta de verificare | **gata** |
| `rust/core` — `gmotion-core`, motorul de comparare | **gata, 50 teste** |
| `rust/db` — `SursaObiecte`: SQL Server + fixture, descoperire instante | **gata, verificat pe SQL Server real** (48 teste) |
| `rust/platforma` — configurare, sesiuni, jurnal, registru, rutare | **gata, DPAPI verificat pe Windows** (40 teste) |
| `rust/module/diff-obiecte` — modulul 1, ca biblioteca | **gata, 59 teste** |
| `rust/cli` — binarul headless | **gata, 31 teste** |
| `rust/fereastra` — fereastra nativa, teme, componente | de scris |
| `rust/app` — binarul cu interfata | de scris |
| `rust/module/upgrade` — modulul 2 | de scris |

O componenta e **gata** doar cand are teste, `CRATE.md`, si poarta verde.

### Ce acopera `gmotion-core`

```
seqmatch.rs     difflib.SequenceMatcher fidel (autojunk=False)
normalizare.rs  norm, pregateste_afisare, hash, rotunjire bancara, scor
segmentare.rs   split pe instructiuni, cu urmarirea literalelor,
                comentariilor bloc, parantezelor si stivei CASE/BEGIN/END
diff.rs         diferente in interiorul blocului, aliniere DP, rezumate
```

Porteaza **semantica JS**, nu pe cea din `motor.py`. Portul JS e varianta mai
noua si corecta; `motor.py` a rams in urma. Testul de conformitate verifica
toate cele 24 de cazuri pe 8 campuri contra fixture-ului produs de motorul JS,
si trece. Detalii in `rust/core/CRATE.md`.

---

## Rulare

**Aplicatia** (fara interfata grafica, functionala):

```
cd rust && cargo build --release -p gmotion-cli
./target/release/gmotion --help
./target/release/gmotion diagnostic
```

**Fereastra** — intra direct in lista, pe un instantaneu sintetic, fara
server:

```
unelte\porneste.bat             # Windows
unelte/porneste.sh              # Linux / WSL (cere ecran)
unelte\porneste.bat --tema luminos
unelte\porneste.bat --server    # ecranul de conectare, gol
```

**Poarta** — se ruleaza inainte de fiecare commit:

```
unelte/verifica.sh              # Linux / WSL / Git Bash
unelte\verifica.bat             # Windows (varianta redusa)
unelte/verifica.sh rapid        # fara wasm si fara harness
```

Verifica format, clippy fara avertismente, toate testele, tinta `wasm32` pentru
`core`, `cargo check` pe `x86_64-pc-windows-msvc` (codul `cfg(windows)` nu e
verificat de nimic altceva pe Linux), directia dependentelor, sincronizarea fiecarui `CRATE.md` cu API-ul lui
public, coerenta starii (`STARE.md` plus exact o etapa activa), si faptul ca
`mostenire/` n-a fost atins.

```
unelte/stare.sh                 # faptele mecanice de pus in STARE.md
```

**Doar motorul:**

```
cd rust
cargo test -p gmotion-core
cargo run --release --example bench     # cere referinta/corpus_bench.json
```

**Harness-ul de referinta** (regenerare explicita, cu motiv notat in `JURNAL.md`):

```
unelte/regenereaza-fixture.sh
python3 -I referinta/compara.py             # raport de divergente Python vs JS
python3 -I referinta/genereaza_corpus.py    # corpus de bench (negitat)
```

**Versiunea Python**, ca inainte:

```
pip install -r mostenire/requirements.txt
python mostenire/app.py
```

---

## Structura

```
STARE.md CLAUDE.md ARHITECTURA.md HARTA.md JURNAL.md
etape/               scopul, criteriile si jurnalul fiecarei etape

mostenire/           GMotion v3.4 in Python — INGHETAT, nu se modifica
referinta/           harness de conformitate: Python vs JS, fixture, bench
unelte/              poarta de verificare, regenerare fixture
rust/
  Cargo.toml         workspace
  core/     GATA     normalizare, segmentare, diff, seqmatch — logica pura
  db/                SursaObiecte: SQL Server | fixture; descoperire instante
  platforma/         configurare, sesiuni, jurnal, registru de module, rutare
  cli/               binar headless: compara, instantaneu, diagnostic, deploy
  fereastra/         fereastra nativa, teme, componente comune
  module/
    diff-obiecte/    modulul 1: orchestrare + ecrane + generator de deploy
    upgrade/         modulul 2
    alocare-multidim/
    stoc/
  app/               binarul cu interfata
```

Directia dependentelor, verificata mecanic de poarta:

```
core  <-  db  <-  platforma  <-  module/*  <-  app, cli
```

Nimic nu depinde la dreapta. **Niciun modul nu depinde de alt modul** — daca
are nevoie de ceva din alt modul, acel ceva apartine platformei.

### Contractul de modul

**Platforma ofera:** o conexiune deja stabilita (modulul nu reintreaba
utilizatorul), un spatiu de nume propriu pentru persistenta, o zona de ecran si
navigatia catre ea, jurnalul, temele si fereastra.

**Modulul declara:** nume si iconita, ecranele lui si cel de intrare, ce expune
catre interfata, si de ce are nevoie de la platforma (ex: „una sau mai multe
baze", „o singura baza", „doua baze si o mapare").

### Tot ce face aplicatia trebuie sa mearga fara interfata

`db` expune `SursaObiecte` ca trait, cu doua implementari: SQL Server real, si
fixture din JSON. Deci tot ce e deasupra `db` se testeaza fara baza de date.

`gmotion instantaneu` scrie obiectele brute dintr-o comparare reala intr-un
fisier; `gmotion compara --sursa fisier` il reia offline. Un bug de pe o baza de
productie devine astfel un test de regresie, reproductibil pe orice masina.

De aceea `cli` se face **inaintea** interfetei.

---

## Plan

Fiecare etapa = un branch si un PR, cu cod functional la final si poarta verde.

| # | Etapa | Rezultat verificabil |
|---|---|---|
| 0 | **Restructurare + reguli** | schelet, `mostenire/` inghetat, documentele de regula, poarta, urmarirea starii. `core` mutat neatins. *(facut)* |
| 1 | **`core` consolidat** | antete de fisier, `CRATE.md`, build `wasm32` verificat, fuzz pe segmentare. Fara schimbari de semantica |
| 2 | **`db`** | `SursaObiecte` + impl fixture + impl SQL Server, descoperire instante. Decis `tiberius`. *(facut; verificat pe server real la etapa 6)* |
| 3 | **`platforma`** | configurare, sesiuni cu cheie `<modul>/<slug>.dat`, jurnal, registru, rutare cu prefix. *(facut)* |
| 4 | **`module/diff-obiecte`, fara UI** | orchestrarea completa ca biblioteca, testata integral pe fixture. *(facut)* |
| 5 | **`cli`** | `compara`, `instantaneu`, `diagnostic`, `deploy`, `export`. **Aplicatie completa si utilizabila.** *(facut)* |
| 6 | **Paritate pe baze reale** | aceleasi baze prin Python si prin CLI, rezultate identice sau diferente explicate ca corecturi. Abia aici se poate retrage Python. *(facut: 0 regresii pe 21.193 obiecte reale)* |
| 7 | **Remedierea auditului** | constatarile critice si inalte din auditul de la `f561bf9` reparate, cu teste vazute cazand pe codul vechi; divergentele deliberate fata de v3.4 listate explicit. *(facut)* |
| 8 | **`module/upgrade`** | primul test real al contractului de modul. Daca cere modificari in platforma, contractul de la etapa 3 era gresit si se corecteaza acum, cat e ieftin. **Asteapta specificatia de la Mihai.** |
| 9 | **Prototip editor diff** | doar editorul, in toolkitul candidat. Scop: aflat costul real al partii grele. Dupa discutia despre GUI |
| 10 | **`fereastra` + interfata** | UI complet, cu `cli` ramas permanent ca mod de depanare |

GUI-ul (9-10) e la final, la decizia lui Mihai din 2026-11-17.

Harness-ul din `referinta/` e plasa de siguranta pentru tot ce urmeaza. Orice
etapa care atinge motorul trebuie sa treaca `cargo test -p gmotion-core`.

---

## Decizii nerezolvate

Toate sunt notate, cu alternativele si motivul, in `JURNAL.md`. Driverul SQL
Server s-a decis la etapa 2: `tiberius`, TDS nativ, fara ODBC pe masina tinta.

**Toolkit-ul de interfata — riscul principal.** Lista de obiecte, filtrele,
conectarea si deployer-ul se fac direct in orice toolkit nativ. Editorul de
diferente nu: cel actual e un `textarea` cu un strat de evidentiere suprapus
exact peste el, cu evidentiere pe token in interiorul liniei, numere de linie,
minimap, conectori SVG intre panouri si scroll sincronizat pe ancore. Niciun
toolkit nativ nu da asta din cutie — va fi nevoie de un view de text desenat
manual.

Pentru RDP si servere fara GPU, toolkit-ul trebuie sa aiba **renderer
software** (Slint are). **Tauri nu e o optiune:** foloseste WebView2 pe
Windows, deci ar reintroduce exact dependenta pe care rescrierea o elimina.
