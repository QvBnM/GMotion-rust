# Machete: functiile din editor care lipsesc

Tinta pentru etapa 21. Trei capturi, pe formatul de acum (etapa 19), cu
**date reale din motor** — blocurile, scorurile si locul unde cade fiecare
bloc sunt iesirea lui `aliniaza` si `plasare::sugereaza`, nu desen de mana.

Cazul: `martor.sql` fata de `modificata.sql` (un `EXEC` de audit scos, un
`SELECT` care a primit coloane, antet cu parametru nou). De verificat oricand:

```
cd rust && cargo run --release -p gmotion-cli -- diferente \
    --sursa <instantaneu> --referinta Martor --baza Modificata --obiect dbo.<nume>
```

| Captura | Ce arata |
|---|---|
| `1-mapare-fantoma-etichete.png` | eticheta pe bloc (ce e si unde ajunge), legatura in canal (plina = are corespondent, punctata cu cerc gol = n-are), randul-fantoma „aici vin blocurile ... din Martor" |
| `2-drag-si-plaseaza-aici.png` | blocul tras din Martor, tintele legale aprinse: banda „Plaseaza aici" intre blocuri si conturul blocului peste care se poate cadea |
| `3-previzualizare-la-hover.png` | rezultatul aplicarii, fara sa aplici |

Umplutura dintr-un bloc (randul gol din dreptul unei linii care exista doar in
cealalta parte) e **hasurata** in culoarea blocului — decizia lui Mihai,
2026-11-20, vezi `JURNAL.md`. Fara hasura, blocul pare rupt in doua si golul
pare o linie pierduta.

## Capturile aplicatiei (etapa 21)

In `capturi/`, pe acelasi caz, 1600x760, ambele teme, plus perechile
macheta | aplicatia (`pereche-*.png`, tema intunecata, ca machetele):

```
cd rust && cargo build --release -p gmotion-fereastra --example captura
./target/release/examples/captura inst.json CharismaERP_GetsBet CharismaERP_GetsBet_Copy \
    dbo.uspLeasing2_GetTemplatesMultipleDocuments iesire.png --latime 1600 --inaltime 760 \
    [--trage 1 | --previzualizeaza 2] [--tema luminos]
```

`inst.json` e un instantaneu cu `martor.sql` in baza `CharismaERP_GetsBet` si
`modificata.sql` in `CharismaERP_GetsBet_Copy`. `--previzualizeaza` plimba
mouse-ul pana ajunge pe sageata, nu pune starea direct.

Abateri fata de machete: in previzualizare, explicatia nu numeste blocul
(„daca aplici blocul", fara numar — eticheta de pe bloc il spune deja);
banda „Plaseaza aici" e pe randul-fantoma, nu o banda separata intre blocuri.
