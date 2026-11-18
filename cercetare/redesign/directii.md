# Trei directii noi (dupa respingerea „Banda")

**2026-11-18.** Mihai, despre „Banda": „pare vechi si anost, vreau cu totul
alta directie". Nu am mai ghicit o singura varianta. Am facut trei, foarte
diferite intre ele, pe acelasi ecran principal si cu aceleasi date:
`cercetare/redesign/Directii.html` (se comuta cu `1` / `2` / `3`).

| | 1. Harta | 2. Teancul | 3. Panza |
|---|---|---|---|
| Ideea | toata baza dintr-o privire: fiecare obiect e un patrat, grupat pe schema si tip | un obiect pe ecran, decizia dintr-o tasta (← omite, ↑ revizuit, → in plan) | codul ocupa tot ecranul; restul pluteste si apare cand il chemi (Ctrl K) |
| Ton | intunecat, viu, spatial | luminos, curajos, jucaus | alb, editorial, minimal |
| Culori | bleumarin `#0d1120`, chihlimbar / menta / corai | lavanda `#e9e4ff`, albastru electric `#3b2bff`, cerneala `#17123b` | alb, negru, accente saturate |
| Litere | Sora + Geist Mono | Bricolage Grotesque + DM Mono | Geist + Geist Mono |
| Cel mai bun la | sute de obiecte, vezi unde sunt problemele | triere rapida, un obiect dupa altul | citit atent un obiect |
| Riscul | harta trebuie desenata de mana in Slint (mii de patrate, virtualizat) | umbrele si animatiile cartii costa pe RDP fara GPU; trebuie simplificate | lista de obiecte ascunsa poate incetini comparatia intre obiecte |

Toate fonturile sunt OFL si se pot include in binar, ca IBM Plex acum.
Combinatiile sunt posibile: de exemplu Harta ca vedere de ansamblu, iar
Teancul ca mod ghidat (lipsa din v3.4).

Nimic din `rust/fereastra` nu se schimba pana nu alege Mihai.
`propunere.md` („Banda") ramane doar ca istoric: a fost respinsa.
