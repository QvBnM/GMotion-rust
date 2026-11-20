# Stare

> **Primul fisier de citit la pornirea unei sesiuni.** Spune unde suntem si ce
> urmeaza. Istoricul a ceea ce s-a facut e in `git log`; aici sta doar ce git nu
> poate spune: ce urmeaza, ce e blocat, si ce te-ar face sa gresesti acum.
>
> Ordinea de pornire e in `CLAUDE.md`, sectiunea 0.

**Actualizat:** 2026-11-20 · branch `claude/upbeat-ramanujan-dm6j8c`
**Poarta:** verde (572 teste; `core` pe wasm32, tot codul si pe windows-msvc; paritate Python/Rust pe instantaneu sintetic si pe fixture real)

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

**Etapa activa:** `etape/21-editorul-functiile-v34.md` — functiile v3.4 din
editor, in formatul de acum, cu machetele-tinta in
`cercetare/machete/editor-functii/`. Etapa 20 (motorul) si 17 (conectarea
noua) s-au incheiat pe 2026-11-20.

```
unelte\porneste.bat                  # Windows: compileaza, ecranul de conectare
unelte/porneste.sh                   # Linux / WSL (cere ecran)
unelte\porneste.bat --tema luminos
unelte\porneste.bat --previzualizare # instantaneul sintetic, fara server
```

---

## Urmatorul pas concret

**Harta inapoi, ca ecran separat** (`h` din lista, dupa macheta lui
Mihai, 2026-11-19): de aratat lui Mihai capturile
`cercetare/capturi/harta-*.png` si de luat „da"-ul pe ele.

**Etapa 19 incheiata** (Object Explorer in coloana editorului, lista
tabel, harta pe `h`, wrap in editor). Mihai trimite sesizari daca apare ceva.
Ramase de confirmat pe Windows: ca filtrul nu mai trece de 100 ms (detaliile
sunt acum pe alt fir) si ca nu mai cade la Unificat cu wrap.

**Etapele 15 si 16 sunt incheiate.** Ramane:

1. **Conectarea noua (etapa 17): facuta, asteapta „da"-ul lui Mihai** pe
   `cercetare/machete/conectare-noua/capturi/pereche-*.png` (macheta |
   aplicatia, 1440x900, ambele teme). Abaterile, listate in etapa: textele
   aplicatiei (Martor / Modificata, „Continua"), butonul ⇄ si „Ascunde" din
   jurnal au disparut (nu erau in macheta). De cerut pe Windows: aceeasi
   fereastra pe un laptop la 150% si pe un monitor mare, si mutata intre
   ele — scara trebuie sa se recalculeze la mutare.
2. **Editorul ca in macheta:** `etape/18-editorul-ca-in-macheta.md`
   (NEPORNITA). Etapa 16 a facut functiile, dar Mihai a gasit ca editorul nu
   seamana cu `cercetare/machete/sina-si-taburi/macheta.png`. Se bifeaza doar
   cu captura pusa langa macheta si vazuta de Mihai. Ordinea fata de 17 o
   alege Mihai.
3. Fara etapa inca: evidentierea de sintaxa **in timp ce scrii** in editare
   (singurul rand din `cercetare/lipsuri-portare.md`). De probat intai daca se
   poate in Slint (`TextInput` nu are culori pe bucati).

La pornire: `python3 -I unelte/harta-cod.py`, apoi cauti in `HARTA-COD.md`
si citesti doar intervalul (CLAUDE.md §0).

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

## Ce a adus etapa 20 (motorul), pe scurt

Nu e in `git log` doar ca lista de commit-uri, ci ca schimbare de capabilitati:

- **alinierea coboara la linie** in interiorul unui bloc imperecheat: `AS` sta
  langa `AS`, nu langa `@UserId INT,`. Liniile goale si comentariile ies din
  imperechere (normalizarea oricum le ignora) si sunt marcate `ignorata`;
- **stratul de plasare exista** (`core::plasare`, port 1:1 din v3.4, cu
  pragurile 55 / 8 / 40 / 70): fara el nu se puteau scrie drag & drop,
  „Plaseaza aici", randul-fantoma si modul ghidat corect;
- **lista si editorul numara la fel** (F-M4 inchis);
- **scorul nu mai e patratic**: 16.000 de linii in 0,53 s in loc de 38 s, cu
  acelasi rezultat;
- `gmotion diferente` arata si alinierea pe linii, si plasarea.

Masuratori curente: `cd rust && cargo run --release --example masura_scor`.

**Urmatorul pas concret:** etapa 21 — maparea bloc -> loc in canal, randul
fantoma „aici vin blocurile ... din Martor", eticheta pe bloc, drag & drop,
previzualizarea la hover. Toate au acum pe cine intreba:
`modul::plasari_obiect`. Tinta: `cercetare/machete/editor-functii/`.

## Ce astept de la Mihai

(Nimic care sa blocheze etapa 21.)

**Decizii inca deschise** (nu blocheaza nimic): dialogul nativ de fisiere
(`rfd`); textul ecranului „Despre" (cerut de licenta Slint); implicitul pentru
`--certificat`; specificatia modulului `upgrade`; repo-ul public; paritatea
portii pe `cmd` (acum cere Git Bash sau WSL).

Deciziile din audit (F-M3, F-A18, F-B5) si din etapa 12 (`DROP`, „Ruleaza")
au primit raspuns pe 2026-11-19: vezi `JURNAL.md`.

## Capcane active

Lucruri care te-ar face sa gresesti chiar acum, daca nu le stii:

- **Imperecherea blocurilor se decide intr-un singur loc:** `plan_blocuri` din
  `core/src/diff.rs`. Si `diff_aliniat`, si `rezumat_rapid` citesc de acolo.
  Daca numeri diferentele altundeva, lista si editorul redevin doua pareri
  (F-M4).
- **Plasarea are doua surse, cu precedenta clara.** Cand diff-ul a imperecheat
  deja blocul (rand `modificat`), aia e plasarea; `core::plasare` raspunde doar
  pentru blocurile fara pereche. Vezi `modul::plasari_obiect`. Daca inversezi
  precedenta, „Aplica" scrie in alt loc decat arata ecranul.
- **`RandAliniat` are cate o pozitie in bloc pe fiecare parte.** Liniile nu mai
  sunt imperecheate pe index, deci un singur `linie_in_bloc` ar fi gresit pe
  una din parti.
- **Fixture-ul de plasare se regenereaza ruland pagina v3.4**, nu extragand
  cod: `unelte/regenereaza-fixture-plasare.sh` (cere `node` si `playwright`;
  poarta sare verificarea daca lipsesc).
- **Garda de cost (`core/tests/cost.rs`) e pe numarul de pasi, nu pe timp.**
  Daca o schimbi in masuratoare de timp, devine instabila pe o masina
  incarcata, si prima oara cand cade nimeni n-o sa o creada.

- **Fara `width: parent.width / N` pe copiii unui layout.** Layout-ul isi ia
  marimea din copii: in 1.19 da „Recursion detected" la rulare (testele o
  prind, compilarea nu). Impartirea egala: `horizontal-stretch: 1` si
  `preferred-width: 1px` pe fiecare.
- **Wrap-ul se calculeaza in Rust** (`rupe_linia`, randuri de continuare),
  la latimea din `coloane-wrap` (Slint). Latimea caracterului e aceeasi in
  ambele locuri (`CARACTER_MONO`, 7,52 px); daca se schimba fontul, se
  schimba in amandoua.
- **Fara `double-clicked` in `.slint`.** Pe Windows nu ajungea (jurnalul
  din 2026-11-19). Dublul clic se numara in Rust, cu `dublu_clic::DubluClic`,
  in handlerul clicului simplu. Daca il pui si in Slint, se deschide de doua ori.
- **Jurnalul de sesiune** e in `%APPDATA%\GMotion\jurnale\` (ultimele 10).
  Fiecare actiune are o linie; ce ruleaza pe alt fir (test, baze, comparare)
  isi scrie si rezultatul. Un buton fara linie in jurnal nu e legat.
- **Obiectul din tabul activ e mereu selectia listei** (`Lista::selecteaza_cheie`).
  O comanda noua pe „obiectul curent" lucreaza pe selectie, ca toate celelalte;
  nu tine un al doilea „curent" in taburi.
- **Pozitia editorului vine din `ocupat` / `stanga`** in `editor.slint`, nu
  din `latime-lista`: coloana poate fi fara pin (peste editor), banda sau pe
  dreapta. Un element nou asezat cu `latime-lista` iese sub coloana.
- **Mediul cloud: `~/.cargo/config.toml` fara debuginfo si incremental.**
  Altfel `target` creste la ~30 GB si linkerul cade cu „Bus error" (disc plin).

- **Harta se picteaza la pixelii fizici** (`picteaza_la(.., factor)`), iar in
  `lista.slint` imaginea are `width = source.width / factor-harta`. O imagine
  noua pictata cu `picteaza` (1x) si pusa acolo iese pe jumatate pe un ecran
  scalat 2x — sau, invers, uitata latimea, iese dubla.
- **Un obiect care „difera doar in hinturi" n-are blocuri in editor**:
  normalizarea uneste hinturile, deci editorul arata 0 diferente, desi lista
  il da modificat. E deliberat (F-M3): se pune in plan, iar deploy-ul duce
  textul exact. Nu e un bug de aliniere.
- **`Limba` sta in `core`**; fiecare ecran o afla cu `limba_curenta(ui)` din
  globalul `Texte`. Un text nou facut in Rust pentru om trece prin
  `limba.alege(ro, en)` sau `limba.numar(..)`, altfel ramane in romana cand
  fereastra e in engleza.

- **Modelul editorului se tine minte pe o cheie** (obiect + amprenta textului
  final + unirea cursorului), si `model_editor()` da un `Ref`, nu o copie. Daca
  adaugi o stare noua de care depinde modelul si n-o pui in `CheieModel`,
  editorul arata ceva vechi. `refaceri_model()` spune de cate ori s-a
  construit: asa se probeaza ca desenarea nu-l reface.
- **Un `Ref` tinut peste o chemare `&mut self` da panica.** De-aia desenarea
  din `main.rs` ia `salvat` inainte si face `drop(model)` dupa `incarca_editor`.
- **Nu mai exista al doilea lexer in `segmentare`.** Linia curatata vine din
  `lexer::segmente`. Daca scrii acolo inca o scanare de caractere, intorci
  P3: `[Case]` inghite tot obiectul.
- **Lucrurile lungi nu se mai fac pe firul ferestrei.** `fir::Lucru<T>` plus un
  slot in `Stare` si o bucata in ceas. Un apel nou care atinge serverul (sau o
  baza mare) merge la fel, altfel fereastra ingheata iar.
- **Spatiul pe disc al containerului se termina.** `rust/target` creste peste
  30 GB, mai ales `debug/incremental` si `debug/examples`. Daca un `cargo`
  cade cu „No space left on device": sterge-le, se refac singure.

- **Capturile se fac si la 1366x768 si la 1024x768**, nu doar la 1440x900.
  Toate erau la 1440 pana la auditul QA, si de aceea butoane iesite din ecran
  (V1-V3) n-au fost vazute niciodata. Comenzile: `cercetare/capturi/README.md`.
- **Intr-un `HorizontalLayout` cu `alignment: start` nimic nu se intinde.**
  `horizontal-stretch` e ignorat acolo; o bucata care trebuie sa ia restul
  latimii primeste `width` calculat (vezi `Jumatate` din `comune.slint`).
  Pus gresit, codul din editor dispare cu totul si raman doar numerele.
- **Cat se poate derula codul pe orizontala se afla din Rust**
  (`latime_continut`: cea mai lunga linie × latimea unui caracter mono).
  Slint nu da latimea unui text fara sa-l deseneze. Daca se schimba fontul sau
  marimea din editor, se schimba si constanta.

- **Un camp din Slint nu mai urmeaza modelul dupa prima tasta.** `text:
  root.text` e o legatura care se rupe la editare. De-aia `Camp` din
  `conectare.slint` rescrie textul pe `changed text` (F-A2). **Campul de
  parola face exceptie**: modelul ii da doar buline cat lungimea parolei, deci
  rescris cu ele prima tasta de dupa ar trimite bulinele **ca parola**. Daca
  scoti exceptia, testul `campurile_din_conectare_...` cade.
- **Fereastra de test n-are dimensiune.** `MinimalSoftwareWindow` porneste la
  0x0 si aranjarea iese degenerata: clicurile nu gasesc nimic din al doilea
  card. Un test cu mouse-ul cheama `window().set_size(...)` intai.
- **Ce se vede pe ecran nu e ce a rulat.** Compararea merge pe alt fir, iar
  formularul primeste tastatura cat ruleaza. De-aia `Comparata::referinta` si
  `Stare::referinta` exista, si de-aia `Stare::parte` intoarce `Option`
  (`None` cat ruleaza): compilatorul cere tratat cazul la fiecare camp (F-A3).
- **`curata` nu atinge bazele din afara compararii de acum.** Fara filtrul pe
  baze, bifele, omisiunile si textele finale ale unei baze comparate alta data
  se sterg toate (C3/C4). `Sesiune::adauga_baze` adauga, nu inlocuieste.
- **Cheia unei sesiuni e (sursa, referinta)**, iar identitatea sursei vine din
  `SursaObiecte::identitate`, nu din `descriere()` (care se schimba cand se
  schimba autentificarea) si nu din calea instantaneului.

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
- **Harta e acum un ecran separat** (`h` sau butonul din bara listei),
  desenat peste lista in `lista.slint` (`arata-harta`). Ce patrat e sub
  mouse se decide doar in `harta::indice_la`; Slint trimite coordonatele.
- **`moved` dintr-un `TouchArea` vine doar cu butonul apasat.** Pentru
  hover se asculta `changed mouse-x` / `mouse-y`. Bula hartii vechi nu
  aparea deloc la simpla trecere a mouse-ului; capturile o aratau doar
  fiindca o chemau cu `invoke_`.
- **Antetul listei curge dupa latimea ferestrei in `lista.slint`** (`bara`:
  grupuri pozitionate din `preferred-width`). Un buton nou intra intr-un
  grup existent, nu ca element separat, altfel nu mai e luat in calcul la
  trecerea pe randul urmator. Sub 760 px fereastra nu se mai strange.
- **Cateva masuri ale hartii sunt scrise de doua ori:** `harta::CAP` (44),
  `Y_BARA` (28), `SPATIU_X / 2` (linia la -20 px) si marginile ecranului
  hartii (`loc_harta` in `lib.rs`) apar si in `lista.slint`.
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
