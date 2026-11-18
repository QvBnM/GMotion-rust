# Etapa 15 — interfata care se tine de treaba, si incredere in motor

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-18 · **Incheiata:** —

## Scop

Aplicatia se foloseste fara sa te incurce: nu pleci din ecran cand deschizi un
obiect, textul incape la orice rezolutie, harta arata curat si la mii de
obiecte, iar denumirile spun ce sunt. Si, inaintea tuturor: dovada, pe cod
scris urat de mana, ca motorul face ce trebuie cap-coada.

Cerut de Mihai pe 2026-11-18, dupa prima rulare pe Windows.

## Criterii de terminare

**A. Motorul si testele** (primul, cerut explicit)
- [x] cazul raportat de Mihai (compararea offline din imaginea 5) intra in
      `referinta/cazuri.json` si trece prin harness: Rust = motorul v3.4
- [x] instantaneul de previzualizare primeste obiecte **deformate real**:
      cursor complet (`DECLARE ... CURSOR` / `OPEN` / `FETCH` / `WHILE` /
      `CLOSE` / `DEALLOCATE`), `WHILE` imbricate, `IF/ELSE`, majuscule
      amestecate pe cuvinte cheie, indentare inconsecventa, linii lipite
- [x] un test cap-coada care ia un obiect deformat, aplica toate blocurile si
      verifica forma normalizata egala cu sursa
      (`pe_cod_deformat_aplica_toate_aduce_martorul_in_baza`)

**B. Continuitate: lista ramane in stanga**
- [x] lista se strange intr-o coloana in stanga, editorul ocupa restul;
      obiectul se schimba cu un clic in coloana, fara sa pleci din editor
- [x] `Esc` (si sageata din capul coloanei) readuce lista la latime intreaga
- [x] antetul (obiect N din M, Revizuit, Omite, In plan) ramane acelasi
- [ ] marginea dintre coloana si editor se trage cu mouse-ul

**C. Incadrare si lizibilitate**
- [ ] la 1366x768 si 1920x1080 nu iese nimic din ecran si nu se taie butoane
- [ ] titlul obiectului: cel mult doua linii, strans
- [ ] comutatorul Alaturat / Unificat mai mare si mai vizibil
- [ ] liniile lungi se pot citi (derulare pe orizontala sau incadrare)

**D. Harta**
- [ ] patratele desenate curat, fara pixelare, la rezolutia ecranului
- [ ] la 4.000 de obiecte care difera: ramane lizibila si se poate derula
- [ ] masurat: timpul de desenare la 1.000 / 10.000 / 50.000 de obiecte

**E. Denumiri si limba**
- [x] „Sursa" / „Baza" devin **„Martor" / „Modificata"** in fereastra
- [ ] aceeasi redenumire in `cli` si in documente
- [x] toate textele din interfata trec printr-un singur loc, nu stau
      imprastiate prin `.slint`
- [x] comutator de limba in interfata: romana si engleza, tinut minte intre
      porniri
- [x] regula din `CLAUDE.md` §6 despre limba se rescrie dupa noua situatie
- [ ] **ramase in romana**: textele facute in Rust in afara ferestrei —
      numele tipurilor (`core::nume_tip`), propozitia „difera in cod, siruri"
      (`core::clasificare`), rezumatele de deploy (`model_deploy`), mesajele
      de eroare si jurnalul

**F. Conectare**
- [ ] cautarea de servere arata ca lucreaza (nu ingheata pana la raspuns)
- [ ] se marcheaza cand referinta si baza comparata sunt pe **aceeasi
      instanta** de SQL Server

**Comun**
- [ ] capturi noi in ambele moduri
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat

## Pasi

1. A — motorul si testele. Daca motorul greseste, restul nu conteaza.
2. E — denumirile si limba: ating fiecare ecran, deci inaintea rearanjarii.
3. B — continuitatea (lista in stanga).
4. C — incadrarea la orice rezolutie.
5. D — harta.
6. F — conectarea.

Separat, in afara etapei: `cercetare/audit-qa-2026-11-18.md` are constatari
blocante nereproduse inca pe implementarea asta. Se triaza inainte de orice
lansare.

## Jurnal de lucru

- **2026-11-18** Etapa deschisa, cu criteriile scrise inainte de cod, din
  lista lui Mihai de dupa prima rulare pe Windows.
- **2026-11-18** B gata, in forma propusa in macheta: coloana de 300 px in
  stanga editorului, cu starea, numele, baza si cat difera, similaritatea si
  bifa; randul deschis marcat; clic = alt obiect pe loc. Randurile se
  construiesc **o singura data** (`randuri_lista_ui`), ca lista intreaga si
  coloana sa nu poata spune altceva despre acelasi obiect.
  Ramane o diferenta fata de macheta: ecranele sunt tot doua ferestre Slint
  care se schimba la Deschide si la Esc; vizual lista nu mai dispare, dar
  trecerea nu e o animatie. O singura fereastra ar insemna mutat tot
  editorul in `lista.slint` — se poate face, dar e o rescriere, nu o
  ajustare.
  Pe drum: `captura_flux` construia aplicatia cu numele bazei scris de mana
  („Client1" in loc de `CharismaERP_Client1`), deci randurile nu-si gaseau
  obiectele si nu se deschidea nimic. Nu se vedea pana acum fiindca nimeni
  nu se uita la ce iese.

- **2026-11-18** E, partea de fereastra: 143 de texte au iesit din `.slint`
  intr-un tabel unic (`fereastra::texte`), in romana si engleza, cu un
  comutator in antet si alegerea tinuta minte in configurare. „Sursa" /
  „Baza" au devenit „Martor" / „Modificata". Pe drum: capturile au iesit
  **goale**, fiindca exemplele aplicau tema dar nu si limba — acum o aplica
  toate. Au ramas in romana textele facute in Rust in afara ferestrei (numele
  tipurilor, propozitia „difera in", rezumatele de deploy, erorile).

- **2026-11-18** Prima verificare din A: cazul din imaginea 5 **nu arata o
  inconsistenta de motor**. Rulat prin motorul JS din v3.4 (`motor_js.mjs`) si
  prin Rust: blocuri identice (`WHile 1=1\nBEgin` / `Stard end` / `END` /
  `Mama 123`) si acelasi sir de diferente. Ce deranjeaza e **cum arata**: doua
  blocuri complet diferite se arata ca sters + adaugat, nu imperecheate, iar
  alinierea lasa gauri hasurate mari. Deci e C, nu A.

- **2026-11-18** Harta din lista refacuta dupa modelul dat de Mihai (in
  afara pachetelor, cerut explicit): coloane pe tip care umplu cartusul,
  bara de proportie, blocuri de 5 randuri, culori proprii, reasezare la
  redimensionare, zoom automat cand sunt prea multe obiecte. Capturi:
  `cercetare/capturi/harta-*.png` (`harta-zoom-*` pe 2980 de obiecte).
  Decizia in `JURNAL.md`.

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
