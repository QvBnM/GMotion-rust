# gmotion-fereastra

<!-- api-hash: d2157e49f311e04f -->

## Rol

Interfata nativa a GMotion, desenata cu Slint (randare software: merge si prin
RDP pe servere fara GPU). Contine ferestrele si componentele, si modelele lor
in Rust (ce se deseneaza pe fiecare rand), testabile fara fereastra.

Aspectul e cel din schita aprobata (`cercetare/schita-slint/`, etapa 13):
conectarea, lista cu harta, editorul cu doua panouri si planul de deploy,
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
| `FereastraConectare` (Slint), `incarca_conectare(ui, c)` | ecranul de conectare: doua carduri (sursa, baza), cautarea serverelor ca iconita, comparari recente, butonul de tema |
| `incarca_jurnal(ui, linii)`, `linie_jurnal(l)`, `LINII_JURNAL` | banda de jurnal de jos: ultimele linii, cu nivelul ca culoare |
| `model_lista::Lista` | lista: filtre (stare, baza, tip, revizuire, scor), cautare, sortare pe orice coloana, contoare, selectie, detalii la cerere |
| `model_lista::{Obiect, Stare, Detalii, Ordine, Filtru, Deploy, FiltruRevizuire}` | modelul listei; cheia unui rand e `(baza, schema.nume)` |
| `incarca_coloana_lista(ui, lista)` | coloana ingusta din editor: aceleasi randuri ca lista intreaga, construite intr-un singur loc |
| `ajutor(limba)`, `AJUTOR`, `AJUTOR_EN` | textul ferestrei de ajutor (F1), acelasi pe toate ecranele |
| `limba_curenta(ui)` | limba pusa de ultimul `aplica_limba`; textele facute in Rust o urmeaza (`model_deploy`, editorul, lista, conectarea) |
| `texte::{Limba, Texte, RO, EN, texte}`, `aplica_limba(ui, limba)`, `cu_textele!` | toate textele interfetei, in romana si engleza; `.slint` nu contine text pentru om |
| `FereastraLista` (Slint), `incarca_lista(ui, antet, lista)`, `AntetLista` | ecranul principal |
| `ordine_din_cod(cod)` | ce coloana a apasat utilizatorul; inversul codurilor din `global Sortare` (`lista.slint`) |
| `model_lista::Lista::{selecteaza_cheie, chei_vizibile}` | selectia dupa cheie (tabul activat; goleste filtrul daca il ascunde) si ordinea curenta pentru `j` / `k` |
| `taburi_ui(taburi, editat)`, `incarca_taburi(ui, taburi, editat)`, `TabUi` | bara de taburi a editorului: fixatele, apoi previzualizarea (inclinata); punctul pe cele editate; `+N` cand nu incap |
| `model_taburi::{Taburi, Activ, Pas, Refuz, Cheie, MAX_FIXATE, taie_pe_mijloc}` | taburile editorului (etapa 16): previzualizare + cel mult 8 fixate, `j` / `k` care sar peste fixate, `Ctrl+W` spre vecinul din stanga; numele taiat pe mijloc |
| `harta::{aseaza, aseaza_la, picteaza, picteaza_bara, picteaza_la, picteaza_bara_la, proportii, indice_la, celula_la, coltul_in_grila, pozitie_randului, latime_titlu, Scara, SCARI, INALTIME_MAXIMA}`, `LATIME_HARTA` | harta de deasupra tabelului: o coloana pe tip, care umplu cartusul; bara de proportie a starilor; patrate pictate in Rust, in ordinea din tabel; zoom automat pe trepte (`SCARI`) cand nu intra in `INALTIME_MAXIMA`; clic → rand |
| `rand_din_harta(ui, grup, x, y)`, `leaga_harta(ui)` | clicul, bula si reasezarea la redimensionare; calculul e doar in `harta::indice_la` |
| `FereastraDeploy` (Slint), `incarca_deploy(ui, ecran)` | planul de deploy: pasii in ordinea din script, garda, revenirea, impartirea fisierelor, scrierea |
| `model_deploy::{EcranDeploy, PasDeploy, GardaDeploy, RezultatDeploy, rezumat, text_garda, text_rezultat}` | ce spune ecranul de deploy, fara fereastra; `GardaDeploy::SeVerifica` = recitirea ruleaza pe alt fir (P4) |
| `model_lista::Obiect::{in_plan, se_poate_deploia}`, `Lista::{comuta_in_plan, scoate_din_plan, in_plan}` | ce e pus in planul de deploy |
| `incarca_previzualizare(ui, sursa, baza, setari)`, `goleste_previzualizarea(ui)` | previzualizarea obiectului selectat |
| `incarca_detalii(ui, panou)`, `PanouDetalii` | panoul de detalii din lista (deschis, impact, nota); nota se rescrie doar cand se schimba obiectul |
| `model_editor::construieste_cu(ref, tinta, uneste_cursor)` | acelasi model, cu ciclul de cursor luat intr-un singur bloc |
| `latime_autofit(lista, coloana)` | latimea ceruta de o coloana la dublu clic pe marginea capului |
| `model_editor::construieste(ref, tinta) -> ModelEditor` | randurile editorului: aliniate (prin `gmotion_core::aliniaza`, aceeasi aliniere ca „Aplica"), numerotate, colorate; `blocuri` = blocul motorului al fiecarei diferente |
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

## Erori tipice

| Simptom | Unde caut |
|---|---|
| Panourile nu mai sunt aliniate | `src/model_editor.rs`, `construieste` |
| Numere de linie gresite | `linii_start_blocuri` in `core`, capcana despre blocuri unite |
| O bucata schimbata nemarcata | `schimbat_pe_caractere` si `diff_intra` din `core` |
| Fereastra nu se deseneaza pe un server fara GPU | backend-ul: `winit` + `software` |

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
