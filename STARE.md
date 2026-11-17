# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (176 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic)

---

## Unde suntem

Etapele 0-5 incheiate. **Exista o aplicatie completa si utilizabila**, fara
interfata grafica: binarul `gmotion`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 37 teste, fuzz pe invariante |
| `rust/db` | gata functional, prima conectare reala in curs, 35 teste |
| `rust/platforma` | gata functional, DPAPI dus-intors verificat pe Windows, 35 teste |
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

Cand vine `instantaneu.json` de la Mihai:

```
cd rust && cargo build --release -p gmotion-cli && cd ..
python3 -I referinta/paritate.py instantaneu.json --referinta <Ref> --gmotion rust/target/release/gmotion --csv export.csv --json raport.json
```

Fiecare `regresie` se repara in Rust (intai `comparare.rs`, apoi `core`).
`corectura` = JS confirma Rust-ul, se noteaza doar. Apoi fixture de regresie
din instantaneu (anonimizat daca Mihai cere) si timpul de citire comparat cu
Python-ul (criteriu in etapa).

**Nu se trece la interfata inainte de paritate.**

---

## Ce aștept de la Mihai

Lista Python nu mai trebuie produsa pe Windows: o produce `paritate.py` din
instantaneu, cu codul vechi. Pe Windows mai e nevoie de **instantaneu**, plus
dovezi pe care nu le pot face de aici. In PowerShell, cu Rust instalat:

```
git clone --branch claude/upbeat-ramanujan-dm6j8c https://github.com/QvBnM/GMotion-rust.git
cd GMotion-rust\rust
cargo test --all
cargo build --release -p gmotion-cli
cd target\release
```

```
.\gmotion.exe diagnostic --json
```

```
$c = Get-Credential -UserName sa -Message "SQL Server"
$env:GMOTION_PAROLA = $c.GetNetworkCredential().Password
.\gmotion.exe baze --server SRV01 --utilizator sa
```

(Pentru autentificare integrata Windows, lasa `--utilizator` afara de tot si
nu seta parola. Daca poti, ruleaza `baze` o data in fiecare fel.)

```
Measure-Command { .\gmotion.exe instantaneu --server SRV01 --utilizator sa --baze Referinta,Client1,Client2 -o instantaneu.json --nota "paritate" } | Select-Object TotalSeconds
```

Trimite incoace:

1. ultimele ~20 de randuri din `cargo test --all` (pe Windows e singura proba
   ca DPAPI cripteaza si decripteaza; daca pica ceva, tot textul testului);
2. ieșirea lui `diagnostic --json` si a lui `baze`;
3. `instantaneu.json` si numarul de secunde din `Measure-Command`;
4. **optional, dar util:** in aplicatia Python, aceleasi baze si aceeasi
   referinta, export CSV **fara niciun filtru activ** in lista, plus cat a
   durat incarcarea (ora din primul si ultimul rand de jurnal). Asta dovedeste
   ca driverul Rust aduce aceleasi obiecte ca `pyodbc`, si da comparatia de
   viteza.

**Daca o comanda cade, trimite mesajul exact.** Are un cod de forma
`GM-DB-001`, si din el ajung direct la cauza.

Daca `instantaneu.json` conține cod pe care nu vrei sa-l pui in repo, spune si
fac un fixture anonimizat din el, ori il folosesc doar pentru verificare.

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
