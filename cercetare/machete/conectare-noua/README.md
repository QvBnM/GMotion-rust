# Macheta: ecranul de conectare nou

Macheta pentru etapa 17. HTML static, cu date inventate. Se citeste ca
specificatie, nu se ruleaza in aplicatie. Fonturile se iau din
`rust/fereastra/fonturi/`, iar culorile sunt cele din
`rust/fereastra/src/jetoane.rs` (`INTUNECAT` / `LUMINOS`).

## Fisierele

| Fisier | Ce arata |
|---|---|
| `conectare-aceeasi.html` | intunecat, bifa „Aceeasi conexiune" pusa |
| `conectare-alta.html` | intunecat, Baza pe alt server |
| `conectare-aceeasi-luminos.html` | luminos, bifa pusa |
| `conectare-alta-luminos.html` | luminos, Baza pe alt server |
| `capturi/*-1440x900.png` | cele patru, la rezolutia de referinta |
| `capturi/rezolutii-*.png` | aceeasi macheta pe 8 rezolutii, de la 1024x640 la 2560x1440, si cu Windows scalat la 150% / 200% |

## Ce s-a decis (Mihai, 2026-11-18 / 19)

- **Sursa si Baza alaturi, simetrice.** Fiecare card are sus conexiunea lui
  (autentificare, server, utilizator, parola, starea, Testeaza), iar
  dedesubt lista bazelor de pe server.
- **O singura baza de fiecare parte**, aleasa cu cerc (radio). Compararea unei
  surse cu mai multe baze deodata (din v3.4) nu mai e pe ecranul asta.
- **Bifa „Aceeasi conexiune"** e in capul cardului Baza. Cand e pusa, campurile
  de conexiune ale Bazei dispar. In locul lor ramane o zona punctata, de
  aceeasi inaltime, cu serverul sursei, ca listele sa inceapa la aceeasi linie.
  In lista Bazei, baza aleasa la Sursa apare gri, cu „e sursa".
- **Fara etichete PROD / TEST / DEV.**
- **Fara randul cu certificatul.** Avertismentul ramane doar in jurnal.
- **Compara la mijloc**, cu „Se citeste, nu se scrie nimic." sub el.
- Dedesubt: **Comparari recente** (cu Reia), apoi **Jurnalul**, „ce se
  intampla acum".

## Proportiile

- **O singura scara pentru tot ecranul.** Macheta e gandita la 1440x900.
  Scara = cat incape din 1440x900 in fereastra, intre 0,85 si 1,3. Cu ea
  cresc sau scad textul, campurile, randurile si spatiile, asa ca ecranul
  arata la fel pe orice rezolutie. Fara scara, pe ecrane mici parea ca nu
  incape, iar pe cele mari ramanea prea mult gol.
- **Pe verticala:** cardurile iau 3/4, jurnalul 1/4. Cardurile cresc pana la
  640 px (la scara 1); ce ramane peste merge in jurnal. Jurnalul are cel putin
  110 px, cardurile cel putin 300 px.
- **Listele de baze deruleaza** in card, cu cel putin 3 randuri vizibile.
  Cardul nu se lungeste dupa numarul de baze.
- **Sub 820 px inaltime** (dupa scara), comparatiile recente dispar (raman in
  tabul Sesiuni), iar spatiile din carduri se strang.
- **Pe ecranele late,** continutul are cel mult 1760 px si sta centrat;
  antetul ramane pe toata latimea.
- Textul lung din campuri (serverul) se taie cu „…", nu se rupe pe doua
  randuri.

Verificat in Chromium pe 1024x640, 1280x720, 1366x768, 1440x900, 1920x1080,
1920x1080 la 150%, 2560x1440 si 2560x1440 la 200%: fara bare de derulare pe
pagina, in ambele teme.

## Ce a ramas deschis

- **Windows scalat la 125% sau mai mult.** Azi scara micsoreaza cu 0,85 si
  peste marirea aleasa in Windows. Recomandat: cand Windows e deja la 125% sau
  peste, aplicatia nu mai micsoreaza. Asteapta raspunsul lui Mihai.
- **Certificatul serverului:** fara randul lui, nu mai exista un loc din ecran
  de unde sa se schimbe verificarea. De decis: ramane pe ce e configurat, sau
  se muta la Ajutor / setari.
- **Banda de 3 px de sus,** in culoarea mediului: inca exista in macheta. E
  acelasi semnal ca etichetele de mediu scoase; nu s-a decis daca ramane.
- **In Slint,** scara ar fi o singura valoare in tema, inmultita in toate
  dimensiunile din `comune.slint`. Nu e probata inca.
