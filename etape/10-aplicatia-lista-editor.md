# Etapa 10 — aplicatia: din lista in editor si inapoi

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

## Scop

Cele doua ecrane existente devin o aplicatie: din lista se deschide editorul pe
tot ecranul (`cercetare/machete/Editor.dc.html`), se trece de la un obiect la
altul fara sa te intorci, si ce ai marcat ca revizuit se tine minte intre
sesiuni — si se pierde singur cand obiectul s-a schimbat pe server.

## Criterii de terminare

- [ ] **starea de revizuire e legata de amprenta continutului:** un obiect
      marcat revizuit nu mai e revizuit daca definitia lui (in sursa sau in
      baza) s-a schimbat. Sta in `diff-obiecte` (modulul isi tine starea), se
      salveaza prin sesiunile din `platforma`, cu teste pe amandoua cazurile
- [ ] **accesibila din `cli`** (CLAUDE.md §3): marcare, listare si golire,
      cu test pe binarul adevarat
- [ ] `rust/app`: crate nou, binar cu fereastra, care leaga ecranele —
      `Deschide` / `Enter` duce in editor, `Esc` / „← Listă" inapoi, `j` / `k`
      trec la obiectul urmator / anterior **fara** sa se intoarca in lista
- [ ] rutarea e testabila fara fereastra: ce ecran e activ, ce obiect e
      deschis, ce se intampla la fiecare comanda — teste fara Slint
- [ ] starea de revizuire marcata in editor se vede imediat in lista (bifa,
      progresul din bara laterala) si invers
- [ ] captura PNG a editorului deschis dintr-un obiect al listei, langa
      machete
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Starea de revizuire in `diff-obiecte`, cu amprenta si teste.
2. Salvarea ei intr-o sesiune (`platforma`), si comanda din `cli`.
3. `rust/app`: rutarea intre ecrane, testata fara fereastra.
4. Binarul cu fereastra si captura.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod. Sesiunile
  exista deja in `platforma` (`sesiune.rs`, cu `date` opac pentru modul), deci
  starea de revizuire e continutul acelui `date`, nu un mecanism nou.

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
