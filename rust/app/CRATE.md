# gmotion-app

<!-- api-hash: 4f535b912dd1c894 -->

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
| `Aplicatie::noua(randuri, sursa, tinta, revizuire)` | starea initiala, din randurile unei comparari |
| `Aplicatie::comanda(Comanda) -> bool` | executa o comanda; `true` = starea de revizuire s-a schimbat (de salvat) |
| `Comanda` | tot ce poate cere utilizatorul din oricare ecran |
| `Ecran` | `Lista` sau `Editor` |
| `Aplicatie::model_editor()`, `texte_obiect()`, `asigura_detalii(de_la, pana_la)` | ce are nevoie desenul |
| `Aplicatie::revizuire_de_salvat()` | starea curatata, gata de pus in sesiune |
| `pornire::deschide(c, parte)` | sursa adevarata (server sau instantaneu) din ce s-a ales pe ecran |
| `pornire::citeste_bazele(c, parte)` | bazele si starea de afisat — „conectat" doar daca chiar a citit |
| `pornire::compara(c) -> Comparata` | citeste ambele baze si compara |

## Invariante

1. **Un singur obiect „curent".** Ce e selectat in lista e si ce se deschide in
   editor; `j` / `k` din editor **mută selectia**, ca intoarcerea in lista sa
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

## Erori tipice

| Simptom | Unde caut |
|---|---|
| Bifa din editor nu se vede in lista | `comuta_revizuit` din `rutare.rs` |
| `j` / `k` te intorc in lista | `treci` din `rutare.rs` |
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
