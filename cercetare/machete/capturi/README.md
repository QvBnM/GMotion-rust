# Capturi ale ecranelor native, langa machete

Ce s-a construit, fotografiat, langa ce s-a aprobat. Toate pe acelasi obiect
(`dbo.TESTSIMILITUDINE_AnalizaVanzari` din
`referinta/instantanee/testsimilitudine.json`), ca sa fie comparabile.

| Fisier | Ce arata | Cu ce se compara |
|---|---|---|
| `Editor-nativ-alaturat.png` | editorul nativ, alaturat, cu plierea pornita | `../Editor.dc.html` |
| `Editor-nativ-unificat.png` | acelasi obiect, modul unificat | — (macheta are tabul, nu si randarea) |
| `Editor-v34.png` | acelasi obiect in interfata v3.4 | referinta de **continut**, nu de aspect |
| `Main-nativ.png` | lista de diferente + previzualizarea | `../Main.dc.html` |
| `Main-v34.png` | lista v3.4, aceeasi comparare | referinta de **continut** |

Se refac asa (din `rust/`):

```
cargo run --release -p gmotion-fereastra --example captura -- \
    ../referinta/instantanee/testsimilitudine.json \
    CharismaERP_GetsBet CharismaERP_GetsBet_Copy \
    dbo.TESTSIMILITUDINE_AnalizaVanzari ../cercetare/machete/capturi/Editor-nativ-alaturat.png
```

Lista (din `rust/`):

```
cargo run --release -p gmotion-app --example captura_lista -- \
    ../referinta/instantanee/testsimilitudine.json \
    CharismaERP_GetsBet CharismaERP_GetsBet_Copy \
    ../cercetare/machete/capturi/Main-nativ.png dbo.TESTSIMILITUDINE_CaseG_StringCuKeywords
```

Captura v3.4 (din radacina, cere `node` si Chromium):

```
python3 -I referinta/captura_v34/date.py \
    referinta/instantanee/testsimilitudine.json CharismaERP_GetsBet /tmp/v34/
node referinta/captura_v34/captura.mjs /tmp/v34 dbo.TESTSIMILITUDINE_AnalizaVanzari
```
