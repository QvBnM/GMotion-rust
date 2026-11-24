# Etapa 29 — remedierea auditului de pe clever-cerf

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-23 · **Incheiata:** —

## Scop

Auditul din `cercetare/audit-qa-2026-11-23-clever-cerf.md` a dat 5/10 pe
`b815d0e`. La final: remedierile din `claude/dreamy-ritchie-jdnfct` sunt unite
aici, iar constatarile care raman dupa unire sunt reparate, pana la cosmetic.

## Criterii de terminare

- [x] `claude/dreamy-ritchie-jdnfct` unit, cu poarta verde si cu coliziunile
      rezolvate explicit (`GM-DB-012`, B4 vs F-A9)
- [x] CRITIC: C-1 (coloana „Obiect"), C-2 (panoul de detalii), D-1 (poarta isi
      rescrie fixture-ul); B-1, B-2, B-3 au venit prin unire
- [x] MARE: B-4 (fals „modificat"), B-5 (garda de cost pe `plasari`), C-M2
      (etichetele blocurilor netraduse), D-2 („core nu are I/O"), A-1, A-2 si
      C-M1 prin unire
- [x] MEDIU: A-3, A-4, A-5, A-6, A-7, A-8, B-6, B-7, C-D2, C-D5, C-D6, C-D7,
      D-3, D-10; D-4 reparat, D-5 / D-8 / D-9 / D-11 erau deja pe dreamy
- [x] MIC si COSMETIC: A-10…A-16, B-8…B-13, C-S1…C-S9, K-1…K-5, D-12…D-23
- [x] fiecare verificare noua din poarta a fost vazuta **cazand**, o data
      (CLAUDE.md §10): fixture-ul regenerat, `std::fs` in `core`, zero etape
      active, callback cu `_` in nume
- [x] capturi noi la 1024x768 si 1366x768 pentru lista, editor si panoul de
      detalii, in ambele teme si in ambele limbi; cele din
      `cercetare/capturi/` refacute
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat cu deciziile luate la unire si la remediere
- [ ] **„da"-ul lui Mihai pe capturile noi**, si probele pe SQL Server /
      Windows din `STARE.md` (F-D7, D-17). Astea nu se pot bifa de aici.

## Pasi

1. Unirea cu `dreamy-ritchie`, conflict cu conflict. **Facut.**
2. Poarta: fixture-ul de referinta verificat, nu rescris; „core fara I/O"
   verificat pe text; „exact o etapa ACTIVA"; callback cu `_`.
3. Motorul: `RE_POATE_CONTINUA` cu `create`/`alter`; garda de cost pe
   `plasari`; unitatile din `marcaje_pe_linii`; ordinea plasarilor.
4. Deploy si sesiune: amprenta exacta la bifa si la textul final; textul final
   egal cu tinta nu intra in script; oprirea intre fisiere; `salvata` scrisa si
   din fereastra.
5. Interfata: latimile coloanelor si panoul de detalii, cu capturi; etichetele
   blocurilor prin `limba.alege`; jurnal la „Aplica diferenta".
6. `cli`: `--json` da JSON si pe eroare; `export` / `instantaneu` scriu ceva;
   `verifica` accesibila din linia de comanda.
7. Mici si cosmetice, in ordinea din raport.

## Jurnal de lucru

- **2026-11-23** Unire cu `dreamy-ritchie` (27 de fisiere cu conflicte). Doua
  coliziuni reale, nu de text: `GM-DB-012` folosit in paralel pentru doua erori
  diferite (a doua a trecut pe `GM-DB-014`), si B4 vs F-A9, care se contraziceau
  pe fond — castiga F-A9, garda B4 ramane pentru obiectele fara niciun text.
  Poarta verde: 749 de teste.
- **2026-11-23** Poarta: harness-ul nu mai rescrie fixture-ul, il compara;
  „core fara I/O" verificat pe text, fiindca `std::fs` compileaza pe wasm32;
  „exact o etapa ACTIVA" cade si la zero. Fiecare, vazuta cazand.

## De ce e inca ACTIVA

Tot ce se putea face de aici e facut si poarta e verde. Raman doua criterii
care nu se pot bifa fara Mihai: „da"-ul pe capturile noi (CLAUDE.md §10 cere
pereche captura-macheta, nu parerea mea) si probele pe SQL Server / Windows
din `STARE.md` — F-D7 (`@@ERROR` dupa `GO`) si D-17 (`SET NOEXEC OFF` in
textul trimis la „Verifica pe server"). Sectiunea de mai jos e scrisa deja,
ca sa nu se piarda.

## Ce a iesit altfel decat planul

**Unirea a fost partea grea, si nu din cauza conflictelor de text.** 27 de
fisiere cu conflicte s-au rezolvat in cateva ore, dar doua lucruri nu erau
conflicte de text si nu le-ar fi prins niciun `git merge`: `GM-DB-012`
folosit in paralel pentru doua erori diferite (ambele parti compilau, poarta
trecea, si codul ar fi insemnat doua lucruri la depanare), si B4 vs F-A9,
care se contraziceau pe fond — testul uneia cadea pe codul celeilalte.
Amandoua sunt acum in `JURNAL.md`. Daca branch-urile ar fi stat separate
inca o saptamana, numarul lor ar fi crescut, nu diferenta de linii.

**Jumatate din constatarile auditului erau deja reparate pe celalalt
branch.** Auditul a rulat pe `clever-cerf`, unde nu erau. Asta nu e o critica
a auditului — a fost cerut asa — dar e masura costului de a tine doua
branch-uri vii: aceeasi munca facuta de doua ori, o data ca reparatie si o
data ca descoperire.

**O reparatie de interfata a fost mai adanca decat parea.** C-1 (coloana
„Obiect" strivita) nu se rezolva cu un `min-width`: latimea tabelului nu se
poate citi din layout fara sa faci o bucla de legaturi in Slint („Recursion
detected", capcana deja scrisa in `STARE.md`). A trebuit calculata din
latimea ferestrei si a panoului, pe fereastra, nu pe layout. Prima varianta,
cu `init` si `changed width`, vedea latime 0 inainte de prima desenare si
ascundea toate coloanele — adica exact bug-ul, mai rau.

**Cateva „reparatii" au fost de fapt documente care minteau,** si au costat
mai mult de verificat decat de schimbat: invarianta 8d descria drept contract
chiar bug-ul F-A5; invarianta 4 din `app/CRATE.md` spunea contrariul codului;
`regenereaza-fixture-plasare.sh` si `STARE.md` promiteau un pas de poarta
inexistent; comentariul din `api-hash.sh` despre `mod teste` era fals. Toate
patru ar fi trimis pe o pista gresita pe cineva care le-ar fi crezut.

**Ce n-am putut repara de aici:** A-7 (oprirea intre fisiere) si D-17
(`SET NOEXEC OFF` in textul trimis la „Verifica pe server") cer un SQL Server.
Primul e documentat in antetul scripturilor impartite, al doilea e o proba in
`STARE.md`. Un mecanism de oprire care n-a fost vazut oprind nu se scrie.
