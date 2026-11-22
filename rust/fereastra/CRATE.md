# gmotion-fereastra

<!-- api-hash: 309db826614366c5 -->

## Rol

Interfata nativa a GMotion, desenata cu Slint (randare software: merge si prin
RDP pe servere fara GPU). Contine ferestrele si componentele, si modelele lor
in Rust (ce se deseneaza pe fiecare rand), testabile fara fereastra.

Aspectul e cel din schita aprobata (`cercetare/schita-slint/`, etapa 13):
conectarea, lista (tabel, harta pe `h`), editorul cu arborele ca Object Explorer in stanga cu doua panouri si planul de deploy,
aceleasi pe ambele moduri; capturile sunt in `cercetare/capturi/`. Jetoanele, in doua palete cu aceleasi nume
(`INTUNECAT`, `LUMINOS`), si fonturile (Geist si Geist Mono, incorporate in
binar) stau in `src/jetoane.rs` — in `.slint` nu se scrie nicio culoare.

Stare: **prototip** — conectarea (etapa 11), lista cu previzualizare (etapa 9)
si editorul de diferente (etapa 8).

## Nu face

- Nu compara si nu decide nimic: blocurile, imperecherea si bucatile schimbate
  vin din `gmotion_core::diff_aliniat`. Aici doar se aseaza pe ecran.
- Nu acceseaza baze de date si nu scrie fisiere.

## API public

| Element | Ce face |
|---|---|
| `model_conectare::Conectare` | ecranul de conectare: campuri, validari (`ce_lipseste`, `ce_lipseste_pentru_baze`), stari, comparari recente; `verifica_certificatul` e alegerea care ajunge in conexiune, `text_certificat()` doar o descrie; `aceeasi_instanta()` (15 F) recunoaste acelasi server sub alte nume; `StareConexiune::SeCauta` = cautarea in retea ruleaza |
| `model_conectare::{Parte, Acreditari, Auth, StareConexiune, Recenta, ServerGasit}` | modelul lui; `Acreditari` **nu are `Debug` derivat**; `Parte::{comuta_baza, baze_alese, e_aleasa}` tin bifele cand se compara mai multe baze; `Recenta::server` nu se deseneaza, dar e jumatatea cheii de sesiune |
| `FereastraConectare` (Slint), `incarca_conectare(ui, c)` | ecranul de conectare (etapa 17, `cercetare/machete/conectare-noua/`): doua carduri simetrice (martor, modificata), fiecare cu conexiunea sus si lista bazelor cu cerc dedesubt; bifa „Aceeasi conexiune"; Compara la mijloc; tabul Sesiuni; Setari (certificatul) |
| `model_conectare::Conectare::{aceeasi_conexiune, pune_aceeasi_conexiune, sincronizeaza, alege_baza_modificata, e_baza_martorului}` | bifa „Aceeasi conexiune": `sincronizeaza` copiaza conexiunea martorului in modificata (se cheama inainte de fiecare desenare); la modificata se alege **o singura** baza, iar baza martorului nu, cand conexiunea e aceeasi |
| `scara::{scara, ScaraFereastra, REFERINTA, MINIM, MAXIM, SISTEM_MARIT}`, `leaga_scara(ui)` | scara unica a conectarii: `min(latime/1440, inaltime/900)` intre 0,85 si 1,3; cu sistemul marit la 125%+ minimul e 1; se pune pe fereastra ca factor de scala (sistem x scara) la fiecare redimensionare |
| `incarca_jurnal(ui, linii)`, `linie_jurnal(l)`, `LINII_JURNAL` | banda de jurnal de jos: ultimele linii, cu nivelul ca culoare |
| `model_lista::Lista` | lista: filtre (stare, baza, tip, revizuire, scor), cautare, sortare pe orice coloana, contoare, selectie, detalii la cerere |
| `model_lista::{Obiect, Stare, Detalii, Ordine, Filtru, Deploy, FiltruRevizuire, Impact}` | modelul listei; cheia unui rand e `(baza, schema.nume)`; `Obiect::{folosit_de, impact}` vin din `diff-obiecte::impact` (fereastra doar le arata, sorteaza si filtreaza: `Ordine::{FolositDe, Impact}`, `Lista::filtreaza_impact`); `Obiect::modificat` si `Ordine::Modificat`: coloana „Ultima modificare", recentele intai (etapa 25) |
| `incarca_coloana_lista(ui, lista)` | coloana ingusta din editor: aceleasi randuri ca lista intreaga, construite intr-un singur loc |
| `ajutor(limba)`, `AJUTOR`, `AJUTOR_EN` | textul ferestrei de ajutor (F1), acelasi pe toate ecranele |
| `text_despre(limba, versiune, jurnal, date)` | textul ecranului „Despre" (din Ajutor, etapa 25): versiunea, licentele componentelor terte, caile jurnalului si datelor; sub el, `AboutSlint` (atribuirea ceruta de licenta Slint) |
| `text_probleme(probleme, limba)`, `StareEditare::probleme` | mesajul „textul final nu se mai compileaza: linia N: ..." din editor, in limba aleasa (etapa 26); `RezultatDeploy::Scris::stricate` |
| „Alege locul" (in `editor.slint`, `alege-*`) | un clic pe un bloc din Martor imparte Modificata in chenarele locurilor (din `loc` al randurilor, ca la tragere); un clic pe chenar cheama `plaseaza`; Esc renunta. Textele `alege_*` in `texte` |
| `leaga_editarea(ui)`, `linii_editare_ui(text)`, `model_editor::{linii_sintaxa, LINII_SINTAXA_LA_SCRIERE}` | sintaxa cat scrii in campul de editare (etapa 25): randuri colorate desenate peste `TextInput`, pozitionate pe caractere si taburi; peste 5.000 de linii, text simplu |
| `limba_curenta(ui)` | limba pusa de ultimul `aplica_limba`; textele facute in Rust o urmeaza (`model_deploy`, editorul, lista, conectarea) |
| `texte::{Limba, Texte, RO, EN, texte}`, `aplica_limba(ui, limba)`, `cu_textele!` | toate textele interfetei, in romana si engleza; `.slint` nu contine text pentru om |
| `FereastraLista` (Slint), `incarca_lista(ui, antet, lista)`, `AntetLista` | ecranul principal |
| `model_lista::Lista::{selecteaza_cheie, chei_vizibile}` | selectia dupa cheie (tabul activat; goleste filtrul daca il ascunde) si ordinea curenta pentru `j` / `k` |
| `RandUi::{continuare, st_lipsa, dr_lipsa}`; in `.slint`, `impacheteaza` si `coloane-wrap` (editor) | wrap: linia lunga continua pe randuri fara numar, rupta la cuvant; partea mai scurta primeste randuri goale, deci stanga si dreapta raman aliniate |
| `model_arbore::{Arbore, Nod, Fel, Dosar}`, `incarca_coloana_lista(ui, lista, arbore)`, `NodUi` | arborele ca Object Explorer, in coloana din stanga editorului: baza › dosar pe tip › obiect, din obiectele vizibile; nodurile inchise raman inchise intre redesenari; `Arbore::cauta` filtreaza doar arborele (cautarea din coloana), cu toate nodurile deschise cat se cauta |
| `harta::{aseaza, aseaza_la, picteaza, picteaza_la, picteaza_bara, picteaza_bara_la, proportii, indice_la, celula_la, coltul_in_grila, pozitie_randului, latime_titlu, Scara, SCARI, INALTIME_MAXIMA}` | harta (`h` din lista, ecran separat): o coloana pe tip, latimi apropiate (`n^0,25`), bara de proportie, patrate pe blocuri de 5 randuri pictate la pixelii fizici, zoom pe trepte ca sa intre in ecran; clic → rand |
| `incarca_harta(ui, selectat)`, `rand_din_harta(ui, grup, x, y)`, `leaga_harta(ui)` | harta in fereastra: deschiderea, reasezarea la redimensionare, bula; calculul patratului e doar in `harta::indice_la` |
| `model_lista::Lista::{pune_detalii_cheie, chei_fara_detalii, detalii_sosite}` | detaliile calculate pe alt fir, puse dupa cheie (filtrul se poate schimba intre timp); `detalii_sosite` resorteaza o lista sortata dupa Linii / Difera in (F-A6) |
| `ordine_din_cod(cod)`, `latime_autofit(lista, coloana)` | sortarea pe capul de coloana si latimea ceruta la dublu clic pe marginea lui |
| `model_lista::Lista::preia_vederea` | filtrul, ordinea, selectia si ce e in plan (daca se mai poate deploia) ale altei liste: recomparare (F-A8) |
| `dublu_clic::DubluClic` | dublul clic numarat in Rust din doua clicuri simple pe aceeasi tinta (500 ms); `double-clicked` din Slint nu ajungea pe Windows |
| `taburi_ui(taburi, editat)`, `incarca_taburi(ui, taburi, editat)`, `TabUi` | bara de taburi a editorului: fixatele, apoi previzualizarea (inclinata); punctul pe cele editate; `+N` cand nu incap |
| `model_taburi::{Taburi, Activ, Pas, Refuz, Cheie, MAX_FIXATE, taie_pe_mijloc}` | taburile editorului (etapa 16): previzualizare + cel mult 8 fixate, `j` / `k` care sar peste fixate, `Ctrl+W` spre vecinul din stanga; numele taiat pe mijloc |
| `FereastraDeploy` (Slint), `incarca_deploy(ui, ecran)` | planul de deploy: pasii in ordinea din script, garda, revenirea, impartirea fisierelor, scrierea |
| `model_deploy::{EcranDeploy, PasDeploy, GardaDeploy, RezultatDeploy, rezumat, text_garda, text_rezultat}` | ce spune ecranul de deploy, fara fereastra; `GardaDeploy::SeVerifica` = recitirea ruleaza pe alt fir (P4) |
| `model_lista::Obiect::{in_plan, se_poate_deploia}`, `Lista::{comuta_in_plan, scoate_din_plan, in_plan}` | ce e pus in planul de deploy |
| `incarca_previzualizare(ui, sursa, baza, setari)`, `goleste_previzualizarea(ui)` | previzualizarea obiectului selectat |
| `incarca_detalii(ui, panou)`, `PanouDetalii` | panoul de detalii din lista (deschis, impact, „Folosit de" / „Foloseste" — primele 8 nume, apasabile, si „+N" —, nota); nota se rescrie doar cand se schimba obiectul |
| `model_editor::construieste_cu(ref, tinta, uneste_cursor)` | acelasi model, cu ciclul de cursor luat intr-un singur bloc |
| `model_editor::construieste(ref, tinta) -> ModelEditor` | randurile editorului: aliniate (prin `gmotion_core::aliniaza`, aceeasi aliniere ca „Aplica"), numerotate, colorate; `blocuri` = blocul motorului al fiecarei diferente; `tinte` = randul catre care se duce fiecare diferenta |

**Unde ajunge un bloc nu se decide aici.** Modelul intreaba
`gmotion_core::plasare::plasari` si doar aseaza raspunsul pe ecran: pastila de
pe bloc (`Rand::eticheta`), randul-fantoma (`Rand::fantoma`) si legatura din
canal (`ModelEditor::tinte`). O a doua parere in fereastra ar insemna ca
„Aplica" scrie in alt loc decat arata ecranul. Cat timp se trage un bloc, se
aprinde **locul calculat de motor**, nu randurile blocului tras: pana la etapa
21 zona de plasare se aprindea acolo unde ar fi cazut blocul pe pozitie. Locul
se tine **pe diferenta** (`tinte-de-la` / `tinte-pana-la`, randuri afisate), nu
pe rand: doua blocuri pot cadea pe acelasi rand-fantoma, iar un singur numar
pe rand il pastra doar pe ultimul.

Locul unde vin blocurile fara corespondent e partea din dreapta a randurilor
lor (`Rand::inserare`; primul are `fantoma` si textul), fara rand separat.
`ModelEditor::grupuri` tine, pentru fiecare diferenta, blocurile motorului pe
care le aplica (`core::grupuri_diferente`); eticheta unui bloc sta pe un rand
al ei (`RandUi::fel == 2`), deasupra blocului, iar legaturile curbe din canal
se deseneaza in Slint din `start-diferente` si `tinte-de-la` / `tinte-pana-la`.

Tragerea (etapa 23): fiecare rand afisat poarta `loc` — ce se intampla cu un
bloc lasat pe el, codificat de `loc_ui` / `loc_din_ui` — si `bloc`, blocul
motorului lui; fereastra cheama `plaseaza(bloc, loc)`, cu `loc` -1 pe locul
sugerat. Fantoma care urmeaza mouse-ul si conturul locului de sub mouse sunt
doar desen; decizia e in `core::aplica_blocuri_la`.

`randuri_ui_pentru_test` (ascunsa din documentatie) da randurile asa cum ajung
in Slint, pentru exemple si probe.
| `model_editor::previzualizare(curent, rezultat, context, max_aduse) -> Vec<LiniePrevizualizare>`, `previzualizare_ui(linii)` | previzualizarea la hover pe sageata din canal: liniile aduse, cu context; `rezultat` vine din `gmotion_core::aplica_blocuri`, deci arata exact ce s-ar scrie. Fereastra cheama `previzualizeaza(k)`, aplicatia pune `previzualizare-linii` |
| `model_editor::plieaza(model, context, limba) -> Vec<RandAfisat>` | sirurile lungi de linii identice, pliate |
| `model_lista`: `Deploy`, `FiltruRevizuire`, `filtreaza_revizuire`, `filtreaza_scor`, `pune_vizibilele_in_plan`, `pune_deploy`, `progres_deploy`; `Ordine` pe orice coloana, sensul in `descrescator()` | lista la paritate cu v3.4 (etapa 14, C — modelul; ecranul inca nu) |
| `model_editor::marcheaza_textul_final(model, baza, final, manual)`, `Marcaj` | liniile din baza aduse din sursa sau scrise de tine |
| `incarca_editarea(ui, &StareEditare)` | starea editarii in editor: omis, text final, Ctrl+Z, modul ghidat, ce copiaza „Copiaza" |
| `model_editor::diferenta_fiecarui_rand(model)` | a cata diferenta e fiecare rand (bara mai lata pe cea curenta) |
| `ModelEditor`, `Rand`, `Parte`, `Bucata`, `Sintaxa`, `Stare`, `RandAfisat` | modelul |
| `jetoane::{Paleta, INTUNECAT, LUMINOS, contrast}` | cele doua palete, cu aceleasi jetoane, si contrastul WCAG |
| `Mod` (`jetoane::Mod`) | intunecat / luminos; `paleta()`, `din_nume("intunecat")`, `nume()`; implicit intunecat |
| `EditorDiferente` (Slint) | fereastra editorului |
| `Tema` (global Slint) | culorile si fonturile, puse din Rust |
| `aplica_tema(ui, mod)` | pune paleta modului in `Tema` (la pornire si la fiecare comutare) |
| `mod_curent(ui)` | modul pus ultima data in `Tema` |
| `tigla_hasura(mod)` | tigla hasurii de pe partea fara linii, in culorile modului |
| `incarca_editor(ui, antet, model)`, `AntetEditor` (cu `id` si `in_plan`) | umple editorul; cand `id` (`baza/schema.nume`) se schimba, **inchide editarea de mana** — altfel campul ramanea cu textul obiectului dinainte (F-A1) |

## Invariante

1. **Un singur motor.** Modelul vine din `diff_aliniat`; nicio logica de
   comparare aici.
2. **Blocurile ocupa aceeasi inaltime in ambele panouri** (umplutura pe partea
   mai scurta), ca in v3.4.
3. **Numerele de linie sunt cele reale** (`linii_start_blocuri`), iar liniile
   pe care segmentarea le arunca (goale, doar-comentariu) apar ca context.
4. **Ambele panouri sunt pe acelasi rand al unui singur `ListView`:**
   derularea e sincronizata fara cod si virtualizata (obiecte uriase).
5. **Sintaxa se coloreaza pe tot blocul**, nu pe linie: string-urile si
   comentariile pe mai multe linii raman colorate corect.
6. **Nicio culoare in `.slint`.** Jetoanele vin din `jetoane.rs`, unde testul
   de contrast le vede, pe ambele palete. `aplica_tema` le pune pe toate,
   dintr-o singura lista (`cu_jetoanele!`): un camp din `Paleta` lipsa din
   lista nu compileaza, iar unul pus gresit face sa cada
   `aplica_tema_pune_fiecare_jeton_in_slint_in_ambele_moduri`.
7. **Cele doua palete au aceleasi nume de jetoane.** Un ecran nu stie in ce
   mod e; daca are nevoie de altceva decat o culoare (ex. umbra), citeste
   `Tema.intunecat`.
8. **Starile se arata in sensul deploy-ului:** ce e doar in sursa se va adauga
   in baza (verde), ce e doar in baza se va scoate (rosu). Motorul le numeste
   invers (fata de referinta); incrucisarea se face intr-un singur loc,
   `cod_stare` din `lib.rs`.
9. **In modul unificat, numerele spun din ce parte vine linia:** doua
   jgheaburi (sursa | baza), fara vreun semn nou — decizia lui Mihai,
   2026-11-17.
10. **Fereastra nu depinde de niciun modul.** Modelul listei primeste randuri
   gata clasificate; `diff-obiecte` apare doar ca dependinta de dezvoltare, in
   exemplele care fac capturi dintr-o comparare adevarata.
11. **Indicii diferentelor sunt in coordonatele listei afisate**, nu ale
   modelului: plierea scoate randuri, iar navigarea deruleaza lista.
12. **Scara conectarii nu se bate cu marirea din sistem.** `ScaraFereastra`
   tine minte factorul pus de ea; altul inseamna ca l-a schimbat sistemul
   (alt monitor, alta marire), iar scara se recalculeaza de la el.
13. **Cu „Aceeasi conexiune", modificata nu are conexiune a ei.** Ce se vede
   e o copie a martorului, refacuta de `sincronizeaza` inainte de fiecare
   desenare; restul aplicatiei vede tot doua parti complete.

## Erori tipice

| Simptom | Unde caut |
|---|---|
| Panourile nu mai sunt aliniate | `src/model_editor.rs`, `construieste` |
| Numere de linie gresite | `linii_start_blocuri` in `core`, capcana despre blocuri unite |
| O bucata schimbata nemarcata | `schimbat_pe_caractere` si `diff_intra` din `core` |
| Fereastra nu se deseneaza pe un server fara GPU | backend-ul: `winit` + `software` |
| Wrap-ul rupe prea devreme / prea tarziu | `CARACTER_MONO` din `lib.rs` si `coloane-wrap` din `.slint` (aceeasi latime de caracter) |
| Dublu clicul nu face nimic (deschide, tab nou, autofit) | `src/dublu_clic.rs` si handlerul din `app/src/main.rs`; in `.slint` nu se foloseste `double-clicked` |

## Testare

`cargo test -p gmotion-fereastra` — modelul, jetoanele (contrast, pe ambele moduri) si fereastra
fara ecran (jetoanele puse, randurile, plierea, unificatul).

Vizual: exemplul `captura` randeaza editorul fara ecran intr-un PNG:

```
cargo run --release -p gmotion-fereastra --example captura -- \
    referinta/instantanee/testsimilitudine.json CharismaERP_GetsBet CharismaERP_GetsBet_Copy \
    dbo.TESTSIMILITUDINE_AnalizaVanzari iesire.png [--unificat] [--fara-pliere]
```

Cu `--font-mono UnNumeInexistent` iese alt PNG: asa se vede ca fonturile
incorporate chiar se folosesc, nu rezerva de sistem.

Conectarea: `--example captura_conectare`. Ecranul listei se fotografiaza din
`app` (`-p gmotion-app --example captura_lista`): captura trece prin
`Aplicatie`, ca sa nu existe o a doua constructie a listei.

Pe obiecte uriase: `cargo run --release -p gmotion-fereastra --example masura`
(implicit 200 / 2.000 / 20.000 de linii, pliat si nepliat); pe liste uriase:
`--example masura_lista` (1.000 / 10.000 / 50.000 de obiecte). Capturi si cifre:
`cercetare/machete/capturi/`, `etape/08-prototip-editor.md`.

Referinta v3.4 (continut, nu aspect): `referinta/captura_v34/`.

## Depanare

`SLINT_DEBUG_PERFORMANCE=refresh_lazy,console` arata timpii de randare (opreste
randarea partiala cat masoara).
