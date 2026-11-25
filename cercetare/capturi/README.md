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
cargo run --release -q -p gmotion-app --example captura_lista -- \
    $INST $REF $BAZA $OUT/lista-1024x768-intunecat.png $OBIECT --latime 1024 --inaltime 768
```

`lista-1024x768-intunecat.png` e proba pentru C-1 din auditul 2026-11-23:
numele obiectelor trebuie sa se citeasca intregi. Coloanele secundare
(„Ultima modificare", „Impact", „Folosit de"...) dispar pe rand cand nu
incape numele — regula e in `Coloane` din `lista.slint`, iar testul
`numele_obiectului_are_loc_la_toate_rezolutiile` o pazeste.

**Toate exemplele primesc si `--limba ro|en`** (auditul 2026-11-23: pana
atunci numai `captura_lista` o accepta, deci interfata in engleza nu se
putea vedea pe 3 din 4 ecrane).

`editor-1024x768-intunecat.png` arata insulele de jos pe **doua randuri**:
asezate pe unul singur, jumatate din butoane ieseau din ecran (V3).


## Etapa 31 (`etapa-31/`): aplicatia adevarata, sub Xvfb

Nu din exemplele `captura*`, ci din `gmotion-ui` pornit sub Xvfb si condus
cu mouse-ul (`etapa-31/conduce_x.py`, cu `python-xlib`): conectare cu
instantaneul de previzualizare → „Adu bazele" pe ambele parti → Compara →
lista → panoul de detalii → sfaturile la hover → Deschide (editor) → In plan
→ Deploy. Toate in aceeasi fereastra. Fiecare ecran la 1366x768 si
1920x1080 (fereastra X redimensionata), in ambele teme:
`<ecran>-<tema>-<latime>x<inaltime>.png`. `comparare-in-curs-*` e prinsa
in timpul compararii (suprapunerea care blocheaza restul).

**Tastele trimise prin XTest nu ajung in aplicatie sub Xvfb**: drumul e
facut doar din clicuri; tastele sunt probate de testele din `fereastra`.

```
Xvfb :99 -screen 0 1920x1080x24 &
DISPLAY=:99 rust/target/proba/gmotion-ui --sursa referinta/instantanee/previzualizare.json --tema luminos &
DISPLAY=:99 python3 cercetare/capturi/etapa-31/conduce_x.py marime 1366 768 clic 368 344
DISPLAY=:99 import -window root -crop 1366x768+0+0 captura.png
```
