# gmotion-app

<!-- api-hash: fa663677e1c8893e -->

## Rol

Binarul cu interfata (`gmotion-ui`): porneste ferestrele din
`gmotion-fereastra`, le leaga de motor si tine starea aplicatiei — ce ecran e
activ, ce obiect e deschis, ce s-a revizuit.

Starea si rutarea stau in `rutare.rs`, **fara Slint**, si se testeaza acolo.
Binarul (`main.rs`) doar deseneaza si trimite comenzi.

## Nu face

- Nu compara si nu clasifica nimic: tot ce tine de motor vine din
  `gmotion-modul-diff-obiecte` si `gmotion-core`.
- Nu tine secrete si nu salveaza acreditari: progresul merge prin sesiunile
  din `gmotion-platforma`, care nu au camp de parola.
- Nu are inca selector de fisiere pentru instantaneu: calea se da la pornire
  (`--sursa`). Vezi `STARE.md`.
- Nu arata progresul unei comparari lungi: pe server real, compararea blocheaza
  fereastra pana se termina. De rezolvat cand apare ecranul de progres.

## API public

| Element | Ce face |
|---|---|
| `Aplicatie::noua(randuri, sursa, tinte, revizuire)` | starea initiala, din randurile unei comparari; `tinte` sunt bazele comparate, una sau mai multe |
| `Aplicatie::{uneste_cursorul, are_cursor}` | starea comutatorului de unire a ciclului de cursor, si daca obiectul curent are cursor |
| `Aplicatie::baze()`, `tinte()` | bazele comparate si obiectele lor pregatite |
| `Aplicatie::comanda(Comanda) -> bool` | executa o comanda; `true` = revizuirea sau textul final s-au schimbat (de salvat) |
| `Aplicatie::cu_editari`, `editari`, `editari_de_salvat`, `texte_finale`, `texte_curente`, `stare_text_final`, `se_poate_anula`, `diferenta_curenta`, `ghidat` | editarea in editor: textul final (Aplica, Aplica toate, Reset, Ctrl+Z, scris de mana), modul ghidat (`Ghidat`), salvate in sesiune |
| `Comanda` | tot ce poate cere utilizatorul din oricare ecran; `SalveazaText { id, text }` duce si cheia obiectului si se refuza daca nu e cel deschis (F-A1) |
| `Aplicatie::id_obiect()` | cheia obiectului deschis, `baza/schema.nume`: pleaca spre fereastra si se intoarce cu textul scris de mana |
| `Ecran` | `Lista`, `Editor` sau `Deploy` |
| `Aplicatie::model_editor()`, `texte_obiect()`, `asigura_detalii(de_la, pana_la)` | ce are nevoie desenul; `model_editor` da un `Ref` la modelul tinut minte pe `(obiect, amprenta textului final, unirea cursorului)`, nu-l reface la fiecare desenare (P1) |
| `Aplicatie::refaceri_model()` | de cate ori s-a construit modelul editorului: masura inghetarilor, si singurul fel in care se poate proba ca desenarea nu-l reface |
| `Aplicatie::detalii_deschise()`, `panou_detalii() -> PanouDetalii` (`pentru_fereastra`), `impact_obiect()`, `nota_obiect()` | panoul de detalii din lista: deschis la `ClicPeRand`, inchis la al doilea clic / `InchideDetalii`; impactul (calculat lenes, o data per comparare) si nota (`PuneNota`) |
| `Aplicatie::revizuire_de_salvat()` | starea curatata, gata de pus in sesiune |
| `Aplicatie::plan(referinta, facut_la)`, `sursa()` | planul de deploy din ce e „in plan" (`Comanda::ComutaInPlan`), cu amprentele compararii; acopera toate bazele comparate |
| `Aplicatie::refuzat_in_plan()` | ultimul „In plan" a fost refuzat (obiect fara cod de deploiat); ecranul spune de ce (F-A18) |
| `Aplicatie::marcheaza_deployate(plan)` | dupa o scriere reusita: obiectele din plan devin „deployate", cu amprenta lor |
| `Aplicatie::randuri_vizibile()`, `exporta_csv(cale)` | lista **filtrata**, si exportul ei CSV (acelasi fisier ca `gmotion export`) |
| `deploy::StareDeploy` (`nou`, `pasi_in_ordine`, `aplica_verificarea(referinta, tinte, la)`, `verificare_esuata`, `poate_scrie`, `scrie(director, acum, texte_finale)`, `alege_impartirea`) | ecranul de deploy: garda pe obiectele recitite, scrierea cu revenirea inainte, aceleasi functii ca `gmotion deploy --plan`; textele finale se dau **la scriere** |
| `deploy::{Cheie, Deschis, asigura_planul(deschis, app, referinta, acum)}` | planul deschis e legat de ce e „in plan", de textele finale si de `Aplicatie::id_comparare`; cand una se schimba, planul se reface fara garda |
| `Aplicatie::id_comparare()`, `amprenta_texte_finale()` | identitatea compararii si a textelor finale: din ele se face `deploy::Cheie` |
| `deploy::{ecran, impartire_din, pozitia_impartirii}`, `Garda`, `PasAfisat` | ce se deseneaza pe ecranul de deploy |
| `pornire::deschide(c, parte)` | sursa adevarata (server sau instantaneu) din ce s-a ales pe ecran |
| `pornire::citeste_bazele(c, parte)` | bazele si starea de afisat — „conectat" doar daca chiar a citit |
| `fir::Rulare` (`porneste`, `verifica`, `anuleaza`, `text`, `timp`, `fractie`) | compararea pe alt fir, cu progres si Anuleaza |
| `fir::Lucru<T>` (`porneste(f)`, `verifica`, `timp`) | orice alt lucru lung pe alt fir: citirea bazelor, „Testeaza conexiunea", cautarea de servere, garda de deploy (P4) |
| `pornire::testeaza(c, parte)` | Testeaza conexiunea: ce raspunde serverul, fara parola in mesaj |
| `pornire::instantanee_gasite()` | fisierele .json de ales in „Deschide un instantaneu", fara dialog nativ |
| `pornire::compara_offline(sursa, tinta, limba)` | compararea a doua bucati de cod lipite, fara server |
| `pornire::citeste(c, progres) -> Citit`, `reciteste(c)` | doar citirea obiectelor, fara comparare: atat ii trebuie garzii de deploy (P4) |
| `pornire::compara(c) -> Comparata`, `compara_cu(c, progres)`, `Pas`, `ANULAT` | citeste referinta si fiecare baza bifata si compara; `compara_cu` da progresul si se poate anula. `Comparata::referinta` e baza **cu care a rulat**, nu ce scrie in formular la final (F-A3) |

## Invariante

1. **Un singur obiect „curent".** Ce e selectat in lista e si ce se deschide in
   editor; `j` / `k` din editor **muta selectia**, ca intoarcerea in lista sa
   cada unde trebuie.
2. **Bifa de revizuit trece prin `Revizuire`,** nu doar prin lista: altfel ar
   arata bifat ceva ce nu s-a salvat.
3. **Detaliile („Difera in", `+N −M`) se calculeaza doar pentru randurile
   cerute.** Pe comparari mari, calculul pentru tot ar fi aruncat aproape
   integral.
4. **Un obiect care exista doar intr-o parte nu deschide editorul:** nu are cu
   ce fi comparat. `ObiectUrmator` sare peste el.
5. **Parola iese din `Acreditari` intr-un singur loc:** `pornire::deschide`.
   Nu ajunge in starea afisata, in sesiuni sau in mesajele de eroare — exista
   test.
6. **Nota se pune pe obiectul primit in comanda (`PuneNota(id, text)`), nu pe
   selectie.** Campul de nota poate pierde focul dupa ce selectia s-a mutat
   deja pe alt rand; legata de selectie, nota ar ajunge pe obiectul gresit.

## Erori tipice

| Simptom | Unde caut |
|---|---|
| Bifa din editor nu se vede in lista | `comuta_revizuit` din `rutare.rs` |
| `j` / `k` te intorc in lista | `treci` din `rutare.rs` |
| Panoul de detalii nu se deschide / nu se inchide la clic | `Comanda::ClicPeRand` din `rutare.rs`; in `lista.slint`, `clic-rand` |
| Nota apare pe alt obiect | `nota-id` din `lista.slint` si `incarca_detalii` in `gmotion-fereastra` |
| Obiectul deschis se schimba dupa un filtru | `Lista::recalculeaza` in `gmotion-fereastra` |
| Fereastra nu porneste pe un server fara GPU | backend-ul Slint: `winit` + `software` |
| Progresul nu se salveaza | sesiunile din `gmotion-platforma`, `GMOTION_DIR_DATE` |

## Testare

```
cargo test -p gmotion-app
cargo run --release -p gmotion-app --example captura_flux -- \
    ../referinta/instantanee/testsimilitudine.json \
    CharismaERP_GetsBet CharismaERP_GetsBet_Copy lista.png editor.png [schema.obiect]
```

`captura_flux` trece chiar prin `Comanda::Deschide`, deci fotografiaza drumul
adevarat din lista in editor, nu un editor deschis de mana.

`tests/flux.rs` face acelasi drum fara fereastra, de la ecranul de conectare
pana in editor: acolo se prinde cand bucatile merg separat, dar nu si legate.

## Depanare

```
GMOTION_JURNAL=gmotion::app=debug,gmotion::platforma=debug gmotion-ui --sursa f.json --referinta R --baza B
GMOTION_DIR_DATE=/tmp/date-de-proba gmotion-ui ...   # sesiuni intr-un loc separat
SLINT_DEBUG_PERFORMANCE=refresh_lazy,console gmotion-ui ...
```
