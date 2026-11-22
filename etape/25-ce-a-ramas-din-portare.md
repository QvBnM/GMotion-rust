# Etapa 25 — ce a ramas din portare: progres, Ultima modificare, Despre, fisiere

**Stare:** INCHEIATA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** 2026-11-22

## Scop

Ultimele lipsuri fata de v3.4 din `cercetare/lipsuri-portare.md`, verificate
pe cod (multe randuri de acolo erau deja facute): la final, lista are progres
si Anuleaza la Recompara, coloana „Ultima modificare", aplicatia are ecranul
„Despre" cerut de licenta Slint, iar instantaneul si folderul de deploy se aleg
cu dialogul nativ.

Alegerile lui Mihai (2026-11-22): dialogul nativ cu `rfd`, da; certificatul
ramane nevalidat implicit; poarta ramane pe Git Bash pe Windows.

## Criterii de terminare

- [x] `lipsuri-portare.md` refacut pe cod: randurile facute sterse, ce ramane
      e adevarat
- [x] Recompara din lista: bara de progres, textul pasului si Anuleaza pe
      ecranul listei, cat ruleaza; lista veche ramane; test pe model
- [x] „Ultima modificare": citita din `sys.objects.modify_date` (server) si din
      instantaneu (camp optional, instantaneele vechi merg mai departe);
      coloana in lista, sortabila; in `gmotion compara --json` si in CSV;
      teste pe fixture
- [x] „Despre": deschis din Ajutor (F1), pe conectare si pe lista (nu din
      bara: antetele raman ca in macheta), cu versiunea, licentele (Slint prin
      `AboutSlint`, fonturile Geist OFL, tiberius) si caile jurnalului si
      datelor; textele in ambele limbi; test pe model si cu mouse-ul
- [x] dialogul nativ (`rfd`, intrare in `JURNAL.md`): „Deschide un
      instantaneu" si „Alege folderul" la deploy; dupa scriere, „Deschide
      folderul"; lista veche de fisiere gasite ramane ca alternativa;
      `cargo check` pe windows-msvc
- [x] evidentierea de sintaxa in timp ce scrii: se poate (randuri colorate
      peste campul de editare); facuta, cu teste si captura; decizia si
      masuratorile in `JURNAL.md`
- [x] capturi la 1600x1000 pentru ecranele atinse (lista in recomparare,
      coloana noua, „Despre", editarea), in `cercetare/machete/etapa-25/`;
      „da"-ul lui Mihai e cerut in `STARE.md` (nu exista macheta pentru ele)
- [x] binarele probate rulandu-le: `gmotion-ui --previzualizare` sub Xvfb,
      capturile din aplicatie; dialogul nativ ramane de probat pe Windows
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat

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
- **2026-11-22** Facute toate. Pe drum: „Despre" pus intai in antetul
  conectarii a mutat „Setari" si a rupt un test pe coordonate; s-a mutat in
  Ajutor. Titlul si butonul Ajutorului erau scrise direct in `.slint`
  (regula 6), acum prin `Texte`. Suprapunerea colorata a alunecat de doua
  ori (inaltimea liniei rotunjita, latimile bucatilor rotunjite) pana sa
  se masoare in Slint. `target` a umplut iar discul: setarile pentru disc
  mic sunt acum in `~/.cargo/config.toml` al containerului.

## Ce a iesit altfel decat planul

- Inventarul din `lipsuri-portare.md` era mult mai mare pe hartie decat in
  cod: jumatate din randuri erau deja facute. Recitirea pe cod a scurtat
  etapa inainte sa inceapa.
- Evidentierea la scriere, trecuta ca „de probat daca se poate", s-a putut,
  dar numai cu masuratori (pas fractionar, tab fara opriri); o suprapunere
  „dupa ochi" aluneca pe fiecare rand.
- „Ultima modificare" a atins 34 de locuri care construiau `Obiect` de mana;
  campul e optional in instantaneu, deci instantaneele vechi merg.
