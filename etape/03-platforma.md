# Etapa 3 — `platforma`

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Un modul primeste de la platforma tot ce nu e specific lui — conexiune,
sesiuni, jurnal, configurare — si nu poate ajunge la starea altui modul.
Adaugarea modulului 2 nu trebuie sa atinga modulul 1.

## Criterii de terminare

- [x] `cai`: unde stau datele, cu `GMOTION_DIR_DATE` pentru teste
- [x] `protejare`: DPAPI pe Windows, marcaj in clar in dezvoltare
- [x] `sesiune`: cheia `<modul>/<slug>.dat` **din start**
- [x] test care dovedeste ca doua module pe aceeasi referinta nu se suprascriu
- [x] `configurare`: implicite in cod, fisier stricat = eroare
- [x] `jurnal`: tinte per crate, tampon marginit
- [x] `modul`: contract, registru, rutare cu prefix, metode comune protejate
- [x] coduri `GM-PLAT-001..011`, in `JURNAL.md`
- [x] o sesiune nu poate contine secrete — proprietate a tipului, testata
- [x] `CRATE.md`
- [x] poarta verde
- [ ] **DPAPI verificat functional pe Windows** — neacoperit aici

## Pasi

1. erori + validarea identificatorilor
2. `cai`, apoi `protejare`
3. `sesiune` (partea care corecteaza bug-ul din versiunea Python)
4. `configurare`, `jurnal`
5. contractul de modul + registru + rutare
6. teste de integrare, `CRATE.md`

## Jurnal de lucru

- **2026-11-17** Inainte de a scrie codul `cfg(windows)`: adaugata tinta
  `x86_64-pc-windows-msvc` si o verificare in poarta
  (`cargo check --target ...`). `cargo check` nu cere linker, doar biblioteca
  standard, si merge inclusiv cu tiberius pe SChannel. Fara asta, tot codul de
  Windows s-ar fi scris orb.
- **2026-11-17** DPAPI scris ca FFI direct catre `crypt32.dll`, fara crate de
  legaturi Windows: doua functii nu justifica o dependinta.
- **2026-11-17** Testele de sesiuni serializate cu un lacat — vezi mai jos.
- **2026-11-17** 32 de teste.

## Ce a iesit altfel decat planul

**Verificarea de Windows din poarta s-a platit in aceeasi zi in care a fost
adaugata.** Prima versiune a FFI-ului DPAPI incerca sa unifice
`CryptProtectData` si `CryptUnprotectData` printr-un pointer de functie
`extern "system"`, si pasa un closure Rust. Nu compila. Pe Linux nu se vedea
nimic: codul e `#[cfg(windows)]`, deci nici `cargo build` nici `clippy` nu se
uitau la el. Ar fi ajuns pe Windows rupt, iar simptomul ar fi fost „nu
compileaza la build", nu „nu merge DPAPI" — deci cel putin nu silentios, dar
descoperit cat mai scump.

Generalizarea, de pus in `CLAUDE.md`: **codul in spatele unui `cfg` pe alta
tinta trebuie verificat pe tinta aia, altfel nu e verificat deloc.** Acum
poarta o face.

**Testele de sesiuni erau instabile si trecusera.** Amandoua schimbau
`GMOTION_DIR_DATE`, care e globala in proces, iar `cargo test` ruleaza in
paralel — deci ar fi picat aleator, probabil mult mai tarziu si pe altceva. Le-am
rulat intai cu `--test-threads=1` fara sa ma gandesc, apoi am observat.
Serializate cu un lacat static si rulate de trei ori cu paralelismul implicit.
**Un test care trece nu inseamna un test corect**, acelasi tip de greseala ca
fuzz-ul de la etapa 1.

**Expectativa mea dintr-un test era gresita, nu codul.** `slug("ImiPlace")`
dadea `miplace-8`, iar eu asteptam `-9`: numarasem octeti, nu caractere.
Discriminantul foloseste `chars().count()`, care e ce trebuie.

**Partea de ecrane din contractul de modul a rams minimala,** deliberat:
interfata nu exista, si un contract de UI scris acum ar fi ghicit. Marcat ca
provizoriu in `modul.rs`. Testul real al contractului e modulul 2, la etapa 9 —
si daca atunci cere schimbari in platforma, contractul de aici era gresit, cat
e ieftin de corectat.