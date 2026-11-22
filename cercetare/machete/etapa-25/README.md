# Capturi pentru etapa 25

La 1600x1000, ambele teme, pentru „da"-ul lui Mihai:

- `lista-recompara-*.png` — Recompara ruleaza: pasul, bara si Anuleaza in
  bara listei (in locul butonului Recompara); coloana noua „Ultima
  modificare", intre Impact si Revizuit.
- `despre-*.png` — „Despre", deschis din Ajutor (F1): versiunea, licentele,
  caile jurnalului si datelor, sigla Slint ceruta de licenta.
- `editare-*.png` — editarea de mana (`e`), cu sintaxa colorata cat scrii.

Comenzile (din `rust/`):

```
I=../referinta/instantanee/previzualizare.json
cargo run -p gmotion-app --example captura_lista -- $I CharismaERP_Referinta \
    CharismaERP_Client1,CharismaERP_Client2_Prod iesire.png \
    --latime 1600 --inaltime 1000 [--recompara | --despre] [--tema intunecat]
cargo run -p gmotion-fereastra --example captura -- \
    ../cercetare/machete/etapa-25/editare-inst.json Martor Modificata \
    dbo.rptInventar_Situatie iesire.png --latime 1600 --inaltime 1000 \
    --fara-pliere --editeaza [--tema intunecat]
```

Dialogul nativ de fisiere (`rfd`) nu se poate fotografia: e al sistemului.
Pe Windows: „Deschide un instantaneu" -> „Alege fisierul…"; in ecranul de
deploy, „Alege folderul…" si, dupa scriere, „Deschide folderul".
