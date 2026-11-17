# Etapa 4 — `module/diff-obiecte`, fara interfata

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-17 · **Incheiata:** 2026-11-17

## Scop

Tot ce face GMotion v3.4 functional, ca biblioteca, fara nicio interfata:
comparare, diff, deploy, export. Testabil cap-coada pe un instantaneu.

## Criterii de terminare

- [x] `comparare`: clasificare lipsa / diferit / extra / identic, cu scor si rezumat
- [x] diferenta cosmetica (hash diferit, scor 100) nu se raporteaza
- [x] `diferente`: diff la cerere, cu id-uri stabile si marcaje pe token
- [x] `deploy`: patru moduri de impartire, `CREATE OR ALTER`, `USE` per baza
- [x] `export`: CSV pentru Excel romanesc (BOM, `;`, CRLF, virgula zecimala)
- [x] implementarea `Modul` si inregistrarea la `Registru`
- [x] coduri `GM-MOD-001..007`, cu codul original pastrat la traversarea granitei
- [x] test de flux complet pe instantaneu, fara baza de date
- [x] `CRATE.md`
- [x] poarta verde

## Pasi

1. `comparare` — clasificarea (portata din `_comparatie`)
2. `diferente` — diff la cerere
3. `deploy` — generatorul (portat din `genereaza_deployer` si `_scrie_deployer`)
4. `export` — CSV
5. `modul` — declaratia catre platforma
6. flux complet pe instantaneu, `CRATE.md`

## Jurnal de lucru

- **2026-11-17** Portat cu `mostenire/app.py` deschis alaturi, pastrand ordinea
  de decizie si deciziile de produs (obiectele `extra` nu se deploiaza, primul
  `CREATE`/`ALTER` devine `CREATE OR ALTER`).
- **2026-11-17** 42 de teste, din care flux complet pe instantaneu. `deploy` are
  cele mai multe: e singura parte unde o greseala se vede dupa ce a rulat pe o
  baza.
- **2026-11-17** Marcajul de timp din scripturi se da din afara, nu se citeste
  din ceas — altfel generatorul n-ar fi testabil.

## Ce a ieșit altfel decat planul

**Am scris `Diferenta` cu indexul din lista drept identificator, si era
greșit.** `gmotion_core::RandDiff` are deja un camp `id` care e **stabil intre
recalculari**: poziția blocului in referinta pentru randurile ancorate, numere
negative unice pentru cele doar-adaugate. Prin el supraviețuiesc deciziile
utilizatorului din editor („linia asta o iau, asta nu") unei recalculari. Cu
indexul din lista, orice schimbare in tinta le-ar fi mutat pe alte linii — un
bug care s-ar fi vazut tarziu si ar fi parut aleator.

Cauza greselii: am presupus forma lui `RandDiff` din memorie in loc sa o
citesc. A ieșit la iveala doar fiindca `serde` nu putea serializa tipurile din
`core` (care nu depinde de `serde`, deliberat), deci am fost obligat sa mă uit.
**Noroc, nu proces.** Lectia: cand portez ceva care traverseaza o granita de
crate, citesc tipul, nu-l presupun.

**`aplica_create_or_alter` a ieșit intai inutil de complicat** — un lanț de
iteratori cu un `let _ = cuvant;` in el, scris ca sa unific doua cazuri care nu
aveau nevoie de unificare. Clippy l-a prins (`let-binding has unit value`).
Rescris simplu: adun pozitiile, le sortez, iau prima care e urmata de un tip de
obiect.

**Doua expectative greșite in testele mele, nu in cod:** `slug("../etc")` si
numaratoarea de obiecte identice din fluxul complet (Client1 da 2, Client2 da 4,
deci 6, nu 5). Al doilea caz e util ca avertisment: intr-un test cu date
construite de mana, cifra asteptata e la fel de ușor de greșit ca si codul. La
`slug` am inlocuit verificarea sirului exact cu verificarea **proprietatii** —
ca rezultatul nu poate forma o cale — care e ce conteaza de fapt.

**Citirea bazelor a rams secventiala.** Versiunea Python folosea 4 conexiuni
simultane. Nu am portat paralelismul: `obiecte_multe` din `db` e secventiala, si
nu am cu ce masura daca merita complicarea. De masurat pe baze reale la etapa de
paritate; notat ca neacoperit in `CRATE.md`.