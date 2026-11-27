# Macheta: meniul principal (ecranul de pornire)

Propunere, **neaprobata inca**. Desenata dupa raspunsurile lui Mihai din
2026-11-25 despre ce intra in meniu si ce inseamna fiecare categorie.

| Fisier | Ce e |
|---|---|
| `meniu.html` | varianta **placi** (2x2), tema intunecata, 1440x900 |
| `meniu-luminos.html` | aceeasi, tema luminoasa |
| `meniu-lista.html` | varianta **lista** (patru randuri pe toata inaltimea), intunecata |
| `meniu-lista-luminos.html` | aceeasi, tema luminoasa |
| `meniu-split.html` | varianta **split**: stanga lucrul, dreapta GMotion; intunecata |
| `meniu-split-luminos.html` | aceeasi, tema luminoasa |
| `meniu-bara.html` | varianta **bara laterala**: fara ecran de meniu; intunecata |
| `meniu-bara-luminos.html` | aceeasi, tema luminoasa |
| `meniu-marca.html` | varianta **marca**: stanga alegerile, dreapta panoul viu cu sigla |
| `globuri.html` | cele patru **forme de glob**, una langa alta, de ales |
| `glob-hex.js` | motorul formelor: fetele, lumina, sortarea pe adancime |
| `*.png` | capturile machetelor (cele ale marcii la 1440x900, restul la deviceScaleFactor 2) |

`meniu-marca.html` ia forma din adresa: `?glob=emisfere` (implicit),
`?glob=diamant`, `?glob=sfera`, `?glob=sticla`, `?glob=cerc`. Tema se schimba
din butonul de jos.

Cinci variante. Trei sunt un ecran de meniu cu continut util in dreapta, una
scoate meniul cu totul, iar a cincea (`meniu-marca`) pune in dreapta panoul viu
al marcii. De ales una (Mihai).

## Varianta marca, de ce arata asa

Ceruta de Mihai pe 2026-11-27, pornind de la o pagina HTML facuta de el:
stanga alegerile, dreapta sigla `gMotion`, benzile aurii-turcoaz si praful de
fundal. Panoul are jos o banda subtire cu serverul, jurnalul si ultimul deploy,
ca sa nu fie doar decor. Textul e pe perechea **Outfit + JetBrains Mono**
(aleasa de Mihai), cu `<` si `/>` in jurul semnaturii.

Nucleul din prisme imprastiate a fost inlocuit, tot la cererea lui, cu forme
hexagonale. `globuri.html` le arata pe toate, cu numarul de fete al fiecareia -
cifra care decide costul, fiindca desenarea in aplicatie e software:

| # | Forma | Fete |
|---|---|---|
| 0 | **sfera taiata in diagonala**: cupola chihlimbar = Before, cea turcoaz = After | 162 |
| 1 | diamant hexagonal (masa, coroana, branta, pavilion) | 19 |
| 2 | sfera acoperita cu placi hexagonale, plina | 162 |
| 3 | aceeasi sfera, translucida (se vad si placile din spate) | 162, desenate toate |
| 4 | cerc de placi hexagonale, cu diamantul in mijloc | 31 |

**Forma 0 e cea propusa** (si cea din capturi). Pleaca de la cupola hexagonala
aprinsa trimisa de Mihai pe 2026-11-27: aceeasi sfera hexagonala, taiata pe o
diagonala si desfacuta in doua cupole. Chihlimbarul e martorul, turcoazul e
modificata, marginea taieturii e aprinsa ca in imagine, bobite trec din una in
alta, iar o unda strabate sfera dinspre Before spre After. Semnul spune ce face
programul fara niciun cuvant - acelasi inteles ca pastila taiata in doua de pe
iconitele categoriilor.

Costul in aplicatie: 162 de fete, desenate cu fata si spatele (cupole
translucide), plus inelele si bobitele. Pentru Slint, asta e varianta **(b)** -
desenare in Rust intr-un `SharedPixelBuffer` - daca vrem mouse-ul; pe cadre
pregenerate, varianta **(a)**, intra orice forma, fiindca pretul e al
generatorului, nu al aplicatiei.

**Macheta ruleaza in browser; aplicatia nu are browser.** Ce se vede aici in
HTML nu se traduce unu-la-unu in Slint:

- *Fundalul si sigla* se pot face: ele sunt deja imagini generate determinist
  (`cercetare/fundal-unde/genereaza.py` scrie aceleasi culori: portocaliu ->
  galben -> turcoaz). Litera cu degrade cere imagine, nu text: `Text` in Slint
  ia o culoare plina, nu un degrade.
- *Intrarea* (literele care intra pe rand, „g" care aluneca, subtitlul care
  apare) se face nativ, cu `animate` si intarzieri pe fiecare litera.
- *Nucleul 3D* nu are echivalent: Slint nu are panza de desen. Doua drumuri:
  **(a)** cadre pregenerate, rulate in bucla dintr-un cronometru - fara mouse,
  cost mic, acelasi material se refoloseste la ecranul de pornire animat;
  **(b)** rasterizare in Rust intr-un `SharedPixelBuffer` pe fiecare cadru -
  pastreaza rotatia dupa mouse si pulsul la clic, dar cere o dependinta noua
  (intrare in `JURNAL.md`) si se masoara cu `--fps`, fiindca randarea e
  software (RDP, fara placa video).
- *Fonturile din comutatorul de jos* (Syne, Outfit, Sora, Montserrat, Space
  Grotesk) nu sunt in binar; azi exista doar Geist. Daca sigla e imagine,
  fontul trebuie doar la generare, nu la rulare.

Comutatorul de fonturi, butonul de tema si „Reia intrarea" sunt unelte ale
machetei. Nu intra in aplicatie.

## Varianta bara laterala, de ce e altfel

Celelalte trei pun un ecran intre tine si lucru: il vezi o data, apesi, si
nu-l mai vezi pana la repornire. Aici nu exista ecran de meniu. Cele patru
directii stau intr-o bara ingusta, vizibila tot timpul, iar fereastra
porneste direct in conectare, ca azi. Se vede mereu unde esti, se trece
dintr-o directie in alta dintr-un clic, si nu se plateste niciun clic in
plus la pornire. `Comparare obiecte` e aprinsa; celelalte au un cerc gol
in loc de `IN CURAND`, fiindca intr-o bara eticheta ar fi zgomot.

## Varianta split, de ce arata asa

Primele doua variante semanau cu un program de instalare: titlu mare,
patru casete de prezentare, text care explica produsul cuiva care nu-l
cunoaste. Ecranul de pornire al unei unelte nu prezinta produsul, il
porneste. Modelul e ecranul de intampinare din IDE-uri (JetBrains: actiuni
intr-un panou, ce ai lucrat ultima data in celalalt; Zed: pagina de pornire
minima, care dispare cand ai deschis ceva).

- **Stanga**: cele patru directii, cate un rand cu iconita si o linie de
  explicatie. Doar prima e apasabila; restul `IN CURAND`, punctate.
- **Dreapta**: numele si ce e GMotion, apoi lucruri **folositoare**:
  compararile recente de reluat cu un clic, comanda echivalenta din `cli`
  (regula §3: nimic nu exista doar in spatele unui buton), starea serverului
  si a jurnalului.
- **Iconitele sunt taiate pe verticala in doua**: jumatatea din stanga
  plina, cea din dreapta goala, despartite de o linie. E acelasi semn peste
  tot, si inseamna Before / After — subiectul intregii aplicatii.

Se citesc ca specificatie, nu se ruleaza in aplicatie. Culorile si fonturile
sunt aceleasi jetoane ca in `conectare-noua/`.

## Ce spune macheta

- Patru placi, 2x2. Doar **Comparare obiecte** e activa; restul sunt
  punctate, cu eticheta `IN CURAND` si fara stare de apasare.
- Fiecare placa isi arata **ramurile** de mai tarziu, ca scopul categoriei sa
  se vada inainte sa existe codul.
- Placa activa arata ultima comparare, ca sa se poata relua de aici.
- Jos, bifa „Porneste direct in Comparare obiecte": cine lucreaza zilnic pe
  comparare nu plateste un clic in plus la fiecare pornire.
- Taste: 1-4 aleg, Enter deschide, Esc inchide aplicatia.

## De decis inainte de cod

1. Numele placii 2: „Kit SQL pe baza" (aici) fata de ecranul Deploy existent,
   care e pasul final al compararii. Nu pot sa se cheme amandoua „Deploy".
2. Daca bifa de pornire directa ramane in meniu sau trece in Configurare.
3. Daca „Configurare" se deschide ca ecran propriu sau ca panou peste meniu.
4. La varianta marca: nucleul 3D pe cadre pregenerate (fara mouse) sau desenat
   in Rust (cu mouse, cost de masurat). Si care pereche de fonturi, daca sigla
   nu ramane pe Geist.
