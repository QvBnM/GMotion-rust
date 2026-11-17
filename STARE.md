# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (98 teste; `core` pe wasm32, tot codul si pe windows-msvc)

---

## Unde suntem

Etapele 0-3 incheiate. Vezi `etape/`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 37 teste, fuzz pe invariante |
| `rust/db` | gata functional, **neverificat pe tinta**, 29 teste |
| `rust/platforma` | gata functional, **DPAPI neverificat pe tinta**, 32 teste |

Poarta verifica si `cargo check --target x86_64-pc-windows-msvc`, deci codul
`cfg(windows)` (DPAPI, auth integrata) e corect la compilare, nu scris orb.

**Etapa activa:** niciuna. Urmatoarea la rand e etapa 4,
`module/diff-obiecte` ca biblioteca, fara interfata.

---

## Urmatorul pas concret

Etapa 4 — `rust/module/diff-obiecte`, **ca biblioteca, fara nicio interfata**.
Orchestrarea completa a modulului 1, portata din `mostenire/app.py`:

1. **compararea** — pentru fiecare baza fata de referinta: obiecte lipsa, in
   plus, identice, modificate; scor si rezumat per obiect (`_comparatie`);
2. **rezultatele** — lista pentru ecran, plus diff-ul la cerere
   (`rezultate`, `elemente`);
3. **export CSV** (`exporta_csv`);
4. **generatorul de deploy** (`genereaza_deployer`, `_scrie_deployer`,
   `_aplica_create_or_alter`) — partea cu cel mai mare risc de a produce ceva
   greșit si de a se observa tarziu, deci cea cu cele mai multe teste;
5. implementarea `Modul` pentru registru.

Testat integral pe instantanee, fara baza de date. Atentie: modulul poate
depinde de `core`, `db`, `platforma` — **de niciun alt modul**. Poarta verifica.

```
unelte/verifica.sh
cp etape/SABLON.md etape/04-diff-obiecte.md     # si pune Stare: ACTIVA
```

---

## Ce aștept de la Mihai

**O rulare pe un SQL Server real**, cand ai timp. Nu blocheaza nimic: continui
pe fixture. Valideaza etapa 2 — conectarea, auth integrata si descoperirea sunt
scrise si compilate, dar netestate pe tinta, fiindca aici nu exista SQL Server.

Comanda exacta o pregatesc la etapa 5, cand exista binarul de linie de comanda.
Pana atunci nu ai nimic de facut.

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
