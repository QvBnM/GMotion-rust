// Fixture de paritate pentru stratul de plasare (`plasare.rs`).
//
// Plasarea nu se poate extrage mecanic din `ui.py` ca motorul de diff: tine de
// starea paginii (`DIF_EL`, textul din editor). Deci rulam pagina v3.4 adevarata
// intr-un browser fara cap, ii dam fiecare caz prin „comparare offline" (care e
// complet locala) si notam ce raspunde `_sugereaza` pentru fiecare rand.
//
// Se ruleaza prin `unelte/regenereaza-fixture-plasare.sh`, nu direct: scriptul
// genereaza intai pagina din `mostenire/ui.py`.
//
//     node referinta/genereaza_plasare.cjs <pagina.html> <cazuri.json> <iesire.json>

// CommonJS, nu ESM: `playwright` e instalat global in containerul de lucru si
// se rezolva prin `require`, nu prin `import`.
const { chromium } = require('playwright');
const { readFileSync, writeFileSync } = require('node:fs');
const { resolve } = require('node:path');

const [pagina, caleCazuri, iesire] = process.argv.slice(2);
if (!pagina || !caleCazuri || !iesire) {
  console.error('folosire: node genereaza_plasare.cjs <pagina.html> <cazuri.json> <iesire.json>');
  process.exit(2);
}

(async () => {
  const cazuri = JSON.parse(readFileSync(caleCazuri, 'utf8'));
  const browser = await chromium.launch();
  const page = await browser.newPage();
  const erori = [];
  page.on('pageerror', (e) => erori.push(String(e)));
  await page.goto('file://' + resolve(pagina));

  const rezultate = [];
  for (const caz of cazuri) {
    const sugestii = await page.evaluate(([ref, cur]) => {
      document.getElementById('offRef').value = ref;
      document.getElementById('offCur').value = cur;
      comparaOffline();
      const out = [];
      DIF_EL.forEach((el, pozitie) => {
        if (el.st == null) return;
        const s = _sugereaza(el.i);
        if (!s) return;
        out.push({
          pozitie,
          tip: s.pick.tip,
          idx: s.pick.idx,
          scor: s.pick.scor,
          randuri: s.srcIndices.slice(),
          text: s.srcText,
        });
      });
      return out;
    }, [caz.ref, caz.cur]);
    rezultate.push({ nume: caz.nume, sugestii });
  }

  await browser.close();
  if (erori.length) {
    console.error('erori in pagina:', erori.slice(0, 5).join(' | '));
    process.exit(1);
  }
  writeFileSync(iesire, JSON.stringify(rezultate, null, 1) + '\n');
  console.log(`scris ${iesire}: ${rezultate.length} cazuri, ` +
    `${rezultate.reduce((n, r) => n + r.sugestii.length, 0)} sugestii`);

})();
