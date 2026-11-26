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
| `*.png` | capturile machetelor (deviceScaleFactor 2) |

Patru variante. Primele trei sunt un ecran de meniu, a patra scoate meniul
cu totul. De ales una (Mihai).

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
