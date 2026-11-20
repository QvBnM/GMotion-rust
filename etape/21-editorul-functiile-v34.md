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

- [ ] **eticheta pe bloc**: ce e (`MODIFICAT` / `DOAR IN MARTOR` /
      `DOAR IN MODIFICATA`) si unde ajunge („inlocuieste blocul N din
      Modificata", „x% similar cu blocul N", „fara corespondent: se
      insereaza"), din `modul::plasari_obiect`
- [ ] **maparea in canal**: legatura bloc -> loc, plina cand are corespondent,
      punctata cu capat gol cand nu are. Acelasi obiect cu care lucrezi, nu
      desen decorativ
- [ ] **randul-fantoma** in Modificata: „aici vin blocurile N si M din Martor,
      daca le aplici", la locul calculat
- [x] **umplutura dintr-un bloc se hasureaza** in culoarea blocului (Mihai,
      2026-11-20) — **exista deja in aplicatie** (`Jumatate.fara-linie` +
      `tigla`); golul din machete era al machetei mele, nu al ecranului.
      Verificat pe captura reala, nu pe desen
- [ ] **drag & drop**: tragi blocul din Martor, se aprind doar tintele legale
      (banda „Plaseaza aici" intre blocuri, contur pe blocul care se poate
      inlocui), cu Ctrl+Z
- [ ] **previzualizare la hover**: rezultatul aplicarii, fara sa aplici
- [ ] **modul ghidat** foloseste plasarea reala, nu pozitii ghicite
- [ ] **„Uneste ciclul cursor"** legat (exista in `core`, nefolosit din
      fereastra)
- [ ] fiecare functie are si test care o apasa, nu doar captura (capcana din
      `STARE.md`: o captura nu dovedeste ca un buton merge)
- [ ] randul corespunzator dispare din `cercetare/lipsuri-portare.md`
- [ ] poarta verde; `STARE.md`; `JURNAL.md` daca s-a decis ceva

## Pasi

1. etichetele si maparea (citesc direct din `plasari_obiect`);
2. randul-fantoma si hasura umpluturii;
3. previzualizarea la hover;
4. drag & drop, cu Ctrl+Z;
5. modul ghidat pe plasarea reala;
6. „Uneste ciclul cursor".

## Jurnal de lucru

- **2026-11-20** Etapa pregatita. Machetele si cazul lor sunt in repo.
- **2026-11-20** Pornita. Prima treaba: captura reala a editorului pe cazul
  machetelor, ca sa stim ce lipseste de fapt. Hasura umpluturii exista deja;
  tragerea blocului exista si ea (fara tinte aprinse). Lipsesc: etichetele pe
  bloc, randul-fantoma, legatura bloc -> loc in canal, previzualizarea la
  hover, tintele legale la tragere, modul ghidat pe plasarea reala si butonul
  „Uneste ciclul cursor".

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
