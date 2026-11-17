# Etapa 11 — ecranul de conectare

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

## Scop

`gmotion-ui` nu mai primeste sursa in linia de comanda: se porneste si se alege
din fereastra ce se compara cu ce, dupa macheta
`cercetare/machete/Conectare.dc.html`. Compararile recente se reiau dintr-un
clic.

## Criterii de terminare

- [ ] ecranul in Slint, fidel machetei: doua coloane Sursa / Baza (server,
      autentificare Windows / SQL, utilizator, parola, baza, starea
      conexiunii), butonul de inversare, certificatul intr-o linie cu
      „Schimbă", butonul Compară
- [ ] **parola nu ajunge niciodata intr-o sesiune, intr-un jurnal sau intr-un
      mesaj de eroare**; tipul care o tine nu are `Debug` derivat (CLAUDE.md §6)
- [ ] starea conexiunii se testeaza fara sa compari: „Se citeste, nu se scrie
      nimic", si lista bazelor se umple din server
- [ ] compararile recente, reluabile: din sesiunile salvate in `platforma`,
      cu data si numele bazelor
- [ ] modelul ecranului (ce e completat, ce lipseste, ce se poate apasa) are
      teste fara fereastra
- [ ] `gmotion-ui` porneste fara argumente si duce in lista dupa o comparare
- [ ] captura PNG langa machetă
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Modelul ecranului de conectare (campuri, validari, stari), cu teste.
2. Ecranul in Slint, pe jetoanele existente.
3. Legarea de `db` (test de conexiune, lista bazelor) si de sesiuni.
4. Pornirea aplicatiei din ecranul de conectare; captura.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod.

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
