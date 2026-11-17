// Rezumatul rapid dat de motorul JS din editor, pentru perechi (ref, cur).
// Intrare pe stdin: JSON [[ref, cur], ...] cu textele BRUTE din baza.
// Ieșire pe stdout: JSON [rezumat, ...], in aceeasi ordine.
//
// Il foloseste referinta/paritate.py ca arbitru: cand Rust si mostenire/app.py
// dau rezumate diferite, raspunsul JS spune care e corectura si care regresia.
// Pregatirea textului e aceeasi ca in genereaza.mjs.

import * as motor from './motor_js.mjs';

let intrare = '';
process.stdin.setEncoding('utf-8');
process.stdin.on('data', (b) => { intrare += b; });
process.stdin.on('end', () => {
  const perechi = JSON.parse(intrare);
  const out = perechi.map(([ref, cur]) =>
    motor.rezumat_rapid(motor.pregateste_afisare(ref), motor.pregateste_afisare(cur)));
  process.stdout.write(JSON.stringify(out));
});
