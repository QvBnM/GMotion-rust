# Cazul pentru etapa 23 (editorul lizibil)

`martor.sql` / `modificata.sql` au forma din sesizarea lui Mihai
(`dbo.rptGenericInventoryObj_SituationAtDate`, 2026-11-20): antet cu
parametri scosi, patru instructiuni consecutive doar in Martor (`declare`,
`set`, doua `select`), o linie lunga modificata.

Capturile din `capturi/` (1600x1000, ambele teme; wrap la 1200):

```
cd rust && cargo build --release -p gmotion-fereastra --example captura
./target/release/examples/captura inst.json Martor Modificata dbo.rptInventar_Situatie \
    iesire.png --latime 1600 --inaltime 1000 --fara-pliere [--wrap] \
    [--trage 1 --sub-mouse 22] [--tema luminos]
```

`inst.json` e un instantaneu cu `martor.sql` in baza `Martor` si
`modificata.sql` in `Modificata`. In captura tragerii starea e pusa direct;
comportamentul il probeaza testele cu mouse-ul
(`hover_pe_sageata_previzualizeaza_si_tragerea_aplica_blocul`).
