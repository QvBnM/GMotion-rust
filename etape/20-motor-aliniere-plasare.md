# Etapa 20 — motorul: aliniere pe linii si plasare

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-19 · **Incheiata:** 2026-11-20

## Scop

Motorul stie azi **ce** difera, dar aproape nimic despre **unde** se aseaza
diferenta. La final: alinierea coboara de la bloc la linie, si exista un strat
de plasare (ce bloc din sursa corespunde cu ce loc din tinta), ca in v3.4.

Fara ele, randurile ramase in `cercetare/lipsuri-portare.md` §4 — drag & drop
de blocuri, „Plaseaza aici", liniile de legatura, previzualizarea la hover —
nu se pot implementa, oricat de bine ar arata ecranul. Nu sunt blocate de
Slint, ci de motor.

Cerut de Mihai, 2026-11-19: „fara motor nu poti merge mai departe" si „toate
functionalitatile alea trebuie implementate corect in noul format".

## Dovezile (masurate pe `d9346d7`, nu presupuse)

Caz de lucru: `uspLeasing2_GetTemplatesMultipleDocuments`, martor fata de
versiunea upgradata (fisierele, in `JURNAL.md` 2026-11-19).

1. **Alinierea se opreste la bloc.** `aliniere.rs:131` imperecheaza liniile
   dintr-un bloc modificat **pe index**: `AS` (sursa 10) ajunge langa
   `@UserId INT,` (baza 4).
2. **Cele doua `SELECT` nu se imperecheaza** (scor 24.00, pragul din
   `perechi_replace` e 44): ies „sters" + „adaugat", pe randuri diferite.
   v3.4 le rateaza si el (pragul lui e 55), **dar arata unde ar veni blocul**.
3. **Stratul de plasare lipseste complet.** v3.4: `_sugereaza`,
   `_celMaiApropiat`, `_gapSugerat` (`mostenire/ui.py` ~3790-3890).
4. **`rezumat_rapid` si `rezumat_diff` divergeaza** — deja in audit ca
   **F-M4**, nereparat.
5. **`scor_similaritate` creste cubic:** 85 ms la 2.000 de linii, 38 s la
   16.000, 400 s la 32.000 (masurat). Prototip cu automat de sufixe: aceleasi
   potriviri — verificat pe 800 de perechi din corpus, nu „apropiate" — in
   0,5 s la 16.000. Acopera partea de motor din **P7** din audit.
6. `MAX_PRODUS_ALINIERE = 144`: peste ~12x12 blocuri intr-un segment
   `replace`, imperecherea cade **tacut** pe pozitie — bug-ul v3.4 reintrodus
   pe obiecte mari.

`linii_start_blocuri` **nu** mai e in lista: reparat la P2 (`3955c7a`).

## Criterii de terminare

**A. Un singur adevar despre diferente**
- [x] F-M4 inchis: lista si editorul dau aceleasi numere pe acelasi obiect;
      `conformitate.rs::lista_si_editorul_numara_la_fel` le compara pe toate
      cazurile, si a fost vazut cazand pe numararea pozitionala veche
- [x] decizia „egal" din imperechere se ia pe chei (invarianta 1b), nu
      renormalizand textul: `norm` nu mai e apelat deloc din `diff.rs`
      (criteriul spunea „contor de apeluri `norm`"; contorul ar fi fost stare
      globala intre teste rulate in paralel, iar absenta apelului se vede
      direct in cod si o tine compilatorul)

**B. Aliniere pe linii**
- [x] liniile dintr-o pereche de blocuri se imperecheaza prin motor, nu pe
      index (`perechi_linii`; `RandAliniat` are acum cate o pozitie in bloc
      pentru fiecare parte)
- [x] liniile goale si cele care sunt doar comentariu nu intra in imperechere;
      se reaseaza dupa, cu `RandAliniat::ignorata`
- [x] fuzz: `compune` cu nimic aplicat = textul din dreapta octet cu octet;
      cu tot aplicat = forma normalizata a stangii. Vazut cazand: cu o linie
      fara pereche aruncata din `perechi_linii`,
      `compunerea_pastreaza_dreapta_si_aplica_tot` pica
- [x] cazul de lucru: cele doua `SELECT` ies **modificat**, fata in fata, cu
      virgula ca singura bucata marcata
      (`un_bloc_fata_de_un_bloc_se_imperecheaza_oricat_de_diferit`, plus proba
      pe binar cu `gmotion diferente`)

**C. Plasarea (portata din `ui.py`, cu pragurile ei)**
- [x] `sugereaza(randuri, pozitie, blocuri_tinta)` -> „inlocuieste blocul k"
      (scor >= 55) sau „insereaza la golul g" (`core/src/plasare.rs`)
- [x] antetul (primul bloc) se potriveste direct cu primul din tinta, fara
      scor — cazul „s-au adaugat parametri si scorul cade sub prag"
- [x] regula de ambiguitate: diferenta sub 8 puncte intre primii doi candidati
      -> decide pozitia (vecin la +-1, scor >= 40); altfel, daca cel mai bun e
      sub 70, nu se sugereaza nimic
- [x] teste pe cazurile numite in comentariile v3.4: antet cu parametri noi;
      doua `UPDATE ... WHERE ...` cu scoruri la mai putin de 8 puncte
- [x] harness de paritate: pagina v3.4 rulata intr-un browser fara cap,
      `referinta/plasare_js.json` (49 de sugestii, 25 de cazuri), comparat de
      `core/tests/plasare.rs`. Vazut cazand: 7 nepotriviri inainte de a repara
      scorul golului (-1, nu 0) si traducerea id -> pozitie

**D. Gruparea pentru afisare**
- [x] motorul spune, per linie: blocul, daca e prima/ultima a blocului
      (`primul_din_bloc` / `ultimul_din_bloc`), si daca e „ignorata la
      comparare" — fereastra le citeste, nu le recalculeaza

**E. Performanta**
- [x] `seqmatch::find_longest_match` prin automat de sufixe peste 250.000 de
      perechi, cu test care compara potrivirile cu bucla difflib pe 200 de
      secvente generate si pe lantul recursiv. **16.000 de linii: 38 s ->
      0,53 s**, cu acelasi scor
- [x] `tokeni_in_literal` si `clasificare::feluri_din_interval` cu cautare
      binara
- [x] `MAX_PRODUS_ALINIERE` 144 -> 4096, acum ca scorul e ieftin; cand tot se
      atinge, se spune: `diff_aliniat_cu_raport` si
      `Aliniere::segmente_pozitionale`
- [x] garda deterministica: `core/tests/cost.rs`, plafon pe numarul de pasi,
      nu pe timp. Vazuta cazand cu automatul oprit: 1.431.181.291 de pasi fata
      de plafonul de 5.000.000

**F. Depanabil fara fereastra**
- [x] `gmotion diferente` arata alinierea pe linii si plasarea sugerata.
      Randarea din `cli` nu-si mai face propria socoteala cu
      `linii_start_blocuri` si umplerea golurilor — citeste alinierea
      motorului, deci a doua implementare a asezarii a disparut

**Comun**
- [x] poarta verde; `CRATE.md` sincronizat pentru `core` si pentru
      `diff-obiecte`; `STARE.md`; `JURNAL.md`

## Pasi

1. A (consistenta) — scoate din drum un numar gresit care ar falsifica
   testele de mai jos.
2. B (alinierea pe linii) — partea noua; riscul e la `compune`.
3. C (plasarea) — port 1:1, cu harness de paritate contra `ui.py`.
4. D — derivat din B si C.
5. E — automatul de sufixe, apoi garda.
6. F — `cli`.

## Jurnal de lucru

- **2026-11-19** Etapa pregatita. v3.4 rulat local pe cazul de lucru
  (comparare offline, in browser fara cap) ca referinta de comportament;
  vezi `JURNAL.md`.
- **2026-11-19** Pornita, dupa ce Mihai a inchis etapa 19.
- **2026-11-19** **A gata.** Imperecherea blocurilor s-a mutat intr-un singur
  loc (`plan_blocuri`), din care citesc si `diff_aliniat`, si `rezumat_rapid`.
  Pe drum: fixture-ul JS pune pe `mihai_offline_while_begin` „2 modificate,
  1 adaugate, 1 sterse" la `rezumat_rapid` si „1 modificate, 2 adaugate,
  2 sterse" la `rezumat_diff` — **motorul JS se contrazice singur**, iar
  campul `tipuri` din acelasi fixture da numerele noastre. Trecut la
  `DIVERGENTE_DELIBERATE`, plus trei intrari in `*.asteptari.json` pentru
  paritatea cu Python (doua obiecte din instantaneul real). Niciuna „ca sa
  treaca poarta": lista si editorul dau acum aceleasi numere, verificat cu
  `gmotion diferente`.
- **2026-11-19** **B gata.** Alinierea coboara la linie: cheia e forma
  normalizata a liniei, segmentele `replace` se imperecheaza pe scor (prag 40),
  iar liniile goale si cele de comentariu ies din imperechere si se reaseaza
  dupa. `RandAliniat` are acum cate o pozitie in bloc pe fiecare parte si
  marcajul `ignorata`; `fereastra::model_editor` s-a mutat pe ele. In plus,
  regula „un bloc fata de un singur bloc se imperecheaza intotdeauna": fara
  concurenta, pragul n-are pe cine sa apere. Paritatea cu Python nu s-a
  clintit. Blocurile prea mari pentru imperecherea pe linii se raporteaza in
  `Aliniere::blocuri_nealiniate`, nu degradeaza in tacere.
- **2026-11-19** **C gata.** `core/src/plasare.rs`, port 1:1 cu pragurile lui
  v3.4. Paritatea nu se putea face ca la motorul de diff (plasarea tine de
  starea paginii, nu se extrage mecanic din `ui.py`), deci fixture-ul se scoate
  ruland **pagina v3.4 adevarata** intr-un browser fara cap, prin „comparare
  offline": `unelte/regenereaza-fixture-plasare.sh`. Doua diferente gasite asa,
  pe care citirea codului nu le-ar fi dat: golul sugerat poarta scor **-1**
  cand niciun candidat nu s-a calificat (nu 0), si `srcIndices` din v3.4 sunt
  id-uri de rand, nu pozitii.
- **2026-11-19** **D si E gata.** Motorul spune acum si unde incepe / se
  termina blocul, iar fereastra nu-si mai face socoteala ei. `seqmatch` cauta
  cea mai lunga potrivire cu un automat de sufixe peste 250.000 de perechi:
  acelasi rezultat (verificat element cu element contra buclei difflib), dar
  16.000 de linii se compara in 0,53 s in loc de 38 s. Cu scorul ieftin,
  plafonul de imperechere a blocurilor a urcat de la 144 la 4096, si cand tot
  se atinge se raporteaza. Garda de cost e pe pasi, nu pe timp.
- **2026-11-19** **F gata.** `gmotion diferente` arata plasarea („linia 26 ->
  inlocuieste blocul 4 (linia 11), potrivire 24%") si deseneaza din alinierea
  motorului, nu dintr-o asezare proprie. Pe drum s-a vazut ca **doua straturi
  dadeau raspunsuri diferite** pentru acelasi bloc: diff-ul imperechea
  `SELECT`-urile, iar plasarea (prag 55) spunea „fara corespondent". Impacate
  in `modul::plasari_obiect`: perechea aleasa de diff bate, plasarea raspunde
  doar pentru blocurile ramase fara pereche. Altfel „Aplica" ar fi scris in alt
  loc decat arata ecranul. Tot aici: `core::marcaje_pe_linii`, ca sa nu-si taie
  fiecare consumator singur marcajele pe `\n`.

## Ce a iesit altfel decat planul

**1. Paritatea pe plasare nu se putea face ca pe motorul de diff.** Planul
spunea „`_sugereaza` rulat in node". Nu merge: plasarea tine de starea paginii
(`DIF_EL`, textul din editor), nu se extrage mecanic din `ui.py`. Solutia a
fost sa rulam **pagina v3.4 intreaga** intr-un browser fara cap si sa-i dam
cazurile prin „comparare offline", care e complet locala. A meritat de doua
ori: a gasit doua lucruri pe care citirea codului nu le-ar fi dat — golul
sugerat poarta scor **-1**, nu 0, cand niciun candidat nu s-a calificat, si
`srcIndices` din v3.4 sunt **id-uri de rand**, nu pozitii.

**2. Doua straturi dadeau raspunsuri diferite pentru acelasi bloc.** Diff-ul
imperechea cele doua `SELECT` (regula 1x1), iar plasarea, cu pragul 55 al lui
v3.4, spunea „fara corespondent". S-a vazut abia cand `gmotion diferente` a
tiparit si una si alta, una sub alta. Daca ramanea asa, „Aplica" ar fi scris
in alt loc decat arata editorul. Impacate in `modul::plasari_obiect`: perechea
aleasa de diff bate, plasarea raspunde doar unde nu exista pereche. Lectia:
doua straturi corecte separat pot fi gresite impreuna, si singurul loc unde se
vede e o iesire care le arata pe amandoua.

**3. Un criteriu a fost scris gresit.** „Contor de apeluri `norm` in test"
cerea stare globala intre teste care ruleaza in paralel — exact capcana din
`CLAUDE.md`. Inlocuit cu ceva pe care il tine compilatorul: `norm` nu mai e
apelat deloc din `diff.rs`. In schimb, contorul **are** sens unde l-am pus la
final: in `SequenceMatcher`, pe instanta, nu global — si pe el sta garda de
cost.

**4. Masuratorile de la deschiderea etapei erau pe un branch vechi.** Intre
timp, alta sesiune impinsese 58 de commit-uri, inclusiv reparatia P2
(`linii_start_blocuri`). Una din cele sase dovezi era deci deja rezolvata, iar
F-M4 si P7 erau deja in auditul lor. Verificat din nou pe codul lor inainte de
a scrie ceva: din sase constatari, cinci au ramas valabile si una a picat.

**5. Castigul pe scor a fost mai mare decat speram** (38 s -> 0,53 s la 16.000
de linii, ~72x), dar a cerut si API nou: `operatii()`, ca garda sa fie pe pasi
si nu pe timp. Alinierea pe linii a costat ~15% in plus la deschiderea
editorului (129 ms -> 148 ms pe 16.000 de linii) — ieftin pentru ce aduce.
