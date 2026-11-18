# Etapa 14 — paritatea functionala cu v3.4, si modul de previzualizare

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-18 · **Incheiata:** 2026-11-18

## Scop

Tot ce facea v3.4 se poate face si in `gmotion-ui`, pe aspectul din etapa 13,
**fara schimbari de design**: elementele noi se iau din schita
(`cercetare/schita-slint/README.md`, unde fiecare functie are locul stabilit).
La final, un mod de previzualizare: aplicatia porneste dintr-un JSON de
conectare si intra singura pe un instantaneu cu obiecte variate, complexe si
scrise dezordonat, ca Mihai sa probeze functiile fara baza de date.

Cerut de Mihai pe 2026-11-18: „implementezi toate functiile, iar la final un
mod preview".

## Criterii de terminare

Fiecare functie: logica in afara ferestrei, cu test; butonul legat, cu test
care il apasa; accesibila din `cli` daca schimba date (CLAUDE.md §3). Randul
corespunzator dispare din `cercetare/lipsuri-portare.md`.

**A. Bug-uri**
- [x] pe instantaneu, lista de baze din conectare vine din instantaneu
- [x] pe instantaneu, randul certificatului nu se arata

**B. Editorul**
- [x] alinierea pe linii sta in `core`; editorul si compunerea textului final
      o folosesc pe aceeasi (un singur motor)
- [x] textul final per obiect: Aplica bloc, Aplica toate, Reset, Ctrl+Z
- [x] Editeaza textul final (panoul Baza devine editabil)
- [x] marcajul liniilor aduse din sursa si al celor scrise de tine
- [x] Omite (`o`), cu stare tinuta minte
- [x] Copiaza sursa / Copiaza rezultatul
- [x] Ascunde diferentele; Re-sincronizeaza (recitirea obiectului)
- [x] modul ghidat: Din sursa / Din tinta / Editeaza / Aplica / Sari, cu
      dialog de final
- [x] deploy-ul foloseste textul final cand exista
- [x] textul final si „omis" supravietuiesc inchiderii (sesiunea)
- [x] din `cli`: aplicarea blocurilor si deploy cu textul final

**C. Lista**
- [x] scorul de similaritate si filtrul min–max %
- [x] filtrul pe stare: nerezolvate / rezolvate / omise / in plan / deployate
- [x] sortare pe fiecare coloana, ambele sensuri
- [x] Export CSV al listei filtrate
- [x] Ctrl+F in cautare
- [x] contoarele „% revizuite" si „% deploy"
- [x] starea „deployat" tinuta minte; „include si ce a fost deja deployat"

**D. Conectarea**
- [x] o referinta comparata cu mai multe baze (bife); filtrul pe baza in lista
- [x] compararea pe alt fir: progres, timp, mesaj, Anuleaza
- [x] Deconecteaza (inapoi la conectare, cu confirmare)
- [x] Testeaza conexiunea, cu detaliile serverului
- [x] etichetele CLUSTER / IMPLICITA pe serverele gasite
- [x] Sterge log in banda de jurnal
- [x] Sesiuni: lista, Reia, Ignora („Comparari recente", cu Ignora pe rand)
- [x] compararea offline: doua bucati de cod lipite
- [x] Ajutor (F1), ecran complet (F11). **Despre** (licenta Slint) nu s-a
      facut: textul de licenta il da Mihai, vezi `STARE.md`

**F. Cele mai grele in Slint** (se fac dupa A-D; daca una nu se poate,
se scrie de ce in `JURNAL.md`, nu dispare tacit)
- [x] coloane redimensionabile, autofit la dublu clic
- [x] drag & drop de blocuri si „Plaseaza aici", cu Ctrl+Z
- [x] liniile de legatura dintre blocuri, in canal
- [x] previzualizarea la hover in modul unificat
- [x] Uneste ciclul de cursor
- [x] selectorul de fisiere — **fara dependinta noua**: alegerea se face in
      aplicatie, din `.json`-urile gasite pe disc. `rfd` ramane de decis
      (`JURNAL.md`); directorul de deploy ramane cel de date

**E. Previzualizarea**
- [x] `gmotion-ui --previzualizare` (si un JSON de conectare) intra direct in
      lista, fara server
- [x] instantaneul de previzualizare acopera: toate categoriile, toate
      tipurile, mai multe baze, obiecte mari, cod dezordonat (majuscule
      amestecate, tab-uri, spatii la capat, comentarii imbricate, string-uri
      cu cuvinte cheie, `GO` in definitie, setari `QUOTED_IDENTIFIER OFF`,
      dependente intre obiecte, criptate)

**Comun**
- [x] capturi noi in `cercetare/capturi/`, in ambele moduri, pentru ecranele
      schimbate
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. A — bug-urile (mic, sigur).
2. B — editorul: alinierea in `core`, apoi textul final in `diff-obiecte`,
   apoi fereastra. E fluxul central din v3.4 si riscul cel mai mare.
3. C — lista.
4. D — conectarea; mai multe baze schimba `rutare`, deci dupa B si C.
5. E — previzualizarea.

## Jurnal de lucru

- **2026-11-18** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-18** A gata. Gasit pe drum, mai grav decat cele doua: alegerea
  „se verifica certificatul" din ecran **nu ajungea in conexiune**
  (`pornire::deschide` punea mereu `Nevalidat`). Acum `Conectare` tine
  `verifica_certificatul: bool`, iar textul se deriva din el.

- **2026-11-18** B gata. Pe drum: fuzz-ul compunerii a gasit ca
  `linii_start_blocuri` aseza blocuri **in interiorul comentariilor** (bug
  vechi, se vedea si in editor), reparat prin lexer. Invarianta „Aplica
  toate da zero diferente" s-a dovedit prea tare (un comentariu lipit de cod
  muta granitele segmentarii); cea corecta e forma normalizata egala.
  Poarta are acum verificarea „fiecare buton are actiune" (vazuta cazand),
  iar `fereastra` un test care apasa tastele editorului.
  „Re-sincronizeaza" din v3.4 doar recalcula diff-ul (nu recitea serverul),
  deci aici redeseneaza. „Uneste cursorul" ramane la F.

- **2026-11-18** C gata. Ce s-a adaugat peste model: comenzile din `rutare`,
  marcarea „deployat" dupa o scriere reusita (si in `gmotion deploy`, ca sa nu
  tina cele doua jumatati evidente separate), exportul CSV al listei filtrate,
  si ecranul: coloanele Similar si Deploy, capete de coloana sortabile,
  filtrul pe stare ca meniu, intervalul min-max, „Toate in plan" cu optiunea
  pentru cele deja trimise, „% deploy" in antet, Ctrl+F.
  Decis pe drum: bifa **nu** re-aplica filtrul pe loc, ca in v3.4
  (`marcheazaRezolvat` nu cheama `renderLista`) — randul nu dispare de sub
  cursor. Doua asteptari din teste au fost gresite, nu codul: ordinea din
  `in_plan()` (lista completa, nu cea afisata) si „inversarea exacta" la
  sortare (numele departajeaza mereu crescator, deci coloanele cu valori
  egale nu se intorc oglinda).

- **2026-11-18** C inceput, **doar modelul** (testat prin testele existente,
  fara teste noi inca): `model_lista` are scorul, starea de deploy
  (Nu / Deployat / VersiuneNoua), filtrul pe revizuire, intervalul de
  similaritate, sortarea pe orice coloana in ambele sensuri,
  `pune_vizibilele_in_plan(cu_deployate)`; `Revizuire` tine deployatele cu
  amprenta. Sesiunea s-a oprit aici, la cererea lui Mihai.

## Ce a iesit altfel decat planul

**Testele au gasit trei bug-uri pe care codul le ascundea bine.**

1. `linii_start_blocuri` aseza blocuri **in interiorul comentariilor** — bug
   vechi, vizibil in editor, gasit de fuzz-ul compunerii, reparat prin lexer.
2. Alegerea „se verifica certificatul" din ecran **nu ajungea in conexiune**:
   `pornire::deschide` punea mereu `Nevalidat`. Ecranul spunea una, conexiunea
   facea alta.
3. Aplicatia **nu arata deloc obiectele identice**: randurile lor nu vin din
   comparare, iar `Aplicatie` nu le adauga. Filtrul „Identice" arata mereu 0,
   iar captura — care si le adauga singura — arata altceva decat aplicatia.
   Gasit de testul instantaneului de previzualizare, adica de pachetul E, nu
   de pachetul C unde fusese scris filtrul.

**A doua implementare apare singura, daca o lasi.** Captura listei isi
construia propriul model (cu `scor: None`, deci fara coloana Similar) si
divergase deja de aplicatie. S-a mutat in `app` si trece prin `Aplicatie`.
Aceeasi regula a salvat si „uneste ciclul cursorului": taierea blocurilor a
ramas in `core`, nu s-a rescris in editor.

**Doua asteptari din teste au fost gresite, nu codul** (regula din
`CLAUDE.md` §6 si-a facut treaba a patra si a cincea oara): ordinea din
`in_plan()` (lista completa, nu cea afisata) si „inversarea exacta" la sortare
— numele departajeaza mereu crescator, deci o coloana cu valori egale nu se
intoarce oglinda.

**Mai multe baze a fost cea mai scumpa schimbare**, cum se banuia: nu codul
nou, ci faptul ca `schema.nume` nu mai e o cheie unica. A trebuit schimbat
peste tot: selectie, sortare, plan, bife, marcaj de deploy, garda, captura.

**Ce n-a intrat, deliberat:** dialogul nativ de fisiere (`rfd`) si ecranul
„Despre" — amandoua asteapta o decizie, nu cod.
