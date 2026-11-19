# Etapa 18 — editorul arata ca macheta

**Stare:** ABANDONATA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-19 · **Incheiata:** 2026-11-19 (abandonata)

> Absorbita in etapa 19: dupa prima proba pe bazele reale, Mihai a cerut
> ecranul de dupa comparare ca Object Explorer, cu cele doua panouri; editorul
> reasezat aici ramane, aprobarea pe captura se cere pe ecranul nou.

> Numarul 17 e tinut pentru conectarea noua (`STARE.md`). Ordinea dintre 17
> si 18 o alege Mihai.

## Scop

Editorul arata ca macheta din `cercetare/machete/sina-si-taburi/macheta.png`,
nu doar are aceleasi functii. Etapa 16 a facut functiile (taburi, coloana cu
pin / banda / mutare, `j` / `k`) si a fost incheiata; Mihai a gasit ca
ecranul „nu seamana deloc" cu macheta: criteriile ei cereau functii, nu
aspect. Logica din etapa 16 nu se rescrie, se reaseaza ecranul.

## Ce se schimba fata de acum

Comparat pe `cercetare/capturi/editor-taburi-intunecat.png`:

- insulele plutitoare de sus (logo / Lista / ‹ ›, si Revizuit / Omite / In
  plan / tema) devin **o singura bara compacta pe toata latimea**: logo,
  martor → modificata (cu eticheta de mediu), progresul revizuirii,
  Revizuit / In plan / Deploy;
- titlul mare (`dbo.uspReproceseazaFacturi` pe doua randuri) si legenda
  culorilor devin **antetul obiectului, pe un rand**: nume, `+/-`, „difera
  in", Revizuit, Alaturat / Unificat;
- taburile stau **lipite de antetul obiectului**, nu plutesc deasupra
  titlului;
- cele doua panouri **umplu tot spatiul ramas**, lipite, fara margini si
  colturi rotunjite care mananca loc; capul fiecarui panou pe un rand subtire;
- bara de butoane de jos (Aplica toate ... Anuleaza) se muta in antetul
  obiectului sau intr-un meniu; jos ramane **o singura linie cu
  scurtaturile**, ca in macheta;
- coloana listei: **grupe pe stare cu titlu si numar**, rand pe o linie
  (cerc revizuit, litera de stare, nume taiat pe mijloc, `+/-`), cautarea
  sus, contoarele jos — ca in macheta;
- nicio functie din etapele 14-16 nu dispare; ce nu are loc vizibil intra
  intr-un meniu, cu tasta pastrata.

## Criterii de terminare

- [x] fiecare punct din „Ce se schimba" e facut (cu diferentele de mai jos)
- [ ] **pus langa macheta:** captura editorului la 1440x900, alaturi de
      `macheta.png`, intr-o singura imagine in `cercetare/capturi/`;
      fiecare diferenta ramasa e scrisa in jurnalul de lucru cu motivul
- [ ] **Mihai a vazut perechea si a spus da** inainte de bifa (data si
      raspunsul in jurnalul de lucru)
- [x] la 1366x768 si 1024x768 nu iese nimic din ecran (capturi)
- [x] testele de taste si de mouse din etapa 16 trec (doua au primit
      coordonatele noi: capul coloanei e sub bara de sus, latimea implicita
      e 244 px ca in macheta; ce verifica e acelasi)
- [ ] fiecare buton mutat intr-un meniu are inca handler si test care il
      apasa
- [x] capturi in ambele moduri
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. Bara de sus si antetul obiectului (scot titlul mare si insulele).
2. Panourile pe toata suprafata; bara de jos devine linia de scurtaturi.
3. Coloana listei ca in macheta.
4. Perechea captura / macheta, aratata lui Mihai.

## Jurnal de lucru

- **2026-11-19** Etapa scrisa, nepornita, dupa ce Mihai a vazut editorul
  din etapa 16.

- **2026-11-19** Pasii 1-3 facuti intr-un singur pas, si perechea in
  `cercetare/capturi/editor-langa-macheta.png`. **Asteapta „da"-ul lui Mihai.**
  Diferentele ramase fata de macheta, fiecare cu motivul:
  - **bara de sus** are in plus Lista (Esc), ‹ › (k / j), tema si limba: nu
    exista alta cale cu mouse-ul spre ele; eticheta de mediu (PROD) lipseste,
    fiindca a fost scoasa peste tot (Mihai, 2026-11-18);
  - **antetul obiectului** are in plus ↑ 1/4 ↓ (n / p) si „Actiuni ▾" (tot ce
    statea in barele de jos: Aplica toate, Reset, Anuleaza, Editeaza, Mod
    ghidat, Omite, Pliaza, Ascunde diferentele, Re-sincronizeaza, Uneste
    cursorul, legenda culorilor), toate cu tastele pastrate;
  - **panourile** scriu „Martor" / „Modificata", nu „Sursa" / „Baza"
    (CLAUDE.md §6), iar capul arata si serverul;
  - **coloana**: lipsesc vederile Stare / Schema / Tip si filtrul „fara
    revizuite", grupele care se pliaza (▸) cu „+ inca N", „se vor crea / se
    sar" si „Export CSV" jos; numele complet la hover apare jos in coloana,
    nu ca bula (intr-un `ListView` bula se taie la margine);
  - **jos**, in dreapta, ora ultimei salvari in loc de „sesiune: data ·
    server".

## Ce a iesit altfel decat planul

(se completeaza la incheiere)
