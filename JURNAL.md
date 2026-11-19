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
ca sa poata regenera fixture-urile, si etapa de paritate are nevoie de o
versiune care nu se misca. Dezvoltarea paralela ar face tinta de paritate
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
fara fereastra. Daca tot ce e valoros traieste in spatele unui GUI Windows,
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
testul de conformitate cade, presupunerea implicita e ca portul Rust e gresit,
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

**Alternative.** (a) `odbc-api` — comportament identic cu azi, dar ramane
dependenta de driver; (b) `tiberius` — TDS nativ, pur Rust.

**Motiv.** (b), si nu pentru argumentul din plan. Scoaterea ODBC de pe masina
tinta era stiuta. Ce a cantarit mai mult: `tiberius` **se compileaza pe Linux**,
in 15 secunde, deci stratul de date poate fi scris si verificat pe masina pe
care se scrie codul. `odbc-api` ar cere headerele de unixODBC, deci ar fi fost
cod pe care nu-l pot nici compila. Verificat intr-un proiect de proba inainte de
a decide: `tiberius` are SQL auth, `AuthMethod::Integrated` prin SSPI pe Windows
(feature `winauth`), si descoperire prin SQL Browser — acopera tot ce face
versiunea Python.

**Consecinte.** Testul pe server real ramane necesar, dar ca **validare**, nu ca
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

**Consecinte.** S-a platit in aceeasi zi: prima versiune a FFI-ului DPAPI nu
compila pe Windows (closure Rust pasat unde se cerea pointer `extern "system"`)
si nimic de pe Linux nu o arata. Regula generala a intrat in `CLAUDE.md`.
Verificarea e sarita in modul `rapid` al portii, deci inainte de a declara
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

## 2026-11-17 — O sesiune nu contine secrete, deci clar pe non-Windows e acceptabil

**Context.** DPAPI exista doar pe Windows. Pe Linux ar trebui ori criptare cu
altceva, ori nimic.

**Alternative.** (a) un alt mecanism de criptare pe Linux; (b) refuz de a salva;
(c) scriere in clar, cu marcaj.

**Motiv.** (c), dar **numai** fiindca o sesiune nu contine secrete prin
constructie: `Sesiune` nu are camp de parola, iar `Autentificare` nu
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
pozitia blocului in referinta pentru randurile ancorate, numere negative unice
pentru cele doar-adaugate. Deciziile utilizatorului din editor („linia asta o
iau in deploy, asta nu") se tin minte pe acel id. Cu indexul din lista, orice
schimbare in tinta le-ar muta pe alte linii.

**Consecinte.** Scris ca invariant in `rust/module/diff-obiecte/CRATE.md`. Cand
apare editorul, cheia de override e `id`, niciodata pozitia.

---

## 2026-11-17 — Citirea bazelor ramane secventiala, deocamdata

**Context.** `aduce_obiecte` din versiunea Python citea bazele cu 4 conexiuni
simultane (`ThreadPoolExecutor`).

**Motiv.** Nu am portat paralelismul: nu am cu ce masura daca merita, si ar
aduce anulare, erori partiale si un runtime in plus in `platforma`.

**Consecinte.** De masurat la etapa de paritate, pe baze reale. Daca citirea
dureaza inacceptabil, se adauga in `db::obiecte_multe`, nu in modul — ca sa
ramana un singur loc. Notat ca neacoperit in `CRATE.md`.

---

## 2026-11-17 — Jurnalul scrie la iesirea de erori, nu la cea standard

**Context.** `tracing_subscriber::fmt::layer()` scrie implicit la iesirea
standard. Linia de comanda scrie JSON tot acolo.

**Motiv.** `gmotion compara --json` producea o iesire neparsabila, fiindca
liniile de jurnal se amestecau cu JSON-ul. Niciun test de unitate nu o arata:
toate chemau functii, nu procese.

**Consecinte.** Jurnalul merge la erori. In mod text progresul merge la iesirea
standard; pe `--json` se muta si el la erori. Acoperit de un test care parseaza
iesirea binarului. Regula generala in `CLAUDE.md`: un binar se probeaza
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

## 2026-11-17 — Citirea bazelor ramane secventiala (confirmat pe date reale)

**Context.** Decizia „Citirea bazelor ramane secventiala, deocamdata" lasa
masuratoarea pe etapa 6.

**Masuratoare.** Doua baze CharismaERP de ~21.000 de obiecte fiecare, acelasi
server: Rust 19 s (secvential, doar citire); Python v3.4 29,6 s (4 conexiuni,
citire + comparatie).

**Motiv.** Rust nu e mai lent decat versiunea pe care o inlocuieste, deci
paralelismul n-ar rezolva o problema existenta si ar aduce anulare, erori
partiale si complexitate in `db`. Se redeschide doar daca un utilizator
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

**Anuleaza** intrarea „Citirea bazelor ramane secventiala (confirmat pe date
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
(`rpt.usp_SoldParteneri`, „comentariu", +1 −1) si, in editor, „4 diferente,
una doar in comentariu". Motorul insa sterge comentariile la normalizare, ca
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
amprentele); `gmotion deploy --plan plan.json` reciteste, verifica, si abia
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

## 2026-11-18 — Fara diacritice nicaieri; textele vor trece in engleza

**Decizie (Mihai).** „Vreau sa dispara diacriticele de orice forma, o sa vreau
sa o fac in engleza." Fara diacritice in interfata, mesaje, comentarii si
documente. Regula 6 din `CLAUDE.md` s-a schimbat in acelasi commit, iar poarta
cade pe orice diacritica noua (`unelte/verifica.sh`, „fara diacritice").

**Exceptia, si de ce.** Datele de test care verifica intentionat nume cu
diacritice ale utilizatorului (o baza cu t-virgula in nume, doua fisiere de
deploy pe nume care difera doar prin a-caciula si a-breve, latimea unui tabel
pe caractere, nu octeti) raman, marcate
cu `diacritice-permise` pe aceeasi linie. Transliterate, doua teste ar fi
trecut verificand altceva. Aplicatia trebuie in continuare sa trateze corect
diacriticele din datele utilizatorului; doar textele noastre nu le mai au.

**Ramane de facut.** Trecerea in engleza. Azi textele sunt imprastiate in
`.slint`, `model_*` si `cli`; o traducere curata cere intai un singur loc
pentru texte. Nu s-a inceput.

---

## 2026-11-18 — Deploy-ul din fereastra: doar scripturi, garda obligatorie

**Context.** Mihai: „la deploy te las pe tine sa alegi cum arata".

**Decizii.** (1) Ecranul spune explicit ca **se scriu doar scripturi** si ca
pe server nu ruleaza nimic: asta face motorul (`deploy::scrie_cu_revenire`),
ca Deployer-ul din v3.4. (2) Scrierea e posibila **doar dupa garda** pe
obiectele recitite atunci, si se scrie din cele recitite, nu din cele de la
comparare. (3) Fara confirmarea prin tastarea numelui bazei din machetele
vechi: nimic nu se executa, iar garda opreste deja o baza atinsa. (4) Scrierea
merge in directorul de date al aplicatiei (`<date>/deploy/<referinta>_<ora>`),
fiindca fereastra n-are selector de fisiere (decizia pe `rfd`, inca deschisa).
(5) Ce e „in plan" nu se tine minte intre porniri; v3.4 tinea starea de
deploy. Ramas in `lipsuri-portare.md`.

**Modul implicit** devine intunecat (stilul hartii, ales primul), acum ca
toate ecranele sunt rescrise; butonul de tema il schimba si il tine minte.

## 2026-11-18 — Panoul de detalii in lista; impactul estimat si notele

Cerut de Mihai, dupa o captura de referinta: lista pe toata latimea, la clic
detaliile in dreapta (redimensionabile, inchise cu X sau al doilea clic),
dublu clic direct in editor. Lucrat intr-o sesiune paralela, pe baza etapei
14; la unire a intrat ca punctul G al etapei 15.

**Impactul estimat** e o estimare grosiera si o spune: fiecare nivel vine cu
motivele lui. Puncte: folosit de 1-4 obiecte din baza comparata +1, de 5+
+2; similar 70-95% +1, sub 70% +2; trigger +1. 0 Scazut, 1-2 Mediu, 3+
Ridicat. Obiect nou: schimbarea nu conteaza (nu inlocuieste nimic); doar in
baza sau criptat: Scazut (nu se deployeaza). Cine foloseste obiectul se ia
din cod, prin lexer, cu **aceeasi** legare nume -> obiect ca ordinea de deploy
(`deploy::referinte`, extrasa din `ordine_dupa_dependente`), nu din
`sys.sql_expression_dependencies`: merge si din instantanee, si nu e a doua
implementare. Respins: dependentele de tabele (nu le citim) si SQL dinamic
(nu se vede in cod) — limita stiuta, aceeasi ca la ordinea de deploy.

**Notele** se tin pe nume, nu pe amprenta: o nota e despre obiect („a ramas
aici"), nu despre versiunea lui; bifele raman pe amprenta. Stau in aceeasi
stare `Revizuire` din sesiune, deci `cli` si fereastra le vad la fel.

---

## 2026-11-18 — Harta din lista, refacuta dupa modelul lui Mihai

Harta veche avea grupuri fixe de 4 randuri, asezate pe o latime constanta
(1372 px): pe putine obiecte statea ingramadita la stanga, cu restul
cartusului gol. Mihai a dat modelul corect (o imagine): cate o coloana pe tip,
despartite prin linii, care umplu impreuna cartusul; titlul cu numarul la
dreapta; o bara cu proportia starilor; patrate pe blocuri de cate 5 randuri;
bula langa patrat, cu punctul starii.

Decis pe drum: (1) patratele stau **in ordinea din tabel**, nu sortate dupa
stare — bara de proportie da rezumatul, iar harta urmeaza sortarea listei.
(2) Toate coloanele de pe un rand primesc cel mai mic numar de randuri care
incape, apoi latimea ramasa se imparte pe masura lor si grila se lateste
cat coloana: banda ramane cat mai joasa deasupra tabelului. (3) Latimea e a
ferestrei, nu o constanta; la redimensionare harta se reaseaza. (4) Culori
proprii pentru harta (`harta_mod/add/del/gol`), mai stinse decat barele din
editor, ca sute de patrate alaturi sa nu orbeasca; patratele din tabel le
folosesc pe aceleasi, ca sa se potriveasca cu harta. (5) Zoom automat,
cerut de Mihai: cand obiectele nu intra in 260 px, patratele coboara pe
trepte fixe (14, 11, 8, 6, 4 px); o singura marime pe toata harta. Abia la
ultima treapta apare derularea. Treptele sunt fixe, nu continue, ca
patratele sa ramana pe pixeli intregi si clicul sa nimereasca exact.

---

## 2026-11-18 — Etapa 14: ce s-a decis pe drum

**Bifa nu re-aplica filtrul pe loc.** Cu filtrul „Nerevizuite" pornit, randul
bifat ramane pe ecran pana se atinge filtrul din nou. Asa face si v3.4
(`marcheazaRezolvat` nu cheama `renderLista`), si e si mai bine: altfel randul
dispare de sub cursor exact cand lucrezi pe el.

**Mai multe baze deodata.** Cheia unui rand devine `(baza, schema.nume)`:
acelasi obiect din doua baze sunt doua randuri, cu bife, plan si marcaj de
deploy separate. Banda de sus arata mediul cel mai riscant dintre tinte — cu o
baza PROD printre ele, ecranul n-are voie sa para DEV.

**Compararea pe alt fir.** Firul trimite **mereu** o veste finala (gata, esuata
sau anulata), ca ecranul sa nu ramana agatat pe „se compara" daca firul moare.
Anularea e un steag citit intre baze, nu o oprire fortata.

**Obiectele identice intra in lista.** Nu vin din comparare (n-au ce spune),
dar fara ele categoria „Identice" arata mereu 0, iar cautarea nu gaseste un
obiect care exista si e la fel. Le adauga `Aplicatie`, nu captura: captura
listei s-a mutat in `app` si trece prin `Aplicatie`, ca sa nu existe o a doua
constructie a listei (era deja divergenta — captura punea `scor: None`).

**Selectorul de fisiere: fara dependinta noua, deocamdata.** „Deschide un
instantaneu" arata `.json`-urile gasite langa aplicatie, in directorul de date
si in `referinta/instantanee`, plus o cale scrisa de mana. Alternativa era
`rfd` (dialogul nativ): o dependinta noua, care aduce GTK sau portalul XDG pe
Linux si **pe care n-o pot proba fara ecran** — adica exact un buton livrat
nedovedit. Ramane de decis de Mihai; daca raspunsul e „da", intra ca dependinta
proprie, cu intrarea ei aici.

**Unirea ciclului de cursor sta in `core`.** `aliniaza_cu` / `diff_aliniat_cu`
duc optiunea pana la `split_statements`, care o avea deja. Un al doilea loc
care decide granite de blocuri ar fi fost bug-ul reparat la etapa 7.

---

## 2026-11-18 — Fixture-ul de referinta, regenerat cu cazul lui Mihai

**Motiv.** Mihai a raportat „inconsistenta motorului" pe o comparare offline
(`WHile 1=1 / BEgin ...` fata de `IF 1=1 / While 1=1 / BEGIN ...`). Cazul a
intrat in `referinta/cazuri.json` ca `mihai_offline_while_begin`, deci
fixture-ul s-a regenerat explicit cu `unelte/regenereaza-fixture.sh`.
Diff-ul adauga **doar** cazul nou.

**Ce a aratat.** Rust da exact ce da motorul JS din v3.4: aceleasi patru
blocuri de fiecare parte, acelasi sir de diferente. Singura divergenta e cea
deja deliberata — continutul string-urilor nu se mai trece in litere mici
(`'Start END'` ramane asa), deci cazul a intrat si in
`DIVERGENTE_DELIBERATE`. Ce deranjeaza la privit tine de **afisare**, nu de
motor: doua blocuri complet diferite se arata ca sters + adaugat, nu
imperecheate.

## 2026-11-18 — Fara eticheta de mediu; antetul listei curge dupa fereastra

**Eticheta PROD / TEST / DEV scoasa de peste tot** (antet, editor, deploy,
conectare, recente), la cererea lui Mihai: mediul se ghicea din numele
bazei, iar GMotion nu are de unde sti la ce e folosita o baza. O eticheta
gresita pe exact baza care conteaza e mai rea decat niciuna. S-a scos si
logica din spate (`Mediu`, `mediu_dupa_nume`, jetoanele `mediu_*`), nu doar
desenul: altfel ramanea o a doua sursa de adevar nefolosita. Intrebarea
deschisa din etapa 11 („de unde se ia mediul") se inchide asa. Daca va fi
nevoie de el (de exemplu pentru garda de deploy), vine configurat explicit,
nu ghicit.

**Antetul listei pe doua niveluri:** pe primul rand doar semnul si bazele
(plus tema si limba); dedesubt, patru grupuri — starile, actiunile,
cautarea, filtrele, planul — care stau alaturi cand incap si trec pe randul
urmator cand nu. Slint n-are un layout care sa se rupa singur pe randuri,
deci pozitiile se calculeaza din `preferred-width` al fiecarui grup.

---

## 2026-11-18 — Editorul primeste taburi; lista din stanga primeste pin si ascundere

**Context.** Mihai a adus ca exemplu un ecran de code review (lista de
fisiere, taburi, comentarii pe linii) si a cerut ce se poate lua de acolo.

**Decizie (Mihai).** (1) Fara note / comentarii pe blocuri sau linii in
editor, nimic legat de ele. (Nota per obiect din panoul de detalii, etapa 15
G, e alt lucru si ramane.) (2) Lista de obiecte langa editor, cu pin, ascundere (`«`) si mutare pe
partea cealalta, ingusta, cu numele complet la hover. (3) Taburi: un tab de
previzualizare plus maxim 8 fixate, pastrand si `j` / `k`; ce vine la `j` /
`k` nu e niciodata un obiect deja deschis intr-un tab fixat.

**Alternative.** Taburi fara limita (4000 de obiecte → 4000 de taburi);
fara taburi deloc (lista + `j` / `k` acopera parcurgerea, dar nu saltul
intre 2-3 obiecte lucrate in paralel).

**Motiv.** Previzualizarea tine parcurgerea la un singur tab indiferent de
numarul de obiecte; limita de 8 tine bara lizibila. Fixarea se face prin
dublu clic si clic dreapta (alegerea asistentului, la cererea lui Mihai):
cele mai simple si fara ambiguitate; tragerea in bara de taburi se poate
adauga dupa, pe mecanismul de drag din etapa 14 F.

**Consecinte.** Etapa 16 (`etape/16-lista-si-taburi.md`), dupa 15, cu regulile
complete. Numele se taie pe mijloc, nu la capat: obiectele au prefixe lungi
comune si taiate la capat arata identic.

---

## 2026-11-18 — Auditul QA, H1: ce s-a decis pe siguranta deploy-ului

Noua constatari reparate (C1, C2, C5, C6, C7, C8, F-D1, F-D2, F-A5). Patru
decizii merita scrise, fiindca nu erau singura varianta posibila.

**1. Planul de deploy e legat de starea care l-a produs, nu de o lista de
nume.** C1 si C2 veneau amandoua din aceeasi cheie prea slaba: „ce e in
plan". Varianta minima ar fi fost `st.deploy = None` in `on_recompara` si
la fiecare comanda care schimba textul — adica o lista de locuri de care
cineva isi aminteste. Alegerea: o cheie explicita (`deploy::Cheie`) din ce e
in plan, amprenta textelor finale si `Aplicatie::id_comparare`, luat dintr-un
numarator la **construire**. O `Aplicatie` noua are mereu alt id, deci
„am uitat sa invalidez" nu se mai poate scrie. In plus `StareDeploy` nu mai
tine textele finale: le primeste la scriere, deci nu pot fi vechi.

**2. Afisarea nu mai atinge interiorul sirurilor — si asta schimba si
editorul, nu doar deploy-ul.** `pregateste_afisare` taia spatiile de la
capatul fiecarei linii fizice si unifica `\r\n` oriunde, inclusiv in
siruri; acelasi text e si ce vede omul in editor si ce trimite „Aplica
toate" in baza. Varianta „taie doar pentru afisare, trimite altceva in baza"
ar fi insemnat doua texte pentru acelasi obiect — exact clasa de bug de la
care am scapat la etapa 7. Alegerea: granitele vin din `lexer::segmente`, ca
orice regula pe text, iar un text final egal cu afisarea referintei nu mai e
socotit editare (se deployeaza `definitie`, octet cu octet).

**3. Setarile de creare conteaza: acelasi cod cu alt `QUOTED_IDENTIFIER` nu
mai e „identic" (C6).** Divergenta deliberata fata de v3.4. Alternativa ar
fi fost o categorie noua („doar setari"), care ar fi atins filtrele, culorile
si modelul listei; am ales un rand `Diferit` cu rezumatul „difera doar
setarile: ...", fara procent de similaritate — textul e acelasi, un procent
ar minti. Deploy-ul scria deja setarile corect, deci randul e deployabil fara
alta schimbare.

**4. Cautarea in baza tinta e acum case-insensitive, dar numai in plan,
garda si revenire (C8).** Colatia implicita la SQL Server nu deosebeste
majusculele, deci `CREATE OR ALTER PROCEDURE dbo.GetX` **modifica** un
`dbo.getx` existent; planul il credea inexistent si revenirea il stergea.
Varianta „uneste perechile lipsa/extra la comparare" ar fi schimbat
clasificarea si paritatea; am atins doar partea care scrie in baza, plus un
rezumat care spune varianta gasita. Pe o colatie case-sensitive (rara)
reparatia lasa in urma un obiect in plus la revenire, in loc sa stearga unul
care exista — partea recuperabila a greselii.

**Nefacut, deliberat:** F-D7 (oprirea intre obiecte, `IF @@ERROR <> 0 SET
NOEXEC ON` dupa fiecare obiect). Nu pot afla pe Linux ce vede `@@ERROR` dupa
un `GO`, si o garda care nu garanteaza nimic e mai rea decat lipsa ei.
Proba ceruta lui Mihai e in `STARE.md`.

---

## 2026-11-18 — Auditul QA, H2: ce s-a decis pe pierderea de date

Sase constatari reparate (C3, C4, F-B4, F-A1, F-A2, F-A3). Patru decizii de
scris.

**1. Curatarea e pe bazele compararii de acum, nu pe tot ce stie sesiunea.**
`curata(randuri)` stergea orice n-avea rand — iar o baza comparata alta data
n-are randuri. Alternativa ar fi fost sa nu curete nimic niciodata (sesiunea
ar creste la nesfarsit cu bife care nu mai spun nimic). Alegerea:
`curata(randuri, baze_comparate)`, plus `Sesiune::adauga_baze`, care adauga
bazele in loc sa le inlocuiasca. Semantica e acum „sesiunea tine munca facuta
pe toate bazele comparate vreodata cu referinta asta".

**2. Identitatea unei surse e un lucru separat de descrierea ei.**
Pentru F-B4 (serverul in cheia sesiunii) aveam nevoie de un sir care sa
insemne „acelasi server" si maine. `descriere()` nu e: contine felul
autentificarii, deci s-ar schimba cand omul trece de pe SQL pe Windows, si
sesiunea ar parea pierduta. Deci `SursaObiecte::identitate`: `host[:port]`
pentru un server, iar pentru un instantaneu **serverul declarat in fisier**,
nu calea — acelasi instantaneu exportat din nou e tot acelasi server. Cheia
veche (doar referinta) se migreaza la prima citire, nu se abandoneaza.
Pe drum: antetul listei arata acum serverul curat, adica jumatate din F-A13.

**3. Campurile din Slint se rescriu la schimbarea modelului — in afara de
parola.** `text: root.text` e o legatura care se rupe la prima tasta, deci
dupa „Inverseaza" campurile aratau vechile valori si comparai in directia
opusa celei de pe ecran (F-A2). Reparatia e `changed text`. Dar campul de
parola **nu** poate fi rescris din model: modelul ii da buline cat lungimea
parolei, si rescris cu ele prima tasta de dupa ar trimite bulinele ca parola
— ceea ce testul a si aratat. Acolo se goleste doar cand modelul spune ca nu
mai exista nicio parola. Ce ramane (dupa „Inverseaza" spre o parte care are
parola, campul tine ce s-a tastat) e notat in „Verdicte"; reparat de tot ar
cere ca parola sa nu mai fie reprezentata prin buline in model.

**4. Unde nu se poate testa, se pune in tipuri.** Blocarea tastaturii cat
ruleaza compararea (F-A3) sta in `main.rs`, care n-are teste. Un `if ruleaza
{ return }` copiat in nouasprezece callback-uri e exact felul de lucru care se
uita la al douazecilea. Deci `Stare::parte` intoarce `Option`, `None` cat
ruleaza: compilatorul cere tratat cazul la fiecare camp. Iar rezultatul
compararii isi poarta referinta cu el (`Comparata::referinta`), ca sa nu mai
depinda de formularul de acum.

---

## 2026-11-19 — Auditul QA, H3: ce s-a decis pe cadere si ecrane

**1. O constatare care nu se mai reproduce primeste tot un test.** Trei din
sase (C9 — caderea hartii, V1 si V2 — antetul listei) disparusera odata cu
refacerea hartii si a antetului de la etapa 15. Tentatia e sa le bifezi si sa
treci mai departe. Dar atunci urmatoarea refacere le poate aduce inapoi fara
ca cineva sa stie. Deci: `nicio_imagine_a_hartii_nu_trece_de_8192_px` (pe
50.000 de obiecte de un singur tip) si capturile la 1366x768 si 1024x768 intra
in setul standard.

**2. Un audit facut pe cod vechi arata simptomul, nu locul.** V9 („bifa unui
rand bifeaza alt obiect") era descrisa ca o problema de `.slint`. In codul de
acum `.slint` trimitea de mult indicele corect, iar legatura din `main.rs` il
ignora. Reparat acolo, cu `Comanda::ComutaRevizuitLa(usize)`. Morala pentru
restul auditului: reproduci intai, apoi citesti unde cade, nu inainte.

**3. Liniile lungi: derulare, nu impachetare.** Pentru V10 se putea si una si
alta. Impachetarea ar muta liniile una fata de alta, iar imperecherea
vizuala stanga-dreapta e tot rostul ecranului. Deci derulare pe orizontala,
**comuna celor doua panouri**, cu bara trasa cu mouse-ul si Shift + sageti de
la tastatura. Cat se poate derula vine din Rust (`latime_continut`): Slint nu
da latimea unui text fara sa-l deseneze.

**4. Insulele de jos trec pe doua randuri, nu intr-un meniu.** Auditul
propunea un meniu „…" sub o latime. Un buton ascuns intr-un meniu e un buton
pe care nu-l mai gaseste nimeni; doua randuri costa 58 px din inaltimea
codului si se vad toate. La 1440 raman pe un rand, ca pana acum.

---

## 2026-11-19 — Auditul QA, H4: ce s-a decis pe inghetari

**1. Un mecanism, nu patru reparatii.** P4 insemna patru locuri care tineau
fereastra (citirea bazelor, „Testeaza conexiunea", cautarea de servere, garda
de deploy). Fiecare mutata de mana pe un fir ar fi insemnat patru feluri de a
astepta. Deci `fir::Lucru<T>`: porneste o functie pe alt fir, se intreaba la
aceeasi bataie de ceas ca si compararea, si are o stare vizibila cat lucreaza
(`SeConecteaza` pe partea de conectare, `GardaDeploy::SeVerifica` pe deploy).
Garda a primit si `pornire::reciteste`: ii trebuie obiectele de acum, nu o
comparare intreaga.

**2. Ce nu se poate masura intr-un test se numara.** P1 (modelul editorului
refacut la fiecare desenare) nu se poate proba cu timpi: un test care
masoara secunde e instabil pe orice masina de CI. Deci `Aplicatie` numara de
cate ori construieste modelul (`refaceri_model`), iar testul se uita la
numarul ala. Cheia modelului e `(obiect, amprenta textului final, unirea
cursorului)` — restul (bifa, „in plan", filtrele) nu-l mai atinge.

**3. Al doilea lexer din `segmentare` a fost scos, nu carpit.** P3 era o
consecinta: scanarea proprie de caractere nu stia de `[...]`, `"..."` si `@`,
deci `SELECT [Case]` deschidea un `CASE` care nu se mai inchidea si tot
obiectul devenea un singur bloc. Se putea adauga `[` si `"` in scanarea aia;
atunci am fi avut doua lexere care trebuie tinute la fel. Linia „curatata"
vine acum din `lexer::segmente`, ca orice regula pe text (regula din
`STARE.md`), iar starea „linia asta incepe intr-un sir deschis mai sus" se
citeste tot din segmente. Divergenta fata de v3.4 si de JS e deliberata:
acolo bug-ul exista, iar paritatea pe fixture n-a cazut.

**4. Masuratorile intra in `bench`, nu raman in capul nostru.** P2 si P3 s-au
masurat pe obiecte construite anume (corpusul n-are obiecte de 20.000 de
instructiuni). Cazurile au ramas in `cargo run --release --example bench`:
2,12 s -> 1,6 ms pe `linii_start_blocuri`, 4,05 s -> 129 ms pe `aliniaza`.

---

## 2026-11-19 — Auditul QA, H5: ce se repara acum si ce se amana

**Regula de taiere.** Dupa H1-H4 ramaneau 60 de constatari. Le-am impartit
dupa un singur criteriu: **schimba ce ajunge in baza, sau ce crede omul ca e
adevarat?** Daca da si e ieftin, s-a facut acum (unsprezece). Daca nu, intra
in tabelul de verdicte cu motivul ei. Amanat fara motiv scris inseamna uitat.

**Ce a intrat, desi e „MIC".** `revizuire --marcheaza` care bifeaza doar
prima baza si `export` care scrie mereu „NU" la Rezolvat mint despre munca
facuta — omul crede ca a revizuit, si n-a revizuit. Ctrl+Z cu istoric global
schimba textul altui obiect fara sa se vada. Obiectele criptate deschise in
editor salvau un text final **gol**, care apoi s-ar fi deployat. Astea sunt
clasa „cod gresit in baza", oricat de mic le-ar fi eticheta.

**Ce s-a amanat, desi e „MEDIU".** F-M1, F-M4, F-M5, F-M6 ating motorul si
paritatea: fiecare cere un caz in `referinta/cazuri.json` si poate o intrare
in `DIVERGENTE_DELIBERATE`. Facute in graba, intre alte zece reparatii, ar
reintroduce exact felul de divergenta pe care fixture-ul il pazeste. Au
nevoie de un pachet propriu.

**MSRV-ul a devenit o verificare, nu o corectura.** `rust-version = 1.74` era
fals de mult (dependentele cereau 1.89, Slint 1.95), si nimic nu-l verifica.
Se putea scrie 1.89 si gata. Dar aceeasi minciuna se reface singura la
urmatoarea dependinta ridicata, deci poarta compara acum declaratia cu ce cer
dependentele, din `cargo metadata` — fara sa ceara un toolchain vechi
instalat. Vazuta cazand pe 1.74 si pe 1.80. Efect colateral: cu MSRV-ul real,
clippy porneste lint-uri pe care le tinea inchise (`is_none_or`).

---

## 2026-11-19 — „Martor" / „Modificata" in `cli`: doar textele, nu interfata

**Context.** Etapa 15 E cerea redenumirea si in `cli`; `CLAUDE.md` §6 spune ca
in cod `referinta` si `tinta` raman, „asa e si in `cli` si in plan".

**Decizie (Mihai, „fa daca este relevant").** Se schimba doar ce citeste
omul: ajutorul, rezumatul, progresul, erorile. Optiunile (`--referinta`,
`--baze`), cheile JSON, formatul planului si antetul scripturilor raman: sunt
interfata pe care o folosesc scripturile si planurile deja salvate, iar o
redenumire le-ar rupe fara castig. `--sursa` ramane: inseamna fisierul de
unde se citeste, nu una dintre parti.

---

## 2026-11-19 — Deciziile deschise din audit si din etapa 12, inchise

**Decizie (Mihai, recomandarile acceptate in bloc).**
- **F-M3, hinturile de blocare:** raman zgomot la comparare, dar un obiect
  care difera **doar** prin ele nu mai iese „identic": apare in lista ca
  „difera doar in hinturi", ca o reparatie de concurenta sa nu se piarda.
- **F-A18, ca in v3.4:** „Aplica toate" marcheaza obiectul revizuit; N din
  „Cate N obiecte per fisier" se schimba din ecran; „In plan" refuzat (obiect
  nedeployabil) spune de ce.
- **F-B5, setarile moarte:** `timeout_conectare_s` si `prag_similaritate` se
  leaga; `uneste_cursor` se scoate (ciclul de cursor se uneste mereu).
- **`DROP`:** deploy-ul nu sterge obiecte; `extra` se sar, ca in v3.4.
- **„Ruleaza":** GMotion doar scrie `.sql`; rularea ramane in SSMS / sqlcmd.
- **Ordinea:** se incheie etapa 15, apoi design-ul nou al conectarii, apoi
  etapa 16.

---

## 2026-11-19 — Erorile si jurnalul raman in romana

**Context.** 15 E cerea toate textele facute in Rust si in engleza.

**Decizie.** Erorile (35 de mesaje cu cod `GM-*`) si jurnalul `tracing` nu se
traduc acum. Erorile ajung in fereastra ca `String` (`e.to_string()`), nu ca
tip; ca sa urmeze limba ar trebui duse tipate pana in ecran, prin `db`,
`platforma`, modul si `app` — o schimbare mare pentru texte care se citesc
rar si se cauta dupa cod. Codul ramane stabil, deci un mesaj in romana se
gaseste oricum. Jurnalul e pentru depanare, nu pentru utilizator.

---

## Coduri de eroare — registru

Un cod alocat nu se refoloseste si nu se renumeroteaza, ca sa ramana
cautabil in jurnale vechi si in rapoarte de bug.

| Prefix | Zona |
|---|---|
| `GM-CORE-*` | motorul de comparare |
| `GM-DB-*` | conexiune, driver, citire obiecte — **alocate 001-011**, vezi `rust/db/CRATE.md` |
| `GM-PLAT-*` | configurare, sesiuni, rutare — **alocate 001-011**, vezi `rust/platforma/CRATE.md` |
| `GM-MOD-*` | module — **diff-obiecte: 001-013**, vezi `rust/module/diff-obiecte/CRATE.md` |
| `GM-UI-*` | fereastra, randare |

`core` nu intoarce erori tipate: functiile lui sunt totale pe intrari valide
UTF-8.

`GM-DB-001` conectare · `002` autentificare · `003` auth integrata indisponibila
· `004` interogare · `005` baza lipsa · `006` descoperire · `007` citire
instantaneu · `008` scriere instantaneu · `009` versiune instantaneu · `010`
nume invalid.
