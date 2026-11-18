# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa gresesti acum.
>
> Ordinea de pornire e in `CLAUDE.md`, sectiunea 0.

**Actualizat:** 2026-11-18 · commit `a35dc85` · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (472 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic si pe fixture real)

---

## Unde suntem

Etapele 0-7 incheiate. **Exista o aplicatie completa si utilizabila**, fara
interfata grafica: binarul `gmotion`.

| Crate | Stare |
|---|---|
| `rust/core` | gata, 50 teste, fuzz pe invariante, lexer T-SQL |
| `rust/db` | gata, verificat pe SQL Server real (auth SQL si Windows), citire paralela, 48 teste |
| `rust/platforma` | gata, DPAPI verificat pe Windows, 40 teste |
| `rust/module/diff-obiecte` | gata, 87 teste; planul de deploy cu garda si revenirea (etapa 12) |
| `rust/cli` | gata, 39 teste, din care 17 pe binarul adevarat; `plan`, `deploy --plan` |
| `rust/fereastra` | aspectul nou (etapa 13): conectarea, lista cu harta, editorul cu doua panouri, planul de deploy; doua moduri (intunecat / luminos) cu butonul de tema. Lista are acum coloanele Similar si Deploy, filtre pe stare si pe scor, sortare pe fiecare coloana, Export CSV, Ctrl+F |
| `rust/app` | `gmotion-ui`: conectare → lista ↔ editor → deploy (garda, revenirea, scrierea); **la paritate cu v3.4**, in afara randurilor ramase in `cercetare/lipsuri-portare.md` |

```
cd rust && cargo build --release -p gmotion-cli
./rust/target/release/gmotion --help
```

**Etapa activa:** `etape/15-interfata-si-incredere.md` — interfata care nu te
incurca (lista ramane in stanga cand deschizi, incadrare la orice rezolutie,
harta curata, denumiri „Martor" / „Modificata", comutator de limba) si
incredere in motor pe cod scris urat. Etapa 14 s-a incheiat pe 2026-11-18.

```
unelte\porneste.bat                  # Windows: compileaza si intra in previzualizare
unelte/porneste.sh                   # Linux / WSL (cere ecran)
unelte\porneste.bat --tema luminos
unelte\porneste.bat --server         # ecranul de conectare, gol
```

---

## Urmatorul pas concret

**Etapa 15, pachetul A (motorul si testele), e in lucru.** Facut: cazul
raportat de Mihai a intrat in harness si s-a dovedit ca motorul da exact ce da
v3.4; instantaneul de previzualizare are acum un ciclu de cursor intreg, cu
`WHILE` imbricat si `IF/ELSE`, scris urat; exista un test cap-coada care aplica
toate blocurile pe el si verifica forma normalizata.

**Facut din E:** textele interfetei au iesit din `.slint` intr-un tabel unic
(`fereastra::texte`), in romana si engleza, cu comutator in antet si alegerea
tinuta minte; „Sursa" / „Baza" sunt acum „Martor" / „Modificata".

**Facut din G:** panoul de detalii din lista (cerut de Mihai): lista pe toata
latimea pana la clic, detaliile in dreapta cu impact estimat si nota, maner
de latime, X / Esc / al doilea clic inchid, dublu clic deschide editorul.
Din `cli`: `gmotion impact`, `revizuire --nota`.

**Facut din B:** lista ramane in stanga cand deschizi un obiect — coloana de
300 px, clic pe rand = alt obiect pe loc, Esc o largeste la loc.

**Auditul QA (etapa 15, pachetul H) e in lucru, inaintea oricarui alt
lucru** — cererea lui Mihai. `cercetare/audit-qa-2026-11-18.md` a fost facut
pe `aec038a`; fiecare constatare se reproduce intai pe codul de acum cu un
test care cade, abia apoi se repara. Verdictele intra in sectiunea
„Verdicte" de la finalul auditului.

**H1 (siguranta deploy-ului) e gata**: C1, C2, C5, C6, C7, C8, F-D1, F-D2,
F-A5, cate un commit fiecare, cu poarta verde inainte de fiecare. Toate noua
s-au reprodus pe codul de acum. F-D7 a ramas nefacut, deliberat: cere o
proba pe Windows (vezi mai jos).

**Tot ce e de facut, in ordine:**

1. ~~**H1** siguranta deploy-ului~~ — gata (2026-11-18).
2. **H2** pierdere de date: C3, C4, F-B4, F-A1, F-A2, F-A3.
3. **H3** cadere si ecrane: C9, V9, V1, V2, V3, V10.
4. **H4** inghetari: P4, P1, P2, P3.
5. **H5** restul auditului: verdict pe fiecare constatare (reparat / nu se
   mai reproduce / amanat cu motiv).
6. **15 E** ce a ramas: „Martor" / „Modificata" si in `cli` si in documente;
   textele facute in Rust in afara ferestrei (`core::nume_tip`,
   `core::clasificare`, rezumatele de deploy, erorile) si in engleza.
7. **15 B** marginea dintre lista si editor trasa cu mouse-ul.
8. **15 C** incadrarea la 1366x768 si 1920x1080, titlul pe doua linii,
   comutatorul Alaturat / Unificat, liniile lungi.
9. **15 D** harta: desen curat, 4.000 de obiecte, timpii masurati.
10. **15 F** cautarea de servere vizibila; marcajul „aceeasi instanta".
11. Incheierea etapei 15: capturi, poarta completa, `STARE.md`, `JURNAL.md`.
12. **Etapa 16** (`etape/16-lista-si-taburi.md`, NEPORNITA): lista din
    stanga cu pin / ascundere / mutare; taburi (previzualizare + maxim 8
    fixate; `j` / `k` sar peste obiectele fixate). Fara note pe blocuri in
    editor (Mihai). Macheta: `cercetare/machete/sina-si-taburi/`.
13. Fara etapa inca: evidentierea de sintaxa **in timp ce scrii** in editare
    (singurul rand ramas in `cercetare/lipsuri-portare.md`); decizia pe `rfd`.

Criteriile: `etape/15-interfata-si-incredere.md` si `etape/16-lista-si-taburi.md`.

- **Platforma cu module:** deocamdata ramane un singur modul, „comparare
  obiecte" (`diff-obiecte`).

**Rulare pe Windows, de cerut lui Mihai cand are timp** (ce n-am putut proba
pe Linux):

```
gmotion-ui --previzualizare
gmotion-ui --tema luminos --previzualizare
```

si, pe un server real, o comparare cu **doua-trei baze bifate**. Ce vreau sa
stiu: (1) daca umbrele insulelor din editor se misca greu prin RDP (daca da,
se scot umbrele, nu insulele); (2) cat tine bara de progres pe bazele lui si
daca „Anuleaza" raspunde pe loc; (3) daca tragerea unui bloc peste panoul din
dreapta se simte natural sau e prea sensibila.

**Si o proba de SQL, pentru F-D7** (oprirea deploy-ului intre obiecte). Nu pot
afla pe Linux ce vede `@@ERROR` **dupa un `GO`**, iar fara asta n-am ce scrie
in script. Pe o baza de test, in SSMS sau `sqlcmd`:

```
CREATE OR ALTER PROCEDURE dbo.pTestEroare AS SELECT * FROM dbo.TabelaCareNuExista;
GO
IF @@ERROR <> 0 PRINT 'eroarea se vede dupa GO'; ELSE PRINT 'nu se vede';
GO
```

Imi trebuie exact ce tipareste al doilea batch. Daca iese „nu se vede",
oprirea intre obiecte se face altfel (`sqlcmd -b`, sau o variabila de
sesiune scrisa de fiecare batch), si atunci e o decizie, nu o reparatie.

Ramase deschise, documentate: performanta patratica pe obiecte uriase (se
vede si in editor: ~3,1 s pentru un obiect de 20.000 de linii, o singura data,
la deschidere), nume case-insensitive intre baze, divergentele Unicode fata de
JS (in afara de BOM), serverul in cheia sesiunii, nume rezervate Windows ca
nume de modul, dependentele prin SQL dinamic sau sinonime.

## Ce astept de la Mihai

**Decizii vechi, inca deschise:** implicitul pentru `--certificat`;
specificatia modulului `upgrade`; repo-ul public.

**Doua decizii noi, din etapa 14:**

6. **Dialogul nativ de fisiere (`rfd`)?** Acum „Deschide un instantaneu" arata
   `.json`-urile gasite pe disc si accepta o cale scrisa. Un dialog nativ ar fi
   o dependinta noua pe care n-o pot proba fara ecran. Daca il vrei, intra cu
   intrarea lui in `JURNAL.md`.
7. **Ecranul „Despre"** (cerut de licenta Slint): textul il dai tu — ce scrie
   acolo despre licenta si versiune.

Intrebarile de design din etapa 8 au primit raspuns (2026-11-17, vezi
`JURNAL.md`): contrastul se repara intunecand minim doua jetoane; modul
unificat primeste doua jgheaburi de numere; comentariul singur ramane zgomot
filtrat, deci randul „difera doar in comentariu" din `Main.dc.html` ramane
nefolosit.

**Doua intrebari din etapa 12** (ecranul de deploy exista acum; raspunsul
schimba ce face, nu cum arata):

4. **Deploy-ul sterge obiectele care nu mai sunt in sursa?** Macheta are un
   pas `DROP`. Acum obiectele `extra` se sar, ca in v3.4.
5. **„Ruleaza" executa scriptul pe server din aplicatie?** Acum GMotion doar
   scrie `.sql`; rularea e in SSMS / sqlcmd. Daca da, e o etapa proprie.

**Trei intrebari din etapa 11:**

1. **Selector de fisiere** (pentru „Deschide un instantaneu" si, acum, pentru
   directorul de deploy)? Ar fi o dependinta noua (`rfd`, dialogul nativ) si
   nu o pot proba pe Linux fara ecran. Pana atunci, instantaneul se da la
   pornire (`gmotion-ui --sursa f.json`), iar deploy-ul se scrie in
   directorul de date.
2. ~~Mediul (PROD / TEST / DEV) de unde se ia?~~ Inchisa: eticheta a fost
   scoasa de peste tot (Mihai, 2026-11-18), fiindca nu exista de unde sti.
3. **Compararea pe server real blocheaza fereastra** pana se termina (12 s pe
   baza ta, din masuratori). Machetele n-au ecran de progres; v3.4 avea. Il
   facem, sau e acceptabil asa?

Ramane deschis si: paritatea portii pe `cmd` (acum cere Git Bash sau WSL).

## Capcane active

Lucruri care te-ar face sa gresesti chiar acum, daca nu le stii:

- **Textul de afisare e si textul care pleaca in baza.** `pregateste_afisare`
  da si ce vede editorul, si ce trimite „Aplica toate". De-aia nu mai atinge
  interiorul sirurilor (C5): granitele vin din `lexer::segmente`. O regula
  noua pe text acolo schimba datele produse de proceduri in PROD.
- **Un text final egal cu textul bazei NU e lipsa de editare**, e decizia
  „pastreaza tinta" (modul ghidat, „din tinta" la tot). De-aia `Editari::pune`
  nu-l mai sterge; doar „Reset" sterge decizia. Daca il stergi iar, deploy-ul
  scrie toata sursa peste ce a ales omul sa pastreze (F-A5).
- **Planul de deploy nu se refoloseste pe o cheie incompleta.**
  `deploy::Cheie` = ce e in plan + amprenta textelor finale +
  `Aplicatie::id_comparare`. Daca adaugi o stare noua de care depinde planul
  si n-o pui in cheie, se intoarce C1/C2: scriere cu text vechi sau cu garda
  mostenita de la alta comparare.
- **Liniile scrise de GMotion in scripturi au `\r\n`, codul obiectului nu se
  atinge.** `crlf()` nu se cheama niciodata pe un text care contine `o.cod`
  (F-D2 si C5 sunt aceeasi regula din doua parti).

- **Dublu clicul pe rand trimite intai doua clicuri.** Slint cheama
  `clicked` de doua ori, apoi `double-clicked`: panoul se deschide, se
  inchide, apoi `Deschide` il redeschide. Un test care asteapta doar
  „deschide" la dublu clic e gresit.
- **Nota nu se rescrie din Rust cat scrii.** `incarca_detalii` pune `nota`
  doar cand se schimba `nota-id`; altfel fiecare tasta ar muta cursorul si ar
  taia spatiul de la capat. Nota se salveaza pe `nota-id`, nu pe selectie.
- **Impactul lexeaza toata baza comparata la prima deschidere a panoului**
  (`OnceCell` in `Aplicatie`). Pe baze mari, primul clic poate dura; nu
  muta calculul in `deseneaza` fara panou deschis.
- **Intre doua ecrane se arata intai fereastra noua, apoi se ascund
  celelalte.** `slint::run_event_loop()` se opreste cand nu mai e **nicio**
  fereastra vizibila, deci „ascunde tot, apoi arata" inchide aplicatia pe loc
  — exact la trecerea de pe conectare pe lista. Gasit de Mihai pe Windows,
  2026-11-18; trecerea sta acum intr-un singur loc, `main::arata`.
- **Nicio fereastra nu se probeaza pe Linux-ul meu.** Capturile randeaza
  ecranul direct, fara bucla de evenimente, deci nu spun nimic despre
  pornire, inchidere sau trecerea intre ecrane. Un bug de ciclu de viata se
  vede doar rulat pe Windows.

- **Cheia unui rand e `(baza, schema.nume)`, nu `schema.nume`.** De cand o
  referinta se compara cu mai multe baze, acelasi obiect apare o data pentru
  fiecare baza. Orice cautare dupa nume singur loveste randul gresit:
  `Lista::{pune_deploy, scoate_din_plan, in_plan}` si `Aplicatie::rand` cer
  amandoua partile.
- **Obiectele identice sunt puse in lista de `Aplicatie`, nu de comparare.**
  Randurile lor nu exista in `Comparare::randuri`. Cine construieste o lista
  pe langa `Aplicatie` pierde categoria „Identice" — si asta a fost exact
  bug-ul din captura.

- **Bifa nu re-aplica filtrul pe loc, si asta e deliberat** (ca in v3.4:
  `marcheazaRezolvat` nu cheama `renderLista`). Cu filtrul „Nerevizuite"
  activ, randul bifat ramane pe ecran pana se atinge filtrul din nou. Un test
  care asteapta sa dispara pe loc e gresit, nu codul.
- **Codurile coloanelor sortabile stau in doua locuri:** `global Sortare` din
  `lista.slint` si `model_lista::Ordine` (prin `cod_ordine` /
  `ordine_din_cod`). Daca una se muta si cealalta nu, clicul pe „Tip" ar
  sorta dupa linii fara ca nimic sa cada; de-aia exista
  `codurile_de_sortare_se_potrivesc`.
- **Un cap de coloana care se intinde primeste `min-width`, nu `width`.**
  Cu `width: 0px` coloana „Obiect" se strivea la zero si titlul disparea,
  desi `horizontal-stretch` era pus.

- **Un buton din schita nu se deseneaza pana nu are logica.** Schita
  (`cercetare/schita-slint/`) are butoane pe care aplicatia inca nu le are
  (Aplica toate, Mod ghidat...). Cand le portezi, adaugi logica si butonul
  in acelasi pas, cu test.
- **Ce patrat de pe harta e sub mouse se decide doar in `harta::indice_la`.**
  Slint trimite coordonatele. Nu repeta calculul in `.slint`.
- **Antetul listei curge dupa latimea ferestrei in `lista.slint`** (`bara`:
  grupuri pozitionate din `preferred-width`). Un buton nou intra intr-un
  grup existent, nu ca element separat, altfel nu mai e luat in calcul la
  trecerea pe randul urmator. Sub 760 px fereastra nu se mai strange.
- **Cateva masuri ale hartii sunt scrise de doua ori:** `harta::CAP` (44),
  `Y_BARA` (28) si `SPATIU_X / 2` (linia la -20 px) apar si in `lista.slint`.
  Treapta de zoom ajunge in fereastra doar ca latura (`patrat`); clicul o
  reface prin `Scara::cu_latura`, deci doua trepte cu aceeasi latura ar
  strica clicul.
  Daca muti una doar intr-o parte, clicul nimereste patratul de sub cel
  aratat. Testele din `harta.rs` nu vad `.slint`.
- **Fara diacritice nicaieri** (poarta cade). Datele de test care verifica
  nume cu diacritice raman, marcate cu `diacritice-permise` pe aceeasi linie;
  `cargo fmt` poate muta comentariul pe randul urmator, si atunci poarta cade.
- **Un jeton nou se adauga in trei locuri:** campul in `jetoane::Paleta` (cu
  valoare in ambele palete), proprietatea in `Tema` din `comune.slint`, si
  linia din `cu_jetoanele!` in `lib.rs`. Compilatorul prinde campul lipsa din
  lista si linia fara proprietate in `Tema`. **Nu prinde** o proprietate pusa
  in `Tema` si uitata in rest: ramane transparenta, iar ecranul o deseneaza
  invizibil.

- **`cercetare/schita-slint/` nu e in workspace si nu trece prin poarta.**
  E schita de fezabilitate a noii directii vizuale. Se construieste cu
  `CARGO_TARGET_DIR=../../rust/target`, ca sa refoloseasca Slint deja compilat.
  Culorile din `ui/schita.slint` se muta in `jetoane.rs` la portare.

- **Paritatea cu v3.4 nu mai e totala, deliberat.** Normalizarea nu mai atinge
  string-urile, „identic" nu mai foloseste scorul, criptatele au categorie
  proprie, iar de la auditul QA acelasi cod cu alt `QUOTED_IDENTIFIER` nu mai
  e „identic" (C6). Orice diferenta noua fata de v3.4 intra in `*.asteptari.json` cu
  motivul ei — niciodata „ca sa treaca poarta". Divergentele fata de JS stau
  in `DIVERGENTE_DELIBERATE` din `core/tests/conformitate.rs`.
- **Granitele cod / string / comentariu se decid intr-un singur loc:**
  `core/src/lexer.rs`. O regula noua pe text (in motor sau in deploy) care nu
  trece prin el reintroduce clasa de bug-uri reparata la etapa 7.
- **Testul de conformitate care cade nu inseamna fixture gresit.** Presupunerea
  implicita e ca portul Rust e gresit. Fixture-ul se regenereaza doar explicit,
  prin `unelte/regenereaza-fixture.sh`, cu motivul in `JURNAL.md`.
- **Un container nou nu are tintele wasm32 si windows-msvc.** Poarta le sare
  si iese tot „verde". Citeste lista de verificari sarite; instaleaza-le cu
  `rustup target add wasm32-unknown-unknown x86_64-pc-windows-msvc`.
- **`paritate.py` ruleaza binarul pe care i-l dai, nu sursa.** Dupa o
  modificare in Rust, reconstruieste inainte de a citi raportul — altfel
  masori binarul vechi. (Poarta il reconstruieste singura.)
- **Nu da comenzi cu parola scrisa in ele, nici in PowerShell.**
  `$env:GMOTION_PAROLA = "..."` ajunge in istoricul PSReadLine, pe disc. Parola
  se ia prin `Get-Credential`.
- **Pe Windows ruleaza cu 2 teste mai putin decat pe Linux, si e corect**
  (`auth_integrata_e_refuzata_devreme_pe_linux`, `pe_linux_marcajul_...`). Pe
  Windows in schimb ruleaza `pe_windows_datele_nu_apar_in_clar`.
- **`mostenire/` nu se atinge.** Poarta cade pe orice modificare de continut
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
- **Nu doar mediul e stare globala intre teste.** Tamponul de jurnal din
  `platforma` si abonatul `tracing` (instalat de primul test care cheama
  `porneste`) sunt partajate de tot procesul. Un test care presupune ca nu
  scrie nimeni altcineva acolo cade ~1 data din 100. Logica se testeaza pe o
  instanta locala.
- **Un tip care tine acreditari nu are `Debug` derivat.** Ar tipari parola la
  primul `dbg!` sau in orice mesaj de panica. Se scrie de mana, cu redactare.
  Vezi `rust/db/src/sqlserver.rs`.
- **O conectare care merge pe Linux nu dovedeste nimic despre Windows.**
  Furnizorul de TLS difera intre tinte (`rustls` vs SChannel), si auth integrata
  nu exista pe Linux deloc.
- **O masuratoare de virtualizare cu plierea pornita nu masoara nimic.**
  `pliaza-identicul` lasa in lista ~160 de randuri dintr-un obiect de 30.000:
  timpul pe cadru iese plat si cu `ListView`, si fara el. `--example masura`
  ruleaza de-aia si nepliat; acolo se vede diferenta (12 ms fata de 711 ms).
- **In `.slint` nu se scrie nicio culoare.** Jetoanele stau in
  `rust/fereastra/src/jetoane.rs`, unde testul de contrast le vede; `Tema` din
  Slint n-are valori implicite, tocmai ca o culoare uitata sa iasa
  transparenta si sa cada testul, nu sa semene cu ceva.
- **Editorul arata culorile in sensul deploy-ului, motorul in sensul
  referintei.** Verde = doar in sursa (se va adauga in baza) = `Stare::Sters`
  in model. Incrucisarea e doar in `cod_stare` din `fereastra/src/lib.rs`.
  Daca o muti sau o dublezi, culorile se inverseaza fara ca vreun test de
  motor sa cada.
- **Poarta verifica acum ca fiecare `callback` al unei ferestre are
  `on_...` in Rust** („fiecare buton are actiune"). Un callback intern (al
  unei componente, nu al ferestrei) nu e prins: acolo raspunde testul care
  apasa tastele (`tastele_editorului_cheama_actiunile`).
- **Editorul arata sursa fata de textul FINAL, nu fata de baza.** Dupa
  „Aplica", diferenta dispare din editor; liniile schimbate fata de baza sunt
  marcate (adus / scris de tine) de `marcheaza_textul_final`. Textul final e
  legat de amprenta: daca obiectul se schimba pe server, editarea cade
  singura (`modul::Editari`).
- **O captura nu dovedeste ca un buton merge.** Uneltele de captura pornesc
  ecranul direct in starea ceruta; un comutator nelegat arata perfect in
  captura. Asa s-au declarat „gata" etapele 8-11 cu 7 butoane moarte. Un
  buton se probeaza prin handler-ul lui (`on_*` in `app/src/main.rs`) si
  printr-un test care il apasa.
- **Directorul de deploy are acum si subdirectorul `revenire/`.** Un test sau
  un script care ia „primul fisier" sau numara intrarile din director
  greseste. Revenirea sta separat tocmai ca `*.sql` din director sa nu o
  includa.
- **Garda de deploy primeste obiecte recitite.** Data cu cele de la comparare,
  compara un lucru cu el insusi si trece mereu.
- **Doua teste care folosesc acelasi director temporar se strica unul pe
  altul.** `cargo test` ruleaza in paralel; un test care sterge directorul la
  final il sterge de sub celalalt. Numele directorului primeste si o eticheta
  a testului, nu doar PID-ul. Pacalit o data la `app/tests/flux.rs`.
- **Un fuzz care trece nu e un fuzz care functioneaza.** Cand adaugi un test de
  invariante, strica deliberat codul si uita-te daca il prinde. Prima versiune a
  fuzz-ului de segmentare trecea si cu urmarirea literalelor scoasa complet.
