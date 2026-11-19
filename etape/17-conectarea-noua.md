# Etapa 17 — ecranul de conectare dupa macheta noua

**Stare:** NEPORNITA
**Branch:** claude/upbeat-ramanujan-dm6j8c
**Deschisa:** 2026-11-19 · **Incheiata:** —

> In lucru in sesiunea `session_01FH9EWVHVrrpH7Qfopnidbk` („Prima pagina
> GMotion Connect"), in paralel cu etapa 19 (ACTIVA, alta sesiune). Poarta
> cere o singura etapa ACTIVA; 17 trece in ACTIVA cand se incheie 19.

## Scop

Ecranul de conectare arata ca `cercetare/machete/conectare-noua/` (aprobata de
Mihai pe 2026-11-19): martorul si modificata alaturi, simetrice, fiecare cu
conexiunea lui si lista bazelor; bifa „Aceeasi conexiune" pe modificata;
o singura baza de fiecare parte; Compara la mijloc; comparari recente si
jurnalul dedesubt; o singura scara pentru tot ecranul.

## Criterii de terminare

- [x] model: bifa „Aceeasi conexiune" — cand e pusa, modificata ia serverul,
      autentificarea, acreditarile, bazele si starea martorului, si le tine
      la zi cand martorul se schimba; baza aleasa ramane doar daca exista in
      lista (test pe model)
- [x] model: la modificata se alege **o singura** baza (un clic pe alta o
      inlocuieste); baza martorului nu se poate alege acolo cand conexiunea
      e aceeasi (test pe model)
- [x] ecran: in fiecare card, sus conexiunea (SQL Server / Windows, server,
      utilizator, parola, starea, Testeaza), dedesubt lista bazelor cu cerc,
      derulabila; cu bifa pusa, conexiunea modificatei e inlocuita de o zona
      de aceeasi inaltime cu serverul martorului
- [x] fara etichete de mediu, fara banda de sus, fara randul cu certificatul;
      certificatul se schimba din **Setari** (antet)
- [x] Compara la mijloc, cu „Se citeste, nu se scrie nimic." dedesubt;
      progresul compararii in locul lui
- [x] proportiile: cardurile 3/4, jurnalul 1/4 (cel putin 130 px, cat cele 4 linii), cardurile
      pana la 640 px; sub 820 px inaltime dispar comparatiile recente;
      continutul cel mult 1760 px, centrat
- [x] scara unica: `min(latime/1440, inaltime/900)`, intre 0,85 si 1,3; cand
      Windows e deja la 125% sau peste, minimul e 1 (test pe functia pura si
      pe fereastra de test)
- [x] fiecare buton nou are actiune si test care il apasa (bifa, cercurile
      din liste, Setari)
- [ ] captura la 1440x900 pusa langa macheta, in ambele teme, si „da"-ul lui
      Mihai pe pereche (CLAUDE.md: un ecran facut dupa macheta arata ca ea)
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md` actualizat
- [x] `JURNAL.md` completat (o singura baza la modificata; certificatul in
      Setari; scara unica)

## Pasi

1. Modelul: `aceeasi_conexiune`, `sincronizeaza`, alegerea unei singure baze.
2. Textele noi in `texte.rs` (romana si engleza).
3. `conectare.slint` rescris dupa macheta; Setari ca fereastra mica in ecran.
4. Scara: functie pura + aplicarea pe fereastra la redimensionare.
5. Legaturile din `rust/app`, testele, captura langa macheta.

## Jurnal de lucru

- **2026-11-19** Etapa deschisa dupa „da"-ul lui Mihai pe macheta si pe cele
  trei puncte ramase (fara micsorare la 150%, fara banda, certificatul la
  Setari).
- **2026-11-19** Facut tot, in afara de „da"-ul lui Mihai pe perechi
  (`cercetare/machete/conectare-noua/capturi/pereche-*.png`, macheta |
  aplicatia, 1440x900, ambele teme). Scara se pune ca factor de scala al
  ferestrei (`WindowEvent::ScaleFactorChanged` + `Resized`), recalculat la
  fiecare redimensionare; probat si pe backend-ul adevarat (winit, X11 pe
  Xvfb): 1920x1080 -> 1,2, 1280x720 -> 0,85. „Scund" il pune Rust: calculat
  in `.slint` din `height` era bucla de layout.
- **2026-11-19** Mihai: „lasam cum este macheta" — butonul ⇄ si „Ascunde"
  din jurnal raman scoase. `Conectare::inverseaza`, callback-ul si
  handler-ul lui s-au scos odata cu ele (erau cod mort).
- Abateri fata de macheta, de vazut cu Mihai: textele sunt cele din aplicatie
  (Martor / Modificata, „Continua" in loc de „Reia"); butonul de inversare
  (⇄) nu e in macheta, deci a disparut de pe ecran (callback-ul a ramas);
  „Ascunde" din jurnal la fel. La scara 0,85 liniile de 1 px ies mai
  palide (randarea software le netezeste).

## Ce a iesit altfel decat planul

(la incheiere)
