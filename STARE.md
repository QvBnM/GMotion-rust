# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · commit `db801ee` · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (31 teste, `core` compileaza pe wasm32)

---

## Unde suntem

Etapa 0 — **restructurare + reguli** — incheiata. Vezi
`etape/00-restructurare.md`.

Repo-ul are acum: Python-ul v3.4 inghetat in `mostenire/`, harness-ul de
conformitate in `referinta/`, documentele de regula (`CLAUDE.md`,
`ARHITECTURA.md`, `HARTA.md`, `JURNAL.md`), si o poarta de verificare
(`unelte/verifica.sh`) care prinde si ce niciun test nu prinde: tinta wasm,
direcția dependentelor, documentatia rămasa in urma codului.

Singurul cod Rust care exista e `rust/core` — motorul de comparare, gata, 31 de
teste, validat contra fixture-ului produs de motorul JS.

**Etapa activa:** niciuna. Urmatoarea la rand e etapa 1, nepornita:
`etape/01-core-consolidat.md`.

---

## Urmatorul pas concret

Etapa 1 — `core` consolidat. Nu schimba semantica, deci e ieftina si fara risc
de regresie:

1. antet de 3-5 linii in fiecare fisier din `rust/core/src/`, cu rolul si
   **categoria de bug** care ii aparține (regula 4 din `CLAUDE.md`);
2. fuzz pe `split_statements` — zona care a produs **toate** divergentele din
   v3.4, deci locul cu cel mai bun raport efort/incredere. Invariantele de
   verificat sunt 4 si 5 din `rust/core/CRATE.md`: nu taie in interiorul
   literalelor/comentariilor/`CASE`, si concatenarea blocurilor acopera tot
   textul de intrare;
3. `cargo fmt`, poarta verde, `STARE.md` actualizat.

Pornire:

```
unelte/verifica.sh                      # confirma ca pornesti de pe verde
cat etape/01-core-consolidat.md
```

---

## Ce aștept de la Mihai

Nimic blocant acum. Ce va fi nevoie, cand ajungem acolo:

- **Etapa 2 (`db`)** — o rulare pe un SQL Server real, ca sa se decida
  `tiberius` vs `odbc-api`. Comanda exacta ti-o dau atunci; nu e nevoie de
  nimic pregatit dinainte.
- **Etapa 6 (paritate)** — aceleasi baze comparate cu versiunea Python si cu
  `cli`, plus un `gmotion instantaneu` ca sa am fixture-ul aici.

Daca vrei paritate completa a porții pe `cmd` (acum verificarile de direcție a
dependentelor si de `CRATE.md` cer bash — Git Bash sau WSL), spune si le rescriu
in Python la etapa 1.

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
