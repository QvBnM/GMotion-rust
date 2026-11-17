// Ruleaza cazurile de test prin motorul JS extras din ui.py si scrie rezultatele.
// Se ruleaza din radacina repo-ului:
//     node referinta/genereaza.mjs

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import * as motor from './motor_js.mjs';

const AICI = dirname(fileURLToPath(import.meta.url));
const cazuri = JSON.parse(readFileSync(join(AICI, 'cazuri.json'), 'utf-8'));

const out = cazuri.map((c) => {
  const ref = motor.pregateste_afisare(c.ref);
  const cur = motor.pregateste_afisare(c.cur);
  const randuri = motor.diff_aliniat(ref, cur);
  const nr = (t) => randuri.filter((r) => r.tip === t).length;
  const p = [];
  if (nr('modificat')) p.push(`${nr('modificat')} modificate`);
  if (nr('adaugat')) p.push(`${nr('adaugat')} adaugate`);
  if (nr('sters')) p.push(`${nr('sters')} sterse`);
  return {
    nume: c.nume,
    norm_ref: motor.normalizeaza(c.ref),
    norm_cur: motor.normalizeaza(c.cur),
    scor: motor.scor_similaritate(ref, cur),
    blocuri_ref: motor._split_statements(ref),
    blocuri_cur: motor._split_statements(cur),
    rezumat_rapid: motor.rezumat_rapid(ref, cur),
    rezumat_diff: p.length ? p.join(', ') : 'diferente minore',
    tipuri: randuri.map((r) => r.tip),
  };
});

// aceeasi forma de serializare ca in genereaza.py, ca diff-ul sa fie comparabil
const cale = join(AICI, 'rezultate_js.json');
writeFileSync(cale, JSON.stringify(out, null, 1) + '\n', 'utf-8');
console.log(`scris: ${cale} (${out.length} cazuri)`);
