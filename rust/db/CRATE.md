# gmotion-db

<!-- api-hash: ee493aef25f3f781 -->

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
- **Nu expune asincronie.** `tiberius` e async, dar runtime-ul ramane inchis in
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
| `identitate()` | **cine** e sursa, pe scurt si stabil: `host[:port]`, sau `instantaneu <server declarat>`. Intra in cheia sesiunii, deci nu se schimba cand se schimba autentificarea (F-B4) |
| `baze()` | bazele disponibile, sortate, fara cele de sistem |
| `obiecte(baza)` | proceduri, view-uri, functii, triggere |
| `obiecte_multe(baze, progres)` | bucla cu raportare de progres; prima eroare opreste citirea |

### Tipuri

`Obiect` (`schema`, `nume`, `tip`, `definitie`, plus `id()` = `schema.nume`),
`Tinta` (server + port optional), `Autentificare` (`Sql` sau `Integrata`),
`Instanta` (cu `implicita` si `cluster`, din `IsClustered`), `InfoServer`.

### Surse

| Tip | Ce e |
|---|---|
| `SursaSqlServer::nou(tinta, auth)` | baza reala. Nu se conecteaza la construire, doar valideaza |
| `SursaSqlServer::cu_timeout_conectare(durata)` | cat se asteapta conexiunea (implicit 30 s); din `timeout_conectare_s` (F-B5) |
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
   iar `Debug` e scris de mana, cu redactare, si pe `Autentificare` si pe
   `SursaSqlServer` (pe `Autentificare` era derivat pana la audit).
2. **Numele se valideaza inainte de conectare,** nu dupa. Serverul: doar
   litere, cifre, `_ - . \ ,` (`valideaza_nume`). Baza: orice nume valid in
   SQL Server, fara caractere de control (`valideaza_baza`) — ajunge doar in
   login-ul TDS, niciodata in text SQL. Configuratia se construieste pe
   campuri, nu prin text de conexiune cu `;`.
3. **Obiectele vin brute.** Nicio transformare in afara de `trim()` pe tip.
   Cu ele vin si setarile de creare (`Obiect::setari`: `uses_ansi_nulls`,
   `uses_quoted_identifier`), optionale in JSON ca instantaneele vechi sa se
   citeasca in continuare.
4. **Ordinea e stabila:** `ORDER BY s.name, o.name` in interogare, `BTreeMap` in
   instantaneu. Altfel diff-ul a doua instantanee e nefolositor.
5. **Un instantaneu dus-intors prin JSON da obiecte identice,** inclusiv CRLF,
   diacritice, ghilimele si apostrofuri escapate.
6. **`obiecte_multe` se opreste la prima eroare.** O comparare pe baze
   incomplete da rezultate false, ceea ce e mai rau decat sa nu dea nimic.
   Pe SQL Server citeste pana la 4 baze deodata, pe conexiuni separate (ca
   v3.4); ordinea rezultatului e cea ceruta.
7. **Un obiect criptat** (`WITH ENCRYPTION`, definitie NULL) se pastreaza cu
   definitie goala (`Obiect::criptat`), cu avertisment. Nu se sare: sarit,
   aparea „lipsa" in tinta si deploy-ul l-ar fi suprascris.
8. **Nicio asteptare fara limita.** Deschiderea (TCP + TLS + login) are
   `TIMEOUT_CONECTARE`, fiecare interogare `TIMEOUT_INTEROGARE`, SQL Browser
   `TIMEOUT_BROWSER`. Dupa orice eroare conexiunea se arunca.
9. **Instanta numita se rezolva prin cod propriu** (`rezolva_port_instanta`),
   pe un socket UDP conectat la gazda intrebata, cu parserul validat din
   `descoperire`. Nu prin `connect_named` din `tiberius` (panica pe raspuns
   scurt, accepta raspuns de la oricine).
10. **Certificatul serverului:** `Tinta::certificat` — `Nevalidat` (implicit,
   ca in v3.4, cu avertisment in jurnal), `Sistem`, `Autoritate(cale)`.
11. **Un instantaneu e intrare nesigura:** la citire se refuza nume de baze
   invalide si obiecte duplicate; scrierea e atomica (temporar + redenumire).

## Erori tipice

Codurile sunt stabile si apar si in jurnal si pe ecran. Registrul e in
`JURNAL.md`.

| Cod | Simptom | Unde caut |
|---|---|---|
| `GM-DB-001` | conexiunea nu se deschide: retea, port, firewall, instanta oprita | `src/sqlserver.rs` |
| `GM-DB-001` cu „No such host is known” la `SRV\INSTANTA` | numele intreg a ajuns la DNS: instanta nu a trecut prin `interpreteaza_server` | `src/sqlserver.rs` |
| `GM-DB-001` cu „SQL Browser ... nu a raspuns” | instanta numita fara port: serviciul SQL Browser (UDP 1434) oprit, blocat de firewall, sau instanta nu exista. Ocolire: `--port` | `src/sqlserver.rs`, `rezolva_port_instanta` |
| `GM-DB-001` cu „timeout dupa 30 s (TCP, TLS si autentificare)” | serverul accepta TCP dar nu termina login-ul: alt serviciu pe port, server blocat | `src/sqlserver.rs`, `conectare` |
| `GM-DB-001` cu eroare de certificat | `--certificat sistem` pe un server cu certificat autosemnat sau cu alt nume | `src/sqlserver.rs`, `configuratie` |
| `GM-DB-002` | autentificare refuzata: parola, utilizator, drepturi pe baza; si cont blocat (18486), parola expirata (18487) sau de schimbat (18488) | `src/sqlserver.rs`, `cod_conectare` |
| `GM-DB-003` | auth integrata ceruta pe un sistem care nu o are (orice non-Windows) | `src/sqlserver.rs` |
| `GM-DB-004` | interogarea cade: drepturi lipsa pe `sys.sql_modules`, baza offline | `src/sqlserver.rs` |
| `GM-DB-011` | nume de baza gol, peste 128 de caractere, sau cu caractere de control | `src/eroare.rs` |
| `GM-DB-005` | baza ceruta nu exista in sursa | `src/sqlserver.rs`, `src/fixture.rs` |
| `GM-DB-006` | descoperirea esueaza: socket UDP blocat local | `src/descoperire.rs` |
| `GM-DB-007` | instantaneul nu se citeste: fisier lipsa sau JSON rupt | `src/fixture.rs` |
| `GM-DB-008` | instantaneul nu se scrie: drepturi, disc plin | `src/fixture.rs` |
| `GM-DB-009` | instantaneu de alta versiune | `src/fixture.rs` |
| `GM-DB-010` | nume de server sau baza invalid | `src/eroare.rs` |
| `GM-DB-012` | obiecte fara definitie care nu sunt criptate: utilizatorul n-are `VIEW DEFINITION` (inainte apareau „criptate”) | `src/sqlserver.rs`, `verifica_definitiile` |

**Un server care exista si nu apare in lista** nu e, de obicei, un bug de cod:
UDP 1434 blocat de firewall, subnet traversat fara IP helper, `Hide Instance =
Yes`, sau SQL Browser oprit. Se verifica intai asta, apoi
`parseaza_raspuns`.

## Testare

```
cd rust && cargo test -p gmotion-db        # 52 teste, fara baza de date
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
gmotion instantaneu --server SRV01 --baze A,B --iesire bug.json
gmotion compara --sursa bug.json
```

Fara pasul asta, un bug de pe un server la care nu avem acces se depaneaza prin
„incearca si spune-mi ce vezi" — cel mai scump mod de lucru posibil.
