// Capturile ecranelor v3.4, ca referinta vizuala pentru interfata nativa.
// Date: `date.py` (Api-ul Python vechi, neatins). Aici `window.pywebview.api`
// e simulat cu raspunsurile lui, iar pagina se fotografiaza in Chromium.
//
//   node referinta/captura_v34/captura.mjs <dir cu pagina.html si date.json> [obiect]
//
// Scrie: 1-conectare.png, 2-lista.png, 3-editor.png in acelasi director.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { chromium } = require('playwright');

const dir = process.argv[2];
const obiect = process.argv[3] || 'dbo.TESTSIMILITUDINE_CaseB_Imbricat';
const date = JSON.parse(readFileSync(join(dir, 'date.json'), 'utf-8'));

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 560, height: 800 } });
await page.addInitScript((date) => {
  const ok = (v) => Promise.resolve(v);
  window.pywebview = {
    api: {
      cauta_servere: () => ok([]),
      test_conexiune: () => ok({ ok: true, baze: date.baze, server: date.server }),
      lista_sesiuni: () => ok([]),
      incarca_sesiune: () => ok(null),
      salveaza_sesiune: () => ok({ ok: true }),
      sterge_sesiune: () => ok({ ok: true }),
      ruleaza: () => ok({ ok: true }),
      progres: () => ok({ mesaj: 'Gata', procent: 100, gata: true, eroare: null, log: [] }),
      log_lines: () => ok({ linii: [], total: 0 }),
      anuleaza: () => ok(null),
      rezultate: () => ok(date.rezultate),
      elemente: (id) => ok(date.elemente[String(id)] || []),
      exporta_csv: () => ok({ ok: true }),
      genereaza_deployer: () => ok({ ok: true }),
      win_resize: () => ok(null),
    },
  };
  window.addEventListener('load', () => window.dispatchEvent(new Event('pywebviewready')));
}, date);
const erori = [];
page.on('pageerror', (e) => erori.push(String(e)));
await page.goto('file://' + join(dir, 'pagina.html'));
await page.waitForTimeout(800);
await page.screenshot({ path: join(dir, '1-conectare.png') });

await page.setViewportSize({ width: 1400, height: 860 });
await page.evaluate(() => incarcaRez());
await page.waitForTimeout(800);
await page.screenshot({ path: join(dir, '2-lista.png') });

const id = date.rezultate.lista.find((r) => r.obiect === obiect)?.id ?? date.rezultate.lista[0].id;
await page.evaluate((id) => deschide(id), id);
await page.waitForTimeout(1200);
await page.screenshot({ path: join(dir, '3-editor.png') });

console.log(erori.length ? 'erori in pagina:\n' + erori.join('\n') : 'fara erori in pagina');
await browser.close();
