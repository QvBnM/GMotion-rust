# gmotion-cli

<!-- api-hash: e1efaad5320ab7fa -->

## Rol

Binarul `gmotion`: tot ce face aplicatia, fara interfata grafica. Exista
**inaintea** interfetei, si rămâne dupa ea ca mod de depanare permanent.

Doua lucruri trebuie sa rămână adevarate:

1. **tot ce face aplicatia se poate acționa de aici** — altfel un bug in spatele
   unui buton nu poate fi depanat decat pe masina utilizatorului;
2. **fiecare comanda care citeste obiecte merge si cu `--sursa <instantaneu>`**,
   nu doar cu `--server`. Daca apare una care cere server, asta e bug-ul.

## Nu face

- **Nu compara si nu genereaza nimic singur.** Toata logica e in
  `gmotion-modul-diff-obiecte`. CLI-ul e argumente, ieșire si coduri de eroare.
- **Nu accepta parola ca argument.** Vezi invarianta 2.
- **Nu scrie nimic pe ieșirea standard in afara de rezultat**, cand e `--json`.

## Comenzi

| Comanda | Ce face |
|---|---|
| `diagnostic` | ce vede GMotion: versiune, cai, configurare, ce e disponibil. Prima comanda cand ceva nu merge |
| `servere` | descoperire prin SQL Browser (UDP 1434) |
| `baze` | bazele disponibile |
| `instantaneu` | obiectele brute intr-un fisier, pentru reluare offline |
| `compara` | lista de diferente |
| `diferente` | diff-ul unui singur obiect, linie cu linie |
| `deploy` | scripturile, cu codul din referinta |
| `export` | CSV pentru Excel |

Argumente comune: `--server`/`--port` sau `--sursa`, `--utilizator`,
`--parola-fisier`/`--parola-stdin`, `--json`.

## Invariante

1. **Eșecul da cod de ieșire nenul, si mesajul merge la ieșirea de erori.**
   Altfel o rulare programata ar raporta succes. Codul original
   (`GM-DB-001`, ...) se pastreaza in mesaj.
2. **Parola nu se da ca argument de linie de comanda.** Ar ajunge in istoricul
   shell-ului si ar fi vizibila in lista de procese pentru orice utilizator de
   pe masina. Caile acceptate: `GMOTION_PAROLA`, `--parola-fisier`,
   `--parola-stdin`. Exista un test care cade daca cineva adauga `--parola`.
3. **Pe `--json`, la ieșirea standard iese numai JSON.** Jurnalul si progresul
   merg la erori. In mod text, progresul merge la ieșirea standard.
4. **Cerintele modulului se verifica inainte de a citi ceva** — altfel s-ar citi
   degeaba, posibil minute.
5. **Marcajul de timp se da catre modul, nu se citeste din ceas acolo**, ca
   generatorul de deploy sa rămână testabil.

## Erori tipice

| Simptom | Cauza | Unde caut |
|---|---|---|
| „da ori --server, ori --sursa" | nu s-a dat niciuna | `src/sursa.rs` |
| „lipseste parola pentru utilizatorul X" | `GMOTION_PAROLA` nu e setata | `src/acreditari.rs` |
| `GM-DB-007` | instantaneul nu se citeste | `gmotion-db`, `src/fixture.rs` |
| „baza de referinta nu exista in sursa" | nume greșit; mesajul listeaza ce exista | `src/comenzi.rs` |
| `--json` da ieșire neparsabila | ceva scrie la ieșirea standard in afara de JSON | `src/afisare.rs`, `platforma/src/jurnal.rs` |
| `GM-MOD-002` | tot ce e marcat exista doar in baza comparata | `gmotion-modul-diff-obiecte` |

## Testare

```
cd rust && cargo test -p gmotion-cli       # 27 teste
```

`tests/cli.rs` ruleaza **binarul adevarat**, nu functii interne, si acopera
anume lucrurile pe care o proba manuala le-ar rata la urmatoarea regresie:
`--json` curat, coduri de ieșire nenule, `--parola` inexistent, `--server` si
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
gmotion instantaneu --server SRV01 --baze A,B -o bug.json --nota "ce se vede greșit"
gmotion compara --sursa bug.json --referinta A
```
