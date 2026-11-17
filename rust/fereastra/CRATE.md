# gmotion-fereastra

<!-- api-hash: b03cefbfd975e3e6 -->

## Rol

Interfata nativa a GMotion, desenata cu Slint (randare software: merge si prin
RDP pe servere fara GPU). Contine ferestrele si componentele, si modelele lor
in Rust (ce se deseneaza pe fiecare rand), testabile fara fereastra.

Stare: **prototip** (etapa 8) — editorul de diferente side-by-side.

## Nu face

- Nu compara si nu decide nimic: blocurile, imperecherea si bucatile schimbate
  vin din `gmotion_core::diff_aliniat`. Aici doar se aseaza pe ecran.
- Nu acceseaza baze de date si nu scrie fisiere.

## API public

| Element | Ce face |
|---|---|
| `model_editor::construieste(ref, tinta) -> ModelEditor` | randurile editorului: aliniate, numerotate, colorate |
| `ModelEditor`, `Rand`, `Parte`, `Bucata`, `Sintaxa`, `Stare` | modelul |
| `EditorDiferente` (Slint) | fereastra editorului |
| `Tema` (global Slint) | culorile si fonturile temei |
| `incarca_editor(ui, antet, model)`, `AntetEditor` | umple editorul |

## Invariante

1. **Un singur motor.** Modelul vine din `diff_aliniat`; nicio logica de
   comparare aici.
2. **Blocurile ocupa aceeasi inaltime in ambele panouri** (umplutura pe partea
   mai scurta), ca in v3.4.
3. **Numerele de linie sunt cele reale** (`linii_start_blocuri`), iar liniile
   pe care segmentarea le arunca (goale, doar-comentariu) apar ca context.
4. **Ambele panouri sunt pe acelasi rand al unui singur `ListView`:**
   derularea e sincronizata fara cod si virtualizata (obiecte uriase).
5. **Sintaxa se coloreaza pe tot blocul**, nu pe linie: string-urile si
   comentariile pe mai multe linii raman colorate corect.

## Erori tipice

| Simptom | Unde caut |
|---|---|
| Panourile nu mai sunt aliniate | `src/model_editor.rs`, `construieste` |
| Numere de linie gresite | `linii_start_blocuri` in `core`, capcana despre blocuri unite |
| O bucata schimbata nemarcata | `schimbat_pe_caractere` si `diff_intra` din `core` |
| Fereastra nu se deseneaza pe un server fara GPU | backend-ul: `winit` + `software` |

## Testare

`cargo test -p gmotion-fereastra` (modelul). Vizual: exemplul `captura`
randeaza editorul fara ecran intr-un PNG:

```
cargo run --release -p gmotion-fereastra --example captura -- instantaneu.json REF BAZA schema.obiect iesire.png [font-mono]
```

Referinta v3.4: `referinta/captura_v34/`.

## Depanare

`SLINT_DEBUG_PERFORMANCE=refresh_lazy,console` arata timpii de randare (opreste
randarea partiala cat masoara).
