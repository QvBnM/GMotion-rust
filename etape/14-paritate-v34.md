# Etapa 14 — paritatea functionala cu v3.4, si modul de previzualizare

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-18 · **Incheiata:** —

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

**G. Panoul de detalii din lista** (cerut de Mihai pe 2026-11-18, dupa o
captura de referinta; schimbare de design venita de la el, deci in afara
regulii „fara schimbari de design")
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

**D. Conectarea**
- [ ] o referinta comparata cu mai multe baze (bife); filtrul pe baza in lista
- [ ] compararea pe alt fir: progres, timp, mesaj, Anuleaza
- [ ] Deconecteaza (inapoi la conectare, cu confirmare)
- [ ] Testeaza conexiunea, cu detaliile serverului
- [ ] etichetele CLUSTER / IMPLICITA pe serverele gasite
- [ ] Sterge log in banda de jurnal
- [ ] Sesiuni: lista, Reia, Ignora
- [ ] compararea offline: doua bucati de cod lipite
- [ ] Ajutor (F1), Despre (cerut de licenta Slint), ecran complet (F11)

**F. Cele mai grele in Slint** (se fac dupa A-D; daca una nu se poate,
se scrie de ce in `JURNAL.md`, nu dispare tacit)
- [ ] coloane redimensionabile, autofit la dublu clic
- [ ] drag & drop de blocuri si „Plaseaza aici", cu Ctrl+Z
- [ ] liniile de legatura dintre blocuri, in canal
- [ ] previzualizarea la hover in modul unificat
- [ ] Uneste ciclul de cursor
- [ ] selectorul de fisiere (instantaneu, director de deploy) — dependinta
      noua, cu intrare in `JURNAL.md`

**E. Previzualizarea**
- [ ] `gmotion-ui --previzualizare` (si un JSON de conectare) intra direct in
      lista, fara server
- [ ] instantaneul de previzualizare acopera: toate categoriile, toate
      tipurile, mai multe baze, obiecte mari, cod dezordonat (majuscule
      amestecate, tab-uri, spatii la capat, comentarii imbricate, string-uri
      cu cuvinte cheie, `GO` in definitie, setari `QUOTED_IDENTIFIER OFF`,
      dependente intre obiecte, criptate)

**Comun**
- [ ] capturi noi in `cercetare/capturi/`, in ambele moduri, pentru ecranele
      schimbate
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

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

- **2026-11-18** G (panoul de detalii) intercalat inainte de D, la cererea
  lui Mihai. Criteriile au fost scrise dupa ce codul era inceput (abatere de
  la CLAUDE.md §10, notata aici). G gata. Testul de mouse cauta randurile si
  X-ul prin clicuri, nu cu coordonate scrise de mana; vazut cazand fara
  `double-clicked` si fara Esc. Dublu clicul in Slint vine dupa doua
  `clicked` (deschis, inchis), deci `Deschide` redeschide panoul: la
  intoarcerea din editor lista arata obiectul de unde ai plecat. Nota se
  trimite cu obiectul ei (`nota-id`), nu pe selectie. Capturi:
  `cercetare/capturi/lista-*.png` (cu panoul) si `lista-fara-detalii-*.png`.

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
