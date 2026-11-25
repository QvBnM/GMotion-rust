# Etapa 32 — runda 2 de remedieri din proba lui Mihai

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-25 · **Incheiata:** —

## Scop

A doua proba a lui Mihai (`Analiza_GMotion_RUST_2.docx`, 2026-11-25 13:29):
denumirile celor doua parti induc in eroare, panoul de detalii si editorul
sunt inghesuite, lista se poate edita din greseala, scroll-ul nu e fluid, iar
jurnalul arata patru simptome reale. La final: Before / After peste tot, codul
are prioritate de spatiu, lista doar selecteaza, un monitor de cadre arata
cifrele, iar jurnalul spune adevarul.

## Criterii de terminare

- [x] **A** etichetele Before / After pe conectare, lista, editor, deploy,
      jurnal, sfaturi; la hover pe orice eticheta de rol, care e dinainte si
      care e dupa (test pe texte; regula din `CLAUDE.md` schimbata)
- [x] **B1** panoul de detalii: un rand de context, pastile mici, codul ia
      restul inaltimii; „Folosit de" / „Foloseste" se vad integral (rupte pe
      latime, derulare proprie cu bara), fara nume taiate
- [x] **B2** editorul: `Wrap lines` pornit implicit (test); textul editabil in
      acelasi font monospatiat ca vizualizarea; mesajele (verificare pe
      server, text final care nu compileaza) intr-o banda proprie sub cod,
      care nu acopera panourile
- [x] **B3** butonul × (tab, mesaj) cu stari normal / hover / apasat in ambele
      teme; marginile rosii din stanga panourilor au legenda + sfat; un bloc
      pus unde nu trebuie are semnal clar si Ctrl+Z
- [x] **C** lista: bifa „Revizuit" si semnul de deploy nu se mai schimba din
      lista (clic = selectie; test); `Reset filter` readuce Modificate,
      Similar crescator (test); semnul de impact nou (nu bare de semnal), cu
      legenda la hover pe capul coloanei, la fel pentru Deploy; panoul de
      detalii mereu deschis cand e ceva selectat (test)
- [ ] **D** monitor de cadre (`--fps` sau `GMOTION_FPS=1`): cadre/s si cel
      mai lung cadru; `masura_ui` masoara si derularea; cifrele inainte / dupa
      in `JURNAL.md`; nicio comanda simpla peste ~50 ms la 12.000 de obiecte.
      *Verificat in container (masura_ui + aplicatia sub Xvfb, 12.000
      sintetice); bifa cere cifrele monitorului pe masina lui Mihai.*
- [x] **E** jurnal: salvarea sesiunii grupata (test), `in_lista` / `diferite`
      in loc de `obiecte` (test), `cod_valid=false` cu eroarea la verificarea
      cazuta (test), etapele compararii in romana, cu Before / After
- [ ] **A2** (din etapa 31) mutarea intre monitoare: proba lui Mihai
- [ ] fluxul cap-coada pe baza reala de ~12.000 (conectare → comparare →
      lista → editor → verificare pe server → deploy): proba lui Mihai, plus
      „da"-ul pe capturi
- [x] capturi in `cercetare/capturi/etapa-32/`, ambele teme, 1366x768 si
      1920x1080, pentru fiecare ecran atins
- [x] poarta verde: `unelte/verifica.sh` (cu wasm32 si windows-msvc)
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat cu deciziile

## Pasi

1. E (jurnalul), apoi A (denumirile).
2. C (lista), B (detalii, editor).
3. D (monitorul de cadre, masuratori).
4. Capturi, documente, poarta, impins.

## Jurnal de lucru

- **2026-11-25** Pornit de pe `claude/clever-cerf-hjk2qh` @ `60ef8d9` (cel mai
  nou; clona locala era cu 12 commit-uri in urma, adusa la zi). Poarta la
  pornire: fmt si clippy verzi; wasm32 / windows-msvc lipseau din container,
  instalate. `vizibile=28405` nu era o dublare: lista tine si cele 16.467 de
  identice, jurnalul numara doar cele 11.938 diferite.

- **2026-11-25 (2)** Lista, panoul, editorul, fluiditatea; deciziile in
  `JURNAL.md`. Gasite pe drum, in afara cererii: planul de deploy O(n·m)
  (1,1 s cu toate in plan) si eticheta gresita a grupurilor de blocuri doar
  in After („blocks 0-0 · BEFORE ONLY" pe un chenar rosu). Capturile in
  `cercetare/capturi/etapa-32/`, facute in aplicatia adevarata sub Xvfb,
  condusa cu mouse-ul (`conduce_x.py`).

## Ce a iesit altfel decat planul

- **„Ecranul asta il vreau mereu pornit" era panoul de detalii**, nu lista
  (conturul rosu din captura 7). Facut asa; lista era deja ecranul de dupa
  comparare.
- **„Nu vreau sa modifici campurile din lista" era bifa Revizuit** din
  lista (captura 3), nu o editare de text; nu exista editare inline.
- **`vizibile=28405` nu era un bug de numarare**: lista tine si identicele;
  jurnalul numara doar diferitele.
- **Liniile rosii erau un bug**, nu o legenda lipsa: bara diferentei curente
  pe liniile identice, in rosu implicit.
- **Monitorul de cadre n-a putut folosi notificarea de desenare**
  (`Unsupported` pe software-renderer cu winit); masoara intervalul dintre
  cadre intr-o desenare continua, doar cat e aprins.
- **Cea mai mare pierdere de fluiditate era un decor**: fundalul hartii,
  ~9 din ~13 ms pe cadru.

