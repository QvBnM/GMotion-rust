# Etapa 15 — interfata care se tine de treaba, si incredere in motor

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-18 · **Incheiata:** 2026-11-19

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
- [x] marginea dintre coloana si editor se trage cu mouse-ul

**C. Incadrare si lizibilitate**
- [ ] ~~la 1366x768 si 1920x1080 nu iese nimic din ecran~~ — mutat la design-ul
      nou al conectarii (singurul ecran care nu incape); restul: da
      — lista, editorul si deploy-ul: da (capturi 2026-11-19); **conectarea**
      nu: la 1366x768 banda de jurnal iese jos. Se face cu design-ul nou al
      conectarii, care vine imediat dupa etapa
- [x] titlul obiectului: cel mult doua linii, strans
- [x] comutatorul Alaturat / Unificat mai mare si mai vizibil
- [x] liniile lungi se pot citi (derulare pe orizontala sau incadrare) — V10, la H3

**D. Harta**
- [x] patratele desenate curat, fara pixelare, la rezolutia ecranului
- [x] la 4.000 de obiecte care difera: ramane lizibila si se poate derula
- [x] masurat: timpul de desenare la 1.000 / 10.000 / 50.000 de obiecte

**E. Denumiri si limba**
- [x] „Sursa" / „Baza" devin **„Martor" / „Modificata"** in fereastra
- [x] aceeasi redenumire in `cli` si in documente
- [x] toate textele din interfata trec printr-un singur loc, nu stau
      imprastiate prin `.slint`
- [x] comutator de limba in interfata: romana si engleza, tinut minte intre
      porniri
- [x] regula din `CLAUDE.md` §6 despre limba se rescrie dupa noua situatie
- [x] textele facute in Rust in afara ferestrei urmeaza limba: numele
      tipurilor, „difera in", rezumatele de deploy, starea conectarii,
      progresul, plierea, ajutorul (F1)
- [ ] ~~mesajele de eroare si jurnalul~~ raman in romana, deliberat (motivul
      in `JURNAL.md`, 2026-11-19); nu blocheaza incheierea etapei

**F. Conectare**
- [x] cautarea de servere arata ca lucreaza (nu ingheata pana la raspuns)
- [x] se marcheaza cand referinta si baza comparata sunt pe **aceeasi
      instanta** de SQL Server

**G. Panoul de detalii din lista** (cerut de Mihai pe 2026-11-18, dupa o
captura de referinta; inceput pe aceeasi baza ca etapa 14, intr-o sesiune
paralela, si adus aici la unire)
- [x] lista pe toata latimea pana la primul clic; clic pe rand = detaliile in
      dreapta; al doilea clic pe acelasi rand, X sau Esc le inchid
- [x] dublu clic pe rand deschide direct editorul
- [x] panoul se redimensioneaza dintr-un maner tras cu mouse-ul
- [x] in panou: stare, modificari, similaritate, impact estimat (cu motive),
      deploy, Revizuit / In plan / Omite / Deschide, previzualizarea
- [x] nota per obiect: se salveaza in sesiune, pe nume (nu cade cand obiectul
      se schimba); din `cli`: `revizuire --nota X --text T`
- [x] impactul estimat: logica in `diff-obiecte` (cine foloseste obiectul in
      baza comparata, prin acelasi lexer ca ordinea de deploy), din `cli`:
      `gmotion impact`
- [x] test care apasa cu mouse-ul (clic, dublu clic, X, nota), vazut cazand

**H. Remedierea auditului QA** (cerut de Mihai pe 2026-11-18: se face
**inaintea** restului etapei). Sursa: `cercetare/audit-qa-2026-11-18.md`,
facut pe `aec038a`, deci pe cod mai vechi decat cel de acum.
Regula pentru fiecare constatare: intai **se reproduce pe codul de acum cu
un test care cade** (vazut cazand), abia apoi se repara. Daca nu se mai
reproduce, se noteaza asa, cu testul care o dovedeste. O constatare care cere
o decizie de produs (ex. C5, F-A18) nu se decide tacit: intra la „Ce astept
de la Mihai" din `STARE.md`.
- [x] H1 siguranta deploy-ului: C1, C2, C5, C6, C7, C8, F-D1, F-D2, F-A5
      (verdictele si testele: sectiunea „Verdicte" din audit; F-D7 amanat,
      cere o proba pe Windows)
- [x] H2 pierdere de date: C3, C4, F-B4, F-A1, F-A2, F-A3
      (campul de parola din F-A2 ramane partial: vezi „Verdicte")
- [x] H3 cadere si ecrane: C9, V9, V1, V2, V3, V10; capturile la 1366x768
      intra in setul standard (si la 1024x768, unde se vedea V3)
- [x] H4 inghetari: P4, P1, P2, P3
- [x] H5 restul: fiecare constatare din audit are verdict in sectiunea
      „Verdicte" de la finalul auditului — reparat (commit), nu se mai
      reproduce (test), sau amanat (motiv si unde e urmarit). **85 de
      constatari, toate cu verdict**; unsprezece reparate in H5.

**Comun**
- [x] capturi noi in ambele moduri (2026-11-19: editor, lista, deploy,
      conectare; editor si lista si la 1366x768)
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat

## Pasi

0. H — remedierea auditului (H1 → H5, in ordinea asta). Mihai a cerut-o
   inaintea restului: un produs care poate scrie in PROD altceva decat s-a
   aprobat nu are rost sa fie aranjat frumos. V1-V3 si V10 se suprapun cu C;
   ce se repara la H3 se bifeaza si la C.
1. A — motorul si testele. Daca motorul greseste, restul nu conteaza.
2. E — denumirile si limba: ating fiecare ecran, deci inaintea rearanjarii.
3. B — continuitatea (lista in stanga).
4. C — incadrarea la orice rezolutie.
5. D — harta.
6. F — conectarea.

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

- **2026-11-18** G (panoul de detalii), lucrat intr-o sesiune paralela pe
  baza etapei 14 si adus aici la unire. Criteriile au fost scrise dupa ce
  codul era inceput (abatere de la CLAUDE.md §10, notata aici). La unire,
  textele panoului au trecut prin `Texte` (romana si engleza). G gata. Testul de mouse cauta randurile si
  X-ul prin clicuri, nu cu coordonate scrise de mana; vazut cazand fara
  `double-clicked` si fara Esc. Dublu clicul in Slint vine dupa doua
  `clicked` (deschis, inchis), deci `Deschide` redeschide panoul: la
  intoarcerea din editor lista arata obiectul de unde ai plecat. Nota se
  trimite cu obiectul ei (`nota-id`), nu pe selectie. Capturi:
  `cercetare/capturi/lista-*.png` (cu panoul) si `lista-fara-detalii-*.png`.

- **2026-11-18** Antetul listei: bazele pe primul rand, restul dedesubt, pe
  randuri care se rup dupa latimea ferestrei (probat la 1440, 1200, 960,
  760). Eticheta de mediu scoasa de peste tot. Decizia in `JURNAL.md`.

- **2026-11-18** Adaugat pachetul H (auditul QA), inaintea restului, la
  cererea lui Mihai. Criteriile scrise inainte de orice reparatie.

- **2026-11-18** H1 (siguranta deploy-ului) gata: C1, C2, C5, C6, C7, C8,
  F-D1, F-D2, F-A5, cate un commit fiecare, fiecare reprodusa intai cu un
  test vazut cazand pe codul de acum, nu pe `aec038a`. Toate noua s-au
  reprodus: niciuna nu disparuse de la sine.
  Trei lucruri ies din tipar:
  - **C5 a schimbat ce vede editorul**, nu doar ce pleaca in baza:
    `pregateste_afisare` e si textul afisat, si textul pe care "Aplica toate"
    il trimite. Acum granitele vin din `lexer::segmente`, ca orice regula pe
    text (capcana din `STARE.md`). Paritatea cu JS n-a cazut.
  - **C6 e o divergenta deliberata fata de v3.4**: acelasi cod cu alt
    `QUOTED_IDENTIFIER` nu mai e "identic". Fixture-ul real n-are astfel de
    perechi, deci paritatea n-a cazut; daca va avea, intrarea merge in
    `*.asteptari.json`.
  - **F-D7 nu s-a facut** si nu e bifat: tiparul `IF @@ERROR <> 0 SET NOEXEC
    ON` dupa fiecare obiect cere stiut ce vede `@@ERROR` peste granita de
    batch (dupa `GO`). Nu se poate proba pe Linux; cerut lui Mihai.

- **2026-11-18** H2 (pierdere de date) gata: C3, C4, F-B4, F-A1, F-A2,
  F-A3. Toate sase s-au reprodus pe codul de acum.
  - **F-A1 si F-A2 se probeaza cu tastatura si mouse-ul, in fereastra**, fara
    ecran: `MinimalSoftwareWindow` plus `dispatch_event`. Pentru F-A2 testul a
    aratat exact ce spunea auditul: dupa „Inverseaza", campul raporta
    „SRV-DEVx1" — vechea valoare cu tastele noi peste.
    Fereastra de test n-are dimensiune: fara `set_size`, aranjarea e
    degenerata si clicurile nu gasesc campurile din al doilea card.
  - **Campul de parola nu se poate rescrie din model**, si asta a iesit abia
    la testare: modelul ii da buline cat lungimea parolei, deci rescris cu ele
    prima tasta de dupa ar trimite bulinele **ca parola**. Exceptia e probata
    cu un test propriu. Rest de facut acolo (dupa „Inverseaza" spre o parte
    care are parola), notat in „Verdicte".
  - **F-A3 nu se poate proba cap-coada**: blocarea tastaturii cat ruleaza
    compararea sta in `main.rs`, care n-are teste. Garantia e pusa in tipuri:
    `Stare::parte` intoarce `Option`, deci compilatorul cere tratat cazul la
    fiecare camp. De confirmat pe Windows.

- **2026-11-18** H3 (cadere si ecrane) gata. Trei din sase **nu se mai
  reproduceau**: C9 (harta), V1 si V2 (antetul listei) — refacerea hartii si
  a antetului de la etapa 15 le-a scos, fara ca cineva sa stie. Fiecare are
  acum testul sau captura care o tine asa.
  - **V9 se reproducea, dar in alta parte decat spunea auditul**: `.slint`
    trimitea de mult indicele randului apasat; legatura din `main.rs` il
    ignora. Un audit facut pe cod vechi arata simptomul, nu locul.
  - **V3 si V10 se reproduceau intocmai.** Pentru V10 (liniile lungi) am ales
    derulare pe orizontala **comuna celor doua panouri**, nu impachetare:
    impachetarea ar muta liniile una fata de alta si ar strica imperecherea
    vizuala, care e tot rostul ecranului.
  - **Capturile erau toate la 1440x900** — de aceea V1-V3 n-au fost vazute
    niciodata. Acum fiecare exemplu primeste `--latime` / `--inaltime`, si
    setul standard are 1366x768 si 1024x768.

- **2026-11-19** H4 (inghetari) gata. Toate patru s-au reprodus, trei dintre
  ele masurat:
  - `linii_start_blocuri` pe 20.000 de blocuri: **2,12 s** (auditul spunea
    3,15 s), acum **1,6 ms**; `aliniaza` 4,05 s -> 129 ms.
  - `[Case]` pe 5.000 de linii: **1 bloc** in loc de 5.000. `@case` la fel,
    `[O'Brien]` dadea jumatate.
  - P1 nu se poate masura intr-un test fara sa iasa instabil, deci
    `Aplicatie` numara de cate ori construieste modelul, si testul se uita la
    numarul ala.
  - **Al doilea lexer din `segmentare` a disparut**, nu a fost carpit: linia
    curatata vine din `lexer::segmente`, ca orice regula pe text. Asta
    inchide si P3 si clasa lui.
  - P4 a cerut un mecanism, nu o reparatie punctuala: `fir::Lucru<T>`, acelasi
    pentru toate lucrurile lungi, plus o stare vizibila pe ecran cat lucreaza.

- **2026-11-19** H5 gata: verdict pe toate cele 85 de constatari ale
  auditului, si inca unsprezece reparate pe drum (F-A4, F-A10 partial,
  F-A11, F-A13 partial, F-A14, F-B1, F-C1, F-C2, F-C3, F-D10, F-D12).
  - **Poarta a primit o verificare noua**: MSRV-ul declarat fata de ce cer
    dependentele. `rust-version` spunea 1.74, iar `db` si `platforma` nici nu
    se construiau pe el. Nu se instaleaza un toolchain vechi: se compara
    `cargo metadata`. Vazuta cazand pe 1.74 si pe 1.80.
  - **Ridicarea MSRV-ului a pornit lint-uri noi de clippy** (`is_none_or`),
    pe care le tinea inchise versiunea veche declarata. Patru locuri.
  - Amanatele nu sunt o lista de bune intentii: fiecare are motivul ei in
    tabel, si cele care cer SQL Server, Windows sau o decizie de produs sunt
    marcate ca atare. Tabelul de verdicte e singura lista de urmarit.

- **2026-11-19** E: „Martor" / „Modificata" si in `cli` (ajutorul, rezumatul
  compararii, progresul, erorile), in mesajele modulului pe care le arata
  `cli` si fereastra (garda planului, GM-MOD-002, progresul compararii) si in
  `README.md`. Raman neschimbate, fiindca sunt cod (CLAUDE.md §6): optiunile
  `--referinta` / `--baze`, cheile JSON, formatul planului si antetul
  `-- DEPLOYER ... | referinta:` din scripturi. Documentele pentru dezvoltator
  (`CRATE.md`, `ARHITECTURA.md`, `HARTA.md`) vorbesc despre cod, deci raman
  si ele pe `referinta` / `tinta`. Probat pe binar: textul, `--help`, eroarea
  cu cod de iesire 1, JSON-ul cu aceleasi chei.

- **2026-11-19** E, restul: `Limba` a coborat in `core` (cu `alege` si
  `numar`, care stie pluralul romanesc: „20 de", dar „101"), iar fiecare ecran
  isi afla limba din globalul `Texte` (`limba_curenta`). Conectarea o tine in
  model, ca `pornire` sa scrie mesajele fara un parametru in plus. Eticheta
  „Difera in" se calculeaza o data, in romana, si se traduce la afisare; un
  test o tine egala cu varianta facuta direct in engleza. Pe drum, patru
  asteptari din teste erau gramatical gresite („42 baze", „23 ore", „60
  minute"), nu codul. Probat pe captura in engleza a listei si a editorului.

- **2026-11-19** B gata: manerul dintre coloana listei si editor (200 px
  minim, editorul pastreaza 640 px); latimea trasa se tine minte, iar Esc /
  sageata o readuc la ea, nu la 300 px. Testul trage cu mouse-ul adevarat
  (evenimente de pointer), nu cheama proprietatea.

- **2026-11-19** C: titlul se rupe pe doua randuri, cu marimea pe trepte
  dupa lungimea numelui (40 / 26 / 20 px). `elide` impreuna cu ruperea pe
  randuri nu merge in Slint: taia dupa primul rand si lasa golul — vazut pe
  captura, de aceea marimea pe trepte. Comutatorul Alaturat / Unificat e acum
  un comutator in doua, cu partea aleasa pe accent; la 1024 px insulele tot
  trec pe doua randuri. Pe drum: in cardul Martor, campul bazei scria
  „Modificata" (ramas de la redenumire); are acum textul lui, „Baza".

- **2026-11-19** D gata. Harta se picteaza la pixelii fizici ai ecranului
  (`picteaza_la(g, paleta, factor)`, factorul din fereastra) si se afiseaza
  la marimea logica: inainte, pe un ecran scalat 125-200%, imaginea se marea
  si patratele ieseau neclare. Masurat cu `--example masura_harta` (release,
  latimea 1440), asezare + pictare, toate coloanele:

  | obiecte | aseaza | picteaza 1x | picteaza 2x | latura | inaltime |
  |---|---|---|---|---|---|
  | 1.000 | 0,0 ms | 0,4 ms | 1,4 ms | 11 px | 219 px |
  | 4.000 | 0,0 ms | 0,1 ms | 0,5 ms | 4 px | 129 px |
  | 10.000 | 0,1 ms | 0,4 ms | 1,8 ms | 4 px | 257 px |
  | 50.000 | 0,8 ms | 3,2 ms | 10,1 ms | 4 px | 1.121 px (se deruleaza) |

  Captura la 4.000 (1.778 modificate, 1366x768): trei coloane curate.

- **2026-11-19** F gata, pe ecranul de acum (design-ul nou al conectarii il
  va reaseza, logica ramane): cautarea rula deja pe alt fir (H4), dar ecranul
  scria „se conecteaza…"; acum are starea ei, „se cauta servere in retea…".
  „Aceeasi instanta" e `Conectare::aceeasi_instanta`: recunoaste `.` /
  `localhost` / `127.0.0.1`, numele scurt fata de cel cu domeniu, instanta
  implicita si portul 1433 scrise sau nu; alt port sau alta instanta pe
  aceeasi masina nu se socotesc la fel.

## Ce a iesit altfel decat planul

- **Auditul QA a intrat in mijlocul etapei** (pachetul H) si a luat cel mai
  mult timp; V1-V3 si V10 din el au rezolvat dinainte o parte din C. O
  etapa de interfata care primeste un audit la jumatate ar trebui sa-si
  rescrie ordinea pasilor, nu sa-i adauge la coada.
- **Cele mai multe „decizii" au avut o abatere la implementare.** F-B5:
  scoaterea unui camp din configurare ar fi stricat fisierele existente
  (`deny_unknown_fields`), deci s-a legat in loc de scos. Erorile si
  jurnalul n-au trecut in engleza: ar fi cerut erori tipate pana in ecran.
  Lectia: o decizie de produs se verifica pe cod inainte de a fi scrisa ca
  „acceptata".
- **Patru asteptari din teste erau gramatical gresite** („42 baze", „23
  ore", „60 minute"), nu codul — a cincea oara cand cifra din test greseste
  (CLAUDE.md §6).
- **Slint: `elide` nu merge cu ruperea pe randuri**; titlul are marimea pe
  trepte. Si `close-policy` nu primeste o expresie.
- **Incadrarea conectarii la 1366x768 a ramas pe dinafara**, mutata la
  design-ul nou, care oricum reaseaza ecranul.
