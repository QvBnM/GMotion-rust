# Macheta: meniul principal (ecranul de pornire)

Propunere, **neaprobata inca**. Desenata dupa raspunsurile lui Mihai din
2026-11-25 despre ce intra in meniu si ce inseamna fiecare categorie.

| Fisier | Ce e |
|---|---|
| `meniu.html` | varianta **placi** (2x2), tema intunecata, 1440x900 |
| `meniu-luminos.html` | aceeasi, tema luminoasa |
| `meniu-lista.html` | varianta **lista** (patru randuri pe toata inaltimea), intunecata |
| `meniu-lista-luminos.html` | aceeasi, tema luminoasa |
| `*.png` | capturile machetelor (deviceScaleFactor 2) |

Doua variante de aspect, acelasi continut. De ales una (Mihai).

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
