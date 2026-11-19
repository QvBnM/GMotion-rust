# Etapa 19 — lista ca Object Explorer, cu cele doua panouri alaturi

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-19 · **Incheiata:** —

## Scop

Prima proba pe bazele reale (GetsBet → CEAAS, 311 obiecte): Mihai a gasit
lista „o tampenie" si a cerut-o ca Object Explorer din SSMS, cu codul in
forma ecranului Side by Side. La final, ecranul de dupa comparare e un arbore
in stanga si cele doua panouri (martor | modificata) in dreapta; harta,
tabelul lat si butonul „Open" plutitor nu mai exista. In editor si in
previzualizare exista un buton de wrap.

## Cum arata (descris de Mihai, 2026-11-19; nu exista macheta)

- **Stanga, arborele**: `Baza` › `Proceduri` / `Functii` / `View-uri` /
  `Triggere` › `schema.nume`. Nodurile se deschid / inchid cu clic pe
  sageata sau pe nume, ca in SSMS; fiecare dosar are numarul de obiecte.
  Obiectul: patratul si litera starii (M / + / −), numele in mono, ±linii.
- **Dreapta, cele doua panouri alaturi**, ca in editor: capul cu numele
  obiectului si actiunile (Revizuit, In plan, Omite, Deschide in editor),
  apoi Martor | Modificata, cu numerele de linie si evidentierea.
- **Sus** raman filtrele, cautarea, Recompara, Export CSV, Deploy.

## Criterii de terminare

- [x] arborele: baza › dosar pe tip › obiect; clic pe dosar il deschide /
      inchide; numarul din dosar = obiectele vizibile din el; filtrele si
      cautarea de sus se aplica arborelui (test pe model)
- [x] clic pe obiect il arata in dreapta; dublu clic sau Enter deschide
      editorul (test cu mouse-ul adevarat, ca la lista de acum)
- [x] dreapta: martor | modificata alaturi, cu numere de linie, ca in
      editor; capul cu Revizuit / In plan / Omite / Deschide
- [x] harta, tabelul si butonul „Open" plutitor scoase, cu tot codul lor
      mort (Rust si `.slint`)
- [x] butonul Wrap: in editor si in previzualizare; liniile lungi se rup,
      iar randurile din stanga si din dreapta raman aliniate
- [x] harta ca ecran separat de ansamblu (Mihai, 2026-11-19, dupa macheta
      lui): se deschide din lista (buton si `h`), se inchide cu Esc; coloane
      pe tip care umplu ecranul, latimi apropiate (mai late pentru tipurile
      mai mari, dar nu proportional), bara de proportie, blocuri de 5
      randuri, zoom automat ca sa intre in ecran; clic pe patrat inchide
      harta si selecteaza obiectul in arbore (test care apasa butonul, tasta
      si un patrat)
- [ ] nicio redesenare peste 100 ms la un filtru sau la un clic, pe 4.000
      de obiecte (masurat, nu presupus)
- [ ] captura la 1920x1080 trimisa lui Mihai si „da"-ul lui pe ea
- [x] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. `model_arbore` in `fereastra`: nodurile aplatizate din `Lista` (vizibile),
   cu starea deschis / inchis; teste.
2. Starea arborelui in `Aplicatie` (`Comanda::ComutaNod`), ca sa se poata
   actiona si testa fara fereastra.
3. `lista.slint`: corpul refacut (arbore | panouri); harta si tabelul scoase.
4. Wrap: numarul de randuri vizuale pe linie se calculeaza in Rust (font
   mono, latime cunoscuta), inaltimea randului = maximul celor doua parti.
5. Masuratori, captura, proba lui Mihai.

## Jurnal de lucru

- **2026-11-19** Deschisa dupa prima proba pe bazele reale. Etapa 18 e
  absorbita aici: ecranul de dupa comparare se reface oricum.
- **2026-11-19** Arborele si panourile facute; harta, tabelul, Open-ul
  plutitor si panoul comutat la clic scoase. `j` / `k` merg in ordinea
  arborelui (`MutaInArbore`), nu a listei: altfel sareau intre dosare.
  `width: parent.width / 2` pe copiii unui `HorizontalLayout` a dat
  „Recursion detected" la rulare; impartirea se face cu `stretch`.
- **2026-11-19** Wrap („Rupe liniile", tasta `w` in editor): Rust rupe
  linia in randuri de continuare, deci alinierea si navigarea pe diferente
  raman cele de dinainte. Latimea panoului se afla abia dupa primul cadru;
  `changed coloane-wrap` redeseneaza.
- **2026-11-19** Viteza: lista cu arbore, pe 4.000 de obiecte, se incarca in
  5 ms si se deseneaza in 10 ms (`masura_lista`). Cele 1,2 s din jurnalul
  lui Mihai veneau din „Difera in" / `+N −M` calculate pe loc pentru 60 de
  obiecte la fiecare filtru; acum se calculeaza toate pe alt fir, imediat
  dupa comparare. De confirmat pe jurnalul urmator de pe Windows.

- **2026-11-19** Harta readusa ca ecran separat (`h`), dupa macheta lui
  Mihai: `harta.rs` refacut din versiunea scoasa, cu latimi `n^0,25`, doua
  trepte de zoom mai mari si bula in dreapta patratului. Testul apasa `h`,
  Esc, butonul si un patrat cu mouse-ul adevarat; a cazut intai pe bula
  (`moved` vine doar cu butonul apasat). Decizia in `JURNAL.md`.

## Ce a iesit altfel decat planul

