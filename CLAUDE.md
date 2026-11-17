# Reguli de lucru pe GMotion

Acest fisier e contractul. Daca o regula de aici intra in conflict cu o idee
care pare mai buna pe moment, regula castiga; ideea se discuta si, daca e
corecta, se schimba regula aici, in acelasi commit.

Fiecare document raspunde la o singura intrebare, ca sa nu se dubleze:

| Intrebare | Unde |
|---|---|
| Unde suntem acum, ce urmeaza, ce e blocat | `STARE.md` |
| Ce ne-am propus la etapa asta si cand e gata | `etape/NN-nume.md` |
| De ce am decis ce am decis | `JURNAL.md` |
| Ce s-a facut, exact | `git log` — **nu se dubleaza in fisiere** |
| Harta crate-urilor, direcția dependentelor | `ARHITECTURA.md` |
| Simptom -> fisierul vinovat | `HARTA.md` |
| Contractul unui crate | `<crate>/CRATE.md` |

---

## 0. Pornire si incheiere de sesiune

Asistentul pornește de la zero la fiecare sesiune, iar utilizatorul poate intra
de pe alt cont. Deci starea trebuie sa fie in repo, nu in memoria sesiunii.

**La pornire, in ordinea asta:**

1. `CLAUDE.md` — asta. Regulile.
2. `STARE.md` — unde suntem, urmatorul pas concret, capcanele active.
3. etapa activa din `etape/` — scopul si criteriile de terminare.
4. `unelte/verifica.sh` — confirma ca pornesti de pe verde. Daca poarta e roșie
   inainte sa atingi ceva, **asta** e prima treaba, si se spune utilizatorului.

Nu te apuci de cod inainte de pasii 2 si 3. Daca `STARE.md` contrazice ce
crezi ca stii despre proiect, `STARE.md` castiga — sau e invechit, si atunci se
spune, nu se ghiceste.

**La incheiere, inainte de ultimul commit:**

1. `unelte/stare.sh` — da faptele mecanice (commit, branch, numar de teste, cat
   de veche e starea).
2. actualizeaza `STARE.md`: unde s-a ajuns, **urmatorul pas concret**, ce
   aștepti de la utilizator, capcane noi.
3. actualizeaza jurnalul de lucru al etapei active. Daca etapa s-a incheiat:
   `Stare: INCHEIATA`, data, si completeaza „Ce a ieșit altfel decat planul" —
   e partea cea mai utila din tot fisierul.
4. decizie luata? intra in `JURNAL.md`.
5. **impinge pe branch.** O stare care n-a fost impinsa nu exista pentru
   sesiunea urmatoare.

O etapa noua se deschide copiind `etape/SABLON.md`. **Exact o etapa poate fi
ACTIVA** — poarta verifica, fiindca doua etape active inseamna ca nimeni nu
mai stie unde suntem.

Scrie in `STARE.md` doar ce `git log` nu poate spune: intentii, blocaje, capcane.
Nu rezuma commit-uri acolo.

---

## 1. Poarta

```
unelte/verifica.sh        # Linux / WSL
unelte\verifica.bat       # Windows
```

Ruleaza: format, clippy fara avertismente, toate testele, conformitatea cu
motorul de referinta, direcția dependentelor, sincronizarea `CRATE.md`.

**Nimic nu se commiteaza daca poarta nu e verde.** Nu se raporteaza „gata"
inainte de a o rula. Daca o verificare e sarita (lipseste o tinta, lipseste
`node`), poarta spune explicit ce a sarit — citeste ieșirea, nu doar codul de
retur.

## 2. Ce nu se atinge

- **`mostenire/`** — Python-ul v3.4, inghetat. E dovada contra careia validam
  paritatea. Nu se modifica, nu se formateaza, nu se „curata". Niciodata.
- **`referinta/rezultate_js.json`** — fixture-ul de referinta. Se regenereaza
  **doar** prin `unelte/regenereaza-fixture.sh`, explicit, ca pas propriu, cu
  motivul notat in `JURNAL.md`. Un fixture regenerat ca sa treaca un test e
  exact mecanismul prin care au divergat cele doua motoare din v3.4. Daca un
  test de conformitate cade, intai presupui ca portul Rust e greșit.

## 3. Arhitectura

**Dependentele curg intr-o singura direcție:**

```
core  <-  db  <-  platforma  <-  module/*  <-  app, cli
```

- Niciun crate nu depinde de unul aflat la dreapta lui.
- **Niciun modul nu depinde de alt modul.** Daca modulul 2 are nevoie de ceva
  din modulul 1, acel ceva aparține platformei.
- `core` nu are I/O: fara `std::fs`, fara retea, fara baza de date, fara nimic
  specific unei interfete. Trebuie sa compileze pe `wasm32-unknown-unknown` —
  de aia exista, ca sa fie UN SINGUR motor in backend si in editorul live.
- **Un singur motor.** Daca apare a doua implementare a aceleiasi logici, aia
  nu e o optimizare, e bug-ul din v3.4 reintrodus.
- Logica se poate acționa fara interfata. Orice lucru pe care-l face aplicatia
  trebuie sa fie accesibil din `cli`. Daca o functie exista doar in spatele unui
  buton, nu poate fi depanata.

## 4. Un fisier se separa doar daca are o cauza de eroare proprie

Nu despicam fisiere ca sa bifam module. `core` are `normalizare` /
`segmentare` / `diff` separat fiindca se strica fiecare altfel: fals
„modificat", blocuri taiate prost, blocuri imperecheate greșit. Partea de Win32
va fi un singur fisier: DPI, monitoare, DWM si handle-uri sunt aceeasi
categorie de bug.

Fiecare fisier incepe cu un antet de 3-5 linii: ce rol are si **ce categorie
de bug ii aparține**.

## 5. Fiecare crate are un `CRATE.md`

Sabloane identice peste tot, ca sa fie clar unde se caută. Sectiuni fixe:
`Rol`, `Nu face`, `API public`, `Invariante`, `Erori tipice`, `Testare`,
`Depanare`. Vezi `rust/core/CRATE.md`.

`CRATE.md` conține o linie `<!-- api-hash: ... -->`. Daca schimbi API-ul public
si nu actualizezi `CRATE.md`, poarta cade. Hash-ul nou se ia din ieșirea porții
sau din `unelte/api-hash.sh <crate>`.

## 6. Cod

- Nicio functie publica fara test.
- Fara `unwrap()` / `expect()` / `panic!` pe cale care vede date de la
  utilizator sau din baza de date. In teste si la invariante imposibile, da.
- Erori cu `thiserror`, fiecare cu **cod stabil** (`GM-DB-003`). Codul apare si
  in jurnal si in mesajul catre utilizator. Un cod nu se refoloseste si nu se
  renumeroteaza; vezi `JURNAL.md`.
- Jurnal cu `tracing`, o tinta per crate (`gmotion::db`, `gmotion::core`), ca sa
  se poată filtra la depanare.
- Textele catre utilizator in romana. Identificatorii din cod in romana,
  consecvent cu ce exista deja. Fara diacritice in identificatori.
- `cargo clippy --all-targets -- -D warnings` trebuie sa taca.

## 7. Dependente

Nicio dependinta noua fara o intrare in `JURNAL.md`: ce rezolva, ce
alternative au fost cantarite, de ce asta. Preferam zero dependente noi intr-un
crate care nu are nevoie de ele. `core` ramane cu amprenta minima — e crate-ul
care merge si pe wasm.

## 8. Commit-uri

- Mici, un singur subiect. Mesaj in romana: **ce** se schimba si **de ce**.
- Prima linie sub 72 de caractere, la imperativ sau substantival, fara punct.
- Nu amesteca mutari de fisiere cu schimbari de logica. Mutarea e un commit,
  logica e altul — altfel diff-ul e ilizibil si nu se poate verifica nimic.
- Branch de lucru: `claude/<nume>`. Nu se impinge pe alt branch fara cerere
  explicita.

## 9. Cand ceva nu merge pe o masina pe care nu o am

Mediul meu e Linux, fara SQL Server si fara fereastra. Deci:

- Bug-ul se transforma in fixture. `gmotion instantaneu` scrie obiectele brute
  intr-un `.json`; aceluia se da drumul prin `gmotion compara --sursa fisier`
  si devine test de regresie. Un bug raportat fara instantaneu e un bug pe care
  nu-l pot reproduce.
- Cand cer o rulare pe Windows, dau comanda exacta si spun ce ieșire imi
  trebuie. Nu cer „incearca si spune-mi".

## 10. Ce inseamna „gata"

O componenta e **gata** doar cand are teste, `CRATE.md` si poarta verde. Pana
atunci e „in lucru", oricat de complet ar arata codul. Tabelul din `README.md`
nu se actualizeaza inainte ca asta sa fie adevarat.

La fel pentru o etapa: criteriile de terminare din `etape/NN-*.md` se scriu
**inainte** de a incepe lucrul, si bifa se pune doar cand criteriul a fost
verificat — nu cand pare facut.

Si pentru verificari: **o verificare care n-a fost vazuta cazand nu e o
verificare.** Cand adaugi una in poarta, incalc-o deliberat o data si uita-te
daca poarta se face roșie.
