# Etapa 9 — lista de diferente si previzualizarea

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Ecranul principal din `cercetare/machete/Main.dc.html`: lista obiectelor care
difera, cu bara laterala de filtre si previzualizarea obiectului selectat.
La final se poate parcurge o comparare intreaga fara linie de comanda — se
vede ce difera, in ce anume difera, si cum arata diferenta — pe o lista de
50.000 de obiecte, fara sa se blocheze.

## Criterii de terminare

- [x] **„Difera in" vine din motor, nu din interfata:** clasificarea
      diferentelor unui obiect pe `cod` / `sir` / `comentariu` / `setari SET`,
      facuta prin `core::lexer` (granitele se decid doar acolo), cu teste pe
      cazurile care au pacalit v3.4 (majuscula intr-un sir, comentariu
      schimbat, `QUOTED_IDENTIFIER`)
- [x] clasificarea e accesibila si din `cli` (CLAUDE.md §3), cu test pe binar
- [x] modelul listei in Rust, fara fereastra, cu teste: filtrare pe categorie
      si tip, cautare, sortare pe linii, contoarele din bara laterala,
      progresul revizuirii, selectia
- [x] ecranul in Slint, fidel machetei: antet (sursa → baza, mediul, cautare,
      Recompara, Deploy cu numarul din plan), bara laterala de 200 px,
      tabelul cu litera de stare / obiect / tip / `+N −M` / „Difera in" / bifa,
      randul selectat plin `selectie`, previzualizarea dedesubt
- [x] fara comutatoare de zgomot, nicaieri (decizia lui Mihai)
- [x] jetoanele noi (daca apar) trec testul de contrast ≥ 4,5:1
- [x] **lista virtualizata, masurata pe 50.000 de obiecte**: incarcare, cadru,
      derulare, filtrare — si **verificarea vazuta cazand** (fara
      virtualizare, cifrele trebuie sa sara)
- [x] captura PNG langa macheta, in `cercetare/machete/capturi/`
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Clasificarea „difera in" in `core` (prin `lexer`), cu teste; expusa in
   `diff-obiecte` si in `cli`.
2. Modelul listei (filtre, contoare, selectie) in `fereastra`, cu teste.
3. Ecranul in Slint, pe jetoanele existente.
4. Previzualizarea: aceeasi constructie de randuri ca editorul (un singur
   motor), cu identicul pliat.
5. Masuratoarea pe 50.000 de obiecte si capturile.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa, cu criteriile scrise inainte de cod. Vine
  dupa etapa 8 (editorul), de unde se refolosesc: jetoanele, fonturile,
  plierea, constructia randurilor de diferente.

- **2026-11-17** Etapa incheiata. „Difera in" sta in `core::clasificare`
  (prin `lexer`), cu propozitia pentru previzualizare acolo unde o vad si
  `cli`, si lista. Detaliile se calculeaza la cerere. Ecranul listei si
  previzualizarea refolosesc randurile editorului.

  **Masurat** (`--example masura_lista`, Linux, randare software, release,
  fara filtru — toate obiectele in lista):

  | obiecte | model | filtru | cautare | incarca | cadru 1 | cadru la capat |
  |---|---|---|---|---|---|---|
  | 1.000 | 1,2 ms | 0,1 ms | 0,0 ms | 0,4 ms | 8,6 ms | 3,6 ms |
  | 10.000 | 15 ms | 2,1 ms | 0,2 ms | 3,7 ms | 4,7 ms | 3,5 ms |
  | 50.000 | 84 ms | 13 ms | 3,1 ms | 20 ms | 7,9 ms | 3,8 ms |

  **Verificarea a fost vazuta cazand:** cu `ListView`-ul inlocuit de
  `Flickable` + `VerticalLayout`, cazul de 50.000 sare la **6.720 ms** primul
  cadru si **217 ms** pe cadru derulat.


## Ce a iesit altfel decat planul

1. **Macheta cerea ceva ce motorul nu poate arata.** Randul „difera doar in
   comentariu" din `Main.dc.html` nu e accesibil: normalizarea sterge
   comentariile, deci obiectul iese identic. Intrebat, Mihai a decis sa ramana
   asa. Lectie: o macheta poate contine, fara sa vrea, o schimbare de motor —
   se vede abia cand incerci sa o implementezi cu datele adevarate.

2. **„Un singur caracter" din macheta nu inseamna ce pare.** Propozitia
   „un singur caracter, intr-un sir: Nu → nu" numara caracterele schimbate, dar
   arata cuvantul din jur. Prima implementare a incercat sa arate chiar
   caracterul (`N → n`) si suna absurd.

3. **Fereastra nu a avut nevoie de modul.** Modelul listei primeste randuri
   gata clasificate, deci `diff-obiecte` a ramas doar dependinta de dezvoltare,
   in exemplele de captura. Asa a ramas si testabil fara baza de date.

4. **Doua mutari de fisiere au fost necesare inainte de ecran** (jetoanele si
   piesele comune, apoi randurile de diff), fiecare in commit propriu, cu
   dovada ca nu s-a schimbat nimic: aceeasi captura, acelasi PNG octet cu
   octet.

5. **Detaliile la cerere au schimbat si `cli`:** `compara --detalii`. Altfel
   ar fi existat un calcul pe care il face doar interfata, adica exact ce
   interzice CLAUDE.md §3.
