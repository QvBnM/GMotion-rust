# Etapa 33 — taburi, pozitia in side-by-side, cadre

**Stare:** ACTIVA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-25 · **Incheiata:** —

## Scop

Jurnalul rularii lui Mihai din 2026-11-25 15:32-15:39 (build `da2f02a`, baza
reala, 11.938 diferite / 28.405 in lista) si „Prompt de lucru #3": un obiect
ales din Object Explorer nu mai suprascrie tabul deschis; transpunerea unui
bloc nu mai muta vederea; in repaus nu se deseneaza nimic; monitorul de cadre
spune adevarul (p99, cadre peste 16,7 ms); salvarea sesiunii iese de pe firul
ferestrei.

## Criterii de terminare

- [x] **A** din lista si din Object Explorer: `Deschide` (in tabul curent, ca
      pana acum) si `+ Tab nou`, amandoua vizibile; un clic simplu in Object
      Explorer doar alege randul (tabul deschis nu se pierde); un obiect deja
      deschis isi activeaza tabul, nu se dubleaza; ordinea, obiectul din
      fiecare tab, previzualizarea si tabul activ trec prin sesiune (teste:
      `tab_nou_*`, `sesiunea_tine_*`, `tabul_activ_si_previzualizarea_*`)
- [x] **B** transpunere / Aplica / Anuleaza / Plaseaza / orice reconstruire pe
      acelasi obiect: derularea ramane exact unde era, diferenta curenta se
      marcheaza fara salt (test `transpunerea_pastreaza_derularea_*`, vazut
      cazand fara reparatie: -5060 in loc de -4400 px)
- [x] **C1** monitorul de cadre nu mai deseneaza el insusi: cadrele vin din
      winit (`RedrawRequested`); raporteaza cadre, p99, cate peste 16,7 ms,
      maxim; nicio linie cand nu s-a desenat nimic (teste in `cadre.rs`)
- [ ] **C1** proba: 30 s de repaus = zero linii `gmotion::cadre` in jurnal pe
      masina lui Mihai. *In container (Xvfb, 12.000): 37 s si 30 s fara niciun
      cadru dupa comparare.*
- [ ] **C2** niciun cadru peste 16,7 ms la `alege_randul`, `activeaza_tab`,
      `comuta_wrap`, `inchide_tab` la 12.000 de obiecte, pe masina lui Mihai.
      *In container: partea de Rust a scazut de la 15-25 ms la 2-4 ms; ramane
      desenarea editorului (~22 ms la schimbarea obiectului, la 1920x1080,
      software-renderer in container). Cifrele de pe Windows le da monitorul.*
- [x] **D** salvarea amanata (1,2 s) se scrie pe alt fir; unirea cu discul
      atinge aplicatia doar cand discul aduce ceva nou; inchiderea asteapta
      firul (test `salvarea_din_fundal_*`, vazut cazand fara unirea in trei
      cai)
- [x] **D** `comuta_wrap` nu mai apare la pornire: schimbarea latimii nu e
      o actiune a omului; grupata la 120 ms, doar cu editorul pe ecran
- [x] **D** serverele gasite: cel mult 5 randuri cu derulare, filtrate de
      textul din camp, „vizibile / gasite" (test `serverele_gasite_*`)
- [x] **D** citirea mai lenta (14,0 / 10,3 s fata de 9,3 / 6,5 s): codul de
      citire e identic intre cele doua build-uri; cauza e pe server (vezi
      `JURNAL.md`)
- [ ] proba lui Mihai pe ~12.000 de obiecte: tab nou din Object Explorer,
      transpunere fara salt, repaus fara cadre
- [ ] **A2** (din etapele 31-32) mutarea intre monitoare, fara inghetare:
      proba lui Mihai
- [x] capturi in `cercetare/capturi/etapa-33/`, ambele teme, 1366x768 si
      1920x1080
- [x] poarta verde: `unelte/verifica.sh` (cu wasm32 si windows-msvc)
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat cu deciziile

## Pasi

1. Monitorul de cadre (masura intai, ca restul sa aiba cifre).
2. B (pozitia), A (taburi).
3. D (salvarea, wrap, serverele), C2 (cadrele lungi, cu `masura_ui`).
4. Capturi, documente, poarta, impins.

## Jurnal de lucru

- **2026-11-25** Pornit de pe `claude/clever-cerf-hjk2qh` @ `da2f02a` (cel
  mai nou; clona locala era cu 19 commit-uri in urma, iar
  `unelte/verifica.bat` aparea modificat doar prin CRLF — readus la versiunea
  din depozit). Etapa 32 inchisa; ce astepta proba lui Mihai a trecut aici.
- **2026-11-25** Cele ~70 de cadre/s in repaus din jurnal erau ale
  monitorului (`animation-tick()`), nu ale aplicatiei. `masura_ui` are acum
  si editorul (schimbarea obiectului, tab nou, taburi, wrap, Ctrl+W), cu
  Rust-ul si desenarea separat; redesenarea scrie la `debug` timpii pe
  bucati (`redesenare pe bucati`).

## Ce a iesit altfel decat planul

- **Nu aplicatia desena continuu, ci monitorul.** Cu `--fps`, `animation-tick()`
  tinea fereastra la ~70 de cadre/s; fara el, nimic.
- **„Redesenare lenta ms=119" nu era desenarea**, ci Rust-ul: coloana din
  editor refacea la fiecare clic toate randurile si nodurile (9-16 ms aici la
  12.000), plus ecranul listei, invizibil din editor.
- **Ce ramane e desenarea editorului** (software-renderer): ~12 ms o pictare
  completa la 1920x1080 in container, ~22 ms la schimbarea obiectului. Sub
  16,7 ms acolo cere alt renderer sau un editor cu mai putine elemente pe
  rand — decizie de luat cu cifrele de pe Windows.
- **Salvarea dadea cadre de 27-45 ms nu din scriere**, ci fiindca unirea cu
  discul se punea mereu inapoi in aplicatie si arunca modelul editorului.
