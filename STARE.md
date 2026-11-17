# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (37 teste, `core` compileaza pe wasm32)

---

## Unde suntem

Etapele 0 si 1 incheiate. Vezi `etape/`.

`gmotion-core` e gata si consolidat: 37 de teste in trei straturi — unitare,
conformitate cu motorul JS (24 cazuri x 8 campuri), si fuzz dirijat pe
invariante (400 de intrari per test). Fuzz-ul a fost pus la proba cu bug-uri
introduse deliberat si le prinde.

Repo-ul are documentele de regula, urmarirea stării (`STARE.md` + `etape/`), si
o poarta (`unelte/verifica.sh`) care verifica si ce niciun test nu prinde:
tinta wasm, direcția dependentelor, documentatia rămasa in urma codului, starea
incoerenta.

**Etapa activa:** niciuna. Urmatoarea la rand e etapa 2, `db`. Nu are inca
fisier de etapa — se deschide din `etape/SABLON.md`.

---

## Urmatorul pas concret

Etapa 2 — `rust/db`. Crate nou. Trei lucruri, in ordine:

1. **`SursaObiecte` ca trait**, plus implementarea din fixture (JSON). Se scrie
   prima, fiindca tot ce urmeaza se testeaza prin ea, fara baza de date.
2. **Implementarea pe SQL Server** + descoperirea instantelor. Aici se ia
   decizia `tiberius` vs `odbc-api` — vezi „ce aștept de la Mihai".
3. **`CRATE.md`**, codurile de eroare `GM-DB-*` in `JURNAL.md`, poarta verde.

Pornire:

```
unelte/verifica.sh
cp etape/SABLON.md etape/02-db.md        # si pune Stare: ACTIVA
```

Atentie la direcția dependentelor: `db` poate depinde de `core`, nu invers.
Poarta verifica.

---

## Ce aștept de la Mihai

Nimic blocant acum. Ce va fi nevoie, cand ajungem acolo:

- **Etapa 2 (`db`), in curs de pornire** — o rulare pe un SQL Server real, ca
  sa se decida `tiberius` vs `odbc-api`. Scriu intai partea care nu depinde de
  decizie (traitul si implementarea din fixture), apoi iti dau comanda exacta.
  Nu e nevoie de nimic pregatit dinainte.
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
- **Un bloc returnat de `split_statements` nu e garantat subsir al textului
  original** (blocurile unite se lipesc cu un singur `\n`). De-aia
  `linii_start_blocuri` are o ramura de rezerva. Conteaza la etapa de editor:
  ancorele de scroll degradeaza silentios acolo. Vezi
  `etape/01-core-consolidat.md`.
- **Un fuzz care trece nu e un fuzz care functioneaza.** Cand adaugi un test de
  invariante, strica deliberat codul si uita-te daca il prinde. Prima versiune a
  fuzz-ului de segmentare trecea si cu urmarirea literalelor scoasa complet.
