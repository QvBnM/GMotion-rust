# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (140 teste; `core` pe wasm32, tot codul si pe windows-msvc)

---

## Unde suntem

Etapele 0-3 incheiate. Vezi `etape/`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 37 teste, fuzz pe invariante |
| `rust/db` | gata functional, **neverificat pe tinta**, 29 teste |
| `rust/platforma` | gata functional, **DPAPI neverificat pe tinta**, 32 teste |
| `rust/module/diff-obiecte` | **gata**, 42 teste, flux complet pe instantaneu |

Poarta verifica si `cargo check --target x86_64-pc-windows-msvc`, deci codul
`cfg(windows)` (DPAPI, auth integrata) e corect la compilare, nu scris orb.

Modulul 1 e functional cap-coada ca biblioteca: comparare, diff, deploy,
export. Ce lipseste e un **binar** care sa-l cheme.

**Etapa activa:** niciuna. Urmatoarea la rand e etapa 5, `cli` — si la finalul
ei exista o aplicatie completa si utilizabila.

---

## Urmatorul pas concret

Etapa 5 — `rust/cli`. **La finalul ei exista o aplicatie completa si
utilizabila**, fara interfata grafica.

Comenzi:

| Comanda | Ce face |
|---|---|
| `gmotion diagnostic` | ce vede GMotion: versiune, cai, configurare, conexiune daca se da un server |
| `gmotion servere` | descoperirea instantelor in retea |
| `gmotion baze --server S` | bazele de pe un server |
| `gmotion instantaneu --server S --baze A,B -o f.json` | obiectele brute intr-un fisier |
| `gmotion compara --sursa f.json --referinta R --baze A,B` | compararea; sursa poate fi un instantaneu **sau** un server |
| `gmotion deploy ...` | scripturile de deploy |
| `gmotion export ...` | CSV |

Doua lucruri de respectat:

1. **Parola nu se da ca argument de linie de comanda** — ar ajunge in istoricul
   shell-ului si in lista de procese. Se citeste din `GMOTION_PAROLA` sau de la
   terminal.
2. Fiecare comanda trebuie sa mearga si cu `--sursa <instantaneu>`, nu doar cu
   un server. Altfel se pierde exact proprietatea pentru care s-a facut
   `SursaObiecte`.

```
unelte/verifica.sh
cp etape/SABLON.md etape/05-cli.md      # si pune Stare: ACTIVA
```

---

## Ce aștept de la Mihai

**O rulare pe un SQL Server real**, cand ai timp. Nu blocheaza nimic: continui
pe fixture. Valideaza etapa 2 — conectarea, auth integrata si descoperirea sunt
scrise si compilate, dar netestate pe tinta, fiindca aici nu exista SQL Server.

Comanda exacta o pregatesc la finalul etapei 5, care urmeaza acum. Pana atunci
nu ai nimic de facut.

La etapa 6 (paritate) va fi nevoie de: aceleasi baze comparate cu versiunea
Python si cu `gmotion`, plus un `gmotion instantaneu` trimis incoace ca sa am
fixture-ul.

Daca vrei paritate completa a porții pe `cmd` (acum cere Git Bash sau WSL),
spune si o rescriu.

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
