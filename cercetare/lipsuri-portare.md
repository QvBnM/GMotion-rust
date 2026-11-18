# Ce lipseste din portare, fata de v3.4

Inventar facut pe 2026-11-18, la cererea lui Mihai: **intai functionalitatile,
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

| Functie v3.4 | Unde in v3.4 | Rust | Nota |
|---|---|---|---|
| Etichete CLUSTER / IMPLICITA in lista de servere | `ui.py:1737` | nu | |
| „Testeaza conexiunea" cu detalii server (editie, OS, uptime, nr. baze) | `app.py:320`, `ui.py:1764` | partial: se citesc bazele, fara detalii | |
| **O referinta comparata cu mai multe baze** (bife) | `ui.py:1812-1877` | GUI **nu** (1 la 1); CLI da (`--baze A,B,C`) | modelul 1 la 1 a venit cu machetele, fara decizie scrisa |
| Progres, timp, mesaje, **Anuleaza** | `app.py:363`, `ui.py:1933` | GUI **nu**: fereastra sta blocata (~12 s pe baza reala) | intrebarea 3 din `STARE.md` |
| „Sterge log" in banda de jurnal | `ui.py:5043` | nu: banda exista (etapa 13), fara stergere | `platforma::jurnal` n-are inca golire |
| Blocarea formularului cat ruleaza, „Deconecteaza" cu confirmare | `ui.py:1980,2056` | nu: din lista nu te mai intorci la conectare | |
| Deschiderea unui instantaneu din fereastra | — (nou) | GUI doar prin `--sursa` la pornire | asteapta decizia pe `rfd` |

## 2. Comparare offline

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|
| Lipesti doua bucati de cod SQL si le compari (Lipeste / Sterge) | `ui.py:1547,3116` | nu, nici in CLI | |

## 3. Lista de rezultate

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|
| Ctrl+F in cautare | `ui.py:5002` | nu (vezi „Ctrl K" mai sus) | |
| Filtru pe baza | `ui.py` `fBaza` | nu (o singura baza) | depinde de „mai multe baze" |
| Filtru pe stare: nerezolvate / rezolvate / omise / deployate / nedeployate | `ui.py` `fStare` | nu | |
| Interval de similaritate min-max % | `ui.py:1116` | nu; **scorul nu apare deloc in GUI** | CLI il arata in `compara` |
| Sortare pe 8 coloane | `ui.py:2393` | partial: doar Nume / Linii ↓ (`model_lista::Ordine`) | |
| Coloane redimensionabile, autofit la dublu clic | `ui.py:2396-2440` | nu | |
| Coloana „Ultima modificare" | `ui.py:2529` | nu, nici in CLI | trebuie citita si din `sys.objects.modify_date` |
| Coloana Deploy (pregatit / deployat / versiune noua / omis / extra) | `ui.py:2533-2549` | nu | |
| Contoare „rezolvate %" si „deploy %" | `ui.py:1054-1059` | partial: doar revizuirea | |
| **Export CSV** al listei filtrate | `app.py:494`, `ui.py:4936` | GUI nu; CLI da (`gmotion export`) | |
| Legenda / Ajutor | `ui.py:1071,2047` | nu | |

## 4. Editorul de diferente

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|
| Drag & drop de blocuri, „Plaseaza aici" | `ui.py:1495,3975` | nu | |
| Previzualizare la hover in modul inline | `ui.py:4385` | nu | |
| Liniile de legatura dintre blocuri, in jgheab | `ui.py:3634` | nu (minimap si derulare sincronizata exista) | |

Rezolvate in etapa 14: tinta editabila (panoul Baza, „Editeaza", `e`),
Aplica bloc (sageata din canal), Aplica toate, Reset, Ctrl+Z, modul ghidat
(Din sursa / Din tinta / Editeaza / Aplica / Sari, cu final), Omite (`o`),
Copiaza sursa / rezultatul, Ascunde diferentele, Re-sincronizeaza, deploy-ul
cu textul final. Evidentierea de sintaxa **in timp ce scrii** lipseste:
editarea de mana e text simplu, culorile revin la salvare.

Exista si merg: navigarea intre diferente (`n`/`p`), obiect anterior/urmator
(`j`/`k`), Esc inapoi la lista, Revizuit (bifa legata de continut, mai buna
decat in v3.4), minimap, derulare sincronizata, diferentele in interiorul
liniei. Alaturat/unificat si plierea **se deseneaza** corect, dar nu se pot
comuta (sectiunea 0).

## 5. Deploy

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|
| „Include si ce a fost deja trimis la deploy" | `ui.py:1161` | nu: starea „deployat" nu exista | |
| Alegerea folderului si deschiderea lui dupa scriere | `app.py:660` | CLI `-d CALE`; GUI scrie in directorul de date, fara alegere | asteapta decizia pe `rfd` (selectorul de fisiere) |
| Starea „deployat" tinuta minte | `ui.py:2533-2549` | nu: ce e „in plan" se pierde la inchidere | |

Nou fata de v3.4, in CLI si (din etapa 13) in fereastra: planul cu
amprente, garda „baza n-a mai fost atinsa", scriptul de revenire, ordinea pe
dependente. Doar in CLI: garda de versiune, setarile `SET` refacute.

## 6. Sesiuni, setari, fereastra

| Functie v3.4 | Unde | Rust | Nota |
|---|---|---|---|
| Tab „Sesiuni": lista cu progres, Reia / Ignora, insigna | `ui.py:869,2218-2310`, `app.py:245-318` | partial: „Recente" pe ecranul de conectare; nu se pot sterge | |
| Indicatorul „salvat" | `ui.py:2313` | nu (se salveaza tacit) | |
| 6 teme, inclusiv intunecate | `ui.py:1075-1085` | partial: doua moduri, intunecat si luminos, cu butonul de tema (etapa 13) | decizia lui Mihai: aceeasi paleta, doua moduri |
| Ecran complet | `app.py:695-870` | nu | |
| „Despre" | — | nu | **cerut de licenta Slint** (vezi `JURNAL.md`) |

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
