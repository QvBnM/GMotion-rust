# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (250 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic si pe fixture real)

---

## Unde suntem

Etapele 0-7 incheiate. **Exista o aplicatie completa si utilizabila**, fara
interfata grafica: binarul `gmotion`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 50 teste, fuzz pe invariante, lexer T-SQL |
| `rust/db` | gata, verificat pe SQL Server real (auth SQL si Windows), citire paralela, 48 teste |
| `rust/platforma` | gata, DPAPI verificat pe Windows, 40 teste |
| `rust/module/diff-obiecte` | gata, 59 teste |
| `rust/cli` | gata, 31 teste, din care 12 pe binarul adevarat |
| `rust/fereastra` | editorul de diferente: gata (etapa 8), 19 teste; restul ecranelor, de scris |

```
cd rust && cargo build --release -p gmotion-cli
./rust/target/release/gmotion --help
```

**Etapa activa:** `etape/09-lista-si-previzualizare.md` — ecranul principal
(lista de diferente + previzualizarea), dupa macheta `Main.dc.html`.

**Etapa 8 e incheiata.** Editorul nativ e pe designul aprobat
(`cercetare/design-aprobat.md`): jetoanele si fonturile IBM Plex in locul
culorilor v3.4, plierea sirurilor identice, modul unificat, navigare de la
tastatura, harta diferentelor. Capturi, langa machete:
`cercetare/machete/capturi/`. Cifrele pe obiecte uriase si ce a ieșit altfel:
`etape/08-prototip-editor.md`.

---

## Urmatorul pas concret

**Etapa 9, pasul 1: clasificarea „difera in" in `core`** (cod / sir /
comentariu / setari SET, prin `core::lexer`), cu teste pe cazurile care au
pacalit v3.4 — mai ales `GenerateOrderByRecommendedOrder`, unde singura
diferenta e o majuscula intr-un mesaj din sir. Apoi expusa in `cli` si
folosita in coloana „Difera in" din lista. Criteriile complete:
`etape/09-lista-si-previzualizare.md`.

Rulari pe Windows: nimic de cerut acum. Ultima (`b4b0e30`) a confirmat
citirea paralela (12,3 s fata de 29,3 s in v3.4), garda de versiune, cele
~4.440 de obiecte (21%) cu `QUOTED_IDENTIFIER OFF` si 401 cu `ANSI_NULLS OFF`,
si 0 regresii.

Ramase deschise, documentate: performanta patratica pe obiecte uriase (se
vede acum si in editor: ~3,1 s pentru un obiect de 20.000 de linii, o singura
data, la deschidere), nume case-insensitive intre baze, divergentele Unicode
fata de JS (in afara de BOM), serverul in cheia sesiunii, nume rezervate
Windows ca nume de modul, dependentele prin SQL dinamic sau sinonime.

## Ce aștept de la Mihai

**Decizii vechi, inca deschise:** implicitul pentru `--certificat`;
specificatia modulului `upgrade`; repo-ul public.

**Doua intrebari de design, din etapa 8** (nu blocheaza etapa 9):

1. In modul **unificat** al editorului, cum se marcheaza care linie e din
   sursa si care din baza? Macheta are tabul „Unificat", dar nu si randarea.
   Acum liniile apar una dupa alta, cu numarul lor de linie si culoarea
   blocului, fara alt semn. Un semn in plus (de pildă o litera `S` / `B` in
   jgheab) ar fi un element care nu e in machete, deci nu l-am facut.
2. „47 linii" din macheta sau „47 de linii"? In cod e a doua (forma corecta
   gramatical, ca „22 de linii identice" din aceeasi macheta).

Rămâne deschis si: paritatea porții pe `cmd` (acum cere Git Bash sau WSL).

## Capcane active

Lucruri care te-ar face sa greșești chiar acum, daca nu le stii:

- **Paritatea cu v3.4 nu mai e totala, deliberat.** Normalizarea nu mai atinge
  string-urile, „identic" nu mai foloseste scorul, criptatele au categorie
  proprie. Orice diferenta noua fata de v3.4 intra in `*.asteptari.json` cu
  motivul ei — niciodata „ca sa treaca poarta". Divergentele fata de JS stau
  in `DIVERGENTE_DELIBERATE` din `core/tests/conformitate.rs`.
- **Granitele cod / string / comentariu se decid intr-un singur loc:**
  `core/src/lexer.rs`. O regula noua pe text (in motor sau in deploy) care nu
  trece prin el reintroduce clasa de bug-uri reparata la etapa 7.
- **Testul de conformitate care cade nu inseamna fixture greșit.** Presupunerea
  implicita e ca portul Rust e greșit. Fixture-ul se regenereaza doar explicit,
  prin `unelte/regenereaza-fixture.sh`, cu motivul in `JURNAL.md`.
- **Un container nou nu are tintele wasm32 si windows-msvc.** Poarta le sare
  si iese tot „verde". Citeste lista de verificari sarite; instaleaza-le cu
  `rustup target add wasm32-unknown-unknown x86_64-pc-windows-msvc`.
- **`paritate.py` ruleaza binarul pe care i-l dai, nu sursa.** Dupa o
  modificare in Rust, reconstruieste inainte de a citi raportul — altfel
  masori binarul vechi. (Poarta il reconstruieste singura.)
- **Nu da comenzi cu parola scrisa in ele, nici in PowerShell.**
  `$env:GMOTION_PAROLA = "..."` ajunge in istoricul PSReadLine, pe disc. Parola
  se ia prin `Get-Credential`.
- **Pe Windows ruleaza cu 2 teste mai putin decat pe Linux, si e corect**
  (`auth_integrata_e_refuzata_devreme_pe_linux`, `pe_linux_marcajul_...`). Pe
  Windows in schimb ruleaza `pe_windows_datele_nu_apar_in_clar`.
- **`mostenire/` nu se atinge.** Poarta cade pe orice modificare de conținut
  acolo. Daca ai nevoie de o schimbare in Python, discuta intai: probabil
  insemna ca trebuie regenerate fixture-urile.
- **Versiunea Rust NU e identica cu `mostenire/motor.py`, si asta e corect.**
  Diferentele sunt corecturi de bug — portul urmeaza semantica motorului JS.
  Nu „repara" Rust-ul ca sa semene cu `motor.py`.
- **`rust/core` nu primeste I/O.** Un singur `std::fs` acolo rupe tinta wasm si
  readuce exact problema celor doua motoare divergente.
- **`referinta/corpus_bench.json` nu e in git.** Daca `cargo run --example bench`
  se plange, ruleaza `python3 -I referinta/genereaza_corpus.py`.
- **Un bloc returnat de `split_statements` nu e garantat subsir al textului
  original** (blocurile unite se lipesc cu un singur `\n`). De-aia
  `linii_start_blocuri` are o ramura de rezerva. Conteaza la etapa de editor:
  ancorele de scroll degradeaza silentios acolo. Vezi
  `etape/01-core-consolidat.md`.
- **Codul in spatele unui `cfg` pe alta tinta nu e verificat de nimic local.**
  Nici `build`, nici `clippy`, nici testele se uita la el. Poarta ruleaza
  `cargo check` si pe windows-msvc, dar verificarea e **sarita in modul
  `rapid`** — deci inainte de a declara gata cod `cfg(windows)`, rulezi poarta
  completa.
- **Un test care umbla la variabile de mediu trebuie serializat cu un lacat.**
  `cargo test` ruleaza in paralel in acelasi proces. Vezi
  `rust/platforma/tests/sesiuni.rs`.
- **Nu doar mediul e stare globala intre teste.** Tamponul de jurnal din
  `platforma` si abonatul `tracing` (instalat de primul test care cheama
  `porneste`) sunt partajate de tot procesul. Un test care presupune ca nu
  scrie nimeni altcineva acolo cade ~1 data din 100. Logica se testeaza pe o
  instanta locala.
- **Un tip care tine acreditari nu are `Debug` derivat.** Ar tipari parola la
  primul `dbg!` sau in orice mesaj de panica. Se scrie de mana, cu redactare.
  Vezi `rust/db/src/sqlserver.rs`.
- **O conectare care merge pe Linux nu dovedeste nimic despre Windows.**
  Furnizorul de TLS difera intre tinte (`rustls` vs SChannel), si auth integrata
  nu exista pe Linux deloc.
- **O masuratoare de virtualizare cu plierea pornita nu masoara nimic.**
  `pliaza-identicul` lasa in lista ~160 de randuri dintr-un obiect de 30.000:
  timpul pe cadru iese plat si cu `ListView`, si fara el. `--example masura`
  ruleaza de-aia si nepliat; acolo se vede diferenta (12 ms fata de 711 ms).
- **In `.slint` nu se scrie nicio culoare.** Jetoanele stau in
  `rust/fereastra/src/jetoane.rs`, unde testul de contrast le vede; `Tema` din
  Slint n-are valori implicite, tocmai ca o culoare uitata sa iasa
  transparenta si sa cada testul, nu sa semene cu ceva.
- **Editorul arata culorile in sensul deploy-ului, motorul in sensul
  referintei.** Verde = doar in sursa (se va adauga in baza) = `Stare::Sters`
  in model. Incrucisarea e doar in `cod_stare` din `fereastra/src/lib.rs`.
  Daca o muti sau o dublezi, culorile se inverseaza fara ca vreun test de
  motor sa cada.
- **Un fuzz care trece nu e un fuzz care functioneaza.** Cand adaugi un test de
  invariante, strica deliberat codul si uita-te daca il prinde. Prima versiune a
  fuzz-ului de segmentare trecea si cu urmarirea literalelor scoasa complet.
