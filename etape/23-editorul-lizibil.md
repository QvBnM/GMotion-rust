# Etapa 23 — editorul lizibil: etichete, grupuri, legaturi, tragere libera

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-20 · **Incheiata:** —

## Scop

Sesizarile lui Mihai pe editor (2026-11-20, cu capturi): etichetele blocurilor
acopera codul si se repeta pe randurile rupte; instructiunile consecutive
„doar in Martor" apar ca zeci de diferente separate; liniile identice se pliaza
fara sa poti alege; legaturile curbe dintre panouri (v3.4) au disparut;
tragerea unui bloc nu arata ca ai luat ceva si nu se poate lasa decat pe locul
sugerat.

Alegerile lui (intrebate inainte): legaturile curbe ca in v3.4; instructiunile
consecutive care merg in acelasi loc devin **o singura diferenta**; implicit
se vede tot codul, plierea e un comutator tinut minte.

## Criterii de terminare

- [x] eticheta blocului sta pe un rand al ei, deasupra blocului, in ambele
      panouri (alinierea ramane), si nu acopera codul; apare o singura data,
      nu si pe randurile rupte de wrap
- [x] instructiunile consecutive fara pereche care merg in acelasi gol sunt o
      singura diferenta: o eticheta („blocurile 21-25"), o sageata care le
      aplica pe toate, un pas la Ctrl+N / Ctrl+P; gruparea sta in `core`, nu
      in fereastra; o instructiune din grup se poate trage singura
- [x] implicit nu se pliaza nimic; comutatorul „Ascunde identicul" e vizibil
      in bara editorului si se tine minte intre sesiuni
- [x] legaturile curbe in canal, de la fiecare diferenta vizibila la locul ei
      din Modificata (plina = inlocuieste, stinsa cu capat gol = se insereaza:
      `Path` din Slint n-are linie punctata), urmaresc derularea
- [x] la tragere: blocul luat urmeaza mouse-ul (o fantoma cu primele lui
      linii), locul sugerat ramane aprins, iar locul de sub mouse se aprinde
      si el
- [x] lasat pe locul sugerat: ca „Aplica"; lasat pe un bloc din Modificata:
      il inlocuieste; lasat intre blocuri: se insereaza acolo. Ctrl+Z il
      scoate. Logica in `core`, accesibila din `cli`
- [x] fiecare are test (in model si cu mouse-ul / tastatura, unde se apasa)
- [ ] capturi in ambele teme pe cazul din sesizare si „da"-ul lui Mihai —
      `cercetare/machete/editor-lizibil/capturi/`; **asteapta „da"-ul**
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. pliere: implicit oprita, comutator, sesiune;
2. eticheta pe rand propriu (si fara repetare la wrap);
3. gruparea in `core`, apoi modelul si aplicarea;
4. legaturile curbe;
5. tragerea: fantoma, locul de sub mouse, lasarea oriunde.

## Jurnal de lucru

- **2026-11-20** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-20** Facute toate, in afara de „da"-ul pe capturi. Pe drum:
  - randul nou de eticheta a mutat `start` (randul cu sageata) pe eticheta;
    prins de testul care verifica primul rand al diferentei;
  - eticheta blocului urmator parea, dupa pozitie, inca un rand al blocului
    tinta (lumina la tragere se intindea peste ea): se numara doar randurile
    de cod;
  - blocurile 4 si 5 din cazul etapei 21 (doua diferente spre acelasi
    rand-fantoma) sunt acum un singur grup; testul de atunci s-a rescris;
  - `gmotion editare` numara acum pe grupuri, ca editorul; altfel
    `--aplica 2` ar fi insemnat altceva in `cli` decat in fereastra;
  - legaturile curbe nu au test propriu: sunt desen, din date testate
    (`start-diferente`, `tinte-*`).

- **2026-11-20** Mihai, pe captura: „nu arata bine" — intrebat ce anume, a
  ales golul hasurat din dreapta. Blocurile doar in Martor care se insereaza
  aveau doua locuri: golul hasurat in dreptul lor si, separat, mai jos,
  randul-fantoma. Acum locul e chiar golul, curat (fara hasura), cu textul
  „aici vin blocurile 5-8 din Martor" pe primul rand; randul separat a
  disparut, iar legatura e dreapta. Hasura ramane doar in interiorul
  blocurilor modificate (decizia lui Mihai din etapa 21).

- **2026-11-20** Mihai a precizat: nu hasura era problema, ci cutia
  blocului. Intrebat, a ales „ca in v3.4": blocul e acum o cutie cu chenar
  subtire, cu eticheta in coltul de sus si „1 / 3" in dreapta, pe partea
  (sau partile) unde blocul are cod. Randul etichetei a devenit antetul
  cutiei. Schimbarea de la golul hasurat (de mai sus) ramane: era tot o
  dublura.
- **2026-11-21** Intrerupere: Mihai a aratat, pe captura, un antet fara corp
  imperecheat „modificat" cu antetul care avea `BEGIN`. Vinovata era
  segmentarea (`BEGIN` / `END` lipite de vecini), iar „Aplica" pe acel bloc
  strica obiectul. Reparat in `core/src/segmentare.rs`; decizia si lista
  completa a golurilor in `JURNAL.md` (2026-11-21).

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
