# gmotion-platforma

<!-- api-hash: cfb613f66a706dbb -->

## Rol

Ce e comun tuturor uneltelor: unde stau datele, sesiuni, configurare, jurnal,
contractul de modul si rutarea.

Regula de impartire: ce foloseste mai mult de un modul sta aici; ce e specific
unei unelte sta in crate-ul uneltei.

## Nu face

- **Nu compara nimic** si nu stie ce e un obiect SQL dincolo de ce-i da `db`.
- **Nu deseneaza nimic.** Partea de ecrane e doar declarativa.
- **Nu tine acreditari.** Parola se cere la fiecare pornire si nu se salveaza
  niciodata — nici criptata.
- **Nu interpreteaza starea unui modul.** `Sesiune::date` e opac pentru
  platforma.

## API public

| Zona | Ce ofera |
|---|---|
| `cai` | `director_date`, `director_sesiuni(modul)`, `cale_configurare`, `director_jurnale` |
| `protejare` | `protejeaza`, `deprotejeaza`, `e_criptat` |
| `sesiune` | `Sesiune` (`adauga_baze` — bazele comparate se adauga, nu se inlocuiesc), `Sesiuni::pentru_modul(m)` (si `in_director(m, dir)`, pentru teste fara variabile de mediu) cu `salveaza`, `incarca(server, referinta)`, `sterge(server, referinta)`, `pune_deoparte(server, referinta)` (o sesiune ilizibila se muta in `.corupt-<marcaj>.dat.bak`, nu se scrie peste ea, F-A15), `lista`; `slug`, `slug_sesiune(server, referinta)`, `server_normalizat` (`SRV`, `srv`, `SRV,1433` = acelasi server; o sesiune sub forma exacta veche se muta singura) |
| `configurare` | `Configurare::citeste/scrie`, cu implicite in cod; `tema` tine modul ferestrei ales cu butonul (`intunecat` / `luminos` / gol); `pliaza_identicul` tine comutatorul din editor (implicit oprit: se vede tot codul) |
| `timp` | `acum()` („2026-11-18 14:52"), `acum_compact()` („20261118_1452"): marcajul de timp, acelasi in `cli` si in fereastra |
| `jurnal` | `porneste_cu_fisier(director)` (jurnalul sesiunii si intr-un fisier `gmotion-<moment>.log`, `gmotion` la debug; se pastreaza `PASTRATE` = 10), `porneste`, `linii(de_la)`, `adauga_linie`, `goleste` (Sterge log: uita liniile fara sa dea indexul inapoi) |
| `modul` | `Modul`, `CerinteBaze`, `Ecran`, `Registru`, `Ruta`, `METODE_PLATFORMA` |

### Contractul de modul

**Platforma ofera:** conexiunea deja stabilita (modulul nu reintreaba
utilizatorul), un spatiu de nume propriu pentru sesiuni, jurnalul, configurarea,
si metodele comune din `METODE_PLATFORMA` — scrise o singura data.

**Modulul declara:** `nume` (identificator strict), `titlu`, `iconita`,
`cerinte`, `ecrane` cu `ecran_intrare`, si `metode` fara prefix. Platforma le
publica drept `<nume>/<metoda>`.

## Invariante

1. **Sesiunile au spatiu de nume per modul:** `<date>/sesiuni/<modul>/<slug>.dat`.
   Doua module pe aceeasi baza de referinta **nu** se suprascriu. In versiunea
   Python cheia era doar `<slug>.dat`, deci coliziunea era garantata de la al
   doilea modul.
2. **O sesiune nu contine secrete.** Nu e o conventie, e o proprietate a
   tipului: `Sesiune` nu are camp de parola, iar `gmotion_db::Autentificare` nu
   implementeaza `Serialize`. Asta face acceptabil ca pe non-Windows fisierul sa
   nu fie criptat. **Daca vreodata trebuie salvat un secret, calea de
   non-Windows din `protejare.rs` trebuie sa refuze, nu sa scrie in clar.**
3. **Un identificator nu poate iesi din directorul de date.** Doar litere mici,
   cifre, cratima — fiindca devine nume de director. `slug()` reduce orice nume
   de baza la ceva care nu contine `/`, `\` sau `..`.
3b. **Doua nume de baza diferite nu impart niciodata un fisier de sesiune:**
   `slug()` se termina cu 48 de biti de SHA-256 pe numele exact (pana la audit,
   lungimea numelui — `Prod.2024` si `Prod_2024` coliziau). `incarca` verifica
   si ca sesiunea citita e a bazei cerute.
3c. **Cheia unei sesiuni e perechea (server, referinta)** (`slug_sesiune`), si
   `incarca` verifica amandoua. Pana la auditul QA era doar referinta, deci
   `PROD\Ref` si `TEST\Ref` isi suprascriau progresul si `incarca` intorcea
   sesiunea celuilalt server (F-B4). O sesiune scrisa sub numele vechi se muta
   singura pe numele nou la prima citire.
3d. **O sesiune ilizibila nu se suprascrie.** Cine nu o poate citi o pune
   deoparte (`pune_deoparte`) inainte sa scrie alta (F-A15).
4. **Doua module pot avea metode cu acelasi nume.** Rutarea e cu prefix.
5. **Un modul nu poate calca o metoda comuna.** Inregistrarea o refuza: altfel
   comportamentul ar depinde de modulul activ.
6. **Un fisier de configurare stricat e eroare, nu implicite in silenta.** Un
   fisier lipsa, in schimb, da implicitele. Un camp scris gresit e eroare
   (`deny_unknown_fields`): altfel utilizatorul crede ca a schimbat o setare.
7. **O sesiune ilizibila e sarita la listare, cu avertisment.** Un singur fisier
   stricat nu ascunde celelalte.
8. **Jurnalul nu creste nemarginit:** 2000 de linii in memorie, cele mai vechi
   se arunca, si fiecare linie are cel mult 4096 de caractere. Liniile pastreaza
   campurile evenimentului (`server=... baza=...`), iar caracterele de control
   devin secvente vizibile (`\n`, `\u{1b}`).
9. **Configurarea si sesiunile se scriu atomic** (temporar + redenumire), iar
   `Configurare::scrie` refuza ce `citeste` n-ar accepta. BOM-ul de la inceputul
   configurarii e acceptat.
10. **Caile din mediu trebuie sa fie absolute** (`GMOTION_DIR_DATE`, `APPDATA`);
   un `XDG_DATA_HOME` relativ se ignora.

## Erori tipice

| Cod | Simptom | Unde caut |
|---|---|---|
| `GM-PLAT-001` | nu gaseste directorul de date (serviciu, cont fara profil) | `src/cai.rs` |
| `GM-PLAT-002/003` | sesiunea nu se scrie / nu se citeste: drepturi, disc | `src/sesiune.rs` |
| `GM-PLAT-004` | sesiunea nu se decripteaza — **comportament corect** daca fisierul vine de la alt utilizator sau alt calculator | `src/protejare.rs` |
| `GM-PLAT-005` | sesiune de alta versiune | `src/sesiune.rs` |
| `GM-PLAT-006` | identificator de modul sau ecran invalid | `src/eroare.rs` |
| `GM-PLAT-007/008` | modul necunoscut / deja inregistrat | `src/modul.rs` |
| `GM-PLAT-009` | ruta invalida, sau un modul calca o metoda comuna | `src/modul.rs` |
| `GM-PLAT-010` | selectia de baze nu satisface cerintele modulului | `src/modul.rs` |
| `GM-PLAT-011` | configurare stricata sau cu valori absurde | `src/configurare.rs` |

## Testare

```
cd rust && cargo test -p gmotion-platforma        # 54 teste
```

Acopera: slug-uri care nu pot iesi din director, coliziunea de sesiuni intre
module (testul care dovedeste ca bug-ul din versiunea Python nu mai e posibil),
nume de baze cu spatii, slash, diacritice si `..`, configurare lipsa / stricata
/ partiala / cu valori absurde, rutare cu prefix si refuzul metodelor comune,
tamponul de jurnal.

**Nu acopera, si nu poate acoperi aici:** DPAPI real. Pe Linux
`protejare.rs` doar pune un marcaj in clar. Calea de Windows e **verificata la
compilare** de poarta (`cargo check --target x86_64-pc-windows-msvc`), dar
netestata functional — vezi `etape/03-platforma.md`.

## Depanare

```
GMOTION_JURNAL=gmotion::platforma=debug cargo run ...
GMOTION_DIR_DATE=/tmp/gmotion-test cargo run ...     # directorul de date, pentru teste
```

`GMOTION_DIR_DATE` e globala in proces: testele care o schimba trebuie
serializate cu un lacat. Vezi `tests/sesiuni.rs` — altfel poarta cade aleator.
