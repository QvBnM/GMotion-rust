"""Pagina si datele interfetei v3.4, pentru capturi de referinta.

Interfata noua (nativa) trebuie sa arate „oarecum la fel" ca v3.4 (Mihai,
2026-11-17). Ca sa existe ceva concret cu care sa se compare, aici se ruleaza
`Api` din `mostenire/app.py` NEATINS pe un instantaneu (ca `paritate.py`) si se
scriu: pagina HTML exacta (`ui.PAGINA` + bara de titlu din `app.py`) si
raspunsurile metodelor pe care pagina le cere (`rezultate`, `elemente(id)`).
`captura.mjs` le incarca in Chromium si fotografiaza ecranele.

  python3 -I referinta/captura_v34/date.py referinta/instantanee/testsimilitudine.json CharismaERP_GetsBet iesire/
"""
import json
import os
import sys

AICI = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(AICI))
import paritate  # noqa: E402  (pyodbc fals, importul aplicatiei vechi)


def main():
    cale, referinta, iesire = sys.argv[1:4]
    os.makedirs(iesire, exist_ok=True)
    inst = paritate.citeste_instantaneu(cale)
    meta, lista = paritate.lista_python(inst, referinta)
    import app  # deja incarcat de lista_python, cu pyodbc fals
    import ui

    api = app.Api()
    baze = [referinta] + sorted(b for b in inst["baze"] if b != referinta)
    api._comparatie(api._gen, inst.get("server", ""), "sql", "u", "p", baze, referinta)
    rezultate = api.rezultate()
    elemente = {str(x["id"]): api.elemente(x["id"]) for x in rezultate["lista"]}
    pagina = ui.PAGINA.replace("</body>", app._TITLEBAR + "</body>", 1)
    with open(os.path.join(iesire, "pagina.html"), "w", encoding="utf-8") as f:
        f.write(pagina)
    with open(os.path.join(iesire, "date.json"), "w", encoding="utf-8") as f:
        json.dump({"rezultate": rezultate, "elemente": elemente, "baze": list(inst["baze"]),
                   "server": inst.get("server", "")}, f, ensure_ascii=False)
    print(f"{len(rezultate['lista'])} randuri, pagina si date in {iesire}")


if __name__ == "__main__":
    main()
