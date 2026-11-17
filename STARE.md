# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (168 teste; `core` pe wasm32, tot codul si pe windows-msvc)

---

## Unde suntem

Etapele 0-5 incheiate. **Exista o aplicatie completa si utilizabila**, fara
interfata grafica: binarul `gmotion`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 37 teste, fuzz pe invariante |
| `rust/db` | gata functional, **neverificat pe tinta**, 30 teste |
| `rust/platforma` | gata functional, **DPAPI neverificat pe tinta**, 32 teste |
| `rust/module/diff-obiecte` | gata, 42 teste |
| `rust/cli` | gata, 27 teste, din care 10 pe binarul adevarat |

```
cd rust && cargo build --release -p gmotion-cli
./rust/target/release/gmotion --help
```

**Etapa activa:** niciuna. Urmatoarea la rand e etapa 6, **paritate pe baze
reale** — si pentru ea e nevoie de tine (vezi mai jos). Dupa ea urmeaza
interfata, despre care Mihai a cerut explicit sa vorbim inainte de a incepe.

---

## Urmatorul pas concret

**Etapa 6 — paritate pe baze reale.** Nu se poate face de aici: are nevoie de o
rulare pe un SQL Server adevarat. Pasii, in ordine:

1. Mihai construieste binarul pe Windows si ruleaza comenzile din secțiunea
   urmatoare.
2. Trimite incoace: ieșirea lui `gmotion diagnostic --json`, un
   `gmotion instantaneu`, si lista de obiecte din versiunea Python pe aceleasi
   baze.
3. Eu compar cele doua liste. Fiecare diferenta se explica: ori e o corectura
   de bug (motorul JS era corect, `motor.py` nu), ori e o regresie si se
   repara.
4. Instantaneul devine fixture de regresie in repo (anonimizat, daca e nevoie).

**Nu se trece la interfata inainte de paritate.** Dupa paritate, versiunea
Python poate fi retrasa, si abia asta face rescrierea sa aiba sens.

---

## Ce aștept de la Mihai

Acum e nevoie de tine, prima data in tot proiectul. Pe Windows, cu Rust
instalat:

```
git clone <repo> && cd GMotion-rust/rust
cargo build --release -p gmotion-cli
```

Binarul iese in `rust/target/release/gmotion.exe`. Apoi, in ordine:

```
gmotion diagnostic --json
```

```
gmotion servere
```

```
set GMOTION_PAROLA=parola-ta
gmotion baze --server SRV01 --utilizator sa
```

(Pentru autentificare integrata Windows, lasa `--utilizator` afara de tot.)

```
gmotion instantaneu --server SRV01 --baze Referinta,Client1,Client2 -o instantaneu.json --nota "paritate"
```

```
gmotion compara --sursa instantaneu.json --referinta Referinta --json > rust.json
```

Trimite incoace `diagnostic`, `rust.json` si `instantaneu.json`, plus lista de
obiecte pe care o da versiunea Python pe aceleasi baze (exportul CSV din
`mostenire/app.py` ajunge).

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
- **Un tip care tine acreditari nu are `Debug` derivat.** Ar tipari parola la
  primul `dbg!` sau in orice mesaj de panica. Se scrie de mana, cu redactare.
  Vezi `rust/db/src/sqlserver.rs`.
- **O conectare care merge pe Linux nu dovedeste nimic despre Windows.**
  Furnizorul de TLS difera intre tinte (`rustls` vs SChannel), si auth integrata
  nu exista pe Linux deloc.
- **Un fuzz care trece nu e un fuzz care functioneaza.** Cand adaugi un test de
  invariante, strica deliberat codul si uita-te daca il prinde. Prima versiune a
  fuzz-ului de segmentare trecea si cu urmarirea literalelor scoasa complet.
