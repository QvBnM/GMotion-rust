# Etapa 26 — verificarea textului final: local si pe server

**Stare:** INCHEIATA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** 2026-11-22

## Scop

Mihai: „daca punem ceva gresit, sa ne zica ca procedura da eroare". La final,
un bloc aplicat sau pus in alt loc care strica structura procedurii se vede
pe loc, in editor si in planul de deploy, fara server; iar un buton trimite
textul final la SQL Server, fara sa creeze nimic, si arata ce spune serverul.

Ales de Mihai (2026-11-22): ambele niveluri.

## Criterii de terminare

- [x] `core::structura::verifica(text)`: `BEGIN`/`END` (si `TRY`/`CATCH`,
      `CASE … END`) echilibrate, paranteze, siruri / identificatori /
      comentarii neinchise, `GO` in interior; fiecare problema cu linia ei;
      fara I/O (merge pe wasm); teste, inclusiv cazuri care **nu** sunt
      probleme (`BEGIN TRAN`, `END CONVERSATION`, cuvinte in siruri si
      comentarii)
- [x] editorul arata problemele textului final (dupa Aplica, plasare,
      editare de mana), cu linia, in ambele limbi; un text final fara
      probleme nu arata nimic
- [x] deploy-ul (fereastra si `gmotion deploy`) avertizeaza pentru obiectele
      cu probleme de structura, in raportul scrierii
- [x] din `cli`: `gmotion editare` spune mereu problemele textului final (si
      in `--json`); cu `--pe-server`, si verificarea pe server (nu o comanda
      noua: `editare` are deja textul final si sesiunea)
- [x] `db`: verificarea pe server fara efecte (`SET PARSEONLY ON`, apoi
      `SET NOEXEC ON`), cu setarile obiectului (`QUOTED_IDENTIFIER`) si in
      forma care se deployeaza (`CREATE OR ALTER`); mesajul si linia
      serverului; refuzata explicit pe instantaneu; teste pe ce se trimite
- [x] butonul „Verifica pe server" in editor, pe alt fir; rezultatul in
      editor; test pe model
- [x] binarele probate rulandu-le (testele pe binar si capturile din
      aplicatie); captura editorului cu o problema in
      `cercetare/machete/etapa-26/`; verificarea pe server **nu** e probata
      pe SQL Server real (nu exista pe masina de dezvoltare) — ceruta in
      `STARE.md`
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat (proba pe SQL Server real ceruta)
- [x] `JURNAL.md` completat

## Pasi

1. `core::structura`.
2. Editorul si planul.
3. `cli`.
4. `db` si butonul.
5. Capturi, stare, jurnal.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-22** Facute ambele niveluri. Verificarea locala, rulata pe toate
  obiectele reale din `testsimilitudine.json`: zero probleme (acolo e testul
  contra falselor pozitive). Pe server, garda „`SELECT 1` nu intoarce
  nimic" inainte de a trimite codul: daca `PARSEONLY` / `NOEXEC` n-au prins,
  codul nu pleaca.

## Ce a iesit altfel decat planul

- Comanda `gmotion verifica` din plan n-a mai fost nevoie: `editare` avea
  deja textul final, sesiunea si iesirea JSON.
- Avertismentul din deploy vine in raportul scrierii, nu in ecranul
  planului: textele finale ajung in `StareDeploy` abia la scriere.
- Partea pe server e scrisa si testata doar pana la granita: ce face
  SQL Server cu `NOEXEC` pe un `CREATE OR ALTER PROCEDURE` (cat compileaza)
  se afla doar pe un server real.
