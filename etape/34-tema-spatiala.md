# Etapa 34 — Tema spatiala: pornire, meniu, ecrane dedicate

**Stare:** NEPORNITA
**Branch:** claude/admiring-mccarthy-o6s86e
**Deschisa:** — · **Incheiata:** —

> **Nu poate porni inca:** etapa 33 e ACTIVA si asteapta probele lui Mihai pe
> Windows (C1 si C2: repausul fara cadre, si niciun cadru peste 16,7 ms la
> clic in Object Explorer, taburi si Ctrl+W, pe 12.000 de obiecte). Exact o
> etapa poate fi ACTIVA. Criteriile de mai jos sunt scrise inainte de lucru,
> cum cere regula; se trece pe `ACTIVA`, cu data, cand 33 se incheie.

## Scop

Aplicatia porneste cu intrarea `goMotion` si intra intr-un meniu din care se
alege una din cele patru directii; fiecare isi deschide ecranul ei. Pornirea
si meniul sunt pe tema spatiala; de la conectare incolo, materialul de azi.
Azi aplicatia porneste direct in conectare si nu exista meniu.

Decizia si limitele: `JURNAL.md`, 2026-11-28. Machetele:
`cercetare/machete/tema-spatiala/`.

## Criterii de terminare

- [ ] **Masuratoarea, inainte de orice cod de desen.** `masura_ui` da costul
      pe cadru al cerului in trei variante: imagine coapta statica, imagine
      coapta cu deriva, desenat integral. Cifra intra in jurnalul de lucru si
      decide varianta. Fara cifra, nu se alege.
- [ ] **Sfera:** se alege cadre pregenerate sau `SharedPixelBuffer`, cu
      motivul si cifra in `JURNAL.md`. O dependinta noua are intrare proprie.
- [ ] Ecran nou `meniu` in `ECRANE` din `rust/fereastra/build_shell.rs`, cu
      `.slint`-ul lui si reexport in `fereastra.slint`.
- [ ] Ecran nou `pornire`, care ruleaza o data pe sesiune si se sare cu orice
      tasta sau clic.
- [ ] Cele patru categorii vin din `Registru` (`rust/platforma/src/modul.rs`),
      nu dintr-o lista scrisa de mana in interfata. `diff-obiecte` e singurul
      inregistrat; celelalte trei sunt declarate ca „in curand", intr-un loc
      unde se vede ca nu sunt module.
- [ ] Fiecare categorie deschide un ecran al ei. Cele trei viitoare deschid un
      ecran propriu cu ramurile planificate si „in curand", nu un mesaj de
      eroare si nu nimic.
- [ ] Taste: `1`-`4` si sagetile aleg, `Enter` deschide, `Esc` intoarce in
      meniu. Probate cu testele headless (`dispatch_event`), nu cu capturi:
      sub Xvfb tastele trimise cu XTest nu ajung in aplicatie.
- [ ] Tot textul nou trece prin `fereastra::texte`, in romana si engleza.
- [ ] Pornirea si meniul raman intunecate si cu comutatorul de tema pe
      „luminos"; comutatorul isi pastreaza efectul de la conectare incolo.
- [ ] Fara diacritice nicaieri, inclusiv in textele noi.
- [ ] Se poate ajunge la orice din meniu si fara meniu: `cli` nu capata un pas
      in plus, iar aplicatia accepta o optiune care sare direct in comparare.
- [ ] **Captura langa macheta**, la aceeasi rezolutie, pereche cu pereche
      (pornire, meniu, un ecran „in curand"), si „da"-ul lui Mihai pe fiecare
      pereche. Fara asta etapa nu se bifeaza, oricat de complet ar fi codul.
- [ ] poarta verde: `unelte/verifica.sh`
- [ ] `STARE.md` actualizat
- [ ] `JURNAL.md` completat, daca s-a luat vreo decizie

## Pasi

1. `masura_ui`: costul cerului in cele trei variante. Se alege varianta.
2. Ecranul `meniu`, static, pe materialul spatial ales la pasul 1. Fara
   animatii inca: asezarea si tastele intai.
3. Rutarea: `Registru` -> randurile meniului -> ecranul fiecarei categorii.
   Cele trei viitoare, cu ramurile lor.
4. Ecranele „in curand", unul singur parametrizat, nu trei copii.
5. Ecranul `pornire`: sfera, literele, bara de incarcare pe pasii adevarati.
   Scannerul urca pe procentul citit.
6. Mutarea lockup-ului intre pornire, meniu si antetul ecranului dedicat.
7. Steaua cazatoare, legata de o lucrare terminata pe firul de fundal, nu de
   un cronometru.
8. Capturile, langa machete, pentru „da"-ul lui Mihai.

## Jurnal de lucru

- **2026-11-28** Criteriile scrise, etapa inca nepornita (33 e activa).
  Machetele si prototipul rulabil sunt gata si
  aprobate; nu s-a atins inca niciun fisier din `rust/`. Presupunerea cu care
  s-au scris criteriile: „tema spatiala" inseamna taietura din `JURNAL.md`
  (spatial pe pornire si meniu, sculptat pe ecranele de lucru). Daca Mihai
  vrea spatial si pe ecranele de lucru, criteriile de mai sus se rescriu
  inainte de a incepe, si se masoara intai derularea listei peste cer.

## Ce a iesit altfel decat planul

Se completeaza la incheiere.
