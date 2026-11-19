# Etapa 20 — motorul: aliniere pe linii si plasare

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-19 · **Incheiata:** —

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
- [ ] `sugereaza(bloc, blocuri_tinta, indice_preferat)` -> „inlocuieste blocul
      k" (scor >= 55) sau „insereaza la golul g"
- [ ] antetul (primul bloc) se potriveste direct cu primul din tinta, fara
      scor — cazul „s-au adaugat parametri si scorul cade sub prag"
- [ ] regula de ambiguitate: diferenta sub 8 puncte intre primii doi candidati
      -> decide pozitia (vecin la +-1, scor >= 40); altfel, daca cel mai bun e
      sub 70, nu se sugereaza nimic
- [ ] teste pe cazurile numite in comentariile v3.4: antet cu parametri noi;
      doua `UPDATE ... WHERE ...` la 0,4 puncte distanta
- [ ] harness de paritate: `_sugereaza` din `ui.py` rulat in node, comparat cu
      Rust pe `referinta/cazuri.json`

**D. Gruparea pentru afisare**
- [ ] motorul spune, per linie: blocul, daca e prima/ultima a blocului, si
      daca e „ignorata la comparare" — fereastra nu recalculeaza nimic

**E. Performanta**
- [ ] `seqmatch::find_longest_match` prin automat de sufixe, cu test care
      compara **toate** potrivirile cu implementarea difflib (corpus + fuzz)
- [ ] `tokeni_in_literal` si `clasificare::feluri_din_interval` cu cautare
      binara
- [ ] `MAX_PRODUS_ALINIERE` ridicat sau inlocuit, acum ca scorul e ieftin;
      daca tot ramane un plafon, se raporteaza cand se atinge, nu tacut
- [ ] garda deterministica in poarta: contor de operatii (nu timp), vazuta
      cazand o data

**F. Depanabil fara fereastra**
- [ ] `gmotion diferente` arata alinierea pe linii si plasarea sugerata

**Comun**
- [ ] poarta verde; `CRATE.md` sincronizat pentru `core`; `STARE.md`;
      `JURNAL.md`

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

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
