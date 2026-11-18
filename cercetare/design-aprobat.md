# Designul aprobat al interfetei native

> **Inlocuit pe 2026-11-18** de directia Harta (lista) + Panza (editor): vezi
> `JURNAL.md` si `cercetare/schita-slint/`. Ramane contractul codului de acum
> pana la portare; principiile despre culoare, litere de stare si zgomot raman.

**Aprobat de Mihai:** 2026-11-17, pe simularea din `cercetare/machete/`.
Asta e contractul vizual pentru `rust/fereastra`. Cercetarea din care vine e
in `cercetare/design-interfata.md`; aici sta doar ce s-a decis.

## Ce a cerut Mihai, in cuvintele lui

- „nu trebuie facuta interfata 1 la 1 [cu v3.4], poti sa fii creativ si sa o
  facem mai frumoasa"
- prima varianta a simularii (paleta GitHub, insigne rotunjite, texte
  explicative) a fost respinsa: „sa nu para facut cu AI-ul"
- „Partea cu ignora mi se pare inutila, asta trb deja filtrata asa" —
  **nu exista comutatoare de zgomot** (spatii, majuscule, comentarii) nici in
  lista, nici in editor. Ce motorul considera zgomot e filtrat implicit; in
  lista ajung doar obiectele care difera cu adevarat.

## Machetele

`cercetare/machete/*.dc.html` — sursa simularii (HTML cu date inline; se
citesc ca specificatie, nu se ruleaza in aplicatie). Cate un ecran, 1440×900:

| Fisier | Ecran |
|---|---|
| `Conectare.dc.html` | Sursa / Baza, autentificare, comparari recente reluabile |
| `Main.dc.html` | Lista de diferente (sus) + previzualizarea obiectului selectat (jos) |
| `Editor.dc.html` | Editorul alaturat, pe tot ecranul |
| `Deploy.dc.html` | Planul de deploy: ordinea, verificarile, confirmarea |
| `Paleta.dc.html` | Paleta de comenzi (Ctrl+K) |

Culorile, dimensiunile si textele exacte se iau din fisierele astea.

## Principii

1. **Unealta de lucru, nu pagina de prezentare.** Dens, linii subtiri (1 px),
   colturi de 3 px, fara umbre, fara degradeuri, fara animatii decorative.
2. **Fara insigne rotunjite.** Starea se spune prin litere si text colorat:
   `M` modificat, `S` doar in sursa, `B` doar in baza; operatiile de deploy
   `ALTER` / `CREATE` / `DROP` ca text mono colorat.
3. **Culoarea nu e singurul semnal.** Fiecare stare are si litera / simbol;
   diferentele au bara in jgheab plus fundal.
4. **Texte scurte, omenesti.** Fara explicatii pentru ce e evident.
   „Baza n-a mai fost atinsa", „Se citeste, nu se scrie nimic".
5. **Tastatura intai, dar discret.** Scurtaturile stau in bara de jos si in
   paleta, nu pe fiecare buton.
6. **Mediul tintei se vede mereu.** Banda de 3 px sus, pe toata latimea
   ferestrei, in culoarea mediului; langa numele bazei `PROD` / `TEST` / `DEV`
   in mono, colorat.

## Jetoane (tema luminoasa)

| Jeton | Valoare | Folosit la |
|---|---|---|
| `fundal` | `#f5f4f0` | fereastra, bara laterala |
| `suprafata` | `#fbfaf7` | antete, lista, panouri |
| `cod-fundal` | `#ffffff` | zona de cod |
| `linie` | `#dedbd3` | separatoare |
| `linie-slaba` | `#ebe8e1` | intre randurile listei |
| `text` | `#1c1b18` | text, butonul principal (fundal) |
| `text-secundar` | `#6b675e` | etichete, contoare |
| `text-tert` | `#9c978b` | schema, numere de linie |
| `selectie` | `#2b4a6f` | randul selectat (text alb) |
| `mediu-prod` | `#b3261e` | banda si eticheta PROD |
| `mediu-test` | `#2b4a6f` | eticheta TEST |
| `modificat` text / fundal / bara / in-linie | `#8a5a00` / `#f8efd9` / `#c48a1a` / `#ecd49a` | |
| `adaugat` text / fundal / bara / in-linie | `#3d6b35` / `#e8f0e0` / `#5a8a4c` / `#c9dfb6` | |
| `sters` text / fundal / bara / in-linie | `#a3392b` / `#f8e6e1` / `#b8543f` / `#efc3b8` | |
| umplutura (linie lipsa) | hasura 135°, `#ffffff` / `#ebe8e1` | partea opusa unui bloc adaugat/sters |

Sintaxa: cuvinte cheie `#2a4d8f`, siruri si numere `#7a3e0f`, variabile
`#6a3d8f`, comentarii `#8a857a` cursiv.

**Contrastul se verifica** (≥ 4,5:1 pentru text normal) cand jetoanele intra in
cod — e criteriu de terminare, nu presupunere.

Tema intunecata si tema pentru daltonism (albastru / portocaliu in loc de
verde / rosu): dupa ce cea luminoasa merge, cu aceleasi nume de jetoane.

## Tipografie

IBM Plex Sans (UI) si IBM Plex Mono (cod, contoare, nume de obiecte, scurtaturi).
Licenta SIL OFL 1.1: se includ in binar (`slint` poate incarca fonturi
incorporate), cu fisierul de licenta alaturi. Rezerva: Segoe UI / Consolas.
Marimi: 13 px baza, 12–12,5 px secundar si cod, 14 px titlul compararii.

## Ecranele

**Lista (ecranul principal).** Antet 48 px: `gmotion`, `Sursa → Baza` +
mediul, serverele si ora compararii, cautare (Ctrl K), Recompara, Deploy cu
numarul din plan. Bara laterala 200 px: progresul revizuirii (linie de 2 px),
categoriile cu contoare, tipurile. Tabelul: litera de stare, obiect (schema
estompata), tip, linii `+N −M`, „Difera in" (cod, siruri, comentariu,
QUOTED_IDENTIFIER — text simplu), bifa de revizuit. Randul selectat: plin
`selectie`. Dedesubt, separat de o linie de 1 px `text`: previzualizarea
obiectului selectat — o propozitie despre ce difera, Revizuit / In plan /
Deschide, diferenta alaturata cu identicul pliat.

**Editorul.** Antet: inapoi la lista, obiectul, „4 diferente, una doar in
comentariu", navigare `2/4`, Revizuit, In plan. Bara: Alaturat / Unificat
(sublinire), „plieaza identicul", „editeaza textul final". Doua jumatati cu
numere de linie reale, bara de 3 px in jgheab (5 px pe blocul curent), fundal
pe linie, caracterele schimbate cu fundal mai tare, hasura pe partea fara
linii, minimap de 14 px. Modelul vine din `gmotion_core` (deja in
`rust/fereastra/src/model_editor.rs`).

**Planul de deploy.** Taburi Ordinea / Atentionari / Script. O propozitie de
rezumat („12 obiecte — 9 modificate, 2 noi, 1 sters"), apoi pasii numerotati in
ordinea dependentelor. In dreapta: verificarile (✓ / !), „salveaza intai
versiunea actuala", si — doar pentru PROD cu DROP — confirmarea prin tastarea
numelui bazei. „Salveaza .sql" mereu disponibil.

**Conectarea.** Doua coloane Sursa / Baza (server, autentificare Windows /
SQL, utilizator, parola, baza, starea conexiunii), butonul de inversare,
certificatul intr-o linie cu „Schimba", Compara. Dedesubt: comparari recente,
reluabile.

**Paleta.** Ctrl K: comenzi si obiecte in aceeasi cautare, potrivirea
subliniata, scurtatura in dreapta.

## Ce nu exista inca in motor (de construit, in afara de UI)

- **„Difera in"**: clasificarea diferentelor unui obiect pe cod / sir /
  comentariu / setari SET. Se face cu `core::lexer` (granitele cod / sir /
  comentariu se decid doar acolo), nu cu o regula noua pe text.
- **Starea de revizuire** legata de amprenta continutului (dispare singura cand
  obiectul se schimba) si **sesiunile reluabile** — in `platforma`.
- **Verificarea „baza n-a mai fost atinsa"** inainte de deploy: recitirea
  obiectelor din plan si compararea amprentelor.
- **Scriptul de revenire** din definitiile actuale, salvat inainte de deploy.
- Fiecare dintre ele trebuie sa fie accesibila si din `cli` (CLAUDE.md §3).
