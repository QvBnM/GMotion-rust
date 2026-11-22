# Capturi pentru etapa 26 (verificarea textului final)

`editor-probleme-*.png` (1600x1000): textul final din `text-final-stricat.sql`
(un `BEGIN` adaugat dupa `AS`, fara `END`). Jos in dreapta, rosu:

- „Textul final nu se mai compileaza: linia 14: BEGIN fara END" — verificarea
  locala, calculata de aplicatie;
- deasupra, mesajul serverului de la „Actiuni -> Verifica pe server".
  **In captura e pus de mana** (`--verificare`): pe masina de dezvoltare nu
  exista SQL Server. Pe un server real, textul e al serverului.

```
cd rust && cargo run -p gmotion-app --example captura_flux -- \
    ../cercetare/machete/etapa-25/editare-inst.json Martor Modificata \
    lista.png editor.png dbo.rptInventar_Situatie --latime 1600 --inaltime 1000 \
    --text-final ../cercetare/machete/etapa-26/text-final-stricat.sql \
    [--verificare "text"] [--tema intunecat]
```
