# Audit QA — 2026-11-23, branch `claude/clever-cerf-hjk2qh`

Audit independent pe commit **`b815d0e946dd9f04a933fcef7e1bc6cb7059bb28`**
(„Tabelele in comparare: existenta, coloane, tipuri (fara deploy)", 2026-11-22).

Patru auditori separati, in paralel, fara acces la `cercetare/audit-qa-*.md` si la
`JURNAL.md` pana la propriile constatari:

- **A** — deploy, revenire, plan, texte finale, revizuire, sesiune
  (`rust/module/diff-obiecte`, `rust/platforma`);
- **B** — motorul `rust/core`, cu fuzz adversarial (45.436 de perechi);
- **C** — `rust/app` si `rust/fereastra`, functional si vizual (39 de capturi la
  1024x768 si 1366x768, ambele teme, privite);
- **D** — `gmotion` (cli) rulat efectiv, `rust/db` prin citire, si poarta
  `unelte/verifica.sh` **incalcata deliberat verificare cu verificare**.

**Poarta: VERDE**, 680 de teste, **nicio verificare sarita** (`wasm32-unknown-unknown`
si `x86_64-pc-windows-msvc` instalate inainte de rulare; `node` si `python3` prezente;
fmt, clippy, avertismente Slint, directia dependentelor, MSRV, api-hash pe toate cele 7
`CRATE.md`, stare/etape, diacritice, butoane cu actiune, `mostenire/` nemodificat,
harness Python si JS, paritate pe ambele fixture-uri).

---

## Nota: **5 / 10**

Pe zone: **deploy si sesiune 7**, **motor 6**, **interfata 6**, **cli + db + poarta 7**
(increderea in poarta, separat: **6**).

Media zonelor e 6,5. Nota totala e mai mica fiindca media ascunde severitatea: pe acest
branch exista **trei drumuri prin care „Aplica" sterge cod SQL in tacere** (reproduse pe
T-SQL banal, nu pe intrari patologice), **ecranul principal pierde complet coloana cu
numele obiectului** la rezolutia ceruta de contract, si **poarta isi rescrie singura
fixture-ul pe care regula 2 il declara intangibil**.

Partea buna, si e partea cea mai importanta a raportului: **aproape tot ce e critic e
deja reparat pe `claude/dreamy-ritchie-jdnfct`** si se rezolva prin unire. Problema reala
a acestui branch nu e ca are bug-uri greu de reparat, ci ca e **nereunit**: cele doua
branch-uri s-au despartit la `4af36b1` (2026-11-21), clever-cerf a mers mai departe cu
etapele 25-28 (26 de commit-uri, 58 de fisiere), dreamy-ritchie cu remedierea a doua
auditurilor (58 de commit-uri, 57 de fisiere), si **42 de fisiere sunt atinse de
amandoua**. Cu cat se intarzie unirea, cu atat reparatiile deja scrise stau nefolosite
intr-un branch pe care nimeni nu-l ruleaza.

Verificarile de mai jos pe `dreamy-ritchie` **nu sunt citite din auditurile lui**: am
rulat eu reproducerile lui B pe amandoua branch-urile, in worktree-uri separate.

| Reproducere | `clever-cerf` (b815d0e) | `dreamy-ritchie` |
|---|---|---|
| B-1 blocuri unite | `"SELECT 1\n\nUNION ALL\nRETURN;"` — **`SELECT 2;` pierdut** | corect |
| B-2 plasare peste bloc imperecheat | `"...\nSELECT @b FROM dbo.t;"` — **`SELECT @a` sters** | corect |
| B-3 antet pe pozitia 0 | `"SET NOCOUNT ON;"` — **`SELECT 1;` sters** | corect |
| B-4 fals „modificat" pe hash identic | `"1 modificate, 1 adaugate"` | **la fel, nereparat** |
| B-5 `plasari`, n=3200 | **8,52 s** | **0,61 s** |

---

## Constatari, in ordinea severitatii

### CRITIC

**B-1 — `linii_start_blocuri` aseaza blocurile unite pe linia gresita; „Aplica" pierde
linii din Martor.** `rust/core/src/segmentare.rs:545`, consumat de
`rust/core/src/aliniere.rs:333-340` si `:570`. Blocul unit (`UNION`, ciclu de cursor,
`INSERT ... SELECT`) nu e subsir al textului original, cautarea literala da `None`, iar
ramura de rezerva pune **linia de start a blocului precedent**. Pe corpusul real: **8 din
109 obiecte** au cel putin un bloc asezat pe linia gresita, intre care
`TESTSIMILITUDINE_ProcesarePlati` (blocul de 39 de linii al cursorului, raportat cu 11
linii mai jos). **VERIFICAT**, reprodus de mine pe ambele branch-uri.
**Deja reparat pe dreamy-ritchie** (`a8ef6b6`, verdictul N3 al auditului de acolo).

**B-2 — `plasari` muta un bloc peste un bloc al tintei pe care diff-ul il imperechease
deja; „Aplica toate" distruge o instructiune a tintei si nu converge.**
`rust/core/src/plasare.rs:216`, `:262`. `plasari` nu-i spune lui `sugereaza` care blocuri
sunt deja luate; la pragul 55 doua instructiuni asemanatoare (normal in cod ERP) trec de
prag. `CRATE.md` promite exact contrariul („perechea aleasa de diff bate"). Cade de
**1406 ori in 20.000** de perechi de T-SQL valid. Comportamentul e mostenit din
`mostenire/ui.py:3811` (acelasi prag, aceeasi lipsa de filtrare) — dar stratul `plasari`
a fost adaugat in Rust tocmai ca sa impace cele doua pareri. **VERIFICAT**.
**Deja reparat pe dreamy-ritchie** (`80aac4a`, `caee92a`).

**B-3 — regula „antetul se potriveste fara scor" se aplica oricarui prim rand, cu scor
fortat 100.** `rust/core/src/plasare.rs:209`: conditia e pur pozitionala
(`pozitie == 0`), nu verifica deloc ca randul e un antet `CREATE`/`ALTER ... AS`. Orice
bloc fara pereche aflat pe pozitia 0 inlocuieste blocul 0 al tintei. **VERIFICAT**.
**Deja reparat pe dreamy-ritchie** (verdictul M3-M6 de acolo: „antetul imperecheat nu se
inlocuieste").

**C-1 — coloana „Obiect" dispare din lista.** `rust/fereastra/ui/lista.slint:110-122`
(latimi fixe), `:1102` (`latime: 0px` pe coloana care se intinde). Latimile fixe insumeaza
**968 px**. La 1024x768 raman ~56 px pentru numele obiectului: patru randuri din sase scriu
identic `dbo.u…`. Cu panoul de detalii deschis (la orice rezolutie, inclusiv 1366 si 1440)
coloana ajunge la **0 px** si **nu mai exista nici capul de coloana** — tabelul incepe
direct cu „Tip". Nu exista derulare orizontala, deci continutul e pierdut, nu ascuns.
Regresie introdusa de etapele 27-28, care au adaugat „Folosit de" (76), „Impact" (72) si
„Ultima modificare" (132) fara sa micsoreze nimic. **VERIFICAT**, am privit capturile.
`.../auditor-c/capturi/lista-1024-intunecat-ro.png`, `det1366-deschis-intunecat.png`.
**Partial reparat pe dreamy-ritchie** (`51dd8a4` rezolva cazul 1024x768) — dar coloana
„Ultima modificare" nu exista acolo, deci **reparatia nu acopera singura din aceste doua
rupturi, si nici panoul de detalii**. Unirea nu e de ajuns; cere lucru dupa.

**C-2 — panoul de detalii iese din ecran.** `rust/fereastra/ui/lista.slint:1246`:
`Math.max(340px, Math.min(latime-fereastra - 860px, latime-detalii))` — planseul de 340 px
invinge limitarea. La 1024 se vede o fasie de ~55 px din panou, fara butoanele
Revizuit / In plan / Omite / Deschide. La 1366 se vad etichetele („Stare", „Similaritate",
„Impact estimat") dar **nicio valoare**: toate sunt la dreapta marginii ferestrei. Bara de
scurtaturi de jos dispare sau e taiata. **VERIFICAT**. **Nereparat nicaieri.**

**D-1 — poarta isi rescrie singura fixture-ul de referinta protejat de regula 2.**
`unelte/verifica.sh:372-382` ruleaza `node referinta/genereaza.mjs`, care **scrie**
`referinta/rezultate_js.json` (`genereaza.mjs:37`, `writeFileSync`); la fel
`referinta/genereaza.py:38`. CLAUDE.md regula 2: fisierul „se regenereaza **doar** prin
`unelte/regenereaza-fixture.sh`, explicit, ca pas propriu, cu motivul notat in
`JURNAL.md`". Cu motorul JS stricat deliberat, pasul raporteaza `ok` si `git diff` arata
`"scor": 88.24` → `"scor": 42`. Ordinea pasilor (teste la 3, harness la 10) inseamna ca o
schimbare in motorul de referinta da poarta rosie **o singura data**, iar a doua rulare e
verde pe fixture-ul nou: exact mecanismul prin care au divergat cele doua motoare din
v3.4, pus **in poarta**. Singura plasa ramasa e ca omul se uita la `git status`.
**VERIFICAT** (am confirmat scrierea eu insumi). **Nereparat pe dreamy-ritchie** — pasul
e identic acolo.

### MARE

**A-1 — `scrie`/`scrie_cu_revenire` sterg recursiv un director strain.**
`rust/module/diff-obiecte/src/deploy.rs:936-937`: `director.with_extension("tmp")`
**inlocuieste** ultima extensie. `/backup/Rapoarte.2026-11-23` → director de lucru
`/backup/Rapoarte.tmp`, sters recursiv si tacut inainte de orice verificare. Si
`a.1` / `a.2` partajeaza acelasi director de lucru. Se ajunge aici prin
`gmotion deploy --director` (`rust/cli/src/comenzi.rs:1131`). **VERIFICAT** (rulat).
**Deja reparat pe dreamy-ritchie** (`eaab16c`).

**A-2 — peste 999 de obiecte pe o baza, numele de fisier rup ordinea dependentelor.**
`rust/module/diff-obiecte/src/deploy.rs:1158`: `format!("{:03}_...")`. `1000_` si `1001_`
sorteaza alfabetic **inaintea** lui `100_`..`999_`, deci un view se compileaza inaintea
obiectelor pe care le foloseste — exact ce trebuia sa previna `ordine_dupa_dependente`.
Nicio verificare nu prinde asta. **VERIFICAT** (rulat, 1001 proceduri in lant).
**Deja reparat pe dreamy-ritchie** (`86cca42`).

**B-4 — fals „modificat" pe un obiect cu hash identic.**
`rust/core/src/segmentare.rs:177` + `:88` (`RE_POATE_CONTINUA` nu contine
`create`/`alter`). O linie goala dupa `CREATE FUNCTION dbo.f(@v ...)` sparge antetul in
doua blocuri. Pe `fnTVA`, obiect real din `referinta/instantanee/previzualizare.json`:
`normalizeaza` egal, `calc_hash` egal, scor 100 — si totusi `rezumat_rapid` spune
„1 modificate, 1 adaugate", `propozitie` spune „difera in cod". Incalca invarianta 1b din
`core/CRATE.md`. 23 din 20.000 de perechi curate. **VERIFICAT**, reprodus de mine.
**Nereparat pe dreamy-ritchie** — am rulat reproducerea acolo si da acelasi rezultat.

**B-5 — `plasare::plasari` e patratic, fara garda de cost.**
`rust/core/src/plasare.rs:262` → `:105`: token-izeaza toate blocurile tintei pentru
fiecare bloc fara pereche. Masurat de mine, release: `aliniaza + plasari_aliniere`
(ce face editorul la deschidere) **8,52 s la 3200 de blocuri / 104 KB**. `tests/cost.rs`
pune plafon doar pe pasii din `seqmatch`; `plasari` nu e acoperit.
**Reparat in mare parte pe dreamy-ritchie**: acelasi test, acolo, **0,61 s** (de 14 ori
mai repede). Ramane superliniar (×3,4 la fiecare dublare), deci plafonul de cost tot
trebuie adaugat.

**C-M1 — editarea de mana nesalvata se pierde tacut.**
`rust/fereastra/src/lib.rs:1092-1095`. In modul de editare `FocusScope` blocheaza doar
**tastele** (`rust/fereastra/ui/editor.slint:442-445`); mouse-ul ramane liber peste
coloana listei (`:1636-1641`), peste taburi (`:543`, `:583`) si peste insule. Un clic
acolo schimba obiectul, overlay-ul se inchide, textul scris dispare — fara avertisment si
**fara linie in jurnal**. **VERIFICAT** (cod + testul existent care descrie efectul).
**Deja reparat pe dreamy-ritchie** (`b391642`: „Un clic pe alt obiect cat scriai de mana
arunca textul; acum se salveaza pe obiectul lui").

**C-M2 — toate etichetele de bloc din editor sunt scrise direct in romana.**
`rust/fereastra/src/model_editor.rs:580, 582, 605-610, 629-640, 653, 656-660`:
„aici vine blocul {n} din Martor", „bloc {} · MODIFICAT", „se insereaza, nu inlocuieste"
s.a.m.d. In acelasi fisier `eticheta_pliere` (`:709`) **primeste** `Limba` si e tradusa —
deci e o scapare, nu o decizie. Pastila se vede pe fiecare bloc. **VERIFICAT**.
**Nereparat pe dreamy-ritchie** (textele se nasc in Rust, iar verificarea noua de acolo
se uita numai la `.slint`).

**D-2 — „core nu are I/O" nu e verificata de nimic, desi poarta pretinde ca o verifica.**
`unelte/verifica.sh:88`. Un `std::fs::read_to_string` adaugat in `rust/core/src/lib.rs`
**compileaza curat** pe `wasm32-unknown-unknown` (`Finished`, exit 0): `std::fs`,
`std::net` si `std::process` sunt stub-uri acolo. Invariantul pe care CLAUDE.md §3 il
numeste motivul existentei crate-ului trece verde. Ce prinde pasul de fapt: doar cod
imposibil pe wasm (`std::os::unix`) sau o dependinta noua. **VERIFICAT**.
**Nereparat pe dreamy-ritchie.**

### MEDIU

**Deploy si sesiune (A)**

- **A-3** — bifa „revizuit" si textul final nu cad la o schimbare pe care normalizarea nu
  o vede (spatii, capete de linie, `QUOTED_IDENTIFIER`). `revizuire.rs:33-38`,
  `editare.rs:52-57` compara `Amprenta::a_randului`, iar hash-urile vin din
  `comparare.rs:122-130` — **fara `setari`**. Doc-comentariul si invarianta 8d promit
  contrariul. Dauna la deploy e oprita de garda (`amprenta_exacta` include setarile), deci
  nu se scrie nimic gresit; raul e ca bifa minte. **VERIFICAT**. Nereparat nicaieri.
- **A-4** — „pastreaza tinta" la tot rescrie obiectul in baza, cu alte capete de linie si
  fara spatiile de la capat. `plan.rs:276-283`: textul final se ignora doar daca e egal cu
  afisarea **referintei**, nu si a tintei. Efect secundar: garda va opri planul urmator pe
  un obiect pe care nimeni nu l-a atins. **VERIFICAT**. Nereparat.
- **A-5** — fereastra suprascrie starea de sesiune scrisa de `cli` (pierdere de
  actualizare, fara incuietoare). `rust/app/src/stare.rs:174-193` vs
  `rust/cli/src/comenzi.rs:974-1004`. **VERIFICAT** (rulat).
  **Deja reparat pe dreamy-ritchie** (`bd02bf4`, „unire in trei cai pe obiect") — dar
  `rust/app/src/stare.rs` **nu exista** pe dreamy, deci reparatia va trebui dusa peste
  structura noua, nu copiata.
- **A-6** — `Sesiune::salvata` nu e scrisa niciodata de fereastra; „Compararile salvate"
  arata data goala si sorteaza gresit. `rust/app/src/stare.rs:180`, consumat de `:998-1012`.
  Doua linii lipsa (`acum()` se calculeaza deja alaturi). **VERIFICAT**. Nereparat.
- **A-7** — oprirea „un obiect cazut nu lasa restul sa ruleze" (F-D7) **nu trece intre
  fisiere**: `deploy.rs:1146-1150` pune `SET NOEXEC OFF` la capatul **fiecarui** fisier. In
  `PerObiect` garda nu are ce proteja, iar ordinea pe dependente e tocmai despre obiecte
  care depind unul de altul si ajung in fisiere diferite. `CRATE.md` 6c' nu spune asta.
  **VERIFICAT**. Nereparat.
- **A-8** — `CRATE.md` se contrazice pe F-A5: invarianta **8d** spune ca un text final egal
  cu textul bazei „se scoate", iar tabelul de API, din acelasi fisier, spune ca se
  pastreaza (si codul confirma a doua varianta). Cine implementeaza dupa 8d reintroduce
  F-A5. **VERIFICAT**.

**Motor (B)**

- **B-6** — `diff::marcaje_pe_linii` da offset-uri in **caractere** (`diff.rs:464`), iar
  `lexer::Segment` in **octeti** (`lexer.rs:33`); nedocumentat. Singurul consumator de azi
  foloseste corect `Vec<char>`, deci nu e bug activ — dar `&b[12..13]` ar da panica.
- **B-7** — `plasari` poate intoarce plasari in dezordine pe `bloc_tinta` (845 din 20.000);
  aplicarea mai multor blocuri intr-o trecere le scrie in alta ordine decat in Martor.

**Interfata (C)**

- **C-D1** — meniul temei e scris direct in `.slint`, in romana
  (`rust/fereastra/ui/comune.slint:548, 550`), desi `texte.rs` are deja `tema`,
  `intunecat`, `luminos` — **nefolosite**. In engleza, butonul de tema deschide un meniu
  romanesc pe toate cele 4 ecrane. **Deja reparat pe dreamy-ritchie**
  (`f6f0417`), care adauga si **o verificare noua in poarta**: „niciun text pentru om
  scris direct in .slint".
- **C-D2** — alte texte pentru om facute in Rust, netraduse: `„{n} baze"`
  (`app/src/main.rs:358`, `app/src/deploy.rs:274`), `„{n} randuri in {cale}"`
  (`app/src/stare.rs:737`), `„{n} revizuite"` (`:1010`), `„verifica intai ca baza n-a mai
  fost atinsa"` (`app/src/deploy.rs:150`), `„lucrul s-a oprit pe neasteptate"`
  (`app/src/fir.rs:62`), `impact.motive`. In plus, starea conexiunii ramane „inghetata" in
  limba de la conectare (`app/src/main.rs:560-565`). Partial reparat pe dreamy
  (`6f57f41`), dar fisierele noi de pe clever-cerf au reintrodus clasa.
- **C-D3** — F1 / „Ajutor" si „Despre" nu exista in Editor si in Deploy, desi textul de
  ajutor (`lib.rs:97-137`) are sectiuni pentru amandoua si se incheie cu „F1 deschide si
  inchide fereastra asta". **Deja reparat pe dreamy-ritchie.**
- **C-D4** — „Deploy (N)" din lista arata activ cand planul e gol si e refuzat tacut
  (`lista.slint:1072-1075` vs `rutare.rs:1005-1009`); acelasi buton in editor e corect
  dezactivat. **Deja reparat pe dreamy-ritchie** (`ButonPrincipal.activ`, V15).
- **C-D5** — „Aplica diferenta" nu lasa linie in jurnalul de sesiune
  (`app/src/main.rs:1046-1048`) — singurul handler de actiune reala fara `actiune(...)`,
  si tocmai cel care schimba textul ce pleaca in baza. „Un buton fara linie in jurnal nu e
  legat" (STARE.md).
- **C-D6** — `captura_deploy` nu mai produce ecranul documentat: de la etapa 28 primele
  randuri din instantaneul de previzualizare sunt **tabele**, refuzate din plan, deci
  captura iese cu planul gol si propozitia rupta. Criteriul din CLAUDE.md §10 („captura
  langa macheta") nu mai poate fi bifat pentru deploy.
- **C-D7** — `rust/app/CRATE.md` afirma lucruri false: „nu are inca selector de fisiere"
  (exista, `rfd`), „nu arata progresul unei comparari lungi" (exista `fir::Rulare`),
  invarianta 4 contrazisa de `rutare.rs:926-929`. `rust/fereastra/CRATE.md` scrie
  „Stare: prototip" la etapa 28.

**cli, db, poarta (D)**

- **D-3** — „Exact o etapa poate fi ACTIVA" (CLAUDE.md §0): poarta verifica doar
  `len(active) > 1` (`verifica.sh:265`), deci **zero** etape active trece verde — si asta e
  starea de acum (0 ACTIVA, 28 INCHEIATA, 1 ABANDONATA). Nereparat pe dreamy.
- **D-4** — `--json` nu produce JSON pe **nicio** cale de eroare: stdout e complet gol,
  codul `GM-DB-007` apare doar in text romanesc pe stderr. Verificat pe 6 cai. Partea
  buna: nu exista niciodata JSON **partial**.
- **D-5** — `gmotion --json export` si `--json instantaneu` nu scriu nimic pe stdout, cu
  exit 0 (`comenzi.rs:946`, `:194`). Celelalte 10 subcomenzi dau JSON valid.
- **D-6** — directia dependentelor se ocoleste cu `[dependencies.gmotion-x]` sau cu
  `baza = { package = "gmotion-db" }` (`verifica.sh:148`, regex pe textul `Cargo.toml`).
  Poarta foloseste deja `cargo metadata` la pasul MSRV. Nereparat pe dreamy.
- **D-7** — `api-hash` nu vede variantele de enum public (`Autentificare::Jeton`) si nici
  semnaturile metodelor de trait (`SursaObiecte::identitate`) — exact tipul de acreditari
  si traitul central. **Deja reparat pe dreamy-ritchie** (`6d88120`).
- **D-8** — `SursaSqlServer::verifica` (editie, versiune, uptime, numar de baze) nu e
  accesibila din CLI, desi comentariul din `sqlserver.rs:300-302` spune ca e pentru
  `gmotion diagnostic`. Incalca CLAUDE.md §3.
- **D-9** — erorile nascute in CLI n-au cod stabil (`Result<(), String>`); cele venite din
  `db` / `modul` si-l pastreaza corect. **Deja reparat pe dreamy-ritchie** (`50187f6`,
  `GM-CLI-001..008`).
- **D-10** — „fiecare buton are actiune" se ocoleste cu un `_` in numele callback-ului
  (`verifica.sh:345`, clasa `[a-z0-9-]+`): `callback compara_tot_acum()` se taie la
  `compara`, gaseste `on_compara` si trece verde. La fel `pure callback`, indentare de 8
  spatii, majuscule, handler cu corp gol. Nereparat pe dreamy.
- **D-11** — un fisier nou nestagiat nu trece prin verificarea de diacritice
  (`git ls-files`), iar fluxul normal e „ruleaza poarta, apoi `git add`".
  **Deja reparat pe dreamy-ritchie** (`--cached --others --exclude-standard`).

### MIC

- **A-9** garda de deploy nu expira, si schimbarea modului de impartire redeschide
  scrierea pe garda veche (`app/src/deploy.rs:132-136`, `:175-183`). **Deja reparat pe
  dreamy** (`6f1f1e5`).
- **A-10** in revenire, un tip de obiect necunoscut tipareste `RAISERROR` dar **nu**
  opreste scriptul, desi comentariul spune ca se opreste (`deploy.rs:1075-1086`, iese
  inainte de `opreste_daca_a_cazut`). **Deja reparat pe dreamy** (`bd02bf4`).
- **A-11** `deploy::scrie` pierde avertismentul „codul nu se inchide" desi nu depinde de
  tinta (`deploy.rs:1227-1234`).
- **A-12** testul de unicitate a codurilor de eroare acopera 12 din 14 variante — lipsesc
  tocmai `GM-MOD-013` si `GM-MOD-014`. **Pe dreamy e mai rau**: 12 din 18.
- **A-13** doua teste din `deploy.rs` folosesc acelasi director temporar `dir("gol")`
  (`:1620`, `:1981`), contra capcanei din STARE.md. Prezent si pe dreamy.
- **A-14** text pentru om scris direct in Rust fara `limba.alege`
  (`app/src/deploy.rs:150`). Prezent si pe dreamy.
- **A-15** `scrie_atomic` foloseste acelasi nume temporar in orice proces si nu face
  `fsync` (`platforma/src/cai.rs:67-74`). **PLAUZIBIL**. **Deja reparat pe dreamy**
  (`d244561`).
- **B-8** `loc_la_linie` da panica pe overflow in debug (`aliniere.rs:479`);
  `saturating_add` rezolva.
- **B-9** / **B-10** invarianta 5 si idempotenta lui `pregateste_afisare` cad pe `[` si `'`
  neinchise — intrare malformata, zero aparitii pe cele 109 obiecte reale.
- **B-11** cinci functii publice fara niciun test (`calc_hash`, `unifica_linii`,
  `clasifica_randuri`, `doar_comentarii`, `diff_aliniat_cu`), trei dintre ele nefolosite
  nicaieri in workspace. CLAUDE.md §6.
- **C-S1** functii publice moarte sau netestate in fereastra (`chei_vizibile`,
  `Stare::categorie`, `Stare::descriere` — ultimele doua inca scriu „Doar in sursa" in loc
  de „Martor"), plus 12 functii publice fara test direct.
- **C-S2**..**C-S9** texte traduse nefolosite; „IMPLICITA"/„CLUSTER" direct in `.slint`;
  `SalveazaText` refuzat se pierde fara mesaj; modul de impartire se reseteaza tacut cand
  se schimba planul; doar `FereastraLista` are `min-width`; `gmotion-ui` nu are `--help`
  si ignora tacut argumentele gresite; capturile din `cercetare/capturi/` arata scurtaturi
  dinaintea regulii „litera cere Ctrl"; uneltele de captura nu pot arata interfata in
  engleza pe 3 din 4 ecrane.
- **D-12** `CRATE.md` nu e verificat pe sectiuni; `rust/cli/CRATE.md` nu are „API public".
  Un `CRATE.md` redus la 3 linii cu hash-ul corect trece verde.
- **D-13** poarta face `touch rust/fereastra/ui/fereastra.slint` (fisier urmarit) la
  fiecare rulare, si scrie `.msrv.json` in radacina, care nu e in `.gitignore`.
- **D-14** verificarea de diacritice are setul incomplet (U+01CE, „a cu caron", lipseste — si exista
  in poarta insasi, `verifica.sh:146`) si nu se uita la `.html` (12 fisiere), `.sql` (5),
  `.txt` (2), `.json`.
- **D-15** stergerea unui fisier din `mostenire/` trece poarta (`--diff-filter=M`).
  **Deja reparat pe dreamy** (`6d88120`, „mostenire fixat pe arbore").
- **D-16** `unelte/regenereaza-fixture-plasare.sh:13` si STARE.md promit un pas de poarta
  pentru fixture-ul de plasare, care **nu exista** (`grep plasare unelte/verifica.sh` →
  nimic). Comportamentul real e cel corect; doar documentele mint.
- **D-17** `--pe-server`: un text final care contine `SET NOEXEC OFF;` ar putea anula garda
  si executa instructiunile urmatoare, desi comanda promite „fara efecte".
  **PLAUZIBIL** — cere SQL Server; proba exacta e mai jos.
- **D-19** `deploy --mod-fisiere cate-n --cate 0` e acceptat in tacere si reinterpretat.
- **D-20** `--port` / `--certificat` cu `--sursa` sunt acceptate si ignorate fara semnal
  (`requires = "server"` nu se aplica fiindca `--server` e in `conflicts_with`).
- **D-24** STARE.md:10 spune „646 teste"; poarta raporteaza 680.

### COSMETIC

`acum()` e in UTC fara sa spuna (antetul fiecarui script de deploy); setarile de sesiune
nu se refac la capatul scriptului; `Revizuire::scoate` nu curata hartile goale; tabelul de
API din `diff-obiecte/CRATE.md` e rupt in doua; un nume de obiect gol trece de
`verifica_inainte`; `CRATE.md` 5b spune „octet cu octet", fals pe CRLF brut;
`structura::verifica` raporteaza `GO` pe ultima linie (4 din 109 obiecte reale);
`unelte/stare.sh` numara testele cu metoda pe care poarta o respinge explicit in propriul
comentariu si nu semnaleaza „nicio etapa activa"; comentariul din `api-hash.sh` despre
`mod teste` e fals; `harta-cod.py` taie rolul fisierului in mijlocul cuvantului; filtrul
„Similar 0–100 %" iese din ecran la 1024; textul de stare din subsol e taiat la toate
rezolutiile; „Reviewed" si „Deploy" se ating in engleza; bula hartii acopera patratele de
langa ea; calea jurnalului din „Despre" se rupe dupa litera unitatii; lipsesc paleta de
comenzi Ctrl+K, banda de mediu DEV/TEST/PROD din `design-aprobat.md` §6, si legenda de
culori permanenta din macheta.

---

## Ce a tinut

**`rust/db` — practic nimic de reprosat la citire.** Zero injectie SQL: toate interogarile
sunt constante, numele de baza merge prin `c.database(baza)` (cu comentariul care explica
de ce nu se foloseste un „ADO string"), `definitie_tabel` dubleaza `]`. Doua validatoare cu
reguli diferite din motive corecte (strict pentru server, permisiv pentru baze, fiindca
numele de baza nu intra in SQL). Acreditarile au `Debug` scris de mana cu redactare, cu
teste de regresie; `Redactat` nu tipareste nici lungimea; probat empiric cu o parola reala
si `RUST_LOG=trace`: **0 potriviri** pe stdout, stderr si in directorul de date.
Timeout-uri peste tot (inclusiv lectia veche: „TCP, TLS si login, impreuna"), `try_get` in
loc de `get` pe fiecare coloana, conexiuni inchise la orice eroare, 13 coduri de eroare
fara refolosire.

**Garzile de deploy.** Am incercat sa le sparg si n-am reusit: injectie prin numele bazei
si al obiectului (`]` dublat, caractere de control refuzate **inainte de orice fisier**,
numele intra in `RAISERROR` ca argument, nu in mesaj); `GO` ascuns in definitie, inclusiv
in `N'...'` si in comentariu bloc; `CREATE` fals din comentariu, din string, din `EXEC(...)`;
nume redenumit / mutat in alta schema / calificat cu baza / in ghilimele (F-D4);
atomicitatea scrierii (directorul apare intreg sau deloc); coliziuni de nume de fisier
(`CON`, 220 de caractere, `../../etc/passwd`); ordinea revenirii (stergeri inainte de
refaceri, invers fata de dependente); pasi repetati si `dbo.p;1`. Garda primeste **obiecte
recitite**, si fara ea `poate_scrie()` e `false`.

**Motorul, pe tot ce nu e in lista de mai sus.** 45.436 de perechi de fuzz, **zero panici**
(in afara de B-8, cu `usize::MAX` dat explicit), zero bucle infinite, zero OOM, zero caz
peste o secunda in release. `norm` idempotenta pe 400.000 de intrari; `lexer::segmente`
acopera textul complet, fara goluri si suprapuneri, mereu pe granite de caracter, inclusiv
pe `/*` imbricate de 50.000 de nivele; `cheie_bloc` echivalenta exact cu egalitatea formei
normalizate (zero coliziuni in ~200.000 de perechi); alinierea nu dubleaza nicio linie;
`compune(..., |_| false) == b` exact pe text pregatit, pe toate cele 68 de comparari
reale. Costul e **liniar** pe toate axele patologice: 1 MB pe o singura linie → 49 ms la
`diff_aliniat`; 20.000 de linii identice (cazul in care `difflib` naiv moare) → 67 ms.
`core` nu are I/O si compileaza pe wasm32; toate cele 30 de `unwrap()` sunt pe regexuri
constante.

**Interfata, pe partea de disciplina.** **Niciun buton mort**: toate cele 161 de `callback`
din `.slint`, inclusiv componentele interne pe care poarta nu le vede, au handler. Zero
`unwrap`/`expect`/`panic!` pe cai cu date de utilizator. Nicio litera simpla legata direct
(regula lui Mihai din 2026-11-20 respectata peste tot), zero conflicte de `Ctrl+<litera>`,
si fiecare scurtatura documentata in `AJUTOR` chiar exista. `texte.rs` e complet: 221 de
campuri in ambele limbi, iar destructurarea fara `..` face imposibila uitarea unuia.
Nicio culoare in `.slint` — toate cele 48 de proprietati vin din Rust, cu test de contrast
AA pe ambele palete. 271 de teste in `app` + `fereastra`, toate verzi pe `b815d0e`.
Ecranul de conectare e cel mai bun din aplicatie: la 1024 si la 1366, in ambele teme,
incape tot si seamana cu macheta aprobata.

**CLI-ul.** Codul de iesire e corect nenul pe **fiecare** cale de eroare probata, mesajul
merge mereu pe stderr, si stdout nu se murdareste niciodata cu JSON partial. Nu exista
`--parola` ca argument, deliberat si cu test. `fara_control()` pe tot ce vine din DB
(un nume de baza nu poate rescrie terminalul). 27 de teste de integrare pe binarul
adevarat, 68 de invocari.

**Poarta, ce face bine.** Nu se opreste la prima eroare si raporteaza explicit ce a sarit —
verificat in viu cand clippy a cazut. Numara testele din rezumatele `test result:`, nu din
randurile `test ... ok`, cu motivul scris. Pasul „avertismente Slint" **functioneaza** si e
singura plasa pentru o clasa de bug-uri pe care nici clippy, nici testele n-o vad. Pasul
windows-msvc e justificat de masuratoare: clippy local e complet orb la `cfg(windows)`
(0 erori local, 2 pe tinta). Paritatea cu `--strict` si `*.asteptari.json` e cea mai buna
verificare din poarta. Verificarile care **prind** cand le incalci deliberat: fmt, clippy,
teste, avertismente Slint, wasm (compilare), windows-msvc, MSRV, api-hash (semnaturi),
>1 etapa activa, diacritice (pe fisiere urmarite), callback mort de fereastra,
`mostenire/` modificat, paritate.

---

## Ordinea recomandata a reparatiilor

1. **Uneste `claude/dreamy-ritchie-jdnfct` in `claude/clever-cerf-hjk2qh`.** Asta inchide
   B-1, B-2, B-3 (cele trei drumuri prin care „Aplica" sterge cod), A-1, A-2, A-9, A-10,
   A-15, C-M1, C-D1, C-D3, C-D4, D-7, D-9, D-11, D-15, aduce B-5 de la 8,5 s la 0,6 s, si
   aduce o verificare noua in poarta („niciun text pentru om in `.slint`"). Nu e o unire
   banala: branch-urile s-au despartit la `4af36b1`, **42 de fisiere sunt atinse de
   amandoua**, iar `rust/app/src/stare.rs` exista doar pe clever-cerf — deci reparatia A-5
   („unire in trei cai pe obiect") trebuie dusa peste structura noua, nu copiata. Dupa
   unire: poarta, si **capturile refacute** la 1024x768 si 1366x768.
2. **C-1 si C-2** — coloana „Obiect" si panoul de detalii. Reparatia de pe dreamy acopera
   doar 1024x768 fara panou; coloana „Ultima modificare" (etapa 27) si panoul deschis
   raman. `min-width` real pe nume (~180-220 px) si coloanele secundare ascunse sub un prag.
3. **D-1** — poarta sa **citeasca** fixture-ul de referinta si sa cada daca regenerarea
   l-ar schimba, in loc sa-l rescrie. Asta e singura constatare care ataca plasa de
   siguranta a intregului proiect.
4. **B-4** — `RE_POATE_CONTINUA` sa acopere `create`/`alter`, ca un obiect cu hash identic
   sa nu mai fie raportat „difera in cod".
5. **A-3, A-4, A-7** — amprenta pe `amprenta_exacta` (bifa si garda sa spuna acelasi
   lucru); textul final egal cu afisarea tintei sa nu intre in script; si o decizie
   explicita pe oprirea intre fisiere: ori `sqlcmd -b` scris in `CRATE.md` si pe ecran,
   ori un marcaj persistent intre fisiere.
6. **D-2, D-3, D-6, D-10** — golurile portii: interdictie textuala pe `std::fs` in
   `rust/core/src`, `len(active) != 1`, dependentele din `cargo metadata` in loc de regex,
   si `[a-z0-9_-]+` plus `(pure )?callback` la butoane. Fiecare incalcata o data, ca sa se
   vada cazand.
7. Restul mediilor (D-4, D-5, D-8, A-6, C-M2, C-D5, C-D6, C-D7), apoi micile si cosmeticele.

---

## De probat pe un SQL Server / pe Windows (nu se poate afla pe Linux)

1. **F-D7, `@@ERROR` dupa `GO`** — proba e deja scrisa in `STARE.md`. Auditorul A o
   confirma drept cea mai importanta intrebare deschisa din zona lui, mai importanta decat
   A-2: daca `@@ERROR` nu supravietuieste separatorului de batch, **toata** oprirea intre
   obiecte e un no-op, in deploy si in revenire.
2. **D-17, `SET NOEXEC OFF` in textul trimis la „Verifica pe server"**, pe o baza de test:

```sql
SET NOEXEC ON;
GO
CREATE TABLE dbo.gmotion_proba_noexec (x INT);
SET NOEXEC OFF;
CREATE TABLE dbo.gmotion_proba_noexec2 (x INT);
GO
SET NOEXEC OFF;
SELECT name FROM sys.tables WHERE name LIKE 'gmotion_proba_noexec%';
```

Daca apare `gmotion_proba_noexec2`, constatarea e reala si `--pe-server` poate avea efecte
pe server, desi promite ca n-are.
