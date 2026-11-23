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
| Harta crate-urilor, directia dependentelor | `ARHITECTURA.md` |
| Simptom -> fisierul vinovat | `HARTA.md` |
| Unde e o functie / componenta, la ce linie | `HARTA-COD.md` (generat, nu e in git) |
| Contractul unui crate | `<crate>/CRATE.md` |

---

## 0. Pornire si incheiere de sesiune

Asistentul porneste de la zero la fiecare sesiune, iar utilizatorul poate intra
de pe alt cont. Deci starea trebuie sa fie in repo, nu in memoria sesiunii.

**La pornire, in ordinea asta:**

0. `unelte/ultima-versiune.sh` (pe Windows `.\unelte\ultima-versiune.ps1`) —
   care e cel mai nou branch si daca exista **lucru paralel neunit**. Daca
   branch-ul curent nu e ultima versiune, sau scriptul iese cu 2 (munca
   impartita pe branch-uri neunite), **se spune utilizatorului inainte de
   orice** si se cere sa aleaga de unde se porneste. S-a lucrat si s-a auditat
   o data o zi intreaga pe o versiune veche, pentru ca `STARE.md` numea alt
   branch (2026-11-23).
1. `CLAUDE.md` — asta. Regulile.
2. `STARE.md` — unde suntem, urmatorul pas concret, capcanele active.
3. etapa activa din `etape/` — scopul si criteriile de terminare.
4. `python3 -I unelte/harta-cod.py` — genereaza `HARTA-COD.md`: fiecare
   fisier, rolul lui si functiile cu numarul de linie. **Cauti in harta, apoi
   citesti doar intervalul** (`sed -n 'A,Bp'`), nu fisiere intregi.
5. `unelte/verifica.sh` — confirma ca pornesti de pe verde. Daca poarta e rosie
   inainte sa atingi ceva, **asta** e prima treaba, si se spune utilizatorului.

Nu te apuci de cod inainte de pasii 2 si 3. Daca `STARE.md` contrazice ce
crezi ca stii despre proiect, `STARE.md` castiga — sau e invechit, si atunci se
spune, nu se ghiceste.

**La incheiere, inainte de ultimul commit:**

1. `unelte/stare.sh` — da faptele mecanice (commit, branch, numar de teste, cat
   de veche e starea).
2. actualizeaza `STARE.md`: unde s-a ajuns, **urmatorul pas concret**, ce
   astepti de la utilizator, capcane noi.
3. actualizeaza jurnalul de lucru al etapei active. Daca etapa s-a incheiat:
   `Stare: INCHEIATA`, data, si completeaza „Ce a iesit altfel decat planul" —
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

## 0b. Cum raspunde asistentul

Minimalist. Fara introduceri, fara rezumate ale cererii, fara recapitulari ale
pasilor, fara oferte de ajutor suplimentar.

Cand lucreaza: doar ce a facut, in 1-2 propozitii.

Exceptia: daca apare o decizie reala sau o ambiguitate, da contextul strict
necesar ca utilizatorul sa aleaga (cateva randuri).

Detalii suplimentare doar daca sunt cerute explicit; dupa aceea, inapoi la
minimalism.

---

## 1. Poarta

```
unelte/verifica.sh        # Linux / WSL
unelte\verifica.bat       # Windows
```

Ruleaza: format, clippy fara avertismente, toate testele, conformitatea cu
motorul de referinta, directia dependentelor, sincronizarea `CRATE.md`.

**Nimic nu se commiteaza daca poarta nu e verde.** Nu se raporteaza „gata"
inainte de a o rula. Daca o verificare e sarita (lipseste o tinta, lipseste
`node`), poarta spune explicit ce a sarit — citeste iesirea, nu doar codul de
retur.

## 2. Ce nu se atinge

- **`mostenire/`** — Python-ul v3.4, inghetat. E dovada contra careia validam
  paritatea. Nu se modifica, nu se formateaza, nu se „curata". Niciodata.
- **`referinta/rezultate_js.json`** — fixture-ul de referinta. Se regenereaza
  **doar** prin `unelte/regenereaza-fixture.sh`, explicit, ca pas propriu, cu
  motivul notat in `JURNAL.md`. Un fixture regenerat ca sa treaca un test e
  exact mecanismul prin care au divergat cele doua motoare din v3.4. Daca un
  test de conformitate cade, intai presupui ca portul Rust e gresit.

## 3. Arhitectura

**Dependentele curg intr-o singura directie:**

```
core  <-  db  <-  platforma  <-  module/*  <-  fereastra, cli  <-  app
```

- Niciun crate nu depinde de unul aflat la dreapta lui.
- **Niciun modul nu depinde de alt modul.** Daca modulul 2 are nevoie de ceva
  din modulul 1, acel ceva apartine platformei.
- `core` nu are I/O: fara `std::fs`, fara retea, fara baza de date, fara nimic
  specific unei interfete. Trebuie sa compileze pe `wasm32-unknown-unknown` —
  de aia exista, ca sa fie UN SINGUR motor in backend si in editorul live.
- **Un singur motor.** Daca apare a doua implementare a aceleiasi logici, aia
  nu e o optimizare, e bug-ul din v3.4 reintrodus.
- Logica se poate actiona fara interfata. Orice lucru pe care-l face aplicatia
  trebuie sa fie accesibil din `cli`. Daca o functie exista doar in spatele unui
  buton, nu poate fi depanata.

## 4. Un fisier se separa doar daca are o cauza de eroare proprie

Nu despicam fisiere ca sa bifam module. `core` are `normalizare` /
`segmentare` / `diff` separat fiindca se strica fiecare altfel: fals
„modificat", blocuri taiate prost, blocuri imperecheate gresit. Partea de Win32
va fi un singur fisier: DPI, monitoare, DWM si handle-uri sunt aceeasi
categorie de bug.

Fiecare fisier incepe cu un antet de 3-5 linii: ce rol are si **ce categorie
de bug ii apartine**.

## 5. Fiecare crate are un `CRATE.md`

Sabloane identice peste tot, ca sa fie clar unde se cauta. Sectiuni fixe:
`Rol`, `Nu face`, `API public`, `Invariante`, `Erori tipice`, `Testare`,
`Depanare`. Vezi `rust/core/CRATE.md`.

`CRATE.md` contine o linie `<!-- api-hash: ... -->`. Daca schimbi API-ul public
si nu actualizezi `CRATE.md`, poarta cade. Hash-ul nou se ia din iesirea portii
sau din `unelte/api-hash.sh <crate>`.

## 6. Cod

- Nicio functie publica fara test.
- Fara `unwrap()` / `expect()` / `panic!` pe cale care vede date de la
  utilizator sau din baza de date. In teste si la invariante imposibile, da.
- Erori cu `thiserror`, fiecare cu **cod stabil** (`GM-DB-003`). Codul apare si
  in jurnal si in mesajul catre utilizator. Un cod nu se refoloseste si nu se
  renumeroteaza; vezi `JURNAL.md`.
- Jurnal cu `tracing`, o tinta per crate (`gmotion::db`, `gmotion::core`), ca sa
  se poata filtra la depanare.
- **Fara diacritice, nicaieri**: nici in interfata, nici in mesaje,
  comentarii sau documente. Poarta verifica. Exceptia: date de test care
  verifica intentionat nume cu diacritice ale utilizatorului, marcate pe
  aceeasi linie cu `diacritice-permise`. Identificatorii din cod in romana,
  consecvent cu ce exista deja.
- **Niciun text pentru om scris direct in `.slint`.** Toate trec prin
  `fereastra::texte`, care le tine in romana si engleza; `.slint` citeste din
  globalul `Texte`. Un text nou se adauga in ambele limbi, altfel nu
  compileaza. Comutatorul de limba din fereastra le schimba pe toate si tine
  minte alegerea (Mihai, 2026-11-18).
- **Denumirile celor doua parti sunt „Martor" si „Modificata"** („Witness" /
  „Modified"): martorul e baza de referinta, modificata e cea care a primit
  schimbari. Nu „sursa" / „baza" / „tinta" in textele catre utilizator;
  in cod, `referinta` si `tinta` raman (asa e si in `cli` si in plan).
- `cargo clippy --all-targets -- -D warnings` trebuie sa taca.
- **Cod in spatele unui `cfg` pentru alta tinta nu e verificat de nimic pe
  masina de dezvoltare** — nici `build`, nici `clippy`, nici testele se uita la
  el. Poarta ruleaza `cargo check` si pe `x86_64-pc-windows-msvc` tocmai pentru
  asta. Cand scrii `#[cfg(windows)]`, rulezi poarta inainte sa spui ca merge.
- **Un tip care tine acreditari nu are `Debug` derivat.** Ar tipari parola la
  primul `dbg!` sau in orice mesaj de panica. Se scrie de mana, cu redactare.
- **Un test care umbla la variabile de mediu trebuie serializat cu un lacat.**
  `cargo test` ruleaza in paralel in acelasi proces. Fara lacat, testul trece
  azi si cade aleator peste o luna.
- **Cand un test cade pe un numar, prima ipoteza e ca numarul din test e
  gresit.** Intr-un test cu date construite de mana, cifra asteptata e cel putin
  la fel de usor de gresit ca si codul. S-a intamplat de trei ori la etapele 4-5.
  Numara din nou inainte sa „repari" codul.
- **Un binar se probeaza rulandu-l.** Compilarea si testele de unitate nu spun
  nimic despre ce ajunge pe iesirea standard, pe cea de erori, sau in codul de
  iesire. `--json` a fost rupt o data exact asa.

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
- Cand cer o rulare pe Windows, dau comanda exacta si spun ce iesire imi
  trebuie. Nu cer „incearca si spune-mi".

## 10. Ce inseamna „gata"

O componenta e **gata** doar cand are teste, `CRATE.md` si poarta verde. Pana
atunci e „in lucru", oricat de complet ar arata codul. Tabelul din `README.md`
nu se actualizeaza inainte ca asta sa fie adevarat.

La fel pentru o etapa: criteriile de terminare din `etape/NN-*.md` se scriu
**inainte** de a incepe lucrul, si bifa se pune doar cand criteriul a fost
verificat — nu cand pare facut.

**Un ecran facut dupa o macheta trebuie sa arate ca macheta.** Macheta e
tinta, nu inspiratie. Criteriile etapei descriu aspectul, nu doar functiile,
iar bifa cere captura la aceeasi rezolutie pusa langa macheta si „da"-ul lui
Mihai pe pereche. O diferenta fata de macheta se cere inainte, nu se
explica dupa (etapa 16 a fost bifata pe functii si nu semana deloc).

Si pentru verificari: **o verificare care n-a fost vazuta cazand nu e o
verificare.** Cand adaugi una in poarta, incalc-o deliberat o data si uita-te
daca poarta se face rosie.
