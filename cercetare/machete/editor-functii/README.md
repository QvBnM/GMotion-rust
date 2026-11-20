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
