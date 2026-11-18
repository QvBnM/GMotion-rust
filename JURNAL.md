# Jurnal de decizii

Append-only. Nu se rescrie o intrare veche; daca o decizie se schimba, se scrie
una noua care o anuleaza explicit, cu motivul. Asa se vede de ce codul arata
cum arata, fara arheologie in istoricul git.

Format: `## <data> — <decizia>`, apoi **Context**, **Alternative**, **Motiv**,
si unde e relevant **Consecinte**.

---

## 2026-11-17 — Codul Python se inghetata in `mostenire/`, nu se sterge

**Context.** GMotion v3.4 e in productie in Python. Rescrierea in Rust porneste
de la aceleasi commit-uri.

**Alternative.** (a) Python-ul ramane in dezvoltare paralela; (b) se sterge
acum, raman doar in istoricul git; (c) se inghetata in repo.

**Motiv.** (c). Harness-ul de conformitate are nevoie de `motor.py` si `ui.py`
ca sa poată regenera fixture-urile, si etapa de paritate are nevoie de o
versiune care nu se mișca. Dezvoltarea paralela ar face tinta de paritate
mobila; stergerea ar scoate plasa de siguranta.

**Consecinte.** `mostenire/` nu se modifica niciodata. Se sterge abia dupa ce
Rust-ul trece paritatea pe baze reale.

---

## 2026-11-17 — CLI headless inainte de interfata

**Context.** Planul initial trecea de la `db` si `platforma` direct la
prototipul de interfata.

**Alternative.** (a) prototip de editor de diferente devreme, ca sa se afle
costul partii grele; (b) CLI si paritate intai, interfata dupa.

**Motiv.** (b). Mediul de dezvoltare al asistentului e Linux, fara SQL Server si
fara fereastra. Daca tot ce e valoros trăiește in spatele unui GUI Windows,
fiecare bug devine o rulare manuala pe masina utilizatorului. Paritatea cu
Python — singurul lucru care permite retragerea versiunii vechi — nu depinde
deloc de interfata, deci nu are sens sa fie blocata de componenta cu cea mai
mare incertitudine de efort.

**Consecinte.** La finalul etapei `cli` exista o aplicatie completa si
utilizabila. `cli` ramane permanent, ca mod de depanare, si dupa ce apare
interfata. Riscul asumat: costul real al editorului de diferente se afla mai
tarziu decat in planul initial.

---

## 2026-11-17 — `SursaObiecte` ca trait, cu implementare din fixture

**Context.** Vezi decizia de mai sus.

**Motiv.** Tot ce e deasupra `db` trebuie testabil fara baza de date. In plus,
`gmotion instantaneu` scrie obiectele brute dintr-o comparare reala intr-un
JSON, care se reia offline — deci un bug de pe o baza de productie devine un
test de regresie.

**Consecinte.** `db` expune un trait, nu un tip concret. Modulele nu stiu
niciodata daca vorbesc cu SQL Server sau cu un fisier.

---

## 2026-11-17 — Portul Rust urmeaza semantica JS, nu `motor.py`

**Context.** Motorul de comparare exista in doua copii divergente. Masurat pe 24
de cazuri: 8 diverg, toata divergenta provenind din segmentare. Detalii in
`referinta/README.md`.

**Motiv.** Portul JS din `ui.py` e varianta mai noua si corecta; `motor.py` a
ramas in urma si produce fals pozitive vizibile utilizatorului (un comentariu
bloc pe mai multe linii il face sa raporteze doua blocuri sterse care nu
exista).

**Consecinte.** Fixture-ul de referinta e `referinta/rezultate_js.json`. Cand
testul de conformitate cade, presupunerea implicita e ca portul Rust e greșit,
nu fixture-ul. Divergentele raman deci intentionate: versiunea Rust **nu** e
identica cu `motor.py`, si asta e corect.

---

## 2026-11-17 — Dependentele lui `gmotion-core`

`regex`, `once_cell`, `md-5`, `sha2`, `hex`. `md-5` doar pentru cheia de bloc,
nu in context de securitate. `serde`/`serde_json` sunt dev-dependencies, fiindca
doar testele de conformitate si exemplul de bench citesc JSON — `core` insusi nu
face I/O si trebuie sa ramana compilabil pe wasm.

---

## 2026-11-17 — Driver SQL Server: `tiberius`

**Context.** Decizia deschisa din etapa 0. Versiunea Python foloseste ODBC si
cere un driver instalat pe masina tinta.

**Alternative.** (a) `odbc-api` — comportament identic cu azi, dar rămâne
dependenta de driver; (b) `tiberius` — TDS nativ, pur Rust.

**Motiv.** (b), si nu pentru argumentul din plan. Scoaterea ODBC de pe masina
tinta era stiuta. Ce a cantarit mai mult: `tiberius` **se compileaza pe Linux**,
in 15 secunde, deci stratul de date poate fi scris si verificat pe masina pe
care se scrie codul. `odbc-api` ar cere headerele de unixODBC, deci ar fi fost
cod pe care nu-l pot nici compila. Verificat intr-un proiect de proba inainte de
a decide: `tiberius` are SQL auth, `AuthMethod::Integrated` prin SSPI pe Windows
(feature `winauth`), si descoperire prin SQL Browser — acopera tot ce face
versiunea Python.

**Consecinte.** Testul pe server real rămâne necesar, dar ca **validare**, nu ca
alegere, si nu blocheaza etapele urmatoare. Daca validarea cade pe
autentificarea integrata intr-un domeniu, alternativa nu e intoarcerea la ODBC
ci `integrated-auth-gssapi`.

---

## 2026-11-17 — TLS: `native-tls` pe Windows, `rustls` pe Linux

**Context.** `tiberius` cere un furnizor de TLS.

**Motiv.** Tinta de rulare e si un Server 2012 curat, unde SQL Server poate
vorbi doar TLS 1.0/1.1. `rustls` nu coboara sub TLS 1.2, deci ar refuza exact
serverele vechi pe care rescrierea trebuie sa mearga. `native-tls` pe Windows e
SChannel, deja in sistem, fara dependinta de instalat.

Pe Linux (doar dezvoltare si teste) se foloseste `rustls`, fiindca `native-tls`
ar cere headerele de OpenSSL. Selectia e pe `[target.'cfg(windows)']` in
`rust/db/Cargo.toml`.

**Consecinte.** Codul de conectare e acelasi; doar furnizorul difera. O
conectare care merge pe Linux nu dovedeste nimic despre TLS pe Windows, si
invers.

---

## 2026-11-17 — Un tip care tine acreditari nu are `Debug` derivat

**Context.** Un test avea nevoie de `Debug` pe `SursaSqlServer`. Reflexul ar fi
fost `#[derive(Debug)]`.

**Motiv.** Ar fi tiparit parola la primul `dbg!` si in orice mesaj de panica.
Scris de mana, cu redactare (`parola` devine numarul de caractere), plus test
care verifica. `Autentificare` nu implementeaza `Serialize` deloc, tot
deliberat.

**Consecinte.** Regula generala, de aplicat la `platforma` cand ajunge la
sesiuni.

---

## 2026-11-17 — `db` intoarce text brut, nenormalizat

**Context.** `aduce_obiecte()` din versiunea Python normaliza, calcula hash-ul
si tinea token-ii la citire, pentru fiecare obiect din fiecare baza.

**Motiv.** Memoria era motivul initial (punctul 3 din `README.md`). Dar e si un
motiv mai bun: daca `db` ar intoarce text deja normalizat, un instantaneu ar fi
legat de versiunea de motor care l-a produs, deci nefolositor ca test de
regresie dupa o schimbare in `core`.

**Consecinte.** `Obiect` tine doar `schema`, `nume`, `tip`, `definitie`.
Normalizarea o face cine are nevoie de ea.

---

## 2026-11-17 — Poarta verifica si tinta Windows

**Context.** Tinta de rulare e Windows; dezvoltarea se face pe Linux. Codul
`#[cfg(windows)]` — DPAPI, auth integrata, viitorul strat Win32 — nu e nici
compilat nici analizat aici.

**Motiv.** `cargo check --target x86_64-pc-windows-msvc` nu cere linker, doar
biblioteca standard a tintei, si functioneaza inclusiv cu `tiberius` pe
SChannel. Deci codul de Windows poate fi verificat la compilare fara Windows.

**Consecinte.** S-a plătit in aceeasi zi: prima versiune a FFI-ului DPAPI nu
compila pe Windows (closure Rust pasat unde se cerea pointer `extern "system"`)
si nimic de pe Linux nu o arata. Regula generala a intrat in `CLAUDE.md`.
Verificarea e sarita in modul `rapid` al porții, deci inainte de a declara
gata cod de Windows se ruleaza poarta completa.

---

## 2026-11-17 — Sesiunile au spatiu de nume per modul

**Context.** In versiunea Python cheia de sesiune era numele bazei de referinta
(`_cale_sesiune(referinta)` -> `<slug>.dat`).

**Motiv.** Cu patru module, al doilea care salveaza progres pe aceeasi baza
suprascrie sesiunea primului. Era un bug garantat, nu probabil.

**Consecinte.** Cheia e `<date>/sesiuni/<modul>/<slug>.dat`, iar un `Sesiuni` e
legat de un modul: un modul nu poate ajunge la sesiunile altuia nici din
greseala. Identificatorul de modul e validat strict, fiindca devine nume de
director.

---

## 2026-11-17 — O sesiune nu conține secrete, deci clar pe non-Windows e acceptabil

**Context.** DPAPI exista doar pe Windows. Pe Linux ar trebui ori criptare cu
altceva, ori nimic.

**Alternative.** (a) un alt mecanism de criptare pe Linux; (b) refuz de a salva;
(c) scriere in clar, cu marcaj.

**Motiv.** (c), dar **numai** fiindca o sesiune nu conține secrete prin
construcție: `Sesiune` nu are camp de parola, iar `Autentificare` nu
implementeaza `Serialize`. Parola se cere la fiecare pornire, ca in versiunea
Python. Non-Windows e oricum doar mediu de dezvoltare.

**Consecinte.** Daca vreodata trebuie salvat un secret, calea de non-Windows
din `protejare.rs` trebuie sa **refuze**, nu sa scrie in clar. Scris in
`protejare.rs` si in `CRATE.md`, ca invariant.

---

## 2026-11-17 — Id-ul de rand de diff e cel stabil din `core`, nu indexul

**Context.** `Diferenta` din modul expune randurile de diff catre interfata si
catre linia de comanda. Prima versiune folosea indexul din lista.

**Motiv.** `gmotion_core::RandDiff` are deja un `id` stabil intre recalculari:
poziția blocului in referinta pentru randurile ancorate, numere negative unice
pentru cele doar-adaugate. Deciziile utilizatorului din editor („linia asta o
iau in deploy, asta nu") se tin minte pe acel id. Cu indexul din lista, orice
schimbare in tinta le-ar muta pe alte linii.

**Consecinte.** Scris ca invariant in `rust/module/diff-obiecte/CRATE.md`. Cand
apare editorul, cheia de override e `id`, niciodata poziția.

---

## 2026-11-17 — Citirea bazelor rămâne secventiala, deocamdata

**Context.** `aduce_obiecte` din versiunea Python citea bazele cu 4 conexiuni
simultane (`ThreadPoolExecutor`).

**Motiv.** Nu am portat paralelismul: nu am cu ce masura daca merita, si ar
aduce anulare, erori parțiale si un runtime in plus in `platforma`.

**Consecinte.** De masurat la etapa de paritate, pe baze reale. Daca citirea
dureaza inacceptabil, se adauga in `db::obiecte_multe`, nu in modul — ca sa
rămână un singur loc. Notat ca neacoperit in `CRATE.md`.

---

## 2026-11-17 — Jurnalul scrie la ieșirea de erori, nu la cea standard

**Context.** `tracing_subscriber::fmt::layer()` scrie implicit la ieșirea
standard. Linia de comanda scrie JSON tot acolo.

**Motiv.** `gmotion compara --json` producea o ieșire neparsabila, fiindca
liniile de jurnal se amestecau cu JSON-ul. Niciun test de unitate nu o arata:
toate chemau functii, nu procese.

**Consecinte.** Jurnalul merge la erori. In mod text progresul merge la ieșirea
standard; pe `--json` se muta si el la erori. Acoperit de un test care parseaza
ieșirea binarului. Regula generala in `CLAUDE.md`: un binar se probeaza
rulandu-l.

---

## 2026-11-17 — `SursaObiecte::obiecte_multe` ia `&mut dyn FnMut`, nu `impl FnMut`

**Context.** Scris la etapa 2 cu un parametru generic, ceea ce face traitul
incompatibil cu `dyn`.

**Motiv.** Linia de comanda alege intre server si instantaneu **la rulare**,
deci are nevoie de `Box<dyn SursaObiecte>`. Defectul era in `db` de la etapa 2 si
nu putea fi observat pana la primul consumator care avea nevoie de asta la
rulare.

**Consecinte.** Schimbat in `db`, nu ocolit in CLI, plus un test in `db` care
verifica explicit compatibilitatea cu `dyn`. E si un argument in plus pentru
ordinea aleasa (CLI inainte de interfata): daca primul consumator ar fi fost
interfata, defectul s-ar fi descoperit mult mai tarziu.

---

## 2026-11-17 — Paritatea ruleaza Python-ul vechi pe instantaneu, nu compara exporturi

**Context.** Planul pentru etapa 6 era: Mihai exporta CSV din aplicatia Python,
eu il compar cu `rust.json`. Asta inseamna o rulare manuala pe Windows pentru
fiecare iteratie, si un CSV care depinde de filtrele active in interfata.

**Alternative.** (a) comparatie de exporturi, ca in plan; (b) reimplementarea
logicii din `Api._comparatie` intr-un script; (c) rularea `Api._comparatie`
din `mostenire/app.py` neatins, cu `pyodbc` inlocuit de un modul care
serveste randurile din instantaneu.

**Motiv.** (c). `_comparatie` si `aduce_obiecte` nu au nevoie de altceva decat
de randurile interogarii. (b) ar fi exact o a doua implementare a aceleiasi
logici — ar putea „trece" fara sa spuna nimic despre codul din productie.
Arbitrul pentru rezumatele diferite e motorul JS extras din `ui.py`, prin
`referinta/rezumat_js.mjs`: daca JS da ce da Rust, diferenta e corectura.

**Consecinte.** `referinta/paritate.py`, rulat si de poarta pe un instantaneu
sintetic din `cazuri.json`. De pe Windows mai trebuie doar instantaneul; CSV-ul
ramane, optional, ca dovada ca `tiberius` si `pyodbc` aduc aceleasi obiecte.
Modulele `pyodbc`/`webview` se inlocuiesc doar in procesul scriptului;
`mostenire/` ramane neatins.

---

## 2026-11-17 — Citirea bazelor rămâne secventiala (confirmat pe date reale)

**Context.** Decizia „Citirea bazelor rămâne secventiala, deocamdata" lasa
masuratoarea pe etapa 6.

**Masuratoare.** Doua baze CharismaERP de ~21.000 de obiecte fiecare, acelasi
server: Rust 19 s (secvential, doar citire); Python v3.4 29,6 s (4 conexiuni,
citire + comparatie).

**Motiv.** Rust nu e mai lent decat versiunea pe care o inlocuieste, deci
paralelismul n-ar rezolva o problema existenta si ar aduce anulare, erori
parțiale si complexitate in `db`. Se redeschide doar daca un utilizator
compara multe baze odata si timpul devine o plangere.

---

## 2026-11-17 — Parola nu-si arata nici lungimea, iar instantaneul retine doar serverul

**Context.** `Redactat` afisa „(N caractere, ascuns)", ca utilizatorul sa
vada ca s-a citit parola potrivita. `gmotion instantaneu` scria in campul
`server` descrierea pentru jurnal, cu tot cu acest text. Primul fixture de pe
o baza reala a ajuns in repo cu lungimea parolei `sa`.

**Alternative.** (a) pastrat lungimea in jurnal, scos doar din instantaneu;
(b) scos de peste tot; (c) si rescrierea istoricului.

**Motiv.** (b). Jurnalele ajung lipite in conversatii si rapoarte de bug, ca
azi; lungimea scurteaza o cautare prin forta bruta si nu ajuta la depanare
mai mult decat „data / goala". Istoricul nu se rescrie: branch-ul e comun cu
Mihai si informatia e de risc mic; Mihai decide daca schimba parola.

**Consecinte.** `Redactat` spune „(data, ascunsa)" sau „(goala)", cu test
care cade pe orice cifra. Instantaneul primeste `ArgumenteSursa::eticheta_server`
(`SRV\INST` / `SRV,PORT`). Invariant in `rust/cli/CRATE.md`.

---

## 2026-11-17 — Istoricul se rescrie, totusi, pentru lungimea parolei

**Anuleaza** partea „istoricul nu se rescrie" din intrarea precedenta.

**Context.** Mihai a cerut explicit scoaterea informatiei din git.

**Ce s-a facut.** Rescris de la `58fbc06`; continut final identic; impins cu
`--force-with-lease`. Clona lui Mihai trebuie readusa la noul istoric
(`git fetch` + `git reset --hard origin/<branch>`), altfel un `pull` ar
reintroduce commit-urile vechi.

**Limita.** GitHub mai poate servi commit-urile vechi dupa SHA pana le curata
singur; stergerea definitiva o face doar suportul GitHub. Nu exista PR sau
fork care sa le tina in viata.

---

## 2026-11-17 — `Cargo.lock` intra in git

**Context.** Era in `.gitignore`. Auditul: fiecare compilare isi alegea singura
versiunile; pe Windows, Mihai a vazut „Locking 194 packages to highest
compatible versions", deci binarul lui si cel verificat aici puteau diferi.

**Motiv.** Pentru o aplicatie (nu o biblioteca), recomandarea Cargo e lock in
git: build reproductibil, si un aviz de securitate se poate verifica pe exact
versiunile livrate. Actualizarea dependentelor devine un commit vizibil
(`cargo update`), nu un efect secundar al unei compilari.

---

## 2026-11-17 — Normalizarea nu mai atinge string-urile; „identic" fara scor

**Context.** Auditul de la `f561bf9` a gasit modificari reale raportate
„identic" (deci absente si din lista si din deploy), toate mostenite din v3.4
si prezente si in motorul JS: regulile de normalizare mergeau pe tot textul,
deci `'--- x'` sau `'/*'` stergeau codul de dupa, `'Activ'` = `'ACTIV'`,
`[a b]` = `a b`, `Sales.dbo.Orders` = `Sales.Orders`; iar pe obiecte de
10.000+ token-uri scorul 99,995 se rotunjea la 100 si „scor 100 -> identic".

**Alternative.** (a) pastrat semantica JS/v3.4 (paritate perfecta, bug-ul
ramane); (b) reparat in Rust si listat explicit fiecare divergenta; (c)
reparat si regenerat fixture-ul JS.

**Motiv.** (b). Paritatea a dovedit ca portul e fidel; nu face un bug corect.
(c) e exact mecanismul interzis de regula 2 din `CLAUDE.md`: fixture-ul JS
descrie ce face JS, si JS are bug-ul.

**Ce s-a facut.** `core/src/lexer.rs` (granite cod / string / identificator /
comentariu, comentarii bloc imbricate); regulile de normalizare se aplica doar
pe cod; string-urile si identificatorii ne-simpli se pastreaza (identificatorii
in litere mici). `comparare`: hash diferit = diferit, scorul afisat al unui
obiect diferit e cel mult 99,99. `cheie_bloc`: 128 de biti de SHA-256 in loc de
32 de MD5 (coliziune gasita la audit in 0,1 s); dependenta `md-5` scoasa din
`core`.

**Consecinte.** Conformitatea cu JS: 4 divergente, toate in `norm` pe cazuri cu
string-uri, listate in `DIVERGENTE_DELIBERATE`. Paritatea cu v3.4:
`paritate.py` clasifica diferentele pe obiecte cu constructii tratate altfel ca
„de verificat", nu regresie; poarta ruleaza strict, cu `*.asteptari.json`.
Pe fixture-ul real: 4 scoruri schimbate, verificate manual (diferente doar in
string-uri; la `CaseM` v3.4 stergea cod dupa un `--` din string). Pe bazele
reale ale lui Mihai: numarul de obiecte „diferite" poate creste — acelea sunt
exact modificarile pe care v3.4 le ascundea.

---

## 2026-11-17 — Conexiunea: timeout-uri, SQL Browser propriu, certificat optional

**Context.** Auditul de la `f561bf9`: (1) certificatul serverului nu se
verifica niciodata — un server TDS fals a primit parola `sa` in clar;
(2) dupa TCP nimic nu avea timeout; (3) `connect_named` din `tiberius` dadea
panica pe un raspuns SQL Browser de 2 octeti si accepta raspunsul de la
oricine de pe LAN; (4) `Autentificare` avea `Debug` derivat.

**Alternative pentru certificat.** (a) validare implicita: sigur, dar cade pe
fiecare server intern cu certificat autosemnat — adica pe serverele reale de
azi, inclusiv `TS2409\MIHAIG`; (b) implicit ca in v3.4, validarea la cerere,
cu avertisment in jurnal la fiecare conexiune; (c) „trust on first use"
(amprenta retinuta) — cere stocare si un flux de confirmare.

**Motiv.** (b), deocamdata: (a) ar rupe toate instalarile existente fara ca
Mihai sa fi decis. Implicitul e decizia lui Mihai (in `STARE.md`). (c) e
candidatul bun pentru interfata.

**Ce s-a facut.** `Tinta::certificat` + `--certificat nevalidat|sistem|<ca>`;
timeout pe deschidere (TCP+TLS+login), pe interogari si pe SQL Browser;
conexiunea se arunca dupa orice eroare; `rezolva_port_instanta` propriu
(socket UDP conectat, parser validat, instanta verificata); `try_get` in loc de
`get`; erorile de interogare `GM-DB-004`, baza inaccesibila (4060) `GM-DB-005`;
`valideaza_baza` (`GM-DB-011`); obiectele criptate pastrate, categorie
`criptat` in modul.

---

## 2026-11-17 — Citirea bazelor devine paralela (anuleaza „ramane secventiala")

**Anuleaza** intrarea „Citirea bazelor rămâne secventiala (confirmat pe date
reale)".

**Context.** A doua rulare a lui Mihai a fost pe alta retea (192.168.1.x, cu
latenta mai mare decat LAN-ul de la prima rulare): aceleasi doua baze
CharismaERP s-au citit in 48,6 s secvential, fata de 28,9 s in v3.4 (4
conexiuni, citire + comparatie). Pe LAN: 19 s fata de 29,6 s. Decizia
anterioara spunea sa se redeschida daca timpul devine o problema; pe o retea
lenta v4 era mai lent decat versiunea pe care o inlocuieste.

**Ce s-a facut.** `SursaSqlServer::obiecte_multe` citeste pana la 4 baze
deodata (ca v3.4), fiecare pe conexiunea ei, intr-un `JoinSet` pe runtime-ul
existent — fara dependinta noua. Ordinea rezultatului e cea ceruta, prima
eroare opreste tot, progresul porneste bazele in ordine. `compara` din CLI
citeste referinta si celelalte baze printr-un singur apel. Sursa din
instantaneu ramane pe implementarea implicita (secventiala, fara retea).

**Masurat** (a treia rulare, aceeasi retea): 12,3 s, fata de 48,6 s secvential si 29,3 s in v3.4.

---

## 2026-11-17 — Interfata nativa, cu Slint

**Context.** Mihai a ales (2026-11-17): interfata se rescrie **nativ**, fara
WebView2, „afisajul oarecum la fel" ca v3.4 dar poate fi si mai frumos; si se
incepe acum, inaintea modulului `upgrade`. Alternativa refuzata: refolosirea
HTML-ului v3.4 intr-un WebView.

**Alternative de toolkit.** Slint; egui (MIT, imediat, dar pe RDP fara GPU are
nevoie de wgpu/WARP, iar toate controalele se deseneaza de mana); iced; Qt
(licenta, C++).

**Motiv.** Slint: randare software nativa (RDP fara GPU), declarativ (layout ca
in CSS), componente standard, virtualizare prin `ListView`, si — decisiv pentru
lucrul fara ecran — randare in buffer, deci capturi PNG comparabile cu v3.4
(`referinta/captura_v34/`). Probat: compileaza si randeaza aici.

**Licenta.** Slint Royalty-free 2.0: gratuit pentru aplicatii desktop, cu
atribuire — fie widget-ul `AboutSlint` intr-un ecran „Despre" accesibil din
meniu, fie insigna „Made with Slint" pe o pagina publica. Nu e permis in
sisteme embedded. Se alege `AboutSlint` in „Despre".

**Consecinte.** Slint 1.19 cere Rust 1.95: doar `gmotion-fereastra` are
`rust-version = 1.95`, restul ramane pe 1.74. Dependente noi doar in
`fereastra` (`slint`, `slint-build`; `image` si `serde_json` doar pentru
exemplul de captura).

---

## 2026-11-17 — Designul interfetei: aprobat pe machete

**Context.** Dupa cercetarea de design (`cercetare/design-interfata.md`), o
simulare cu cinci ecrane. Prima varianta (paleta GitHub, insigne rotunjite,
texte explicative peste tot) a fost respinsa de Mihai: „sa nu para facut cu
AI-ul".

**Decizie.** Varianta a doua: unealta densa, fond cald, linii subtiri, IBM
Plex, starea prin litere si text colorat, mediul tintei ca banda sus. Fara
comutatoare de zgomot (Mihai: „asta trb deja filtrata asa") — ce motorul
considera zgomot nu se arata deloc. Detaliile si jetoanele:
`cercetare/design-aprobat.md`; machetele: `cercetare/machete/`.

**Consecinte.** Fonturile IBM Plex (OFL) se includ in binar, cu licenta.
Ecranele cer si lucru in motor: clasificarea „difera in" (prin `core::lexer`),
starea de revizuire si sesiunile (`platforma`), verificarea ca baza nu s-a
schimbat si scriptul de revenire la deploy.

---

## 2026-11-17 — Contrastul: doua jetoane intunecate fata de macheta

**Context.** Jetoanele aprobate au intrat in cod (`rust/fereastra/src/jetoane.rs`),
iar criteriul de contrast din `design-aprobat.md` (≥ 4,5:1 pentru text normal)
a fost scris ca test peste fiecare pereche text / fundal pe care o deseneaza
editorul. Testul a cazut pe doua culori: `text-tert` `#9c978b` (numere de
linie, schema) avea 2,9:1 pe fundalul de cod, iar comentariile `#8a857a`
aveau 3,7:1.

**Decizie (Mihai).** Se intuneca minim, restul designului nu se atinge:
`text-tert` → `#716c61`, comentarii → `#6a665b` (al doilea pas a fost nevoie
ca sa treaca si pe fundalul blocurilor sterse, `#f8e6e1`).

**Consecinta.** Numarul de linie **din interiorul unui bloc diferit** ia
culoarea blocului (`mod-text` / `add-text` / `del-text`), nu gri-ul tert: pe
fundalurile tinte, un gri destul de inchis cat sa treaca AA nu se mai
deosebea de `text-secundar`, adica se pierdea ierarhia.

**Abatere acceptata, notata in cod:** caracterele schimbate **dintr-un
comentariu** stau pe fundalul de in-linie (galben / verde / roz tare) si
raman la ~3,7:1. Acolo fundalul mai tare e semnal peste cel al blocului, nu
fundalul de referinta al textului; nu se poate ridica fara sa dispara chiar
evidentierea (la 4,5:1 fundalul de in-linie nu se mai distinge de cel al
blocului). Lista abaterilor e in `jetoane.rs` si are test propriu.

---

## 2026-11-17 — Sensul culorilor in editor: deploy, nu referinta

**Context.** Motorul numeste blocurile fata de referinta: `Sters` = e in
sursa si lipseste din baza. Machetele coloreaza invers: in `Editor.dc.html` o
linie care e doar in sursa (stanga) e **verde**, una care e doar in baza
(dreapta) e **rosie**.

**Decizie.** Macheta are dreptate pentru utilizator: el se uita la ce o sa
faca deploy-ul — ce e doar in sursa se **adauga** in baza, ce e doar in baza
se **scoate**. Incrucisarea se face intr-un singur loc, `cod_stare` din
`rust/fereastra/src/lib.rs`, cu test. Numele din motor nu se schimba.

---

## 2026-11-17 — Comentariul singur ramane zgomot filtrat

**Context.** Machetele arata un obiect care difera doar intr-un comentariu
(`rpt.usp_SoldParteneri`, „comentariu", +1 −1) si, in editor, „4 diferențe,
una doar în comentariu". Motorul insa sterge comentariile la normalizare, ca
in v3.4: un asemenea obiect iese „identic" si nu ajunge nici in lista, nici in
editor. Alternativa era sa pastram comentariile in hash si in segmentare.

**Decizie (Mihai).** Ramane ca azi. Motorul nu se atinge; randul doar-comentariu
din macheta ramane nefolosit. Coloana „Difera in" arata „comentariu" doar cand
comentariul s-a schimbat intr-un bloc care difera si in cod („cod, comentariu").

**De ce conteaza.** A schimba regula ar fi insemnat obiecte noi in lista pe
baza reala (cate, nu se stie fara o rulare pe Windows), o abatere noua fata de
v3.4 in `*.asteptari.json`, si fixture-uri de regenerat. Testul
`comentariul_singur_ramane_zgomot_filtrat` din `core/src/clasificare.rs` tine
regula pe loc: daca cineva o schimba, testul cade si intrebarea se pune din nou.

---

## 2026-11-17 — Modul unificat: doua jgheaburi de numere

**Context.** Macheta editorului are tabul „Unificat", dar nu arata randarea.
Fara niciun semn, nu se vede care linie e din sursa si care din baza.

**Decizie (Mihai).** Doua coloane de numere (sursa | baza), ca in diff-urile
clasice: linia din sursa are numar doar in stanga, cea din baza doar in
dreapta, iar randurile egale au amandoua. Niciun element nou (litere, semne
`+` / `−`) in afara machetelor.

---

## 2026-11-17 — „Difera in" si „+N −M" se calculeaza la cerere

**Context.** Lista are nevoie, pentru fiecare obiect, de clasificarea
diferentei si de numarul de linii mutate. Amandoua cer alinierea completa a
obiectului (`diff_aliniat`), adica partea scumpa — pe comparari cu zeci de mii
de obiecte ar fi fost calculata pentru tot si aruncata aproape toata.

**Decizie.** Se calculeaza la cerere, ca si diff-ul: `detalii_obiect` in
`diff-obiecte`, chemat de lista doar pentru randurile de pe ecran
(`Lista::fara_detalii`) si de `cli` doar cu `compara --detalii`. Compararea
simpla ramane la fel de rapida ca pana acum (hash, apoi scor).

**Masurat:** pe 50.000 de obiecte, lista se construieste in 84 ms, un filtru ia
13 ms, o cautare 3 ms, iar un cadru ramane la ~8 ms.

---

## 2026-11-17 — Ecranul de conectare: cand se conecteaza, si ce nu stim inca

**Decizie.** Bazele se citesc **cand le ceri** (clic pe „Baza"), nu dupa
fiecare tasta. Starea „conectat" se scrie numai dupa ce s-a citit ceva de pe
server: altfel ar fi o promisiune, nu un fapt.

**Parola.** Iese din `Acreditari` intr-un singur loc (`pornire::deschide`), si
`Acreditari` n-are `Debug` derivat. Exista test ca nu apare nici in mesajul de
eroare al unei conectari esuate, nici in campul din fereastra (care arata doar
puncte).

**Ramase de decis cu Mihai** (notate si in `STARE.md`): selectorul de fisiere
pentru instantaneu (ar fi dependinta `rfd`), de unde se ia mediul PROD / TEST
/ DEV (acum se ghiceste din numele bazei), si daca e nevoie de un ecran de
progres cat dureaza compararea pe server real (acum fereastra sta).

---

## 2026-11-18 — Garda de deploy pe amprenta exacta, revenirea langa deploy

**Context.** Etapa 12: inainte de deploy trebuie stiut ca baza n-a mai fost
atinsa de la comparare, si trebuie salvat un script de revenire. Compararea
are deja hash-uri (`Rand::hash_referinta` / `hash_curent`), dar sunt pe forma
**normalizata**: nu vad spatiile si nici `ANSI_NULLS` / `QUOTED_IDENTIFIER`.

**Alternative.** (a) garda pe hash-ul compararii: gratis, dar o schimbare de
setari facuta intre timp pe tinta ar fi suprascrisa fara avertisment; (b)
amprenta exacta: SHA-256 peste tip, setari si definitia neatinsa
(`plan::amprenta_exacta`), calculata la cerere, doar pentru obiectele din plan.

**Decizie.** (b). Deploy-ul inlocuieste textul exact, deci garda se uita la
textul exact. Se verifica **ambele** parti: tinta (cineva a lucrat pe baza) si
referinta (codul deployat nu mai e cel revizuit). Obiect aparut sau disparut
opreste la fel ca unul modificat. Oprirea e `GM-MOD-012`, cu fiecare obiect pe
nume.

**Forma in `cli`.** `gmotion plan -o plan.json` scrie planul (pasii, cu
amprentele); `gmotion deploy --plan plan.json` recitește, verifica, si abia
apoi scrie. Fara `--plan`, deploy-ul face planul pe loc din ce tocmai a citit.
Planul e JSON fiindca e si forma in care fereastra il va tine intre ecrane.

**Revenirea.** Se scrie **inainte** de deploy, in `<director>/revenire/`, nu
langa scripturile de deploy: cine ruleaza `*.sql` din director nu trebuie sa
ruleze si revenirea. Totul (deploy si revenire) se verifica inainte de primul
fisier: daca revenirea nu se poate scrie (o definitie actuala cu `GO`), nu se
scrie nici deploy-ul. Ordinea: intai se refac obiectele vechi (dupa
dependentele codului vechi), apoi se sterg cele create de deploy, in ordinea
**inversa** dependentelor codului nou — un view nou `WITH SCHEMABINDING` ar
bloca altfel stergerea functiei noi.

**Limita cunoscuta.** `DROP` taie `schema.nume` la primul punct (`Baza` nu
tine partile separat); o schema cu punct in nume ar iesi gresit.

---

## 2026-11-18 — Intai paritatea functionala cu v3.4, apoi aspectul

**Context.** Mihai a rulat `gmotion-ui` pe Windows: nu e multumit vizual si
simte ca s-au pierdut functii (exemplul lui: cautarea serverelor). Inventarul
(`cercetare/lipsuri-portare.md`) i-a dat dreptate: editarea tintei, modul
ghidat, cautarea serverelor, compararea cu mai multe baze, progresul, starea
„Omis", exportul din fereastra si altele au disparut odata cu machetele noi,
**fara nicio decizie scrisa**. In plus, 7 butoane din fereastra nu erau legate
la nimic, desi etapele 8-11 au fost declarate incheiate.

**Decizie (Mihai).** Intai toate functionalitatile, abia apoi aspectul
vizual, intr-o sesiune separata. Deocamdata un singur modul, „comparare
obiecte"; platforma cu mai multe module ramane in arhitectura, dar celelalte
module se decid mai tarziu. Ecranul de deploy din etapa 12 se opreste pana
cand Mihai stabileste ordinea lipsurilor.

**Consecinta pentru proces.** O functie din v3.4 nu se mai scoate tacit: sau
se porteaza, sau se scrie aici de ce nu.

---

## 2026-11-18 — Noua directie vizuala: Harta pentru lista, Panza pentru editor

**Context.** Designul aprobat pe 2026-11-17 (fond cald, linii subtiri) si
propunerea „Banda" au fost respinse de Mihai („pare vechi si anost"). Din trei
directii prototipate in HTML (`cercetare/redesign/Directii.html`), Mihai a ales
doua, cu conditia sa se poata face in Rust.

**Decizie (Mihai).** Lista obiectelor = **Harta**: intunecat, fiecare obiect
diferit e un patrat colorat, grupat pe schema si tip, cu panoul obiectului in
dreapta. Interiorul editorului = **Panza**: alb, codul pe tot ecranul, titlu
mare, comenzile in insule care plutesc, lista de obiecte in `Ctrl K`.
Inlocuieste `cercetare/design-aprobat.md`, dar **nu schimba ordinea**: intai
paritatea cu v3.4, apoi aspectul.

**Fezabilitate, dovedita.** `cercetare/schita-slint/` deseneaza ambele ecrane
cu Slint 1.19 si randare software, pe diferenta reala din motor: primul cadru
12–24 ms la 1440×900. Patratele hartii se picteaza in Rust intr-o imagine
(0,4 ms), nu ca sute de elemente Slint. Ce s-a adaptat ca sa se poata face si
ce ramane de dovedit (HiDPI, tastatura si cititorul de ecran pe harta, umbrele
prin RDP): `cercetare/schita-slint/README.md`.

**Consecinte.** O singura familie de litere, Geist + Geist Mono (OFL), in loc
de IBM Plex. Sora a cazut: ii lipsesc sagetile. Identicele nu se deseneaza ca
patrate, doar ca proportie sub fiecare grup. Contrastul jetoanelor noi trece
4,5:1 dupa intunecari minime, verificate la portare de testul din `jetoane.rs`.

---

## 2026-11-18 — Aspect nou, aceleasi functii: trei ecrane legate

**Context.** Dupa ce a ales Harta si Panza, Mihai a precizat: „vreau sa
pastrez functionalitatile, doar ca vreau sa le vad diferit". Prima schita a
hartii inlocuia tabelul din v3.4 si nu avea butoanele editorului.

**Decizie (Mihai).** (1) Ecranul de conectare intra in acelasi stil, iar cele
trei ecrane au continuitate intre ele. (2) Ecranul cu patrate **contine lista
din v3.4**: tabelul cu toate coloanele, filtrele, CSV si Deployer, iar harta
sta deasupra ca vedere de ansamblu, legata de tabel. (3) Editorul pastreaza
**toate butoanele din v3.4** (Aplica toate, Reset, navigarea, Mod ghidat,
Side-by-side / Inline, Ascunde diferentele, Re-sincronizeaza, Uneste
cursorul, Copiaza, Am ajuns la o concluzie, Omite, fereastra modului ghidat).

**Cum.** `cercetare/schita-slint/` deseneaza toate trei in Slint, cu motorul
adevarat in editor. `README.md` de acolo are tabelul „functie din v3.4 → unde
e acum", cu ce e desenat si ce are doar locul stabilit. Regula din intrarea
de mai sus ramane: o functie din v3.4 nu se scoate tacit.

---

## 2026-11-18 — Aceeasi paleta peste tot, in mod intunecat si luminos; incepe rescrierea

**Context.** Schita avea conectarea si lista pe fond inchis, editorul pe alb.
Mihai: „vreau sa fie paleta de culori asemanatoare la toate ecranele", iar la
alegerea intre intunecat, luminos sau amandoua: „integrat ca dark si light
mode". Si: „imi place cum arata, poti incepe sa rescrii designul".

**Decizie (Mihai).** Doua palete, `INTUNECAT` si `LUMINOS`, cu **aceleasi
nume de jetoane**, aplicate pe toate ecranele; modul se comuta. Rescrierea
ferestrelor incepe acum (etapa 13), inaintea paritatii cu v3.4 — schimbare
fata de intrarea „Intai paritatea functionala", la cererea lui Mihai. Etapa
12 se inchide cu ecranul de deploy mutat dupa etapa 13.

**Cum.** `jetoane::Paleta` e o singura structura; `aplica_tema(ui, Mod)` o
pune in `Tema` dintr-o lista verificata de compilator. Testul de contrast
ruleaza pe ambele palete (73 de perechi, abaterile comentariilor raman la
3,5). Geist si Geist Mono inlocuiesc IBM Plex (schita le-a dovedit, cu
diacritice). Implicitul ramane luminos pana sunt rescrise toate ecranele.

---

## Coduri de eroare — registru

Un cod alocat nu se refoloseste si nu se renumeroteaza, ca sa rămână
cautabil in jurnale vechi si in rapoarte de bug.

| Prefix | Zona |
|---|---|
| `GM-CORE-*` | motorul de comparare |
| `GM-DB-*` | conexiune, driver, citire obiecte — **alocate 001-011**, vezi `rust/db/CRATE.md` |
| `GM-PLAT-*` | configurare, sesiuni, rutare — **alocate 001-011**, vezi `rust/platforma/CRATE.md` |
| `GM-MOD-*` | module — **diff-obiecte: 001-012**, vezi `rust/module/diff-obiecte/CRATE.md` |
| `GM-UI-*` | fereastra, randare |

`core` nu intoarce erori tipate: functiile lui sunt totale pe intrari valide
UTF-8.

`GM-DB-001` conectare · `002` autentificare · `003` auth integrata indisponibila
· `004` interogare · `005` baza lipsa · `006` descoperire · `007` citire
instantaneu · `008` scriere instantaneu · `009` versiune instantaneu · `010`
nume invalid.
