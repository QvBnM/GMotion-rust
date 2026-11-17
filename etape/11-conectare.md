# Etapa 11 — ecranul de conectare

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

`gmotion-ui` nu mai primeste sursa in linia de comanda: se porneste si se alege
din fereastra ce se compara cu ce, dupa macheta
`cercetare/machete/Conectare.dc.html`. Compararile recente se reiau dintr-un
clic.

## Criterii de terminare

- [x] ecranul in Slint, fidel machetei: doua coloane Sursa / Baza (server,
      autentificare Windows / SQL, utilizator, parola, baza, starea
      conexiunii), butonul de inversare, certificatul intr-o linie cu
      „Schimbă", butonul Compară
- [x] **parola nu ajunge niciodata intr-o sesiune, intr-un jurnal sau intr-un
      mesaj de eroare**; tipul care o tine nu are `Debug` derivat (CLAUDE.md §6)
- [x] starea conexiunii se testeaza fara sa compari: „Se citeste, nu se scrie
      nimic", si lista bazelor se umple din server
- [x] compararile recente, reluabile: din sesiunile salvate in `platforma`,
      cu data si numele bazelor
- [x] modelul ecranului (ce e completat, ce lipseste, ce se poate apasa) are
      teste fara fereastra
- [x] `gmotion-ui` porneste fara argumente si duce in lista dupa o comparare
- [x] captura PNG langa machetă
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Modelul ecranului de conectare (campuri, validari, stari), cu teste.
2. Ecranul in Slint, pe jetoanele existente.
3. Legarea de `db` (test de conexiune, lista bazelor) si de sesiuni.
4. Pornirea aplicatiei din ecranul de conectare; captura.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-17** Etapa incheiata. `model_conectare.rs` (7 teste), ecranul in
  Slint, `pornire.rs` in `app` (5 teste: despartirea portului, citirea
  bazelor, parola care nu apare in eroare), si drumul intreg in
  `app/tests/flux.rs`. Captura: `cercetare/machete/capturi/Conectare-nativ.png`.

## Ce a ieșit altfel decat planul

1. **Bazele se citesc cand le ceri, nu cand tastezi.** Prima idee era sa se
   conecteze singur dupa ce se completeaza campurile; ar fi insemnat o
   conectare la fiecare tasta. Acum lista de baze se umple la clic pe „Baza",
   si tot atunci se afla daca serverul raspunde. Starea „conectat" se scrie
   numai dupa ce s-a citit ceva de acolo — altfel ar fi o promisiune, nu un
   fapt.

2. **„Deschide un instantaneu…" nu are selector de fisiere.** Ar fi o
   dependinta noua (`rfd`), si nu se poate proba pe Linux fara ecran. Pana
   atunci, calea se da la pornire (`--sursa`), iar legatura spune asta.
   Intrebare pentru Mihai, notata in `STARE.md`.

3. **Mediul (PROD / TEST / DEV) se ghiceste din numele bazei.** Machetele il
   arata peste tot, dar nu spun de unde vine. Ghicitul e util si poate fi
   gresit — de-aia e notat ca decizie deschisa: ar trebui configurat explicit
   pe server sau pe baza.

4. **Un test de flux paralel si-a sters singur instantaneul.** Cele doua teste
   din `tests/flux.rs` foloseau acelasi director temporar. A trecut de doua
   ori, apoi a cazut: exact capcana din `STARE.md` despre starea globala intre
   teste, dar pe disc, nu in memorie.
