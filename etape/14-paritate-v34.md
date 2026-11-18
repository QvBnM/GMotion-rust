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
- [ ] pe instantaneu, lista de baze din conectare vine din instantaneu
- [ ] pe instantaneu, randul certificatului nu se arata

**B. Editorul**
- [ ] alinierea pe linii sta in `core`; editorul si compunerea textului final
      o folosesc pe aceeasi (un singur motor)
- [ ] textul final per obiect: Aplica bloc, Aplica toate, Reset, Ctrl+Z
- [ ] Editeaza textul final (panoul Baza devine editabil)
- [ ] marcajul liniilor aduse din sursa si al celor scrise de tine
- [ ] Omite (`o`), cu stare tinuta minte
- [ ] Copiaza sursa / Copiaza rezultatul
- [ ] Ascunde diferentele; Re-sincronizeaza (recitirea obiectului)
- [ ] modul ghidat: Din sursa / Din tinta / Editeaza / Aplica / Sari, cu
      dialog de final
- [ ] deploy-ul foloseste textul final cand exista
- [ ] textul final si „omis" supravietuiesc inchiderii (sesiunea)
- [ ] din `cli`: aplicarea blocurilor si deploy cu textul final

**C. Lista**
- [ ] scorul de similaritate si filtrul min–max %
- [ ] filtrul pe stare: nerezolvate / rezolvate / omise / in plan / deployate
- [ ] sortare pe fiecare coloana, ambele sensuri
- [ ] Export CSV al listei filtrate
- [ ] Ctrl+F in cautare
- [ ] contoarele „% revizuite" si „% deploy"
- [ ] starea „deployat" tinuta minte; „include si ce a fost deja deployat"

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

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
