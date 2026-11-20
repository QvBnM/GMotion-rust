# Etapa 21 — editorul: functiile v3.4, in formatul nou

**Stare:** ACTIVA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-20 · **Incheiata:** —

## Scop

Randurile ramase in `cercetare/lipsuri-portare.md` §4, implementate **in
formatul de acum**, nu portate ca aspect. Pana la etapa 20 erau blocate de
motor: fara stratul de plasare nu aveai cui cere unde cade un bloc. Acum ai:
`modul::plasari_obiect`.

Cerut de Mihai, 2026-11-19: „toate functionalitatile alea trebuie
implementate corect in noul format".

## Tinta

`cercetare/machete/editor-functii/` — trei capturi cu **date reale din
motor**, plus cazul pe care au fost facute. Bifa pe o functie cere captura
proprie pusa langa macheta si „da"-ul lui Mihai pe pereche (CLAUDE.md §10).

## Criterii de terminare

- [x] **eticheta pe bloc**: ce e (`MODIFICAT` / `DOAR IN MARTOR` /
      `DOAR IN MODIFICATA`) si unde ajunge („inlocuieste blocul N din
      Modificata", „x% similar cu blocul N", „fara corespondent: se
      insereaza"), din `modul::plasari_obiect`
- [x] **maparea in canal**: legatura bloc -> loc, plina cand are corespondent,
      punctata cu capat gol cand nu are. Acelasi obiect cu care lucrezi, nu
      desen decorativ
- [x] **randul-fantoma** in Modificata: „aici vin blocurile N si M din Martor,
      daca le aplici", la locul calculat
- [x] **umplutura dintr-un bloc se hasureaza** in culoarea blocului (Mihai,
      2026-11-20) — **exista deja in aplicatie** (`Jumatate.fara-linie` +
      `tigla`); golul din machete era al machetei mele, nu al ecranului.
      Verificat pe captura reala, nu pe desen
- [ ] **drag & drop**: tragi blocul din Martor, se aprind doar tintele legale
      (banda „Plaseaza aici" intre blocuri, contur pe blocul care se poate
      inlocui), cu Ctrl+Z — **facut si testat, asteapta „da"-ul lui Mihai**
      pe `capturi/pereche-2-trage.png`
- [ ] **previzualizare la hover**: rezultatul aplicarii, fara sa aplici —
      **facut si testat, asteapta „da"-ul lui Mihai** pe
      `capturi/pereche-3-hover.png`
- [x] **modul ghidat** foloseste plasarea reala, nu pozitii ghicite
- [x] **„Uneste ciclul cursor"** legat (exista in `core`, nefolosit din
      fereastra)
- [x] fiecare functie are si test care o apasa, nu doar captura (capcana din
      `STARE.md`: o captura nu dovedeste ca un buton merge)
- [x] randul corespunzator dispare din `cercetare/lipsuri-portare.md`
- [x] poarta verde; `STARE.md`; `JURNAL.md` daca s-a decis ceva

## Pasi

1. etichetele si maparea (citesc direct din `plasari_obiect`);
2. randul-fantoma si hasura umpluturii;
3. previzualizarea la hover;
4. drag & drop, cu Ctrl+Z;
5. modul ghidat pe plasarea reala;
6. „Uneste ciclul cursor".

## Jurnal de lucru

- **2026-11-20** Etapa pregatita. Machetele si cazul lor sunt in repo.
- **2026-11-20** (sesiunea a doua) Lasarea, sageata, modul ghidat si `cli`
  scriu acum prin `core::aplica_blocuri`, pe aceeasi plasare ca ecranul:
  pana acum un bloc fara pereche care „inlocuieste blocul N" se insera la
  pozitia lui din martor, deci iesea dublat (JURNAL). Aplicarea nu primea nici
  „uneste ciclul cursor" (indicele arata alt bloc). Previzualizarea la hover
  pe sageata, din aceeasi functie. Locul aprins la tragere se tine pe
  diferenta: blocurile 4 si 5 cad pe acelasi rand-fantoma, si tragerea lui 4
  nu aprindea nimic. „Uneste ciclul cursor" era deja legat (cu test); randul
  din criterii era vechi. Capturile si perechile: `cercetare/machete/
  editor-functii/capturi/`. Raman doar „da"-urile lui Mihai.
- **2026-11-20** La tragere se aprinde locul calculat de motor (blocul
  inlocuit sau randul-fantoma), nu randurile blocului tras. Sesiunea s-a
  oprit aici; drag & drop ramane nebifat pana se probeaza lasarea in acelasi
  loc si Ctrl+Z, fiecare cu test.
- **2026-11-20** Eticheta, fantoma si legatura din canal: facute. Decizia
  „unde ajunge blocul" s-a mutat in `core::plasare::plasari` — fereastra nu
  depinde de module, deci nu putea chema `modul::plasari_obiect`; acum amandoi
  citesc din acelasi loc. Pastila se deseneaza peste cod, in capatul
  jumatatii: altfel codul ar sari din loc cand apare eticheta.
- **2026-11-20** Pornita. Prima treaba: captura reala a editorului pe cazul
  machetelor, ca sa stim ce lipseste de fapt. Hasura umpluturii exista deja;
  tragerea blocului exista si ea (fara tinte aprinse). Lipsesc: etichetele pe
  bloc, randul-fantoma, legatura bloc -> loc in canal, previzualizarea la
  hover, tintele legale la tragere, modul ghidat pe plasarea reala si butonul
  „Uneste ciclul cursor".

## Ce a iesit altfel decat planul

(se completeaza la incheiere, dupa „da"-urile lui Mihai)

- Pasul cel mai important n-a fost in plan: ecranul si „Aplica" luau doua
  pareri despre unde cade blocul. Fara el, tragerea aprindea corect si scria
  gresit.
- Modul ghidat n-a cerut cod propriu: trece prin aceeasi aplicare.
