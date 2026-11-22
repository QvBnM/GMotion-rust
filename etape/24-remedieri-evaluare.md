# Etapa 24 — remedieri dupa evaluare: deploy, fereastra, main.rs

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** —

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

- [ ] dupa fiecare obiect din script (si din scriptul de revenire): `GO`,
      `IF @@ERROR <> 0 SET NOEXEC ON`, `GO`; la final `SET NOEXEC OFF`; test
      pe textul generat, si `gmotion deploy` il scrie
- [ ] revenirea reface si sterge in ordine topologica inversa (un view cu
      `SCHEMABINDING` peste o functie se reface/sterge inaintea ei); test
- [ ] planul avertizeaza pentru view-uri cu index (`CREATE ... INDEX` pe
      view in definitie) si pentru triggere (ordinea `sp_settriggerorder` se
      pierde); avertismentul se vede si in `cli`, si in ecranul de deploy

Pachetul 2, fereastra (fiecare cu test in `rutare` / model, fara fereastra):

- [ ] P8: „Anuleaza" opreste compararea in curs (firul se termina in cel mult
      un obiect dupa apasare); test cu un fir care verifica oprirea
- [ ] F-A6: „Linii" si „Difera in" calculate pentru toata lista pe fir;
      sortarea pe ele e corecta peste randul 60; test
- [ ] F-A8: „Recompara" pastreaza „in plan", filtrele, sortarea si selectia;
      o recomparare esuata se spune pe ecranul listei; test
- [ ] F-A12: eroarea de comparare are camp propriu, nu e pusa pe referinta
- [ ] F-A15: o sesiune corupta e pastrata ca `.corupt` si se spune
- [ ] F-A16: starea de la conexiunea anterioara se goleste la conectare noua
- [ ] F-A17: „X din N nerevizuite" numara nerevizuitele; coloana Baza ramane
      cand sunt 2+ baze bifate

Pachetul 3, `main.rs`:

- [ ] mutarea (commit separat, fara logica) a starii si a deciziilor din
      `main.rs` intr-un modul testat; in `main.rs` raman legaturile Slint
- [ ] `main.rs` sub 1.200 de linii; codul mutat are teste

Pachetul 4, functii noi:

- [ ] F-A9: un obiect doar in Martor sau doar in Modificata se deschide in
      editor (tot codul ca adaugat / sters), cu test; accesibil si din `cli`
- [ ] numele se imperecheaza fara diferenta de majuscule intre baze
      (`dbo.Proc` = `DBO.proc`), cu caz in `referinta/` sau in
      `DIVERGENTE_DELIBERATE` daca v3.4 face altfel

Comune:

- [ ] fiecare binar atins, probat rulandu-l (`gmotion`, `gmotion-ui
      --previzualizare` sub ecran virtual, daca exista)
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat (F-D7 neprobat pe SQL Server; numele
      case-insensitive)

## Pasi

1. Pachetul 1 (`module/diff-obiecte/src/deploy.rs`, `plan.rs`).
2. Pachetul 3: intai mutarea, apoi teste — ca F-A-urile sa se poata testa.
3. Pachetul 2, pe codul mutat.
4. Pachetul 4.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
