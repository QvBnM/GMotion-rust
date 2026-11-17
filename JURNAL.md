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

## 2026-11-17 — Driver SQL Server: `tiberius`

**Context.** Decizia deschisa din etapa 0. Versiunea Python foloseste ODBC si
cere un driver instalat pe masina tinta.

**Alternative.** (a) `odbc-api` — comportament identic cu azi, dar rămâne
dependenta de driver; (b) `tiberius` — TDS nativ, pur Rust.

**Motiv.** (b), si nu pentru argumentul din plan. Scoaterea ODBC de pe masina
tinta era stiuta. Ce a cantarit mai mult: `tiberius` **se compileaza pe Linux**,
in 15 secunde, deci stratul de date poate fi scris si verificat pe masina pe
care se scrie codul. `odbc-api` ar cere headerele de unixODBC, deci ar fi fost
cod pe care nu-l pot nici compila. Verificat intr-un proiect de proba inainte de
a decide: `tiberius` are SQL auth, `AuthMethod::Integrated` prin SSPI pe Windows
(feature `winauth`), si descoperire prin SQL Browser — acopera tot ce face
versiunea Python.

**Consecinte.** Testul pe server real rămâne necesar, dar ca **validare**, nu ca
alegere, si nu blocheaza etapele urmatoare. Daca validarea cade pe
autentificarea integrata intr-un domeniu, alternativa nu e intoarcerea la ODBC
ci `integrated-auth-gssapi`.

---

## 2026-11-17 — TLS: `native-tls` pe Windows, `rustls` pe Linux

**Context.** `tiberius` cere un furnizor de TLS.

**Motiv.** Tinta de rulare e si un Server 2012 curat, unde SQL Server poate
vorbi doar TLS 1.0/1.1. `rustls` nu coboara sub TLS 1.2, deci ar refuza exact
serverele vechi pe care rescrierea trebuie sa mearga. `native-tls` pe Windows e
SChannel, deja in sistem, fara dependinta de instalat.

Pe Linux (doar dezvoltare si teste) se foloseste `rustls`, fiindca `native-tls`
ar cere headerele de OpenSSL. Selectia e pe `[target.'cfg(windows)']` in
`rust/db/Cargo.toml`.

**Consecinte.** Codul de conectare e acelasi; doar furnizorul difera. O
conectare care merge pe Linux nu dovedeste nimic despre TLS pe Windows, si
invers.

---

## 2026-11-17 — Un tip care tine acreditari nu are `Debug` derivat

**Context.** Un test avea nevoie de `Debug` pe `SursaSqlServer`. Reflexul ar fi
fost `#[derive(Debug)]`.

**Motiv.** Ar fi tiparit parola la primul `dbg!` si in orice mesaj de panica.
Scris de mana, cu redactare (`parola` devine numarul de caractere), plus test
care verifica. `Autentificare` nu implementeaza `Serialize` deloc, tot
deliberat.

**Consecinte.** Regula generala, de aplicat la `platforma` cand ajunge la
sesiuni.

---

## 2026-11-17 — `db` intoarce text brut, nenormalizat

**Context.** `aduce_obiecte()` din versiunea Python normaliza, calcula hash-ul
si tinea token-ii la citire, pentru fiecare obiect din fiecare baza.

**Motiv.** Memoria era motivul initial (punctul 3 din `README.md`). Dar e si un
motiv mai bun: daca `db` ar intoarce text deja normalizat, un instantaneu ar fi
legat de versiunea de motor care l-a produs, deci nefolositor ca test de
regresie dupa o schimbare in `core`.

**Consecinte.** `Obiect` tine doar `schema`, `nume`, `tip`, `definitie`.
Normalizarea o face cine are nevoie de ea.

---

## Coduri de eroare — registru

Un cod alocat nu se refoloseste si nu se renumeroteaza, ca sa rămână
cautabil in jurnale vechi si in rapoarte de bug.

| Prefix | Zona |
|---|---|
| `GM-CORE-*` | motorul de comparare |
| `GM-DB-*` | conexiune, driver, citire obiecte — **alocate 001-010**, vezi `rust/db/CRATE.md` |
| `GM-PLAT-*` | configurare, sesiuni, rutare |
| `GM-MOD-*` | module |
| `GM-UI-*` | fereastra, randare |

`core` nu intoarce erori tipate: functiile lui sunt totale pe intrari valide
UTF-8.

`GM-DB-001` conectare · `002` autentificare · `003` auth integrata indisponibila
· `004` interogare · `005` baza lipsa · `006` descoperire · `007` citire
instantaneu · `008` scriere instantaneu · `009` versiune instantaneu · `010`
nume invalid.
