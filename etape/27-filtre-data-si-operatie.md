# Etapa 27 — lista: filtre pe data crearii, data modificarii si operatie

**Stare:** INCHEIATA
**Branch:** claude/clever-cerf-hjk2qh
**Deschisa:** 2026-11-22 · **Incheiata:** 2026-11-22

## Scop

Mihai: filtre in lista dupa data crearii si a modificarii, si dupa CREATE /
ALTER / DROP. La final, lista se poate strange la obiectele create sau
modificate recent si la ce ar face deploy-ul cu ele.

Alegerile lui Mihai (2026-11-22): perioade gata facute (azi, 7 zile, 30 de
zile, oricand); DROP ramane doar filtru (deploy-ul tot nu sterge nimic,
decizia din 2026-11-19).

## Criterii de terminare

- [x] data crearii: citita din `sys.objects.create_date` (server) si din
      instantaneu (camp optional), purtata pana pe randul listei; in
      `--json` si in CSV
- [x] filtrul „Creat" si filtrul „Modificat": oricand / azi / ultimele 7
      zile / ultimele 30 de zile, socotite de la data de azi; obiectele fara
      data nu trec de un filtru activ; teste pe model, inclusiv trecerea
      peste luna si an
- [x] filtrul „Operatie": orice / CREATE (doar in Martor) / ALTER (modificat)
      / DROP (doar in Modificata); identicele si criptatele nu trec de un
      filtru activ; teste
- [x] filtrele se combina cu celelalte, raman la recomparare
      (`preia_vederea`) si se vad in bara de filtre, in ambele limbi
- [x] captura listei cu filtrele, la 1600x1000, ambele teme
- [x] poarta verde: `unelte/verifica.sh`
- [x] `STARE.md`, `JURNAL.md`

## Pasi

1. Data crearii (`db`, modul, lista).
2. Modelul filtrelor si testele.
3. Bara de filtre si legaturile.
4. Capturi, stare, jurnal.

## Jurnal de lucru

- **2026-11-22** Etapa deschisa, cu criteriile scrise inainte de cod.
- **2026-11-22** Facute. Data crearii pe acelasi drum ca „Ultima modificare"
  (server, instantaneu, rand, CSV). Datele se compara ca text `AAAA-LL-ZZ`;
  prima zi a perioadei se socoteste fara biblioteca de date. „Azi" e data
  calculatorului, iar datele vin de pe server: la miezul noptii pot diferi
  cu o zi.

## Ce a iesit altfel decat planul

- Filtrul CREATE / ALTER / DROP se suprapune cu categoriile din stanga sus
  (Doar in martor / Modificate / Extra); l-am facut tot, ca filtru separat,
  fiindca spune ce ar face deploy-ul si se combina cu ele.
- Bara de filtre a trecut pe doua randuri la 1600 px; aranjarea existenta
  (filtrele coboara cand nu incap) a mers fara schimbari.
