# Etapa 26 — verificarea textului final: local si pe server

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** —

## Scop

Mihai: „daca punem ceva gresit, sa ne zica ca procedura da eroare". La final,
un bloc aplicat sau pus in alt loc care strica structura procedurii se vede
pe loc, in editor si in planul de deploy, fara server; iar un buton trimite
textul final la SQL Server, fara sa creeze nimic, si arata ce spune serverul.

Ales de Mihai (2026-11-22): ambele niveluri.

## Criterii de terminare

- [ ] `core::structura::verifica(text)`: `BEGIN`/`END` (si `TRY`/`CATCH`,
      `CASE … END`) echilibrate, paranteze, siruri / identificatori /
      comentarii neinchise, `GO` in interior; fiecare problema cu linia ei;
      fara I/O (merge pe wasm); teste, inclusiv cazuri care **nu** sunt
      probleme (`BEGIN TRAN`, `END CONVERSATION`, cuvinte in siruri si
      comentarii)
- [ ] editorul arata problemele textului final (dupa Aplica, plasare,
      editare de mana), cu linia, in ambele limbi; un text final fara
      probleme nu arata nimic
- [ ] planul de deploy (fereastra si `gmotion plan` / `deploy`) avertizeaza
      pentru obiectele cu probleme de structura
- [ ] `gmotion verifica --referinta --baza --obiect`: problemele locale ale
      textului final; cu `--pe-server`, si verificarea pe server
- [ ] `db`: verificarea pe server fara efecte (`SET PARSEONLY ON`, apoi
      `SET NOEXEC ON`), cu setarile obiectului (`QUOTED_IDENTIFIER`) si in
      forma care se deployeaza (`CREATE OR ALTER`); mesajul si linia
      serverului; refuzata explicit pe instantaneu; teste pe ce se trimite
- [ ] butonul „Verifica pe server" in editor, pe alt fir; rezultatul in
      editor; test pe model
- [ ] binarele probate rulandu-le; captura editorului cu o problema
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat (proba pe SQL Server real ceruta)
- [ ] `JURNAL.md` completat

## Pasi

1. `core::structura`.
2. Editorul si planul.
3. `cli`.
4. `db` si butonul.
5. Capturi, stare, jurnal.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
