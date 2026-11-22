# Etapa 28 — tabelele: existenta, coloane si tipuri (doar comparare)

**Stare:** INCHEIATA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** 2026-11-22

## Scop

Mihai: sa intre si tabelele — care exista si care nu, si daca difera
coloanele sau tipul coloanelor. Ales: doar comparare, fara deploy.

La final, tabelele apar in lista ca orice obiect (doar in Martor, doar in
Modificata, difera), iar in editor se vad coloanele lipsa, in plus si cele cu
alt tip / NULL / IDENTITY; deploy-ul le refuza explicit.

**Cum, ca sa ramana un singur motor:** un tabel devine un `Obiect` de tip `U`
cu definitia construita din `sys.columns`: `CREATE TABLE [s].[t] (` apoi o
coloana pe linie, in ordinea din tabel (`column_id`). Compararea, lista,
editorul, filtrele si „Folosit de" lucreaza pe ea fara cod nou.

## Criterii de terminare

- [x] `db`: tabelele citite din `sys.tables` / `sys.columns` / `sys.types`,
      si serial, si in paralel; definitia construita de o functie pura, cu
      teste pe tipuri (`varchar(50)`, `nvarchar(max)`, `decimal(18,2)`,
      `datetime2(3)`, `int`, `IDENTITY`, `NULL` / `NOT NULL`); data crearii
      si a modificarii ca la celelalte
- [x] `core`: tipul `U` are nume („tabel" / „table") si intra in tipurile
      cunoscute; in lista, filtrul pe tip arata „Tabele"
- [x] comparare pe instantaneu: tabel doar intr-o parte, coloana lipsa,
      coloana in plus, tip schimbat, NULL schimbat — fiecare iese diferenta,
      cu linia coloanei; un tabel identic iese identic
- [x] deploy-ul nu scrie tabele: sar (ca extra), cu motivul, in `cli` si in
      fereastra; „In plan" e refuzat pentru ele
- [x] captura listei si a editorului pe un tabel, 1600x1000, ambele teme
- [x] poarta verde; `STARE.md` (proba pe server real), `JURNAL.md`

## Pasi

1. `db` (interogarea, definitia sintetica).
2. `core` / fereastra (numele tipului).
3. Deploy-ul care refuza tabelele.
4. Instantaneul de previzualizare cu tabele, capturi.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-22** Facute. Interogarea de coloane ruleaza dupa cea de obiecte,
  pe aceeasi conexiune (si in citirea paralela). In coloana din stanga a
  editorului, tabelele au dosarul lor, „Tabele". Neprobat pe SQL Server
  real: proba in `STARE.md`.

## Ce a iesit altfel decat planul

- Un `CREATE TABLE` e o singura instructiune pentru motor: in editor
  alinierea pe linii arata exact coloana schimbata, dar coloana „Linii" din
  lista numara tot tabelul (`+6 −5` pentru o coloana lipsa), ca la orice
  instructiune modificata. Ramas pentru Mihai: numararea pe linii, pentru
  toate obiectele, sau doar pentru tabele.
- Sageata „Aplica" ramane si pe tabele: schimba doar textul final, care nu
  ajunge in deploy (tabelele sunt sarite).
