# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (230 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic si pe fixture real)

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

```
cd rust && cargo build --release -p gmotion-cli
./rust/target/release/gmotion --help
```

**Etapa activa:** `etape/08-prototip-editor.md` — prototipul nativ (Slint) al
editorului de diferente. Mihai a decis (2026-11-17): interfata nativa, fara
WebView2, aspect apropiat de v3.4 dar poate fi mai frumos, si se incepe acum,
inaintea modulului `upgrade`. Cercetarea de design (tool-uri de referinta,
vizualizarea diferentelor, review la scara, deploy sigur, design vizual,
fezabilitate Slint): `cercetare/design-interfata.md`.

**Designul e aprobat** (Mihai, 2026-11-17): `cercetare/design-aprobat.md` +
machetele din `cercetare/machete/`. E contractul vizual; nu se deseneaza un
ecran care nu e acolo fara sa intrebi. Atentie: prototipul existent din
`rust/fereastra/ui/editor.slint` foloseste inca culorile v3.4 — se aliniaza la
jetoanele aprobate.

---

## Urmatorul pas concret

A treia rulare pe Windows (`b4b0e30`) a confirmat tot ce era deschis:
citirea paralela **12,3 s** (fata de 48,6 s secvential si 29,3 s in v3.4, pe
aceeasi retea); garda de versiune corecta pe ambele ramuri pe serverul real;
**~4.440 de obiecte (21%) au `QUOTED_IDENTIFIER OFF`** si 401
`ANSI_NULLS OFF` — un deploy v3.4 rulat din SSMS le schimba comportamentul;
0 regresii. Obiectul ascuns de v3.4 (`GenerateOrderByRecommendedOrder`):
diferenta e intr-un mesaj din string (`'... pentru partenerul '`), adica
exact ce normalizarea v3.4 inghitea. `gmotion diferente` nu arata insa ce s-a
schimbat; reparat (ambele versiuni, `[-...-]` / `{+...+}`, inclusiv spatiile
din string-uri).

1. ~~`diferente` pe `GenerateOrderByRecommendedOrder`~~ — rulat de Mihai:
   `'nu s-au gasit ...'` fata de `'Nu s-au gasit ...'`, majuscula intr-un
   mesaj din string. Confirmat: exact ce v3.4 ascundea. Afisarea noua
   (`[-nu-]` / `{+Nu+}`) merge pe Windows.
2. **Decizii de la Mihai:** implicitul pentru `--certificat`; specificatia
   modulului `upgrade`; repo-ul public.
3. Ramase deschise (documentate): performanta patratica pe obiecte uriase,
   nume case-insensitive intre baze, divergentele Unicode fata de JS (in
   afara de BOM), serverul in cheia sesiunii, nume rezervate Windows ca nume de
   modul, dependentele prin SQL dinamic sau sinonime.

**Urmatorul pas:** etapa 8 se termina pe designul aprobat (jetoanele si
fonturile in `editor.slint`, navigare, virtualizare pe 20.000 de linii, captura
noua), apoi se deschide etapa 9 — lista si previzualizarea (`Main.dc.html`).

---

## Ce aștept de la Mihai

Deciziile de la punctul 2: implicitul pentru `--certificat`, specificatia
modulului `upgrade`, repo-ul public. Nimic de rulat.

Rămâne deschis si: paritatea porții pe `cmd` (acum cere Git Bash sau WSL).

---

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
- **Un fuzz care trece nu e un fuzz care functioneaza.** Cand adaugi un test de
  invariante, strica deliberat codul si uita-te daca il prinde. Prima versiune a
  fuzz-ului de segmentare trecea si cu urmarirea literalelor scoasa complet.
