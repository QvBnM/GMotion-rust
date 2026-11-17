# gmotion-db

<!-- api-hash: 243dbd8f17bf7a44 -->

## Rol

Accesul la datele din SQL Server: descoperirea instantelor in retea, listarea
bazelor, citirea obiectelor comparabile. Singurul crate cu I/O de retea.

## Nu face

- **Nu normalizeaza si nu calculeaza hash-uri.** Intoarce text brut din
  `sys.sql_modules`. Normalizarea e `gmotion-core`, si o face cine are nevoie de
  ea. Versiunea Python calcula totul la citire si tinea token-ii in memorie
  pentru fiecare obiect din fiecare baza — al treilea motiv al rescrierii.
- **Nu compara nimic.** Ce se compara cu ce e treaba modulului.
- **Nu salveaza acreditari.** Sesiunile sunt `gmotion-platforma`. Parola nu se
  serializeaza, nu ajunge in `Debug`, nu ajunge in jurnal.
- **Nu expune asincronie.** `tiberius` e async, dar runtime-ul rămâne inchis in
  `sqlserver.rs`. Altfel `async` ar urca prin platforma si module pana in
  interfata, pentru o aplicatie care face citiri in lot.

## API public

### Contractul

`SursaObiecte` — de unde vin obiectele. Doua implementari: baza reala si
instantaneu din fisier. Tot ce e deasupra `db` vorbeste numai prin trait, deci
se testeaza fara SQL Server.

| Metoda | Ce face |
|---|---|
| `descriere()` | pentru jurnal si diagnostic. Niciodata parole |
| `baze()` | bazele disponibile, sortate, fara cele de sistem |
| `obiecte(baza)` | proceduri, view-uri, functii, triggere |
| `obiecte_multe(baze, progres)` | bucla cu raportare de progres; prima eroare oprește citirea |

### Tipuri

`Obiect` (`schema`, `nume`, `tip`, `definitie`, plus `id()` = `schema.nume`),
`Tinta` (server + port opțional), `Autentificare` (`Sql` sau `Integrata`),
`Instanta`, `InfoServer`.

### Surse

| Tip | Ce e |
|---|---|
| `SursaSqlServer::nou(tinta, auth)` | baza reala. Nu se conecteaza la construire, doar valideaza |
| `SursaSqlServer::verifica(baza)` | test de conexiune, intoarce `InfoServer` |
| `SursaFixture::din_fisier(cale)` / `din_instantaneu(i)` | instantaneu |
| `Instantaneu::nou/scrie/citeste` | formatul de instantaneu, versiune `1` |

### Descoperire si validare

`cauta_instante(timeout)` — SSRP broadcast pe UDP 1434.
`parseaza_raspuns(date, ip)` — parsarea, separata de retea ca sa fie testabila.
`valideaza_nume(n)` — nume de server/baza, inainte de a ajunge in conexiune.

## Invariante

1. **Parola nu iese niciodata:** nici in `Display`, nici in `Debug`, nici in
   jurnal, nici in instantaneu. `Autentificare` nu implementeaza `Serialize`,
   iar `Debug` pe `SursaSqlServer` e scris de mana, nu derivat.
2. **Numele se valideaza inainte de conectare,** nu dupa. Doar litere, cifre,
   `_ - . \ ,`. Configuratia se construieste pe campuri, nu prin text de
   conexiune cu `;`.
3. **Obiectele vin brute.** Nicio transformare in afara de `trim()` pe tip.
4. **Ordinea e stabila:** `ORDER BY s.name, o.name` in interogare, `BTreeMap` in
   instantaneu. Altfel diff-ul a doua instantanee e nefolositor.
5. **Un instantaneu dus-intors prin JSON da obiecte identice,** inclusiv CRLF,
   diacritice, ghilimele si apostrofuri escapate.
6. **`obiecte_multe` se oprește la prima eroare.** O comparare pe baze
   incomplete da rezultate false, ceea ce e mai rau decat sa nu dea nimic.
7. Un obiect fara definitie (criptat cu `WITH ENCRYPTION`) se sare cu
   avertisment; nu e eroare.

## Erori tipice

Codurile sunt stabile si apar si in jurnal si pe ecran. Registrul e in
`JURNAL.md`.

| Cod | Simptom | Unde caut |
|---|---|---|
| `GM-DB-001` | conexiunea nu se deschide: retea, port, firewall, instanta oprita | `src/sqlserver.rs` |
| `GM-DB-002` | autentificare refuzata: parola, utilizator, drepturi pe baza | `src/sqlserver.rs` |
| `GM-DB-003` | auth integrata cerută pe un sistem care nu o are (orice non-Windows) | `src/sqlserver.rs` |
| `GM-DB-004` | interogarea cade: drepturi lipsa pe `sys.sql_modules`, baza offline | `src/sqlserver.rs` |
| `GM-DB-005` | baza cerută nu exista in sursa | `src/sqlserver.rs`, `src/fixture.rs` |
| `GM-DB-006` | descoperirea eșueaza: socket UDP blocat local | `src/descoperire.rs` |
| `GM-DB-007` | instantaneul nu se citeste: fisier lipsa sau JSON rupt | `src/fixture.rs` |
| `GM-DB-008` | instantaneul nu se scrie: drepturi, disc plin | `src/fixture.rs` |
| `GM-DB-009` | instantaneu de alta versiune | `src/fixture.rs` |
| `GM-DB-010` | nume de server sau baza invalid | `src/eroare.rs` |

**Un server care exista si nu apare in lista** nu e, de obicei, un bug de cod:
UDP 1434 blocat de firewall, subnet traversat fara IP helper, `Hide Instance =
Yes`, sau SQL Browser oprit. Se verifica intai asta, apoi
`parseaza_raspuns`.

## Testare

```
cd rust && cargo test -p gmotion-db        # 29 teste, fara baza de date
```

Toate testele ruleaza fara SQL Server si fara retea. Acopera: parsarea SSRP
(inclusiv raspunsuri rupte si octeti invalizi UTF-8), validarea numelor,
unicitatea codurilor de eroare, redactarea parolei in `Debug` si `descriere`,
dus-intorsul instantaneului, si granita cu `core`.

**Nu acopera, si nu poate acoperi aici:** conectarea reala, autentificarea
integrata Windows, si descoperirea pe o retea cu servere adevarate. Pentru
astea exista etapa de verificare pe un server real — vezi
`etape/02-db.md`.

## Depanare

Jurnalul are tinta `gmotion::db`:

```
RUST_LOG=gmotion::db=debug cargo run ...
```

Un bug de pe o baza reala se transforma in fixture, si de acolo in test de
regresie:

```
gmotion instantaneu --server SRV01 --baze A,B --ieșire bug.json
gmotion compara --sursa bug.json
```

Fara pasul asta, un bug de pe un server la care nu avem acces se depaneaza prin
„incearca si spune-mi ce vezi" — cel mai scump mod de lucru posibil.
