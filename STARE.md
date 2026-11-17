# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (228 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic si pe fixture real)

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

**Etapa activa:** niciuna. Etapa 7 (remedierea auditului de la `f561bf9`)
e **incheiata**: toate constatarile critice si inalte reparate, cu teste.
Detalii: `etape/07-remediere-audit.md`. Urmatoarea: 8 (`module/upgrade`),
blocata pe specificatie. GUI-ul (9-10) e la final, la decizia lui Mihai.

---

## Urmatorul pas concret

A doua rulare pe Windows (dupa etapa 7): **0 regresii**, aceleasi 42.427 de
obiecte prin pyodbc si tiberius, instanta numita rezolvata de codul propriu,
auth SQL si Windows. 45 de diferente „de verificat" (scoruri si rezumate pe
obiecte cu string-uri) si **o modificare pe care v3.4 o ascundea**:
`dbo.GenerateOrderByRecommendedOrder` (v3.4: identic, v4: diferit).

Dupa rulare s-au facut si cele trei puncte deschise ale deploy-ului (setarile
de creare per obiect, ordinea dupa dependente, garda de versiune) si citirea
paralela (pe reteaua lui Mihai v4 era mai lent decat v3.4: 48,6 s fata de
28,9 s).

1. **Mihai ruleaza din nou `unelte/etapa6.ps1`.** Pasi noi: 7 (garda de
   versiune rulata pe `master`, doar citire, ca atare si cu versiunea
   simulata 12.0) si 8 (cate obiecte au `ANSI_NULLS`/`QUOTED_IDENTIFIER` OFF).
   De urmarit: timpul de la 4a (asteptat sub 48,6 s), pasul 7 „GARDA
   CORECTA".
2. **Mihai se uita la `GenerateOrderByRecommendedOrder`** (comanda in „Ce
   astept"), si spune doar daca diferenta e reala — nu trebuie trimis codul.
3. **Decizii de la Mihai:** implicitul pentru `--certificat`; specificatia
   modulului `upgrade`; repo-ul public.
4. Ramase deschise (documentate): performanta patratica pe obiecte uriase,
   nume case-insensitive intre baze, divergentele Unicode fata de JS (in
   afara de BOM), serverul in cheia sesiunii, nume rezervate Windows ca nume de
   modul, dependentele prin SQL dinamic sau sinonime (nu se vad in cod).

**Nu se deschide etapa de interfata fara discutia despre GUI ceruta de Mihai.**

---

## Ce aștept de la Mihai

```
git pull
```

```
powershell -ExecutionPolicy Bypass -File unelte\etapa6.ps1
```

Si, pentru obiectul ascuns de v3.4 (in aceeasi fereastra, din radacina):

```
rust\target\release\gmotion.exe diferente --sursa rust\target\release\instantaneu.json --referinta CharismaERP_GetsBet --baza CharismaERP_GetsBet_Copy --obiect dbo.GenerateOrderByRecommendedOrder
```

Trimite `etapa6-rezultat.txt`; despre `diferente` ajunge o propozitie (ce
difera: majuscule intr-un string, cod dupa un `--` dintr-un string, alt
lucru). Daca o comanda cade, mesajul exact (are un cod `GM-...`).

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
