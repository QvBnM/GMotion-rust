# gmotion-modul-diff-obiecte

<!-- api-hash: 35b437f6ad0ab45f -->

## Rol

Modulul 1: compara obiectele SQL ale mai multor baze fata de o baza de
referinta si genereaza scripturi de deploy. Toata orchestrarea, **ca
biblioteca** — fara nicio interfata.

## Nu face

- **Nu citeste din baza.** Primeste obiecte prin `gmotion_db::SursaObiecte`,
  deci nu stie daca vin de pe un server sau din instantaneu.
- **Nu compara texte.** Normalizarea, segmentarea si diff-ul sunt
  `gmotion-core`. Un al doilea motor aici ar fi exact bug-ul din v3.4.
- **Nu desenează si nu deschide dialoguri.** Caile de ieșire vin ca argument.
- **Nu depinde de alt modul.** Daca ar avea nevoie, acel ceva aparține
  platformei.
- **Nu citeste ceasul.** Marcajul de timp din scripturi se da din afara:
  altfel rezultatul n-ar fi testabil.

## API public

| Zona | Ce ofera |
|---|---|
| `comparare` | `pregateste(obiecte) -> Baza`, `compara(...) -> Comparare`, `Rand`, `Categorie`, `Numaratoare`, `Progres` |
| `diferente` | `diferente_obiect(ref, cur) -> (Vec<Diferenta>, rezumat)`, `Fel`, `Bucata`, `Marcaj` |
| `deploy` | `aplica_create_or_alter`, `scrie(...)`, `DeDeployat`, `ModImpartire`, `Raport` |
| `export` | `scrie_csv(cale, randuri, stare)`, `Stare` |
| `modul` | `ModulDiffObiecte` — declaratia catre `Registru` |
| rădăcină | `NUME` = `"diff-obiecte"` |

## Invariante

1. **Ordinea de clasificare:** lipsa -> hash egal -> scor 100 -> diferit.
   Comparatia de hash **inainte** de scor e optimizarea care conteaza: pe baze
   aproape identice majoritatea obiectelor se opresc acolo.
2. **Hash diferit cu scor 100 inseamna identic.** Diferenta e cosmetica, adica
   exact ce sterge normalizarea. Nu se raporteaza.
3. **`Diferenta::id` NU e poziția in lista.** E id-ul stabil dat de
   `gmotion_core`: poziția blocului in referinta pentru randurile ancorate,
   numere negative unice pentru cele doar-adaugate. Prin el supraviețuiesc
   deciziile utilizatorului („linia asta o iau, asta nu") unei recalculari. Cu
   indexul din lista, orice schimbare in tinta le-ar muta pe alte linii.
4. **Obiectele `extra` nu se pot deploia** — exista doar in baza comparata, deci
   nu exista cod in referinta de adus. Se numara ca `sarite`, si daca **tot** ce
   e marcat e extra, se intoarce `GM-MOD-002`, nu un director gol.
5. **Codul deployat vine din referinta,** nu din tinta.
6. **Primul `CREATE`/`ALTER <tip>` devine `CREATE OR ALTER`,** doar primul:
   restul pot fi in comentarii sau in SQL dinamic. `CREATE TABLE` nu se atinge
   (`CREATE OR ALTER TABLE` nu exista). Codul deja `CREATE OR ALTER` rămâne
   neatins.
7. **Fiecare baza isi aduce propriul `USE [...]`,** inclusiv in modul „tot
   intr-un fisier": altfel al doilea set ar rula pe baza greșita.
8. **Un nume de baza sau obiect nu poate forma o cale.** Separatorii devin `_`.
9. **CSV-ul are BOM UTF-8, separator `;`, CRLF si virgula zecimala** — altfel
   Excel romanesc strica diacriticele sau pune totul pe o coloana.

## Erori tipice

| Cod | Simptom | Unde caut |
|---|---|---|
| `GM-MOD-001` | se cer rezultate fara sa fi rulat o comparare | `src/eroare.rs` |
| `GM-MOD-002` | „niciun obiect de deployat": tot ce e marcat e `extra` | `src/deploy.rs` |
| `GM-MOD-003` | scripturile nu se scriu: drepturi, disc | `src/deploy.rs` |
| `GM-MOD-004` | id de rand inexistent | `src/eroare.rs` |
| `GM-MOD-005` | exportul CSV cade, sau nimic de exportat | `src/export.rs` |
| `GM-MOD-006/007` | eroare venita de la platforma / de la date. **Codul original rămâne in text** | `src/eroare.rs` |

**Un obiect raportat ca „modificat" care se deschide identic** era bug-ul tipic
din v3.4, cu lista si editorul pe motoare diferite. Aici ambele cheama
`gmotion_core`, deci cauza nu mai poate fi aia: caută in `core/src/segmentare.rs`.

## Testare

```
cd rust && cargo test -p gmotion-modul-diff-obiecte      # 42 teste
```

`tests/flux_complet.rs` ruleaza tot lanțul pe un instantaneu: citire prin
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
