# Etapa 24 — remedieri dupa evaluare: deploy, fereastra, main.rs

**Stare:** INCHEIATA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** 2026-11-22

## Scop

Evaluarea din 2026-11-22 (nota 7,5/10) a gasit riscul cel mai mare in deploy
(un obiect care cade lasa restul sa ruleze) si fragilitatea cea mai mare in
`app/src/main.rs` (2.300 de linii, zero teste). La final: deploy-ul se
opreste la primul obiect care cade, revenirea nu se impiedica de
`SCHEMABINDING`, constatarile F-A ramase din audit sunt inchise cu teste, iar
logica din `main.rs` sta in cod testat fara fereastra.

Alegerile lui Mihai (2026-11-22): F-D7 cu tiparul `IF @@ERROR <> 0 SET NOEXEC
ON` acum, cu proba pe SQL Server ca o confirmare ulterioara; toate cele patru
pachete.

## Criterii de terminare

Pachetul 1, deploy:

- [x] dupa fiecare obiect din script (si din scriptul de revenire): `GO`,
      `IF @@ERROR <> 0 SET NOEXEC ON`, `GO`; la final `SET NOEXEC OFF`; test
      pe textul generat, si `gmotion deploy` il scrie
- [x] revenirea sterge intai obiectele noi (ordine inversa dependentelor),
      apoi reface; un view nou cu `SCHEMABINDING` peste o functie existenta
      nu mai blocheaza refacerea ei; test (formularea initiala, „se reface
      inaintea ei", nu se poate garanta cu `ALTER`: vezi mai jos)
- [x] raportul deploy-ului avertizeaza pentru view-uri care pot avea indecsi
      (cele existente cu `SCHEMABINDING`: indecsii nu sunt in definitie) si
      pentru triggere existente (ordinea `sp_settriggerorder` se
      pierde); avertismentul se vede si in `cli`, si in ecranul de deploy

Pachetul 2, fereastra (fiecare cu test in `rutare` / model, fara fereastra):

- [x] P8: „Anuleaza" opreste compararea in curs (firul se termina in cel mult
      un obiect dupa apasare); test cu un fir care verifica oprirea
- [x] F-A6: „Linii" si „Difera in" calculate pentru toata lista pe fir;
      sortarea pe ele e corecta peste randul 60; test
- [x] F-A8: „Recompara" pastreaza „in plan", filtrele, sortarea si selectia;
      o recomparare esuata se spune pe ecranul listei; test
- [x] F-A12: eroarea de comparare are camp propriu, nu e pusa pe referinta
- [x] F-A15: o sesiune corupta e pastrata ca `.corupt` si se spune
- [x] F-A16: starea de la conexiunea anterioara se goleste la conectare noua
- [x] F-A17: coloana Baza ramane cand sunt 2+ baze comparate (textul „X din
      N" si contorul Identice erau deja reparate)

Pachetul 3, `main.rs`:

- [x] mutarea starii si a deciziilor din `main.rs` intr-un modul testat; in
      `main.rs` raman legaturile Slint (commit separat de reparatii, dar
      impreuna cu macro-urile de legare, fara schimbare de comportament)
- [x] `main.rs` sub 1.200 de linii; codul mutat are teste

Pachetul 4, functii noi:

- [x] F-A9: un obiect doar in Martor sau doar in Modificata se deschide in
      editor (tot codul ca adaugat / sters), cu test; accesibil si din `cli`
- ~~numele se imperecheaza fara diferenta de majuscule intre baze~~ —
      **scos** (Mihai, 2026-11-22): ramane decizia C8, vezi `JURNAL.md`

Comune:

- [x] fiecare binar atins, probat rulandu-l: `gmotion deploy` /
      `diferente` (teste pe binar si de mana), `gmotion-ui --previzualizare`
      sub Xvfb (compara si arata lista, fara cadere)
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat (F-D7 neprobat pe SQL Server; numele
      case-insensitive)

## Pasi

1. Pachetul 1 (`module/diff-obiecte/src/deploy.rs`, `plan.rs`).
2. Pachetul 3: intai mutarea, apoi teste — ca F-A-urile sa se poata testa.
3. Pachetul 2, pe codul mutat.
4. Pachetul 4.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-22** Toate pachetele facute. Pe drum:
  - discul sesiunii s-a umplut de doua ori (`target` ajunsese la 35 GB);
    poarta cadea cu „Bus error" la linker si la `cargo check` pe Windows.
    Cu `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0` ramane sub 5 GB;
  - `gmotion-ui` sub Xvfb cerea `libxkbcommon-x11-0`, instalat cu `apt`;
  - testele vechi pentru F-A9 descriau exact comportamentul schimbat
    voit si au fost rescrise.

## Ce a iesit altfel decat planul

- Trei constatari erau deja reparate de etapele 19-22 fara sa fie bifate in
  audit: contorul „Identice", textul „X din N nerevizuite", detaliile pe
  toata lista (F-A6 cerea doar resortarea la sosirea lor). Auditul trebuia
  recitit pe cod inainte de a scrie criteriile.
- Revenirea nu poate garanta ordinea „view inainte de functie" la refacere:
  `ALTER` pe o functie legata cu `SCHEMABINDING` cade oricum. S-a facut ce se
  poate garanta (stergerile intai) si s-a pus avertismentul.
- Imperecherea fara majuscule era deja decisa altfel (C8); intrebarea trebuia
  pusa la scrierea criteriilor, nu la implementare.
- `main.rs` a coborat la 1.100 de linii mai mult prin macro-uri de legare
  decat prin mutare; ce era logica e acum in `stare.rs` (17 teste).
