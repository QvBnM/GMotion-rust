# Propunere de redesign: „Banda"

**Stare:** propunere, neaprobata. Nu inlocuieste `cercetare/design-aprobat.md`
pana nu o aproba Mihai; pana atunci nu se atinge nimic in `rust/fereastra`.
Prototipul interactiv: `cercetare/redesign/Banda.html` (se deschide in browser,
fara server).

## Ce am presupus (sablonul cererii a venit necompletat)

- **Aplicatia:** GMotion. Compara obiectele programabile (proceduri, functii,
  view-uri, triggere) intre doua baze SQL Server si face deploy-ul diferentelor.
- **Stack:** Rust + Slint, randare software prin RDP fara GPU. Prototipul e
  HTML doar ca specificatie, ca machetele existente.
- **Utilizatori:** dezvoltatori si DBA care trec prin sute de obiecte diferite
  si trimit o parte dintre ele in productie.
- **Problema:** nu aspectul (designul aprobat e curat), ci fluxul. Lista,
  editorul si deploy-ul sunt trei ecrane separate. Planul se vede abia la final,
  iar decizia „trimit asta?" se ia fara sa vezi ce mai e deja in plan.

## Auditul designului aprobat

1. **Trei ecrane pentru un singur gest.** Utilizatorul face de sute de ori
   ciclul „citesc diferenta, decid, trec mai departe". Machetele il impart in
   lista → editor → deploy, cu planul vizibil doar in ultimul ecran.
2. **Bara laterala de categorii nu se poate citi inapoi.** Dupa trei clicuri,
   nimic de pe ecran nu iti spune exact ce filtru e activ si cum il refaci.
3. **Lista nu arata forma diferentei.** `+48 −31` nu-ti spune daca e un bloc
   sau o rescriere peste tot. Asta e prima intrebare la triere.
4. **Estetica e foarte aproape de varianta implicita a unui generator.** Fond
   crem `#f5f4f0`, linii subtiri si densitate de ziar sunt chiar tiparul pe care
   Mihai a vrut sa-l evite („sa nu para facut cu AI-ul").

## Ideea mare

**Un singur ecran, o banda: Coada → Obiectul → Planul.** Lumina inseamna ca
citesti. Cerneala inchisa inseamna ce urmeaza sa se execute. Planul e o placa
inchisa la culoare, mereu prezenta in dreapta, si e singurul element indraznet
din interfata. Restul ramane linistit.

### Trei mecanici noi

1. **„p" muta obiectul in plan, la locul lui.** Obiectul intra in placa direct
   in ordinea dependentelor. Este singura animatie din aplicatie (260 ms, doar
   ca raspuns la `p`, oprita la `prefers-reduced-motion`). In coada ramane o
   crestatura de cerneala pe marginea dreapta a randului: puntea dintre cele
   doua zone. Deploy-ul nu mai e un ecran separat la care ajungi, ci ceva ce
   construiesti sub ochi.
2. **Semnatura diferentei.** Fiecare rand din coada are o banda de 72 px, cat
   lungimea obiectului, cu blocurile diferite la pozitia lor reala: chihlimbar
   pentru modificat, verde pentru adaugat, rosu pentru sters. Un comentariu e
   desenat doar cu contur, iar o setare SET e un patrat gol. Aceeasi geometrie
   e harta din dreapta editorului, deci forma invatata in coada se regaseste in
   obiect. Trierea se face dupa forma, inainte de citire.
3. **Filtrul e text, iar gardurile sunt o secventa.** Categoriile din bara
   laterala devin o linie de filtrare: `/ stare:M difera:cod nerevizuit`.
   Fatetele cu contoare scriu in ea, iar ce scrii e exact ce vezi, deci se poate
   copia, relua sau pune in `cli`. In placa, verificarile chiar sunt o
   secventa, fiecare o deschide pe urmatoarea: baza n-a mai fost atinsa →
   versiunea actuala salvata → (doar PROD cu DROP) numele bazei tastat. Butonul
   de rulare se aprinde in rosu de productie abia cand toate trei sunt bifate.

## Sistemul vizual

| Jeton | Valoare | Rol |
|---|---|---|
| `hartie` | `#e6e9e5` | fundalul ferestrei: rece si mineral, nu crem |
| `suprafata` | `#f3f5f2` | coada, antete |
| `cod` | `#ffffff` | zona de cod |
| `linie` / `linie-slaba` | `#cbd1ca` / `#e0e4de` | separatoare |
| `text` / `-sec` / `-tert` | `#161b1f` / `#4f575c` / `#656d73` | tert ≥ 4,5:1 pe alb |
| `cerneala` / `-2` / `-linie` | `#1c2640` / `#26324f` / `#36436a` | placa planului, randul selectat |
| `pe-cerneala` / `-sec` | `#e4e8f1` / `#a9b3c7` | text pe placa |
| `ok` / `atentie` / `sters` pe cerneala | `#93cf93` / `#e9bb5f` / `#f29d8b` | CREATE, ALTER, DROP in plan |
| `prod` | `#b3261e` | banda de sus, marginea placii, butonul de rulare |
| starea: mod / add / del | aceleasi familii ca in designul aprobat, text putin mai inchis | |

**Ce ramane din designul aprobat:** IBM Plex Sans si Mono (deja incluse in
binar, cu licenta), literele de stare M / S / B, culoarea care nu e niciodata
singurul semnal, nicio comutare de zgomot, colturile de 3 px, fara umbre, fara
degradeuri, banda mediului si tastatura intai.

**Tipografie:** 13 px baza; 11,5–12,5 px pentru meta, cod si nume in Mono;
15 px pentru titlul placii; 19 px Mono pentru numele obiectului deschis, cel mai
mare text de pe ecran, fiindca e lucrul la care te uiti.

**Miscare:** una singura, cea de la `p`. Restul se schimba instant, ceea ce e si
cerinta RDP fara GPU din `design-interfata.md`.

## Asezarea

```
▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔ PROD 3px
gmotion  CharismaDev → CharismaProd PROD   [/ stare:M nerevizuit    Ctrl K] Recompara
┌ Coada 372 ───────────────┬ Obiectul ───────────────────────┐┃ Planul 352 ─────────┐
│ M modif 12  S 3  B 1      │ dbo.fn_PretCuTva     [r] [p]     │┃ Deploy pe CharismaProd│
│ M dbo.usp_Facturare  ▮ ▮ ▮│ Cota de TVA: 19 → @CotaTva       │┃ 1 ALTER fn_Zile…     │
│   procedura, +22 −19      │ ┌ acum ──────┬ dupa deploy ─────┐│┃ 2 ALTER fn_PretCuTva │
│ S dbo.vw_Stoc…  ████████ ▌│ │            │                  │▮┃ 3 DROP  fn_CursVechi │
│ ...                       │ └────────────┴──────────────────┘│┃ ✓ baza neatinsa      │
│                           │                                  │┃ ○ salveaza revenirea │
├───────────────────────────┴──────────────────────────────────┤┃ [Ruleaza 3 pasi]    │
│ j k obiect  n nerevizuit  F7  r  p  /        revizuite 6/16 ━││                      │
└──────────────────────────────────────────────────────────────┘┸──────────────────────┘
```

Totul e aliniat la stanga. Sub 1180 px placa devine sertar (tasta `d` sau
butonul „Planul (N)"). Sub 760 px coada si obiectul se aseaza una sub alta.

## Tastatura

`j`/`k` obiectul, `n` urmatorul nerevizuit, `F7`/`Shift+F7` diferenta, `r`
revizuit, `p` in plan / scoate din plan, `v` salveaza revenirea, `/` filtrul,
`Esc` goleste filtrul, `Ctrl K` paleta (comenzi si obiecte), `Ctrl ↵` ruleaza,
`d` sertarul planului.

## Ce ar insemna pentru proiect

- **Motorul nu se schimba.** Semnatura vine din blocurile pe care `core` le
  produce deja (`model_editor.rs` are pozitiile). Gramatica filtrului e logica
  pura, care ar sta in `diff-obiecte` si ar fi accesibila din `cli`
  (`gmotion diferente --filtru "stare:M difera:cod"`), conform CLAUDE.md §3.
- **In Slint:** `lista.slint` si `editor.slint` se unesc intr-un singur ecran,
  iar placa planului e ecranul de deploy al etapei 12. Asta inseamna ca etapa
  12 si-ar schimba doar criteriul de ecran: „taburi Ordinea / Atentionari /
  Script" devine „placa Ordinea / Script cu gardurile in secventa". Garda si
  scriptul de revenire raman exact cum sunt planificate.
- **Jetoanele noi** intra in `jetoane.rs` si trec prin testul de contrast
  existent. Primul pas e sa vad daca vreuna cade.

## Unde incap functiile lipsa fata de v3.4

Decizia din 2026-11-18 (`JURNAL.md`) e: intai paritatea, apoi aspectul, si nicio
functie din v3.4 nu se mai scoate tacit. Machetele aprobate au pierdut functii
tocmai pentru ca nu aveau loc pentru ele. Mai jos e locul fiecarei lipse din
`cercetare/lipsuri-portare.md` in Banda. **Prototipul nu le deseneaza inca**;
le adaug dupa ce Mihai alege directia.

| Lipsa (v3.4) | Locul in Banda |
|---|---|
| Comparare cu mai multe baze | o placa de plan pe baza tinta, cu taburi pe numele bazelor; in filtru, `baza:X` |
| Progres, Anuleaza | coada se umple pe masura ce se citesc obiectele; linia de 2 px din antet arata progresul, `Esc` anuleaza |
| Mod ghidat | `n` exista deja; `Enter` aplica blocul curent, `Esc` il sare, iar la final placa iti spune ce a ramas |
| Tinta editabila, Aplica bloc / toate, Reset, Ctrl+Z | jumatatea dreapta („dupa deploy") devine editabila; sageata de aplicare sta in jgheabul fiecarui bloc |
| Omite | tasta `o`; in coada, randul se estompeaza cu „omis"; in filtru, `omis` / `neomis` |
| Filtru pe stare (rezolvate / omise / deployate) | `revizuit`, `omis`, `deployat` in filtru, cu fatetele lor |
| Interval de similaritate | `scor:<90` in filtru; scorul apare in meta-ul randului |
| Sortare | `ordine:linii` / `ordine:modificat` in filtru, la fel de reluabil ca restul |
| Ultima modificare | in meta-ul randului si `modificat:>2026-11-01` |
| Export CSV | in paleta, „Exporta ce vezi"; foloseste exact textul filtrului, la fel ca `gmotion export --filtru` |
| Coloana Deploy (deployat / versiune noua) | crestatura de pe marginea randului isi schimba forma: plina = in plan, contur = deployat, contur rupt = versiune noua dupa deploy |
| Cautarea serverelor, testarea conexiunii | ecranul de conectare ramane separat; primeste doar jetoanele noi |
| Comparare offline (lipesti doua texte) | in paleta, „Compara doua texte"; deschide Obiectul fara Coada |
| Jurnal live | in sertarul de jos, `Ctrl J`, ascuns implicit |
