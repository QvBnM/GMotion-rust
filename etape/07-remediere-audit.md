# Etapa 7 — remedierea constatarilor din audit

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Constatarile critice si inalte din auditul de la `f561bf9` sunt reparate, cu
teste care cad pe codul vechi. Unde reparatia face v4 sa difere intentionat de
v3.4 / motorul JS, diferenta e listata explicit si motivata, nu ascunsa prin
regenerarea fixture-ului.

Mihai a cerut (2026-11-17): „ocupa-te tu de ce trebuie facut". GUI-ul ramane
la final. Planul din README se renumeroteaza: 7 remediere, 8 `upgrade`,
9-10 interfata.

## Criterii de terminare

Comparatorul (critic: modificari reale raportate „identic"):

- [x] decizia „identic" nu mai foloseste scorul rotunjit (un `NOT` sters
      dintr-o procedura de 10.000+ token-uri iese `diferit`)
- [x] normalizarea stie de string-uri, identificatori in paranteze si
      comentarii imbricate: `--`/`/*` din string nu mai sterg cod; continutul
      literalelor nu se mai modifica (majuscule, spatii, `[0-9]`, `dbo.`)
- [x] `[a b]` ≠ `a b`, `Sales.dbo.Orders` ≠ `Sales.Orders`; cosmeticele reale
      (spatii, majuscule in cod, comentarii, `[simplu]`, `dbo.`) raman identice
- [x] cheia de bloc are cel putin 64 de biti si „egal" cere text normalizat
      egal, nu doar cheie egala
- [x] fiecare divergenta deliberata fata de JS / v3.4 e listata in testul de
      conformitate si in asteptarile de paritate, cu motivul in `JURNAL.md`

Deploy-ul (critic: script injectabil sau stricat):

- [x] `USE [..]` cu `]` dublat; nume cu caractere de control refuzate;
      comentariile din script nu pot contine rand nou
- [x] o linie `GO` in definitie => obiectul e refuzat cu eroare clara
- [x] `CREATE OR ALTER` ignora comentariile si string-urile; fara panica pe
      `K`/`İ` <!-- diacritice-permise -->
- [x] se deployeaza definitia originala, octet cu octet, nu textul de afisare
- [x] fara coliziuni de fisiere; director de iesire ne-gol refuzat; nume
      rezervate Windows ocolite
- [x] obiectele criptate nu apar „lipsa" si nu se deployeaza peste

Conexiunea:

- [x] timeout pe login, TLS si interogari; conexiunea stricata nu se refoloseste
- [x] rezolvarea instantei prin SQL Browser e cod propriu: fara panica pe
      raspuns scurt, raspunsul trebuie sa vina de la gazda tinta
- [x] validarea certificatului e posibila (optiune CLI `--certificat`);
      implicitul il decide Mihai (ramas cel din v3.4, cu avertisment)
- [x] `Autentificare` are `Debug` scris de mana, cu redactare
- [x] nume de baze cu spatii/diacritice acceptate; erorile de interogare dau
      `GM-DB-004`, baza inaccesibila `GM-DB-005`; `try_get` in loc de `get`

Restul:

- [x] BOM in configurare si in fisierul de parola; `\r` singur la final
- [x] CSV: celulele care incep cu `= + - @` nu mai sunt formule
- [x] fisierul de sesiune e unic per nume exact si verificat la incarcare
- [x] scrieri atomice: configurare, sesiune, instantaneu
- [x] jurnalul din memorie pastreaza campurile; caracterele de control se
      neutralizeaza
- [x] descoperirea: tampon de 64 KB, o eroare de citire nu opreste cautarea
- [x] cursorul cu nume urias nu mai da panica
- [x] `--baze A,A` fara dubluri; `--parola-stdin` citeste o linie
- [x] `Cargo.lock` in git
- [x] `etapa6.ps1`: parola se cere dupa compilare
- [x] fiecare reparatie are un test vazut cazand pe codul vechi
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat

In afara etapei, documentat ca deschis: performanta patratica pe obiecte
uriase (doar optimizari ieftine aici), `SET ANSI_NULLS/QUOTED_IDENTIFIER` per
obiect, ordinea dependentelor, verificarea versiunii serverului, nume
case-insensitive intre baze, divergentele Unicode fata de JS (in afara de BOM),
repo-ul public (decizia lui Mihai).

## Pasi

1. `Cargo.lock`, `etapa6.ps1` — mici, independente.
2. Comparatorul: lexer T-SQL comun in `core`, normalizare peste el, decizia
   „identic", cheia de bloc. Conformitate si paritate cu divergente listate.
3. Deploy: acelasi lexer pentru `CREATE OR ALTER` si `GO`; escape; definitia
   bruta; fisiere.
4. `db`: timeout-uri, SQL Browser propriu, certificat, `Debug`, nume, coduri,
   obiecte criptate, scrieri atomice.
5. `platforma`: BOM, sesiuni, jurnal, scrieri atomice, cai relative.
6. `cli`: parola, `--baze`, CSV.

## Jurnal de lucru

- **2026-11-17** Etapa deschisa dupa audit (4 auditori independenti pe
  `f561bf9`, constatarile critice reproduse de mine). Cele trei probleme mari
  sunt mostenite din v3.4.

- **2026-11-17** Comparatorul: lexer T-SQL in `core`, normalizarea doar pe cod;
  „identic" = hash egal; cheie de bloc pe 128 de biti; `SequenceMatcher` pe
  vectori densi (9,7 s -> 0,6 s, identic cu `difflib` pe 3000 de cazuri);
  fara panica la cursor si la limite. Conformitate JS: 4 divergente listate.
  Fixture-ul real: 4 scoruri schimbate, verificate manual, in
  `testsimilitudine.asteptari.json`.
- **2026-11-17** Deploy: escape, nume cu control refuzate, `GO` refuzat,
  `CREATE OR ALTER` pe lexer, definitia bruta, fisiere unice, director gol,
  criptate sarite. Reproducerile auditorului (`s1`-`s4`) refuzate cu cod clar;
  pe `s4`, cele 3 modificari ascunse ies acum „diferit".
- **2026-11-17** Conexiune: timeout-uri pe tot, SQL Browser propriu, optiunea
  de certificat, `Debug` redactat, nume de baze, coduri, `try_get`, criptate,
  descoperire, instantanee validate si scrise atomic.
- **2026-11-17** Platforma si CLI: sesiuni pe hash, jurnal cu campuri si
  curatat, BOM, scrieri atomice, cai relative; parola, `--baze`, CSV.
- **2026-11-17** Vazut cazand pe codul vechi: motor (normalizare, cursor,
  limite), deploy (8 teste + definitia bruta pe binar), platforma (sesiuni,
  BOM, validare), descoperire, CSV, parola. Testele de retea (`rezolva_port_
  instanta`, timeout) testeaza cod nou; regresia pe codul vechi a fost
  reprodusa de auditor (panica, raspuns strain, blocare 45 s+). `lista` din
  CLI: verificata prin rationament (codul vechi nu se compila izolat).

- **2026-11-17, dupa incheiere** A doua rulare pe Windows: 0 regresii, o
  modificare ascunsa de v3.4 gasita pe date reale
  (`GenerateOrderByRecommendedOrder`). La cererea lui Mihai s-au facut si cele
  trei puncte lasate deschise: setarile de creare per obiect in deploy, ordinea
  dupa dependente, garda de versiune (`CREATE OR ALTER` cere 13.0.4001+), plus
  citirea paralela (v4 era mai lent decat v3.4 pe o retea cu latenta mare).

## Ce a iesit altfel decat planul

**Paritatea perfecta cu v3.4 nu mai e tinta, si asta a cerut unelte noi.**
Etapa 6 a dovedit ca portul reproduce v3.4 — inclusiv bug-urile. Reparatia
motorului face v4 sa difere deliberat. Fara un mecanism explicit, „paritate"
ar fi devenit fie imposibila, fie o lista de exceptii ascunse. `paritate.py`
separa acum trei lucruri: diferente asteptate explicit (cu motiv, verificate
in ambele sensuri), „de verificat" (obiecte cu constructii tratate altfel) si
regresii. Pe bazele reale ale lui Mihai numarul de obiecte „diferite" va
creste: acelea sunt exact modificarile pe care v3.4 le ascundea.

**Cea mai buna reparatie a fost una de structura, nu de regula.** Trei
constatari critice (comentarii in string-uri, `CREATE` din comentarii, `GO`)
aveau aceeasi cauza: reguli pe text fara granite. Un singur lexer mic le-a
inchis pe toate trei, si e folosit de motor si de deploy — un singur loc unde
se decide ce e cod.

**O dependenta in minus.** `md-5` a disparut din `core` odata cu cheia pe 32
de biti; cheia foloseste acum SHA-256-ul care exista deja.

**Ce n-am putut verifica aici:** comportamentul pe Windows al socket-ului UDP
conectat, `--certificat sistem` pe un server real, si efectul normalizarii noi
pe bazele lui Mihai. De rulat `unelte/etapa6.ps1` din nou.
