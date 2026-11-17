# Jurnal de decizii

Append-only. Nu se rescrie o intrare veche; daca o decizie se schimba, se scrie
una noua care o anuleaza explicit, cu motivul. Asa se vede de ce codul arata
cum arata, fara arheologie in istoricul git.

Format: `## <data> — <decizia>`, apoi **Context**, **Alternative**, **Motiv**,
si unde e relevant **Consecinte**.

---

## 2026-11-17 — Codul Python se inghetata in `mostenire/`, nu se sterge

**Context.** GMotion v3.4 e in productie in Python. Rescrierea in Rust porneste
de la aceleasi commit-uri.

**Alternative.** (a) Python-ul ramane in dezvoltare paralela; (b) se sterge
acum, raman doar in istoricul git; (c) se inghetata in repo.

**Motiv.** (c). Harness-ul de conformitate are nevoie de `motor.py` si `ui.py`
ca sa poată regenera fixture-urile, si etapa de paritate are nevoie de o
versiune care nu se mișca. Dezvoltarea paralela ar face tinta de paritate
mobila; stergerea ar scoate plasa de siguranta.

**Consecinte.** `mostenire/` nu se modifica niciodata. Se sterge abia dupa ce
Rust-ul trece paritatea pe baze reale.

---

## 2026-11-17 — CLI headless inainte de interfata

**Context.** Planul initial trecea de la `db` si `platforma` direct la
prototipul de interfata.

**Alternative.** (a) prototip de editor de diferente devreme, ca sa se afle
costul partii grele; (b) CLI si paritate intai, interfata dupa.

**Motiv.** (b). Mediul de dezvoltare al asistentului e Linux, fara SQL Server si
fara fereastra. Daca tot ce e valoros trăiește in spatele unui GUI Windows,
fiecare bug devine o rulare manuala pe masina utilizatorului. Paritatea cu
Python — singurul lucru care permite retragerea versiunii vechi — nu depinde
deloc de interfata, deci nu are sens sa fie blocata de componenta cu cea mai
mare incertitudine de efort.

**Consecinte.** La finalul etapei `cli` exista o aplicatie completa si
utilizabila. `cli` ramane permanent, ca mod de depanare, si dupa ce apare
interfata. Riscul asumat: costul real al editorului de diferente se afla mai
tarziu decat in planul initial.

---

## 2026-11-17 — `SursaObiecte` ca trait, cu implementare din fixture

**Context.** Vezi decizia de mai sus.

**Motiv.** Tot ce e deasupra `db` trebuie testabil fara baza de date. In plus,
`gmotion instantaneu` scrie obiectele brute dintr-o comparare reala intr-un
JSON, care se reia offline — deci un bug de pe o baza de productie devine un
test de regresie.

**Consecinte.** `db` expune un trait, nu un tip concret. Modulele nu stiu
niciodata daca vorbesc cu SQL Server sau cu un fisier.

---

## 2026-11-17 — Portul Rust urmeaza semantica JS, nu `motor.py`

**Context.** Motorul de comparare exista in doua copii divergente. Masurat pe 24
de cazuri: 8 diverg, toata divergenta provenind din segmentare. Detalii in
`referinta/README.md`.

**Motiv.** Portul JS din `ui.py` e varianta mai noua si corecta; `motor.py` a
ramas in urma si produce fals pozitive vizibile utilizatorului (un comentariu
bloc pe mai multe linii il face sa raporteze doua blocuri sterse care nu
exista).

**Consecinte.** Fixture-ul de referinta e `referinta/rezultate_js.json`. Cand
testul de conformitate cade, presupunerea implicita e ca portul Rust e greșit,
nu fixture-ul. Divergentele raman deci intentionate: versiunea Rust **nu** e
identica cu `motor.py`, si asta e corect.

---

## 2026-11-17 — Dependentele lui `gmotion-core`

`regex`, `once_cell`, `md-5`, `sha2`, `hex`. `md-5` doar pentru cheia de bloc,
nu in context de securitate. `serde`/`serde_json` sunt dev-dependencies, fiindca
doar testele de conformitate si exemplul de bench citesc JSON — `core` insusi nu
face I/O si trebuie sa ramana compilabil pe wasm.

---

## Coduri de eroare — registru

Un cod alocat nu se refoloseste si nu se renumeroteaza, ca sa rămână
cautabil in jurnale vechi si in rapoarte de bug.

| Prefix | Zona |
|---|---|
| `GM-CORE-*` | motorul de comparare |
| `GM-DB-*` | conexiune, driver, citire obiecte |
| `GM-PLAT-*` | configurare, sesiuni, rutare |
| `GM-MOD-*` | module |
| `GM-UI-*` | fereastra, randare |

Niciun cod alocat deocamdata: `core` nu intoarce erori tipate (functiile lui
sunt totale pe intrari valide UTF-8).
