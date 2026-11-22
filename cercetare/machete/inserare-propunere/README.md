# Locul unde intra blocurile doar in Martor (2026-11-22)

Mihai: „sa se vada mai bine unde ar merge". Variantele propuse: cutie
punctata cat tot blocul, sau un rand colorat. Ales: **randul colorat, mai
mic, rosu** („ca e gen o lipsa").

- `acum-luminos.png` — inainte: o linie rosie subtire deasupra textului.
- `ales-*.png` — banda rosie de 16 px pe primul rand al golului, cu bara in
  stanga si textul „aici vin blocurile ... din Martor" in ea; legatura din
  canal se termina in dreptul ei.

Comanda (din `rust/`):

```
cargo run -p gmotion-fereastra --example captura -- \
    ../cercetare/machete/etapa-25/editare-inst.json Martor Modificata \
    dbo.rptInventar_Situatie iesire.png --latime 1600 --inaltime 1000 \
    --fara-pliere [--tema intunecat]
```
