# Etapa 31 — fereastra unica si remedierile de interfata din proba lui Mihai

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-25 · **Incheiata:** —

## Scop

Proba lui Mihai din 2026-11-25 (`Analiza_GMotion_RUST.docx`, cererea
`PROMPT-GMotion-UX.md`): aplicatia ingheata la mutare si se inchide cu
-805306369, fiecare ecran e o fereastra care se inchide si se redeschide
in alt loc, iar lista, panoul de detalii si conectarea au zeci de lucruri
care incurca. La final: o singura fereastra, pe monitorul potrivit, fara
inghetari, cu ecranele trecand unul in altul, si toate punctele A-H din
cerere rezolvate sau lasate cu motiv in `cercetare/lipsuri-portare.md`.

## Criterii de terminare

- [ ] **A1** porneste pe monitorul cursorului, centrata, cel mult cat zona de
      lucru; reia locul de data trecuta daca monitorul mai exista (test pe
      `monitor::alege_geometria`)
- [ ] **A2** mutarea intre monitoare cu alta marire nu mai intra in bucla
      (fara scara proprie peste sistem; test cu zece factori pe fiecare ecran);
      ceas de garda care scrie in jurnal daca interfata nu raspunde
- [ ] **A3** fantoma de tragere, previzualizarea la hover si textul de hover
      din coloana editorului pleaca odata cu mouse-ul (test cu mouse-ul iesit
      din fereastra)
- [ ] **A4** clic pe rand fara „redesenare lenta" pe previzualizare
- [ ] **B** conectare: lupa curata; animatie cat se cauta / se citesc bazele;
      buton explicit „Adu bazele"; bara de derulare mereu vizibila; doar
      ultima sesiune + legatura spre Sesiuni; Martor / Modificata pe ambele
      panouri si mai departe (lista, editor, deploy)
- [ ] **C** o singura fereastra (`Aplicatie`), ecranele cu tranzitie; cat
      ruleaza compararea, restul e blocat, cu progres pe etape
- [ ] **D** lista: implicit Modificate + Similar crescator; fara coloana
      Linii; buton Deschide pe rand, fara dublu clic; nume fara `dbo.`, taiat
      pe mijloc, intreg la hover; tip prescurtat; data fara ora; Impact si
      Deploy ca puncte colorate cu text la hover (si celelalte pastile);
      fundalul hartii, estompat, sub lista
- [ ] **E** panoul de detalii: doar blocurile diferite, rupte pe latime si
      mai mici; rezumatul de linii (adaugate / sterse / modificate / mutate);
      „Foloseste" / „Folosit de" ca text selectabil, fara navigare
- [ ] **F** jurnal: etapele compararii cu durata; fara server / baza /
      utilizator in jurnalul din fereastra; certificatul neverificat o data pe
      sesiune
- [ ] **G** `cercetare/lipsuri-portare.md` la zi
- [ ] capturi in `cercetare/capturi/` pentru fiecare ecran atins, ambele
      teme, 1366x768 si 1920x1080
- [ ] poarta verde: `unelte/verifica.sh` (cu windows-msvc)
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat cu deciziile

## Pasi

1. Fereastra unica (shell generat), apoi pozitia la pornire, DPI, garda.
2. Componentele comune: bara de derulare, sfatul la hover, rotitorul.
3. Conectarea, lista, panoul de detalii, jurnalul.
4. Capturi, documente, poarta, impins.

## Jurnal de lucru

- **2026-11-25** Pornit de pe `claude/clever-cerf-hjk2qh` (cel mai nou).
  Poarta verde la pornire (cu wasm32 si windows-msvc instalate in container).
  Fereastra unica: `build.rs` genereaza `Aplicatie` din declaratiile de pe
  radacina celor patru ecrane, si cate un tip Rust per ecran cu aceleasi
  metode, ca restul codului sa nu se schimbe. Blocajul: ultima actiune din
  jurnalul lui Mihai e `plaseaza(0,-1)` in editor; pe Linux, sub Xvfb, aceeasi
  tragere nu ingheata. Cauza cea mai probabila e scara proprie a conectarii
  (trimitea `ScaleFactorChanged` peste cel al sistemului si se batea cu el la
  schimbarea monitorului); scoasa. Ceas de garda adaugat, ca o inghetare
  viitoare sa lase urma in jurnal.

## Ce a iesit altfel decat planul

—
