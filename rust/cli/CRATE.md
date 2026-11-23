# gmotion-cli

<!-- api-hash: bec346e59fe1af2c -->

## Rol

Binarul `gmotion`: tot ce face aplicatia, fara interfata grafica. Exista
**inaintea** interfetei, si ramane dupa ea ca mod de depanare permanent.

Doua lucruri trebuie sa ramana adevarate:

1. **tot ce face aplicatia se poate actiona de aici** — altfel un bug in spatele
   unui buton nu poate fi depanat decat pe masina utilizatorului;
2. **fiecare comanda care citeste obiecte merge si cu `--sursa <instantaneu>`**,
   nu doar cu `--server`. Daca apare una care cere server, asta e bug-ul.

## Nu face

- **Nu compara si nu genereaza nimic singur.** Toata logica e in
  `gmotion-modul-diff-obiecte`. CLI-ul e argumente, iesire si coduri de eroare.
- **Nu accepta parola ca argument.** Vezi invarianta 2.
- **Nu scrie nimic pe iesirea standard in afara de rezultat**, cand e `--json`.

## Comenzi

| Comanda | Ce face |
|---|---|
| `diagnostic` | ce vede GMotion: versiune, cai, configurare, ce e disponibil. Prima comanda cand ceva nu merge |
| `servere` | descoperire prin SQL Browser (UDP 1434) |
| `baze` | bazele disponibile |
| `testeaza` | testul de conexiune, ca butonul din fereastra: editia, versiunea, de cand merge, cate baze; doar pe server |
| `instantaneu` | obiectele brute intr-un fisier, pentru reluare offline |
| `compara` | lista de diferente (`--detalii`: in ce difera fiecare obiect si cate linii se misca) |
| `revizuire` | ce s-a marcat ca revizuit sau omis (`--omite`); bifa cade cand obiectul s-a schimbat; note (`--nota X --text T`, text gol = sterge); `--goleste` sterge doar bifele (textele finale, notele si marcajele de deploy raman) |
| `impact` | impactul estimat al deploy-ului fiecarui obiect care difera; cu `--obiect`, si cine il foloseste |
| `editare` | textul final al unui obiect: diferentele numerotate, `--aplica 1,3`, `--aplica-toate`, `--reset`, `--la N` (lasa diferenta pe linia N, ca tragerea), `--din-fisier` (scris de mana; un text care n-ar trece de deploy se refuza pe loc), `--arata-final` (textul final pe iesirea standard); aceeasi sesiune si acelasi `uneste_cursor` (din configurare) ca fereastra |
| `diferente` | diff-ul unui singur obiect, linie cu linie |
| `plan` | ce intra in deploy, cu amprentele de acum, intr-un `.json` |
| `deploy` | scripturile, cu codul din referinta (sau textul final din editor, daca e valabil; fara cele omise), si revenirea in `revenire/`. Cu `--plan`, intai verifica ca baza n-a mai fost atinsa (`GM-MOD-012`) |
| `export` | CSV pentru Excel |

Argumente comune: `--server`/`--port`/`--certificat` sau `--sursa`,
`--utilizator`, `--parola-fisier`/`--parola-stdin`, `--json`.
`--certificat`: `nevalidat` (implicit, ca in v3.4), `sistem`, sau calea unei
autoritati de certificare.

## Invariante

1. **Esecul da cod de iesire nenul, si mesajul merge la iesirea de erori.**
   Altfel o rulare programata ar raporta succes. Codul original
   (`GM-DB-001`, ...) se pastreaza in mesaj.
2. **Parola nu se da ca argument de linie de comanda.** Ar ajunge in istoricul
   shell-ului si ar fi vizibila in lista de procese pentru orice utilizator de
   pe masina. Caile acceptate: `GMOTION_PAROLA`, `--parola-fisier`,
   `--parola-stdin`. Exista un test care cade daca cineva adauga `--parola`.
3. **Pe `--json`, la iesirea standard iese numai JSON.** Jurnalul si progresul
   merg la erori. In mod text, progresul merge la iesirea standard.
4. **Cerintele modulului se verifica inainte de a citi ceva** — altfel s-ar citi
   degeaba, posibil minute.
5. **O sesiune care exista dar nu se poate citi opreste `deploy`, `plan` si
   `export`.** Fara ea, omisele ar ajunge in deploy si textele finale s-ar
   pierde fara semn (B1, audit 2026-11-22). Lipsa sesiunii nu e eroare.
6. **Autentificarea se rezolva o singura data pe rulare** (`ArgumenteAuth::
   rezolva`): sursa se deschide de mai multe ori, iar `--parola-stdin` se
   poate citi o singura data (B3).
7. **Marcajul de timp se da catre modul, nu se citeste din ceas acolo**, ca
   generatorul de deploy sa ramana testabil.
6. **Un instantaneu retine doar numele serverului** (`SRV\INST` sau
   `SRV,PORT`, din `ArgumenteSursa::eticheta_server`), nu descrierea pentru
   jurnal. Instantaneele ajung in repo ca fixture; o data a ajuns acolo
   lungimea parolei pe calea asta.
7. **Cheia sesiunii e perechea (sursa, referinta)**, iar identitatea sursei
   vine din sursa deschisa (`ArgumenteSursa::identitate` ->
   `SursaObiecte::identitate`), nu ghicita din argumente: pentru un
   instantaneu e serverul declarat in fisier, nu calea lui (F-B4).

## Erori tipice

| Simptom | Cauza | Unde caut |
|---|---|---|
| `GM-CLI-001` | argumente lipsa sau care nu merg impreuna („da ori --server, ori --sursa", `--la` fara o singura diferenta) | `src/main.rs`, `src/sursa.rs` |
| `GM-CLI-002` | parola: lipsa (`GMOTION_PAROLA` nesetata), goala, fisier sau intrare standard ilizibile | `src/acreditari.rs` |
| `GM-CLI-003` | un fisier de intrare (text final, plan) nu se citeste | `src/comenzi.rs` |
| `GM-CLI-004` | planul nu e JSON valid sau e facut cu alt martor | `src/comenzi.rs`, `deploy` |
| `GM-CLI-005` | un nume (baza, obiect, `--doar`) nu e in comparare; mesajul listeaza ce exista | `src/comenzi.rs` |
| `GM-CLI-006` | nimic de facut (nicio baza, nimic de exportat) | `src/comenzi.rs` |
| `GM-CLI-007` | un fisier de iesire nu se scrie | `src/comenzi.rs` |
| `GM-CLI-008` | martorul nu s-a putut citi | `src/comenzi.rs` |
| `GM-DB-007` | instantaneul nu se citeste | `gmotion-db`, `src/fixture.rs` |
| `--json` da iesire neparsabila | ceva scrie la iesirea standard in afara de JSON | `src/afisare.rs`, `platforma/src/jurnal.rs` |
| `GM-MOD-002` | tot ce e marcat exista doar in baza comparata | `gmotion-modul-diff-obiecte` |

## Testare

```
cd rust && cargo test -p gmotion-cli       # 55 teste
```

`tests/cli.rs` ruleaza **binarul adevarat**, nu functii interne, si acopera
anume lucrurile pe care o proba manuala le-ar rata la urmatoarea regresie:
`--json` curat, coduri de iesire nenule, `--parola` inexistent, `--server` si
`--sursa` incompatibile, cele patru moduri de impartire a deploy-ului, si
fluxul complet instantaneu -> comparare -> diff -> deploy -> CSV.

**Nu acopera:** conectarea la un SQL Server real si descoperirea in retea. Vezi
`etape/02-db.md`.

## Depanare

```
gmotion diagnostic
GMOTION_JURNAL=gmotion::cli=debug,gmotion::db=debug gmotion compara --sursa f.json --referinta R
gmotion compara --sursa f.json --referinta R --json | python3 -m json.tool
```

Pentru un bug de pe o baza reala:

```
gmotion instantaneu --server SRV01 --baze A,B -o bug.json --nota "ce se vede gresit"
gmotion compara --sursa bug.json --referinta A
```
