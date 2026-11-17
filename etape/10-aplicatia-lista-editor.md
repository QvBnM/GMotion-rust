# Etapa 10 — aplicatia: din lista in editor si inapoi

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Cele doua ecrane existente devin o aplicatie: din lista se deschide editorul pe
tot ecranul (`cercetare/machete/Editor.dc.html`), se trece de la un obiect la
altul fara sa te intorci, si ce ai marcat ca revizuit se tine minte intre
sesiuni — si se pierde singur cand obiectul s-a schimbat pe server.

## Criterii de terminare

- [x] **starea de revizuire e legata de amprenta continutului:** un obiect
      marcat revizuit nu mai e revizuit daca definitia lui (in sursa sau in
      baza) s-a schimbat. Sta in `diff-obiecte` (modulul isi tine starea), se
      salveaza prin sesiunile din `platforma`, cu teste pe amandoua cazurile
- [x] **accesibila din `cli`** (CLAUDE.md §3): marcare, listare si golire,
      cu test pe binarul adevarat
- [x] `rust/app`: crate nou, binar cu fereastra, care leaga ecranele —
      `Deschide` / `Enter` duce in editor, `Esc` / „← Listă" inapoi, `j` / `k`
      trec la obiectul urmator / anterior **fara** sa se intoarca in lista
- [x] rutarea e testabila fara fereastra: ce ecran e activ, ce obiect e
      deschis, ce se intampla la fiecare comanda — teste fara Slint
- [x] starea de revizuire marcata in editor se vede imediat in lista (bifa,
      progresul din bara laterala) si invers
- [x] captura PNG a editorului deschis dintr-un obiect al listei, langa
      machete
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Starea de revizuire in `diff-obiecte`, cu amprenta si teste.
2. Salvarea ei intr-o sesiune (`platforma`), si comanda din `cli`.
3. `rust/app`: rutarea intre ecrane, testata fara fereastra.
4. Binarul cu fereastra si captura.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod. Sesiunile
  exista deja in `platforma` (`sesiune.rs`, cu `date` opac pentru modul), deci
  starea de revizuire e continutul acelui `date`, nu un mecanism nou.
- **2026-11-17** Etapa incheiata. `Revizuire` in `diff-obiecte` (amprenta
  celor doua definitii), comanda `gmotion revizuire` in `cli` cu test pe
  binar, crate-ul `rust/app` cu rutarea testata fara fereastra (8 teste) si
  binarul `gmotion-ui`. Captura drumului adevarat (lista -> `Deschide` ->
  editor): `cercetare/machete/capturi/Editor-deschis-din-lista.png`, facuta de
  `--example captura_flux`, care trece chiar prin comenzile aplicatiei.

## Ce a ieșit altfel decat planul

1. **Poarta a prins o regula de arhitectura lipsa.** `app -> fereastra` iesea
   „dependinta laterala", fiindca tabelul de niveluri le punea pe amandoua pe
   4. Regula s-a schimbat in acelasi commit (CLAUDE.md, ARHITECTURA.md,
   `unelte/verifica.sh`): lantul e acum
   `core <- db <- platforma <- module/* <- fereastra, cli <- app`. Exact
   cazul pentru care CLAUDE.md spune ca regula castiga, dar se poate schimba.

2. **Verificarea amprentei a fost vazuta cazand.** Cu comparatia de amprente
   scoasa, testul pe binar (`revizuirea_se_tine_minte_si_cade_cand_obiectul_se_schimba`)
   cade imediat: bifa veche ar fi ramas valabila pe un obiect schimbat.

3. **`j` / `k` din editor trebuie sa sara peste obiectele care nu se pot
   deschide** (cele care exista doar intr-o parte). Fara asta, editorul s-ar
   deschide gol, sau aplicatia ar parea blocata pe ultimul obiect.

4. **Binarul nu are inca de unde sa stie la ce se conecteaza:** pana la
   ecranul de conectare (etapa 11), `gmotion-ui` primeste instantaneul in
   linia de comanda. Argumentele lui sunt minime deliberat — nu are rost sa
   crestem o interfata de linie de comanda care dispare.
