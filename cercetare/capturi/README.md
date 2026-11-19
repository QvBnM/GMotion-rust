# Capturi ale ecranelor

Randate fara ecran, cu exemplele `captura*`. **O captura nu dovedeste ca un
buton merge** (vezi capcana din `STARE.md`); dovedeste ce incape si cum arata.

Toate se refac din `rust/`, pe instantaneul de previzualizare:

```
INST=../referinta/instantanee/previzualizare.json
REF=CharismaERP_Referinta
BAZA=CharismaERP_Client1
OBIECT=dbo.uspReproceseazaFacturi
OUT=../cercetare/capturi
```

| Fisier | Comanda |
|---|---|
| `editor-{intunecat,luminos}.png` | `cargo run --release -q -p gmotion-fereastra --example captura -- $INST $REF $BAZA $OBIECT $OUT/editor-TEMA.png --tema TEMA` |
| `lista-*.png`, `lista-fara-detalii-*.png` | `cargo run --release -q -p gmotion-app --example captura_lista -- $INST $REF $BAZA $OUT/lista-TEMA.png [OBIECT] --tema TEMA` |
| `deploy-*.png` | `cargo run --release -q -p gmotion-app --example captura_deploy -- ...` |
| `conectare-*.png` | `cargo run --release -q -p gmotion-fereastra --example captura_conectare -- ...` |

## Rezolutii mici

Pana la auditul QA din 2026-11-18, **toate** capturile erau la 1440x900, si de
aceea V1-V3 (butoane iesite din ecran la 1366 si mai jos) n-au fost vazute
niciodata. Fiecare exemplu primeste acum `--latime` si `--inaltime`, iar setul
standard are si cel mai comun ecran de laptop:

```
cargo run --release -q -p gmotion-fereastra --example captura -- \
    $INST $REF $BAZA $OBIECT $OUT/editor-1366x768-intunecat.png --latime 1366 --inaltime 768
cargo run --release -q -p gmotion-app --example captura_lista -- \
    $INST $REF $BAZA $OUT/lista-1366x768-intunecat.png --latime 1366 --inaltime 768
cargo run --release -q -p gmotion-fereastra --example captura -- \
    $INST $REF $BAZA $OBIECT $OUT/editor-1024x768-intunecat.png --latime 1024 --inaltime 768
```

`editor-1024x768-intunecat.png` arata insulele de jos pe **doua randuri**:
asezate pe unul singur, jumatate din butoane ieseau din ecran (V3).
