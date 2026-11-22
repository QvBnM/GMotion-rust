# Etapa 25 — ce a ramas din portare: progres, Ultima modificare, Despre, fisiere

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** —

## Scop

Ultimele lipsuri fata de v3.4 din `cercetare/lipsuri-portare.md`, verificate
pe cod (multe randuri de acolo erau deja facute): la final, lista are progres
si Anuleaza la Recompara, coloana „Ultima modificare", aplicatia are ecranul
„Despre" cerut de licenta Slint, iar instantaneul si folderul de deploy se aleg
cu dialogul nativ.

Alegerile lui Mihai (2026-11-22): dialogul nativ cu `rfd`, da; certificatul
ramane nevalidat implicit; poarta ramane pe Git Bash pe Windows.

## Criterii de terminare

- [ ] `lipsuri-portare.md` refacut pe cod: randurile facute sterse, ce ramane
      e adevarat
- [ ] Recompara din lista: bara de progres, textul pasului si Anuleaza pe
      ecranul listei, cat ruleaza; lista veche ramane; test pe model
- [ ] „Ultima modificare": citita din `sys.objects.modify_date` (server) si din
      instantaneu (camp optional, instantaneele vechi merg mai departe);
      coloana in lista, sortabila; in `gmotion compara --json` si in CSV;
      teste pe fixture
- [ ] „Despre": ecran din bara listei, cu versiunea, licentele (Slint prin
      `AboutSlint`, fonturile IBM Plex OFL) si calea jurnalului; textele in
      ambele limbi; test pe model
- [ ] dialogul nativ (`rfd`, intrare in `JURNAL.md`): „Deschide un
      instantaneu" si „Alege folderul" la deploy; dupa scriere, „Deschide
      folderul"; lista veche de fisiere gasite ramane ca alternativa;
      `cargo check` pe windows-msvc
- [ ] evidentierea de sintaxa in timp ce scrii: proba de fezabilitate in Slint;
      daca se poate, facuta, cu test; daca nu, decizia in `JURNAL.md`
- [ ] capturi la 1600x1000 pentru ecranele atinse (lista in recomparare,
      coloana noua, „Despre"), pentru „da"-ul lui Mihai
- [ ] binarele probate rulandu-le (`gmotion`, `gmotion-ui` sub Xvfb)
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat

## Pasi

1. `lipsuri-portare.md`.
2. Progresul la Recompara.
3. „Ultima modificare" (`db`, instantaneu, modul, lista, cli, CSV).
4. „Despre".
5. `rfd`.
6. Evidentierea in timp ce scrii.
7. Capturi, stare, jurnal.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
