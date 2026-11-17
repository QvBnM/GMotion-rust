# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (178 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic)

---

## Unde suntem

Etapele 0-5 incheiate. **Exista o aplicatie completa si utilizabila**, fara
interfata grafica: binarul `gmotion`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 37 teste, fuzz pe invariante |
| `rust/db` | gata, verificat pe SQL Server real (auth SQL si Windows), 35 teste |
| `rust/platforma` | gata, DPAPI verificat pe Windows, 35 teste |
| `rust/module/diff-obiecte` | gata, 42 teste |
| `rust/cli` | gata, 27 teste, din care 10 pe binarul adevarat |

```
cd rust && cargo build --release -p gmotion-cli
./rust/target/release/gmotion --help
```

**Etapa activa:** `etape/06-paritate.md` — **paritate pe baze reale**. Partea
care se putea face fara Windows e gata: `referinta/paritate.py` ruleaza codul
Python v3.4 **neatins** (`Api._comparatie` din `mostenire/app.py`) pe un
instantaneu si compara cu `gmotion compara`; poarta il ruleaza pe un
instantaneu sintetic. Restul asteapta rulari de la Mihai. Dupa etapa 6 urmeaza
interfata, despre care Mihai a cerut explicit sa vorbim **inainte** de a
incepe — nu se deschide etapa 7 fara discutia asta.

---

## Urmatorul pas concret

Etapa 6 e aproape inchisa. Verificat pe masina lui Mihai (`TS2409\MIHAIG`,
SQL Server 17): auth SQL si integrata, 0 regresii de paritate, **pyodbc si
tiberius aduc aceleasi 42.427 obiecte**, Rust citeste mai repede (19 s fata de
29,6 s). Detalii in `etape/06-paritate.md`.

Ramane un singur criteriu: **fixture-ul `TESTSIMILITUDINE`** (18+18 obiecte,
48,5 KB, extras deja la Mihai in `referinta\instantanee\testsimilitudine.json`)
intra in repo **daca Mihai confirma** ca obiectele pot fi publice in repo.
Atunci: commit-ul il face Mihai (fisierul e doar la el), iar poarta primeste un
pas care ruleaza `paritate.py` si pe el (asteptat: 18 randuri, 10 corecturi,
0 regresii). Daca nu poate intra: criteriul se inchide ca „refuzat", cu
motivul, si ramane doar instantaneul sintetic.

Apoi: etapa se inchide (`Stare: INCHEIATA`, „Ce a iesit altfel"). **Etapa 7
(interfata) nu se deschide fara discutia despre GUI ceruta de Mihai.**

---

## Ce aștept de la Mihai

1. Da sau nu: pot intra `TESTSIMILITUDINE_*` in repo? Daca da:

```
git add referinta\instantanee\testsimilitudine.json
```

```
git commit -m "Fixture TESTSIMILITUDINE din CharismaERP_GetsBet"
```

```
git push
```

2. Optional: in editorul aplicatiei v3.4, `TESTSIMILITUDINE_CaseB_Imbricat` ar
   trebui sa arate `1 modificate`, iar `Payment_BankAccountListAllBanks`
   `1 modificate, 5 adaugate` — confirma ca `motor_js.mjs` e egal cu ce
   ruleaza in aplicatie.
3. Discutia despre GUI.

Rămâne deschis si: paritatea porții pe `cmd` (acum cere Git Bash sau WSL).

---

## Capcane active

Lucruri care te-ar face sa greșești chiar acum, daca nu le stii:

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
