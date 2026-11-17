# Arhitectura

Harta repo-ului: ce exista, ce depinde de ce, si de ce e asa. Pentru reguli vezi
`CLAUDE.md`; pentru „unde caut un simptom" vezi `HARTA.md`.

---

## Doua decizii de la baza

**1. Platforma vs module.** Ce e comun tuturor uneltelor sta in platforma.
Fiecare unealta e un crate separat. Adaugarea modulului 2 nu trebuie sa atinga
modulul 1 — iar daca o atinge, contractul de modul era greșit.

**2. Un fisier se separa doar daca are o cauza de eroare proprie.** Nu
despicam ca sa bifam module.

---

## Direcția dependentelor

```
                       +-----------+
                       |   core    |  logica pura, fara I/O, merge pe wasm
                       +-----------+
                             ^
                       +-----------+
                       |    db     |  SursaObiecte: SQL Server | fixture
                       +-----------+
                             ^
                       +-----------+
                       | platforma |  config, sesiuni, jurnal, registru, rutare
                       +-----------+
                             ^
              +--------------+--------------+
              |              |              |
     +----------------+  +--------+  +-------------+
     | diff-obiecte   |  | upgrade|  |  ...        |   module/*
     +----------------+  +--------+  +-------------+
              ^                             ^
              +--------------+--------------+
                             |
                     +---------------+
                     |   cli | app   |  binare
                     +---------------+
```

Nimic nu depinde la dreapta. Niciun modul nu depinde de alt modul. Verificat
mecanic de `unelte/verifica.sh`.

---

## Harta pe directoare

| Cale | Ce e | Stare |
|---|---|---|
| `mostenire/` | GMotion v3.4 in Python, **inghetat**. Referinta de paritate. | inghetat |
| `referinta/` | Harness de conformitate: aceleasi cazuri prin motorul Python si prin cel JS, plus fixture-ul contra caruia se valideaza Rust-ul. Si bench. | gata |
| `rust/core/` | `gmotion-core` — motorul de comparare. Logica pura. | gata, 31 teste |
| `rust/db/` | Conexiune SQL Server, descoperire instante, citire obiecte. `SursaObiecte` ca trait, cu implementare reala si una din fixture. | de scris |
| `rust/platforma/` | Configurare, sesiuni (DPAPI), jurnal, registru de module, rutare. | de scris |
| `rust/module/` | Un crate per unealta. | de scris |
| `rust/cli/` | Binarul headless: `compara`, `instantaneu`, `diagnostic`, `deploy`, `export`. | de scris |
| `rust/fereastra/` | Fereastra nativa, teme, componente comune. | de scris |
| `rust/app/` | Binarul cu interfata: porneste fereastra, ruteaza catre module. | de scris |
| `unelte/` | Poarta de verificare, urmarirea stării, scripturile de regenerare. | gata |
| `etape/` | Scopul, criteriile de terminare si jurnalul fiecarei etape. Exact una e `ACTIVA`. | gata |

---

## De ce `SursaObiecte` e un trait

Mediul de dezvoltare al asistentului e Linux, fara SQL Server si fara fereastra.
Daca singura cale de a rula logica trece prin ODBC si printr-un GUI Windows,
fiecare bug devine „ruleaza tu si spune-mi ce vezi" — cel mai scump mod de
lucru.

Cu `SursaObiecte` ca trait si doua implementari — SQL Server real, si fixture
din JSON — tot ce e deasupra `db` se testeaza fara baza de date. Iar
`gmotion instantaneu` transforma un bug de pe o baza reala intr-un fixture
reproductibil, deci intr-un test de regresie.

Asta e si motivul pentru care `cli` se face **inaintea** interfetei.

---

## De ce `core` trebuie sa mearga pe wasm

In v3.4 motorul de comparare exista in doua copii: `mostenire/motor.py`
(backend) si portul lui in JS din `mostenire/ui.py` (editorul live). Pe 24 de
cazuri masurate, 8 diverg, 2 vizibil pentru utilizator. Detalii in
`referinta/README.md`.

Acelasi crate compilat in binar nativ si in WebAssembly inseamna ca backendul
si editorul live folosesc **fizic** acelasi motor. Nu mai pot diverge. De aceea
`core` nu are I/O: un singur `std::fs` acolo rupe tinta wasm si readuce
problema.

---

## Decizii deschise

Sunt notate, cu alternativele, in `JURNAL.md`. Pe scurt:

- **Driver SQL Server:** `tiberius` (TDS pur Rust, scoate ODBC din ecuatie si
  are descoperire de instante) vs `odbc-api` (comportament identic cu azi,
  inclusiv autentificare integrata Windows, dar ramane dependenta de driverul
  instalat). Se decide la etapa `db`, cu test pe server real.
- **Toolkit de interfata:** riscul principal. Editorul de diferente (textarea
  cu strat de evidentiere suprapus, evidentiere pe token, minimap, conectori
  SVG, scroll sincronizat) nu exista in niciun toolkit nativ din cutie. Trebuie
  renderer software, pentru RDP si servere fara GPU. **Tauri e exclus:**
  foloseste WebView2, deci reintroduce exact dependenta pe care rescrierea o
  elimina.
