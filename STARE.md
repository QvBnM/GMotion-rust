# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa greșești acum.
>
> Ordinea de pornire e in `CLAUDE.md`, secțiunea 0.

**Actualizat:** 2026-11-17 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (66 teste, `core` compileaza pe wasm32)

---

## Unde suntem

Etapele 0, 1 si 2 incheiate. Vezi `etape/`.

- `rust/core` — motorul, gata si consolidat: 37 teste, inclusiv fuzz pe
  invariante.
- `rust/db` — accesul la date, gata **functional, neverificat pe tinta**: 29 de
  teste, toate fara SQL Server si fara retea. `SursaObiecte` ca trait, cu baza
  reala (TDS nativ, fara ODBC) si instantaneu din fisier.

**Etapa activa:** niciuna. Urmatoarea la rand e etapa 3, `platforma`.

---

## Urmatorul pas concret

Etapa 3 — `rust/platforma`. Patru lucruri:

1. **configurare** — ce se citeste de unde, cu valori implicite sanatoase;
2. **sesiuni** — cheia `<modul>/<slug>.dat` din start, ca sa nu se repete
   coliziunea din versiunea Python. DPAPI pe Windows; pe alt sistem **nu se
   salveaza secrete**, se refuza cinstit (vezi regula de la `Debug` din
   `JURNAL.md`);
3. **jurnal** — `tracing`, cu tinta per crate, filtrabil din mediu;
4. **registru de module + rutare cu prefix** — metodele comune o singura data,
   cele specifice inregistrate cu prefix de modul.

```
unelte/verifica.sh
cp etape/SABLON.md etape/03-platforma.md     # si pune Stare: ACTIVA
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
- **Un tip care tine acreditari nu are `Debug` derivat.** Ar tipari parola la
  primul `dbg!` sau in orice mesaj de panica. Se scrie de mana, cu redactare.
  Vezi `rust/db/src/sqlserver.rs`.
- **O conectare care merge pe Linux nu dovedeste nimic despre Windows.**
  Furnizorul de TLS difera intre tinte (`rustls` vs SChannel), si auth integrata
  nu exista pe Linux deloc.
- **Un fuzz care trece nu e un fuzz care functioneaza.** Cand adaugi un test de
  invariante, strica deliberat codul si uita-te daca il prinde. Prima versiune a
  fuzz-ului de segmentare trecea si cu urmarirea literalelor scoasa complet.
