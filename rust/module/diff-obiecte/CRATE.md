# gmotion-modul-diff-obiecte

<!-- api-hash: 0298bb9adb4b9103 -->

## Rol

Modulul 1: compara obiectele SQL ale mai multor baze fata de o baza de
referinta si genereaza scripturi de deploy. Toata orchestrarea, **ca
biblioteca** — fara nicio interfata.

## Nu face

- **Nu citeste din baza.** Primeste obiecte prin `gmotion_db::SursaObiecte`,
  deci nu stie daca vin de pe un server sau din instantaneu.
- **Nu compara texte.** Normalizarea, segmentarea si diff-ul sunt
  `gmotion-core`. Un al doilea motor aici ar fi exact bug-ul din v3.4.
- **Nu deseneaza si nu deschide dialoguri.** Caile de iesire vin ca argument.
- **Nu depinde de alt modul.** Daca ar avea nevoie, acel ceva apartine
  platformei.
- **Nu citeste ceasul.** Marcajul de timp din scripturi se da din afara:
  altfel rezultatul n-ar fi testabil.

## API public

| Zona | Ce ofera |
|---|---|
| `comparare` | `pregateste(obiecte) -> Baza`, `compara(...) -> Comparare`, `setari_diferite(a, b)`, `Rand`, `Categorie`, `Numaratoare`, `Progres` |
| `diferente` | `diferente_obiect(ref, cur) -> (Vec<Diferenta>, rezumat)`, `detalii_obiect(ref, cur, setari) -> Detalii`, `Fel`, `Bucata`, `Marcaj` |
| `deploy` | `aplica_create_or_alter`, `scrie(...)`, `scrie_cu_revenire(...)`, `DeDeployat`, `PasRevenire`, `ModImpartire`, `Raport` |
| `plan` | `Plan` (`din_comparare`, `verifica`, `verifica_sau_opreste`, `de_deployat`, `de_deployat_cu` — cu textele finale, `revenire`), `Pas`, `Schimbare`, `Parte`, `FelSchimbare`, `amprenta_exacta` |
| `export` | `scrie_csv(cale, randuri, stare)`, `Stare` |
| `impact` | `folosit_de(baza)` — cine foloseste fiecare obiect (prin lexer, aceeasi legare ca ordinea de deploy); `estimeaza(rand, folosit_de) -> Impact` (`Nivel` Scazut / Mediu / Ridicat, cu `motive`) |
| `revizuire` | `Revizuire` — ce s-a revizuit, **omis** (`omite`, `e_omis`) sau **deployat** (`marcheaza_deployat`, `e_deployat`, `a_fost_deployat` = versiune noua), legat de amprenta continutului; **note** pe nume (`pune_nota`, `nota`, `note`); `din_sesiune` / `in_sesiune` |
| `editare` | `Editari` — textul final per obiect: `aplica_blocuri`, `aplica_toate`, `reset`, `restaureaza` (Ctrl+Z), `pune` (scris de mana), `pentru_deploy`, `curata(randuri, baze_comparate)`; legat de amprenta, salvat in sesiune; `TextFinal`. Un text final **egal cu textul bazei** e decizia „pastreaza tinta" si se pastreaza: doar `reset` o sterge (F-A5) |
| `modul` | `ModulDiffObiecte` — declaratia catre `Registru` |
| radacina | `NUME` = `"diff-obiecte"` |

## Invariante

1. **Ordinea de clasificare:** lipsa -> criptat (definitie goala intr-o
   parte) -> hash egal (identic) -> diferit.
   Comparatia de hash e si optimizarea care conteaza: pe baze aproape identice
   majoritatea obiectelor se opresc acolo.
2. **Hash diferit inseamna diferit, si scorul afisat e cel mult 99,99.**
   v3.4 avea „scor 100 -> identic"; scorul e rotunjit, deci pe obiecte mari
   un `NOT` sters disparea (audit 2026-11-17). Ce e cosmetic e deja prins de
   hash, care e calculat pe forma normalizata.
3. **`Diferenta::id` NU e pozitia in lista.** E id-ul stabil dat de
   `gmotion_core`: pozitia blocului in referinta pentru randurile ancorate,
   numere negative unice pentru cele doar-adaugate. Prin el supravietuiesc
   deciziile utilizatorului („linia asta o iau, asta nu") unei recalculari. Cu
   indexul din lista, orice schimbare in tinta le-ar muta pe alte linii.
4. **Obiectele `extra` nu se pot deploia** — exista doar in baza comparata, deci
   nu exista cod in referinta de adus. Se numara ca `sarite`, si daca **tot** ce
   e marcat e extra, se intoarce `GM-MOD-002`, nu un director gol.
5. **Codul deployat vine din referinta, exact cum e in baza** (`definitie`),
   nu textul de afisare: acela taie spatiile de la capatul liniilor de cod si
   liniile goale de la capete. Un text final **egal cu afisarea referintei**
   (ce pune „Aplica toate") nu e o editare: se deployeaza `definitie`.
   Afisarea nu mai atinge interiorul string-urilor (C5, auditul QA
   2026-11-18), dar nici asa nu e textul din baza.
6. **Primul `CREATE`/`ALTER <tip>` din cod devine `CREATE OR ALTER`.** Se
   cauta doar in cod (prin `gmotion_core::lexer`), nu in comentarii sau
   string-uri; pozitiile sunt in textul original. `CREATE TABLE` nu se atinge.
   Codul deja `CREATE OR ALTER` ramane neatins.
6b. **Nimic nu se scrie daca ceva e periculos** (verificare inainte de orice
   fisier): nume cu caractere de control (`GM-MOD-008`), o linie `GO` in
   definitie (`GM-MOD-009`; sqlcmd taie batch-ul chiar si intr-un string),
   obiect fara cod / criptat (`GM-MOD-011`). Numele bazei intra in `USE [..]`
   cu `]` dublat.
6c. **Fiecare obiect isi reface setarile de creare** (`SET ANSI_NULLS` /
   `SET QUOTED_IDENTIFIER`, intr-un batch inaintea codului), exact cum erau in
   referinta; necunoscute (instantaneu vechi) -> `ON`/`ON`, ca SSMS.
6d. **Obiectele unei baze se scriu dupa dependente** (`ordine_dupa_dependente`):
   referintele din cod (prin lexer) la alte obiecte din acelasi deploy; la
   egalitate functii, view-uri, proceduri, triggere. Ciclurile nu pierd
   obiecte.
6e. **Fiecare fisier incepe cu garda de versiune** (`CREATE OR ALTER` cere
   13.0.4001+; Azure exceptat): pe un server vechi `RAISERROR` + `SET NOEXEC
   ON`, inainte de orice `USE`. Fiecare fisier se termina cu `SET NOEXEC OFF`.
7. **Fiecare baza isi aduce propriul `USE [...]`,** inclusiv in modul „tot
   intr-un fisier": altfel al doilea set ar rula pe baza gresita. Dupa `USE`
   urmeaza `IF DB_NAME() <> N'...' RAISERROR + SET NOEXEC ON`: un `USE` care
   cade opreste doar batch-ul lui, iar SSMS si `sqlcmd` fara `-b` ar rula
   restul pe baza curenta (F-D1, auditul QA).
7b. **Scripturile se scriu cu BOM UTF-8, iar liniile scrise de GMotion cu
   `\r\n`** (F-D2): fara BOM, `sqlcmd` le citeste in pagina de cod OEM si un
   literal `N'...'` non-ASCII ajunge mojibake in codul deployat. Codul
   obiectului se lipeste **neatins**, cu capetele lui de linie — acolo sunt
   datele utilizatorului.
8. **Un nume de baza sau obiect nu poate forma o cale.** Separatorii devin `_`,
   punctele de la capete se taie. Numele de fisier sunt unice in cadrul unei
   rulari (fara a deosebi majusculele), numele rezervate Windows (`CON`,
   `NUL`, ...) primesc `_` in fata, iar directorul de iesire trebuie sa nu
   existe sau sa fie gol (`GM-MOD-010`).
8b. **Garda „baza n-a mai fost atinsa" e pe amprenta exacta** (tip, setari,
   definitia neatinsa), nu pe hash-ul normalizat al compararii: o schimbare
   doar de spatii sau de `QUOTED_IDENTIFIER` tot opreste deploy-ul. Se
   verifica si tinta, si referinta; aparut / disparut / modificat opresc la
   fel (`GM-MOD-012`), cu fiecare obiect pe nume. Garda primeste obiecte
   **recitite** — cu cele de la comparare ar compara un lucru cu el insusi.
8c. **Revenirea se scrie inainte de deploy, in `<director>/revenire/`**, din
   definitiile actuale ale tintei (cod si setari ale tintei): ce exista se
   reface, ce deploy-ul creeaza se sterge (`DROP ... IF EXISTS`). Intai
   refacerile, apoi stergerile in ordinea inversa dependentelor. Daca
   revenirea nu se poate scrie, nu se scrie nici deploy-ul.
8d. **Textul final e legat de amprenta obiectului** (ca bifa): daca sursa
   sau baza s-au schimbat de la editare, nu mai e valabil si nu intra in
   deploy. Un text final egal cu textul bazei nu e editare (se scoate).
   „Aplica toate" pune textul exact al sursei; „Aplica" compune prin
   `gmotion_core::compune`, aceeasi aliniere ca editorul.
8e. **Un obiect e ori revizuit, ori omis.** Omisele nu intra in planul facut
   din `cli` fara `--doar`.
9. **CSV-ul are BOM UTF-8, separator `;`, CRLF si virgula zecimala** — altfel
   Excel romanesc strica diacriticele sau pune totul pe o coloana.
10. **Un nume din cod se leaga de un obiect intr-un singur loc**
   (`deploy::referinte`): ordinea de deploy si impactul estimat il folosesc pe
   acelasi. Un nume scurt ambiguu nu se leaga; comentariile si string-urile nu
   sunt referinte.
11. **Notele se tin pe nume, bifele pe amprenta.** O nota e despre obiect
   („a ramas aici"), deci nu cade cand obiectul se schimba si nu o sterge
   `curata`.
12. **Curatarea atinge doar bazele din compararea de acum**
   (`curata(randuri, baze_comparate)`, si la `Revizuire` si la `Editari`).
   O baza care nu e comparata acum n-are randuri: fara filtru, toata munca
   facuta pe ea alta data se stergea, fara niciun mesaj (C3/C4, auditul QA
   2026-11-18). Un „omis" pierdut inseamna obiect ajuns inapoi in deploy.

## Erori tipice

| Cod | Simptom | Unde caut |
|---|---|---|
| `GM-MOD-001` | se cer rezultate fara sa fi rulat o comparare | `src/eroare.rs` |
| `GM-MOD-002` | „niciun obiect de deployat": tot ce e marcat e `extra` | `src/deploy.rs` |
| `GM-MOD-003` | scripturile nu se scriu: drepturi, disc | `src/deploy.rs` |
| `GM-MOD-004` | id de rand inexistent | `src/eroare.rs` |
| `GM-MOD-005` | exportul CSV cade, sau nimic de exportat | `src/export.rs` |
| `GM-MOD-006/007` | eroare venita de la platforma / de la date. **Codul original ramane in text** | `src/eroare.rs` |
| `GM-MOD-008` | nume de baza / obiect cu caractere de control: ar injecta cod in script | `src/deploy.rs` |
| `GM-MOD-009` | linie `GO` in definitie: obiectul se deployeaza de mana | `src/deploy.rs` |
| `GM-MOD-010` | directorul de deploy nu e gol | `src/deploy.rs` |
| `GM-MOD-011` | obiect criptat sau fara cod: nu se compara, nu se deployeaza | `src/deploy.rs`, `src/comparare.rs` |
| `GM-MOD-012` | baza sau referinta s-a schimbat intre plan si deploy; mesajul spune ce obiect | `src/plan.rs` |

**Un obiect raportat ca „modificat" care se deschide identic** era bug-ul tipic
din v3.4, cu lista si editorul pe motoare diferite. Aici ambele cheama
`gmotion_core`, deci cauza nu mai poate fi aia: cauta in `core/src/segmentare.rs`.

## Testare

```
cd rust && cargo test -p gmotion-modul-diff-obiecte      # 87 teste
```

`tests/flux_complet.rs` ruleaza tot lantul pe un instantaneu: citire prin
`SursaObiecte` -> comparare -> diff -> deploy -> CSV, fara baza de date si fara
interfata. **E si forma in care un bug de pe o baza reala devine test de
regresie:** se schimba doar instantaneul.

`deploy.rs` are cele mai multe teste, fiindca e singura parte unde o greseala se
vede dupa ce a rulat pe o baza.

**Nu acopera:** citirea paralela a bazelor (versiunea Python folosea 4
conexiuni simultane; aici citirea e secventiala prin `obiecte_multe` — de
masurat pe baze reale inainte de a complica), si anularea unei comparari in curs.

## Depanare

```
GMOTION_JURNAL=gmotion::modul=debug gmotion compara --sursa instantaneu.json
```

Pentru un bug de clasificare: compara `hash_referinta` si `hash_curent` din
`Rand`. Egale = ar fi trebuit identic. Diferite cu scor 100 = diferenta
cosmetica, deci corect identic.
