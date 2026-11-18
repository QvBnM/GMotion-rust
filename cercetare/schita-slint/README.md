# Schita in Slint: conectarea, lista si editorul

**Ce dovedeste:** noua directie vizuala se poate desena in Rust, cu Slint 1.19
si randare software, exact cum ruleaza aplicatia prin RDP fara GPU. Cele trei
ecrane sunt legate intre ele si **pastreaza toate functiile din v3.4**:

| Ecran | Directia | Captura |
|---|---|---|
| Conectarea | intunecat, ca harta | `capturi/conectare.png`, `capturi/conectare-progres.png` |
| Lista | harta sus, tabelul din v3.4 dedesubt | `capturi/lista.png` |
| Editorul | panza alba, cu butoanele din v3.4 | `capturi/panza.png`, `capturi/panza-ghidat.png`, `capturi/panza-paleta.png` |

Ce le leaga: acelasi semn g•motion, acelasi drum „sursa → baza", aceleasi
etichete de mediu (DEV / TEST / PROD), aceleasi culori de stare (chihlimbar
modificat, verde adus sau lipsa, rosu extra, violet doar SET), acelasi font
(Geist si Geist Mono) si aceleasi scurtaturi in subsol si pe butoane.

Diferenta din editor vine din motorul adevarat (`model_editor::construieste`),
pe `referinta/instantanee/testsimilitudine.json`. Similaritatea calculata de
Rust pe `AnalizaVanzari` e 89,97%, ca in v3.4. Lista foloseste o baza
generata de 21.183 de obiecte.

**Nu e cod livrat.** Crate-ul nu e in workspace-ul aplicatiei si nu trece prin
poarta. La portare, culorile din `ui/comune.slint` se muta in
`rust/fereastra/src/jetoane.rs` (regula „nicio culoare in `.slint`").

```bash
cd cercetare/schita-slint
B="cargo run --release --offline --"
export CARGO_TARGET_DIR=../../rust/target
$B conectare capturi/conectare.png
$B conectare capturi/conectare-progres.png --progres
$B lista     capturi/lista.png
$B panza     capturi/panza.png
$B panza     capturi/panza-ghidat.png --ghidat
$B panza     capturi/panza-paleta.png --paleta
```

## Masuratori (Linux, release, randare software, 1440×900)

| Ecran | Pregatire | Primul cadru |
|---|---|---|
| Conectarea | — | 9–11 ms |
| Lista (336 de randuri, 8 grupuri pe harta) | 0,5 ms pictura hartii | 12 ms |
| Editorul (71 de randuri, 4 blocuri) | 8–10 ms modelul din motor | 16–19 ms |
| Editorul cu paleta Ctrl K deschisa | idem | 25 ms |

## Fiecare functie din v3.4, si unde e acum

„Desenat" = se vede in capturi. „Locul stabilit" = are loc pe ecran si e
descris aici, dar schita nu-l deseneaza inca.

### Conectarea

| Functie v3.4 | Unde | Stare |
|---|---|---|
| SQL Auth / Windows | comutatorul din coloana serverului | desenat |
| Server SQL + cautarea in retea | camp + „Cauta in retea"; rezultatele dedesubt | desenat |
| Etichetele CLUSTER / IMPLICITA | pe fiecare server gasit | desenat |
| Utilizator, Parola | sub server; dispar la Windows | desenat |
| Testeaza conexiunea + detaliile serverului (editie, OS, uptime, nr. baze) | buton + starea dedesubt | desenat |
| Lista bazelor, referinta (o singura) + bazele comparate (bife) | coloana din mijloc, cu mediul fiecarei baze | desenat |
| Porneste verificarea | butonul mare chihlimbar, cu ce se compara | desenat |
| Comparare offline (copy/paste) | buton langa pornire | desenat; fereastra cu cele doua campuri si „Lipeste": locul stabilit |
| Progres, timp, mesaj, **Anuleaza** | in locul butoanelor, cat ruleaza | desenat (`--progres`) |
| Conexiune activa → Deconecteaza | sub progres | desenat |
| Jurnalul (Output backend) + Sterge log | coloana din dreapta | desenat |
| Tab Sesiuni, cu insigna | tab langa Conectare | desenat (tabul); lista cu progres, Reia / Ignora: locul stabilit |
| Banner „Sesiune gasita" + Anuleaza | sus in coloana serverului | desenat |
| Legenda / Ajutor | „Ajutor F1" in antet | locul stabilit |
| *(nou in Rust)* certificatul serverului, cu Schimba | sub starea conexiunii | desenat |
| *(nou in Rust)* deschiderea unui instantaneu | buton langa pornire | desenat |

### Lista

| Functie v3.4 | Unde | Stare |
|---|---|---|
| Contoarele dif / lipsa / ok | antet, cu culoarea starii (si „extra", separat) | desenat |
| % rezolvate, % deploy | antet, cu bara | desenat |
| Deconectare, Ajutor, Tema | antet, dreapta | desenat |
| Cautare (Ctrl+F) | primul camp din bara de filtre | desenat |
| Filtrele: baza, tip, categorie, stare | liste derulante in bara de filtre | desenat |
| Similaritate min–max % | in bara de filtre | desenat |
| Export CSV | in bara de filtre | desenat |
| Deployer (N) | dreapta, chihlimbar | desenat |
| Coloanele #, Obiect, Baza, Tip, Similaritate (bara + %), Diferente, Stare, Ultima modif., Deploy, Actiune (Diff) | tabelul | desenat |
| Sortare pe coloane | sageata pe coloana sortata | desenat (o coloana); restul: acelasi mecanism |
| Coloane redimensionabile, autofit | marginea capului de coloana | locul stabilit |
| Starea: rezolvat / nerezolvat / omis | cercul din coloana Stare | desenat |
| Deploy: pregatit / trimis / versiune noua / omis / neintrodus | textul colorat din coloana Deploy | desenat |
| Numarul de obiecte afisate | subsol | desenat |
| *(nou)* harta pe grupuri, legata de tabel | intre filtre si tabel; patratul selectat = randul selectat | desenat |

### Editorul

| Functie v3.4 | Unde | Stare |
|---|---|---|
| Inapoi la Lista | insula din stanga sus, `Esc` | desenat |
| Obiectul anterior / urmator (17/18) | aceeasi insula, `PgUp` / `PgDn` | desenat |
| Am ajuns la o concluzie | insula din dreapta sus, `r` | desenat |
| Omite | aceeasi insula, `o` | desenat |
| Similaritatea, cate modificate / adaugate / sterse | propozitia de sub titlu | desenat |
| „Modificarile se aplica in timp real" + ora salvarii | dreapta, sub titlu | desenat |
| Aplica toate, Reset | bara de jos | desenat |
| Diferenta anterioara / urmatoare (1/44) | bara de jos, `⇧F7` / `F7` | desenat |
| Mod ghidat, cu cate au ramas | bara de jos, `g` | desenat |
| Side-by-side / Inline | bara de jos: Alaturat / Unificat, `u` | desenat (comutatorul); randarea unificata exista deja in `rust/fereastra` |
| Ascunde diferentele | a doua insula de jos | desenat |
| Re-sincronizeaza | a doua insula de jos | desenat |
| Uneste ciclul de cursor | a doua insula de jos | desenat |
| Copiaza sursa / Copiaza rezultatul | capul fiecarei coloane | desenat |
| Legenda: modificat / adus / scris de tine | capul coloanei Baza | desenat |
| Baza editabila | coloana dreapta, marcata „editabil" | desenat (marcajul); editarea textului: locul stabilit |
| Aplicarea unui bloc + liniile de legatura | sageata `→` intre coloane, la fiecare bloc; `✓` dupa aplicare | desenat |
| Fereastra modului ghidat: Din sursa / Din tinta / Editeaza / Aplica / Sari peste | sub cod, deasupra barei, cand modul ghidat e pornit | desenat (`--ghidat`) |
| Harta diferentelor (minimap) | rigla din stanga | desenat |
| Anuleaza plasarea (Ctrl+Z), Plaseaza aici | apar in a doua insula, doar dupa o plasare | locul stabilit |
| Drag & drop de blocuri | tragi blocul pe randul tinta din coloana Baza | locul stabilit |
| Previzualizarea la hover in modul inline | bula peste rand, ca bula hartii | locul stabilit |
| *(nou)* paleta Ctrl K: obiecte si comenzi | strat peste panza | desenat (`--paleta`) |

## Cum e facut fiecare lucru greu, in Slint

| Element | Cum | Stare |
|---|---|---|
| Patratele hartii | pictate in Rust intr-o imagine (`SharedPixelBuffer`), una pe grup, nu sute de elemente | dovedit, 0,5 ms |
| Bula la hover pe harta | `TouchArea` da coordonatele; Rust afla patratul | dovedit |
| Patratul selectat ↔ randul selectat | un contur pus la coordonatele calculate in Rust | dovedit |
| Tabelul cu 10 coloane | `ListView` virtualizat, latimi comune cap / rand | dovedit |
| Randul schimbat cu degrade, blocul adus in verde | `@linear-gradient`, suportat de randarea software | dovedit |
| Insulele care plutesc, cu umbra | `drop-shadow-*`, suportat de randarea software | dovedit; costul prin RDP, de masurat pe Windows |
| Derularea la blocul curent | `content-y` legat in ambele sensuri, calculat in Rust | dovedit |
| Bifele, butoanele radio, sagetile | `Path` si `Rectangle`, fara imagini | dovedit |
| Fonturile Geist si Geist Mono | `import "*.ttf"`, incluse in binar, licenta OFL | dovedit, cu diacritice |

## Ce ramane de dovedit la portare

1. **HiDPI:** imaginea hartii se picteaza la factorul de scala al ferestrei.
2. **Tastatura pe harta:** sagetile muta selectia intre patrate.
3. **Cititorul de ecran nu vede patratele unei imagini:** tabelul de dedesubt
   ramane calea accesibila, iar harta primeste o eticheta pe grup.
4. **RDP pe Windows:** umbrele insulelor, masurate pe o sesiune reala. Daca
   sunt scumpe, se renunta la umbre, nu la insule.
5. **Trecerea de la lista (inchisa) la editor (alb):** de vazut pe un ecran
   adevarat daca deranjeaza ochiul.
