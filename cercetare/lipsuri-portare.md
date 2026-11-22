# Ce lipseste din portare, fata de v3.4

Inventar facut pe 2026-11-18 si refacut pe cod pe 2026-11-22 (etapa 25), la cererea lui Mihai: **intai functionalitatile,
apoi aspectul vizual**. Comparat codul din `mostenire/` (v3.4) cu `rust/`.
Afirmatiile despre butoane nelegate sunt verificate in cod (handler `on_*`
lipsa in `rust/app/src/main.rs`), nu doar deduse.

Cand o lipsa e rezolvata, randul se sterge de aici, in acelasi commit cu
codul. Cand se decide abandonarea uneia, randul se muta in `JURNAL.md`, cu
motivul.

Legenda coloanei „Rust": **GUI** = `gmotion-ui`; **CLI** = `gmotion`.

---

## 0. Butoane care exista pe ecran, dar nu fac nimic

Callback declarat in `.slint`, fara handler in `rust/app/src/main.rs`. Sunt
primele de reparat: un buton mort e mai rau decat unul lipsa.

Rezolvate in etapa 13: Recompara, Deploy (N), In plan, Alaturat / Unificat,
pliaza identicul au acum actiune; eticheta „Ctrl K" nu se mai deseneaza.
„Editeaza textul final" nu se mai deseneaza pana nu exista tinta editabila
(sectiunea 4); callback-ul a ramas declarat in `editor.slint`, fara buton.

Bug-urile gasite la rularea lui Mihai pe Windows (lista de baze pe
instantaneu, randul certificatului pe instantaneu) si certificatul ales in
ecran care nu ajungea in conexiune: rezolvate in etapa 14.

## 1. Conectare si servere

Facute in etapa 25: progres si Anuleaza la Recompara, pe lista; dialogul
nativ pentru instantaneu (`rfd`).

| Functie v3.4 | Unde in v3.4 | Rust | Nota |
|---|---|---|---|

## 3b. Facute in etapa 25

Coloana „Ultima modificare" (si in `--json` si CSV), sintaxa colorata cat
scrii in editare, alegerea folderului de deploy si „Deschide folderul",
ecranul „Despre" (din Ajutor).

## 2. Comparare offline

Facuta (lipire de cod, `pornire::compara_offline`, ecranul offline din
conectare).

## 3. Lista de rezultate

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|

Rezolvate in etapa 14: coloane redimensionabile cu autofit la dublu clic,
Ajutor (F1), filtrul pe baza, Ctrl+F in cautare, filtrul pe stare (sase cazuri),
intervalul de similaritate si coloana Similar, sortarea pe fiecare coloana in
ambele sensuri, coloana Deploy, contorul „% deploy", Export CSV al listei
filtrate, „Toate in plan" cu optiunea pentru cele deja trimise. O referinta
comparata cu mai multe baze (bife in conectare), cu filtrul si coloana Baza.
Compararea din conectare pe alt fir: mesaj, bara, timp, Anuleaza, formularul
blocat cat ruleaza. Deconecteaza (din lista, cu confirmare), Sterge in banda
de jurnal, etichetele CLUSTER / IMPLICITA (si in `gmotion servere`), Testeaza
conexiunea cu detaliile serverului (neverificat inca pe server real).

## 4. Editorul de diferente

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|

Rezolvate in etapa 14: tinta editabila (panoul Baza, „Editeaza", `e`),
Aplica bloc (sageata din canal), Aplica toate, Reset, Ctrl+Z, modul ghidat
(Din sursa / Din tinta / Editeaza / Aplica / Sari, cu final), Omite (`o`),
Copiaza sursa / rezultatul, Ascunde diferentele, Re-sincronizeaza, deploy-ul
cu textul final, drag & drop de blocuri cu „Plaseaza aici", liniile de
legatura din canal, previzualizarea la hover in modul unificat si „Uneste
ciclul cursor". Evidentierea de sintaxa **in timp ce scrii** lipseste:
editarea de mana e text simplu, culorile revin la salvare.

Etapa 21 le-a refacut **in formatul nou**, pe plasarea din motor: eticheta
pe bloc, legatura din canal, randul-fantoma, locul aprins la tragere,
previzualizarea la hover pe sageata (ce ar scrie „Aplica"), modul ghidat si
tragerea scriu acolo unde arata ecranul (`core::aplica_blocuri`).

Exista si merg: navigarea intre diferente (`n`/`p`), obiect anterior/urmator
(`j`/`k`), Esc inapoi la lista, Revizuit (bifa legata de continut, mai buna
decat in v3.4), minimap, derulare sincronizata, diferentele in interiorul
liniei. Alaturat/unificat si plierea **se deseneaza** corect, dar nu se pot
comuta (sectiunea 0).

## 5. Deploy

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|

Facute: „Toate in plan" cu cele deja trimise, starea „deployat" tinuta in
sesiune pe amprenta (`Aplicatie::marcheaza_deployate`), planul pastrat la
recomparare (etapa 24).

Nou fata de v3.4, in CLI si (din etapa 13) in fereastra: planul cu
amprente, garda „baza n-a mai fost atinsa", scriptul de revenire, ordinea pe
dependente. Doar in CLI: garda de versiune, setarile `SET` refacute.

## 6. Sesiuni, setari, fereastra

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|
| 6 teme, inclusiv intunecate | `ui.py:1075-1085` | doua moduri, intunecat si luminos | decizia lui Mihai: aceeasi paleta, doua moduri |

Facute: „Recente" pe conectare cu progres, Reia si Ignora (sterge sesiunea);
indicatorul „salvat la 14:52" in editor; ecranul complet (F11, pe toate
ecranele).

---

## Ce ar observa Mihai primul, in ordine

1. Butoanele moarte (sectiunea 0).
2. Tinta editabila, aplicarea blocurilor, modul ghidat: fluxul central din v3.4.
3. Deploy-ul din fereastra.
4. Cautarea serverelor in retea.
5. O referinta comparata cu mai multe baze.
6. Progres si Anuleaza la comparare.
7. Starea „Omis", starea de deploy, filtrele pe stare.
8. Export CSV si „Copiaza" din fereastra.
9. Scorul, filtrul de similaritate, sortarea pe orice coloana.
10. Compararea offline prin lipire de cod.
