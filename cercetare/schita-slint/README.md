# Schita in Slint: Harta si Panza

**Ce dovedeste:** cele doua directii alese de Mihai pe 2026-11-18 (Harta pentru
lista obiectelor, Panza pentru editorul de comparare) se pot desena in Rust, cu
Slint 1.19 si randare software, exact cum ruleaza aplicatia prin RDP fara GPU.
Diferenta afisata vine din motorul adevarat (`model_editor::construieste` pe
`referinta/instantanee/testsimilitudine.json`). Harta foloseste o baza generata
de 21.183 de obiecte, cu distributia din machete.

**Nu e cod livrat.** Crate-ul nu e in workspace-ul aplicatiei si nu trece prin
poarta. La portare, culorile se muta in `rust/fereastra/src/jetoane.rs` (regula
„nicio culoare in `.slint`"), iar ecranele in `rust/fereastra/ui/`.

```bash
cd cercetare/schita-slint
CARGO_TARGET_DIR=../../rust/target cargo run --release -- harta capturi/harta.png
CARGO_TARGET_DIR=../../rust/target cargo run --release -- panza capturi/panza.png
CARGO_TARGET_DIR=../../rust/target cargo run --release -- panza capturi/panza-paleta.png --paleta
```

Capturile, 1440×900, sunt in `capturi/`.

## Masuratori (Linux, release, randare software, 1440×900)

| Ecran | Pregatire | Primul cadru |
|---|---|---|
| Harta (8 grupuri, 336 de patrate) | 0,4 ms pictura patratelor | 12–17 ms |
| Panza (71 de randuri, 4 diferente) | 8–12 ms modelul din motor | 14 ms |
| Panza cu paleta Ctrl K deschisa | idem | 24 ms |

## Fiecare element, si cum se face in Slint

| Element din machete | Cum e facut aici | Stare |
|---|---|---|
| Patratele hartii | **pictate in Rust intr-o imagine** (`SharedPixelBuffer`), una pe grup, nu sute de elemente Slint | dovedit |
| Bula la hover, click pe patrat | `TouchArea` da coordonatele; Rust calculeaza patratul (`rand × coloane + coloana`) | dovedit (bula) |
| Conturul patratului selectat | un `Rectangle` pus peste imagine, la coordonatele calculate in Rust | dovedit |
| Grupurile asezate „zidarie" | Slint n-are asezare cu rupere de rand; pozitiile se calculeaza in Rust (cea mai scurta coloana) | dovedit |
| Codul din panoul hartii, unificat | `ListView` virtualizat, bucatile colorate ca in editorul actual | dovedit |
| Titlul uriaș al panzei | `Text` 48 px, `letter-spacing` negativ | dovedit |
| Randul schimbat cu degrade | `@linear-gradient`, suportat de randarea software | dovedit |
| Insulele care plutesc, cu umbra | `drop-shadow-*`, suportat de randarea software (`box_shadow.rs`) | dovedit; costul prin RDP, de masurat pe Windows |
| Inelul de progres | `Path` cu arc, calculat in Rust | dovedit |
| Rigla diferentelor din stanga | randuri de `Rectangle`, din `model.diferente` | dovedit |
| Paleta Ctrl K | strat peste panza, in aceeasi fereastra (nu `PopupWindow`) | dovedit |
| Fonturile Geist si Geist Mono | `import "*.ttf"` in `.slint`, incluse in binar, licenta OFL | dovedit, cu diacritice |

## Ce s-a schimbat fata de prototipul HTML, ca sa se poata face

- **Identicele nu mai sunt patrate.** Pe 21.183 de obiecte nu incap si nu
  ajuta. Fiecare grup arata doar ce difera, iar sub el o bara cu proportia si
  „9.271 identice din 9.412".
- **O singura familie de litere: Geist.** Harta avea Sora, careia ii lipsesc
  sagetile (`→`, `←`) si bifa. O aplicatie are acum un singur set de fonturi.
- **Titlul panzei sta fix sus** cand derulezi codul. `ListView` derulează
  doar codul, iar asa vezi mereu ce obiect citesti.
- **Plierea are linii continue,** nu punctate: Slint nu deseneaza contur
  punctat.
- **Contrastul trece 4,5:1** (3:1 pentru „dbo." de 48 px). Prototipul HTML
  avea numerele de linie la 2:1 si PROD la 3,3:1; aici sunt intunecate minim.

## Ce ramane de dovedit la portare

1. **Ecran HiDPI.** Imaginea hartii trebuie pictata la factorul de scala al
   ferestrei, altfel patratele ies neclare pe 150%.
2. **Tastatura pe harta:** sagetile muta selectia intre patrate (logica in
   Rust, testabila fara fereastra).
3. **Cititorul de ecran nu vede patratele unei imagini.** Harta are nevoie de
   o eticheta accesibila pe grup si de navigarea cu tastatura de la punctul 2.
4. **RDP pe Windows:** umbrele insulelor si degradeurile, masurate pe o
   sesiune RDP reala. Daca sunt scumpe, se renunta la umbre, nu la insule.
5. **Trecerea de la harta (inchisa) la panza (alba)** pe acelasi monitor:
   de vazut pe ecran adevarat daca deranjeaza ochiul.
