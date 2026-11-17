# Etapa 7 — remedierea constatarilor din audit

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** —

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

- [ ] decizia „identic" nu mai foloseste scorul rotunjit (un `NOT` sters
      dintr-o procedura de 10.000+ token-uri iese `diferit`)
- [ ] normalizarea stie de string-uri, identificatori in paranteze si
      comentarii imbricate: `--`/`/*` din string nu mai sterg cod; continutul
      literalelor nu se mai modifica (majuscule, spatii, `[0-9]`, `dbo.`)
- [ ] `[a b]` ≠ `a b`, `Sales.dbo.Orders` ≠ `Sales.Orders`; cosmeticele reale
      (spatii, majuscule in cod, comentarii, `[simplu]`, `dbo.`) raman identice
- [ ] cheia de bloc are cel putin 64 de biti si „egal" cere text normalizat
      egal, nu doar cheie egala
- [ ] fiecare divergenta deliberata fata de JS / v3.4 e listata in testul de
      conformitate si in asteptarile de paritate, cu motivul in `JURNAL.md`

Deploy-ul (critic: script injectabil sau stricat):

- [ ] `USE [..]` cu `]` dublat; nume cu caractere de control refuzate;
      comentariile din script nu pot contine rand nou
- [ ] o linie `GO` in definitie => obiectul e refuzat cu eroare clara
- [ ] `CREATE OR ALTER` ignora comentariile si string-urile; fara panica pe
      `K`/`İ`
- [ ] se deployeaza definitia originala, octet cu octet, nu textul de afisare
- [ ] fara coliziuni de fisiere; director de iesire ne-gol refuzat; nume
      rezervate Windows ocolite
- [ ] obiectele criptate nu apar „lipsa" si nu se deployeaza peste

Conexiunea:

- [ ] timeout pe login, TLS si interogari; conexiunea stricata nu se refoloseste
- [ ] rezolvarea instantei prin SQL Browser e cod propriu: fara panica pe
      raspuns scurt, raspunsul trebuie sa vina de la gazda tinta
- [ ] validarea certificatului e posibila (setare + optiune CLI); implicitul
      il decide Mihai
- [ ] `Autentificare` are `Debug` scris de mana, cu redactare
- [ ] nume de baze cu spatii/diacritice acceptate; erorile de interogare dau
      `GM-DB-004`, baza inaccesibila `GM-DB-005`; `try_get` in loc de `get`

Restul:

- [ ] BOM in configurare si in fisierul de parola; `\r` singur la final
- [ ] CSV: celulele care incep cu `= + - @` nu mai sunt formule
- [ ] fisierul de sesiune e unic per nume exact si verificat la incarcare
- [ ] scrieri atomice: configurare, sesiune, instantaneu
- [ ] jurnalul din memorie pastreaza campurile; caracterele de control se
      neutralizeaza
- [ ] descoperirea: tampon de 64 KB, o eroare de citire nu opreste cautarea
- [ ] cursorul cu nume urias nu mai da panica
- [ ] `--baze A,A` fara dubluri; `--parola-stdin` citeste o linie
- [ ] `Cargo.lock` in git
- [ ] `etapa6.ps1`: parola se cere dupa compilare
- [ ] fiecare reparatie are un test vazut cazand pe codul vechi
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat

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

## Ce a ieșit altfel decat planul

(se completeaza la incheiere)
