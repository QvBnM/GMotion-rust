# Macheta: tema spatiala (pornire + meniu)

Aprobata de Mihai pe 2026-11-28: „o sa merg pe tema asta spatiala". Decizia si
limitele ei sunt in `JURNAL.md`, 2026-11-28. Etapa care o duce in aplicatie:
`etape/34-tema-spatiala.md`.

| Fisier | Ce e |
|---|---|
| `pornire-si-meniu.html` | prototipul rulabil: intrarea, incarcarea, meniul side by side, ecranele dedicate |
| `cosmic-fata-de-sculptat.html` | aceleasi doua ecrane in ambele materiale, plus taietura propusa |
| `schite-sfera.html` | cele opt idei in jurul sferei, cu tabelul de corespondenta ochi -> date |
| `meniu-1440x900.png` | captura meniului din prototip |

Machetele ruleaza in browser pe `canvas`. **Aplicatia nu are canvas si
deseneaza software.** Ce se vede aici nu se traduce unu-la-unu in Slint; ce
se traduce si cum, e in etapa 34.

## Ce arata prototipul

Trei stari, un singur fundal:

1. **intrare** — marca in centru, bara de incarcare cu pasii adevarati, iar
   scannerul de pe sfera urca pe procentul citit, nu pe un cronometru. La
   100% nucleul se aprinde.
2. **meniu** — alegerile stanga, marca vie dreapta. Nicio data pe ecran.
3. **ecran dedicat** — categoria aleasa, singura pe scena. Lockup-ul zboara
   mic in antet; `Esc` sau „Meniu" intorc.

Lockup-ul e **acelasi obiect** in toate trei: se muta prin transform, nu se
redeseneaza. Steaua cazatoare cade la 6-13 s si loveste litera „o"; in
aplicatie ea nu cade pe cronometru, ci cand se termina o lucrare pe firul de
fundal (schita 8).

## Ce ramane de decis

- forma sferei in aplicatie: cadre coapte inainte (ieftin, fara mouse) sau
  rasterizare in Rust intr-un `SharedPixelBuffer` (scump, cu mouse)
- cat de repede se stinge cerul la intrarea in conectare
- fonturile: machetele folosesc Outfit + JetBrains Mono, binarul are azi
  doar Geist

## Ce nu intra in aplicatie

Butoanele „Reia intrarea" si „Sari la meniu" din dreapta jos sunt unelte ale
machetei. Textele scrise direct in HTML trec, in aplicatie, prin
`fereastra::texte`, in romana si engleza.
