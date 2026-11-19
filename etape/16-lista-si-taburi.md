# Etapa 16 — lista de obiecte in editor si taburile

**Stare:** INCHEIATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-19 · **Incheiata:** 2026-11-19

> Porneste dupa etapa 15, sau inainte daca Mihai cere. Extinde 15 B (lista
> ramane in stanga, coloana de 300 px, clic = alt obiect pe loc): aici
> primeste pin / ascundere / mutare si taburile. Numele grupurilor urmeaza
> denumirile din 15 E („Martor" / „Modificata"), nu cele din macheta.
> Decizia si motivele:
> `JURNAL.md`, intrarea din 2026-11-18 „Editorul primeste lista de obiecte si
> taburi". Macheta: `cercetare/machete/sina-si-taburi/`.

## Scop

In editor se poate trece de la un obiect la altul fara intoarcere in lista
mare: o lista ingusta de obiecte langa editor si taburi pentru obiectele la
care lucrezi in paralel. `j` / `k` raman calea principala de parcurgere.

## Regulile taburilor

- **Un singur tab de previzualizare** (titlu inclinat). Clic simplu in lista
  si `j` / `k` incarca obiectul in el, inlocuind ce era.
- **Tab fixat** se deschide prin: dublu clic pe un obiect din lista, clic
  dreapta → „Deschide in tab nou", dublu clic pe tabul de previzualizare, sau
  automat cand se editeaza textul final al obiectului din previzualizare.
  Tragerea unui obiect in bara de taburi nu intra aici; se poate adauga
  dupa, pe mecanismul de drag deja facut la etapa 14 F.
- **Maxim 8 taburi fixate.** Al noulea inchide tabul fixat folosit cel mai de
  demult **si needitat**. Daca toate 8 au text final editat, deschiderea se
  refuza cu mesaj; nu se inchide nimic singur.
- **Un obiect sta in cel mult un tab.** Clic in lista pe un obiect deja fixat
  activeaza tabul lui; nu se incarca si in previzualizare.
- **`j` / `k` sar peste obiectele deschise in taburi fixate.** Pornesc de la
  pozitia obiectului activ (din previzualizare sau dintr-un tab fixat) in
  ordinea curenta a listei (filtru + sortare) si iau urmatorul obiect care nu
  e fixat. La capat: raman pe loc, cu mesaj. Daca obiectul activ nu mai e in
  lista filtrata, se porneste de la inceputul listei.
- **Cheia unui tab** e cheia din sesiune: tip + `schema.nume` (dupa etapa 14 D,
  si baza). Pe tab scrie numele scurt; numele complet apare la hover.
- `Ctrl+W` inchide tabul activ; activ devine vecinul din stanga.

## Regulile listei din editor

- Grupata pe stare (Modificate / Doar in sursa / Doar in baza / Identice), cu
  cercul de revizuit si `+/-`; aceeasi ordine si acelasi filtru ca lista mare.
- **Pin** (fixa langa editor) / **«** ascunde (banda ingusta cu contoare,
  tasta `[`) / **⇄** muta pe partea cealalta. Fara pin, apare peste editor la
  hover pe margine.
- Latimea se trage de margine, **intre 200 si 360 px**.
- Numele lung se taie **pe mijloc** (finalul distinge obiectele); la hover,
  numele complet, tipul, `+/-`, similaritatea, unde difera.
- Pozitia, latimea, pinul si taburile fixate supravietuiesc inchiderii
  (sesiunea).

## Criterii de terminare

- [x] logica taburilor intr-un singur loc, in Rust, fara Slint, cu teste pe
      fiecare regula de mai sus (inclusiv `j` cu toate obiectele urmatoare
      fixate, al noulea tab cu toate 8 editate, filtru care scoate obiectul
      activ) — `fereastra::model_taburi`, legat in `app::rutare`
- [x] taierea pe mijloc a numelui: functie Rust, testata (latime, unicode)
- [x] testul care apasa tastele acopera `j`, `k`, `[`, `Ctrl+W`, dublu clic
- [x] fiecare buton nou (pin, ascunde, muta, inchide tab, `+N ▾`) are handler
      si test care il apasa (pin / muta / banda cu mouse-ul; tabul si
      meniul `+N` prin callback-uri)
- [x] starea listei si a taburilor in sesiune, testata la reincarcare
- [x] capturi in ambele moduri in `cercetare/capturi/` (`editor-taburi-*`,
      `editor-coloana-*`)
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Modelul taburilor (pur, testat), apoi taierea numelui.
2. Lista din editor, legata de modelul listei existent.
3. Bara de taburi si tastele.
4. Sesiunea.

## Jurnal de lucru

- **2026-11-18** Etapa scrisa, nepornita. Regulile vin din discutia cu Mihai
  pe macheta.

- **2026-11-19** Etapa facuta intr-o zi, in patru commit-uri: modelul
  taburilor (pur), legarea in `rutare` (obiectul din tabul activ e mereu
  selectia listei), bara de taburi si tastele, coloana (grupe, pin, banda,
  mutare, sesiune). Pe drum, testul de anulare (F-A4) s-a dovedit ca nu
  testa ce spunea: avea un singur obiect, deci „alt obiect" nu exista; acum
  are doua. Doua asteptari de taiere pe mijloc numarate gresit in test.

## Ce a iesit altfel decat planul

- **Un singur „obiect curent" a ramas regula.** Planul nu spunea cum stau
  taburile fata de selectia listei; a doua stare ar fi dublat toate comenzile.
  Activarea unui tab muta selectia (golind filtrul, daca il ascunde).
- **Hover-ul pe rand nu e o bula**: intr-un `ListView`, o bula se taie la
  marginea listei. Detaliile apar jos in coloana.
- **Pin, muta si banda sunt simboluri (◆ ⇄ «)**, nu cuvinte: nu cer texte
  in doua limbi si incap in 300 px.
- **Discul mediului cloud s-a umplut de trei ori** (`target` la ~30 GB);
  `~/.cargo/config.toml` fara debuginfo si fara incremental il tine la ~2 GB.
  Nu e o setare a repo-ului.
