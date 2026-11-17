# GMotion — rescriere in Rust

Rescrierea GMotion in Rust, ca platforma pe module.

Porneste din aceleasi commit-uri ca repo-ul original (`GMotion`), care rămâne
versiunea Python in productie. Codul Python e pastrat aici neatins, ca
referinta: din el se valideaza fiecare bucata portata.

**Originalul:** `QvBnM/GMotion`, tag `v3.4`. Problemele versiunii Python sunt
documentate in README-ul de acolo; aici le reiau doar cand explica o decizie de
proiectare.

---

## De ce rescriere, si nu reparatii

Trei motive, in ordinea greutatii.

**1. Motorul de comparare exista in doua copii care au divergat deja.**
`motor.py` (backend: lista, scoruri, rezumate) si portul lui din `ui.py`
(editorul live) ar trebui sa dea rezultate identice. Masurat pe 24 de cazuri:
8 diverg. In doua, diferenta ajunge in fata utilizatorului — un comentariu bloc
pe mai multe linii face `motor.py` sa raporteze doua blocuri sterse care nu
exista. Vezi `rust/referinta/README.md`.

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
comparabila e ~4x, nu ordinele de magnitudine la care te-ai aștepta. Cifrele si
explicatia sunt in `rust/referinta/README.md`.

---

## Stare

| Componenta | Stare |
|---|---|
| `rust/core` — `gmotion-core`, motorul de comparare | **gata, 31 teste** |
| `rust/referinta` — harness de conformitate + bench | **gata** |
| `rust/db` — conexiune SQL Server, descoperire instante, citire obiecte | de scris |
| `rust/platforma` — sesiuni DPAPI, configurare, registru de module | de scris |
| `rust/fereastra` — fereastra nativa, teme, componente | de scris |
| `rust/module/diff-obiecte` — modulul 1 | de scris |
| `rust/app` — binarul | de scris |

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
si trece.

---

## Rulare

```
cd rust
cargo test                              # 31 teste, inclusiv conformitatea cu motorul JS
cargo clippy --all-targets              # fara avertismente
cargo run --release --example bench     # masuratori (cere corpus_bench.json)
```

Regenerarea fixture-urilor, cand se schimba `ui.py` sau `motor.py` in original:

```
python3 -I rust/referinta/genereaza_corpus.py   # corpus de bench (negitat)
node     rust/referinta/genereaza.mjs           # fixture de referinta (motorul JS)
python3 -I rust/referinta/genereaza.py          # rezultatele motorului Python
python3 -I rust/referinta/compara.py            # raport de divergente
```

Versiunea Python se ruleaza ca inainte: `pip install -r requirements.txt`,
`python app.py`.

---

## Structura tinta

Doua decizii stau la baza:

1. **Platforma vs module.** Ce e comun tuturor uneltelor sta in platforma.
   Fiecare unealta e un crate separat. Adaugarea modulului 2 nu trebuie sa
   atinga modulul 1.
2. **Un fisier se separa daca are o cauza de eroare proprie.** Nu despart ca sa
   bifez module. De-asta `core` are `normalizare` / `segmentare` / `diff`
   separat — se strica fiecare altfel: fals „modificat", blocuri taiate prost,
   blocuri imperecheate greșit. Partea de Win32, in schimb, va fi un singur
   fisier: DPI, monitoare, DWM si handle-uri sunt aceeasi categorie de bug.

```
rust/
  Cargo.toml                workspace
  core/          GATA      normalizare, segmentare, diff, seqmatch
  referinta/     GATA      harness de conformitate + bench
  db/                      conexiune, descoperire instante, citire obiecte
  platforma/               sesiuni (DPAPI), configurare, registru de module
  fereastra/               fereastra nativa, teme, componente comune
  module/
    diff-obiecte/          modulul 1: orchestrare + ecrane + generator de deploy
    upgrade/               modulul 2
    alocare-multidim/      modulul 3
    stoc/                  modulul 4
  app/                     binarul: porneste fereastra si ruteaza catre module
```

### Contractul de modul

**Platforma ofera:** o conexiune deja stabilita (modulul nu reintreaba
utilizatorul), un spatiu de nume propriu pentru persistenta, o zona de ecran si
navigatia catre ea, jurnalul, temele si fereastra.

**Modulul declara:** nume si iconita, ecranele lui si cel de intrare, ce expune
catre interfata, si de ce are nevoie de la platforma (ex: „una sau mai multe
baze", „o singura baza", „doua baze si o mapare").

### Doua lucruri de corectat fata de versiunea Python

**Sesiunile se vor ciocni.** Cheia de sesiune e acum doar numele bazei de
referinta (`_cale_sesiune(referinta)` -> `<slug>.dat`). Daca modulul Upgrade
salveaza progres pe aceeasi baza, suprascrie sesiunea modulului de diff. Cheia
trebuie sa devina `<modul>/<slug>.dat`.

**Expunerea catre interfata trebuie sa devina rutare.** Clasa `Api` din
`app.py` expune metodele modulului de diff direct la nivelul de sus. Cu patru
module, metodele comune (`test_conexiune`, `cauta_servere`, `progres`,
`log_lines`, controalele de fereastra) stau la nivelul platformei si se scriu o
singura data, iar cele specifice se inregistreaza cu prefix.

---

## Decizii nerezolvate

**Driver SQL Server.** `tiberius` (TDS nativ, pur Rust) ar elimina complet
cerinta de driver ODBC pe masina tinta, si are suport pentru descoperirea
instantelor prin SQL Browser — ceea ce inlocuieste broadcast-ul UDP 1434 scris
de mana in `app.py`. Alternativa, `odbc-api`, pastreaza comportamentul identic
cu cel de azi, inclusiv autentificarea integrata Windows, dar rămâne dependenta
de driverul instalat. De testat pe un server real inainte de a alege.

**Toolkit-ul de interfata — riscul principal.** Lista de obiecte, filtrele,
conectarea si deployer-ul se fac direct in orice toolkit nativ. Editorul de
diferente nu: cel actual e un `textarea` cu un strat de evidentiere suprapus
exact peste el, cu evidentiere pe token in interiorul liniei, numere de linie,
minimap, conectori SVG intre panouri si scroll sincronizat pe ancore. Niciun
toolkit nativ nu da asta din cutie — va fi nevoie de un view de text desenat
manual.

Pentru RDP si servere fara GPU, toolkit-ul trebuie sa aiba **renderer
software** (Slint are). **Tauri nu e o opțiune:** foloseste WebView2 pe
Windows, deci ar reintroduce exact dependenta pe care rescrierea o elimina.

---

## Plan

Fiecare etapa = un branch si un PR, cu codul functional la final.

1. **Harness de conformitate** — dovada divergentei, fixture de referinta.
   *(facut)*
2. **`gmotion-core`** — motorul, validat contra motorului JS. *(facut)*
3. **Prototip de editor de diferente** in toolkit-ul ales. Scop: aflat cat
   costa partea grea, inainte de a investi in restul. E singura componenta cu
   incertitudine mare de efort.
4. **`db`** — conexiune, descoperire instante, citire obiecte. Decizia
   tiberius vs odbc-api se ia aici, cu un test pe un server real.
5. **`platforma`** + contractul de modul — sesiuni cu spatiu de nume, registru,
   rutare.
6. **`module/diff-obiecte`** — orchestrare, ecrane, generator de deploy.
   Primul modul complet in Rust.
7. **Paritate verificata** — aceleasi baze comparate cu ambele versiuni,
   rezultate identice. Abia dupa asta se retrage versiunea Python.
8. **`module/upgrade`** — primul test real al contractului. Daca are nevoie de
   modificari in platforma, contractul de la etapa 5 era greșit si se
   corecteaza acum, cat e ieftin.

Harness-ul de la etapa 1 e plasa de siguranta pentru tot ce urmeaza. Orice
etapa care atinge motorul trebuie sa treaca `cargo test -p gmotion-core`.
