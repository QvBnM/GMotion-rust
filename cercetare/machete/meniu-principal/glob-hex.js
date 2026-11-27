/* Formele de glob pentru panoul marcii: un diamant hexagonal in loc de nucleul
   din prisme imprastiate. Un singur motor pentru toate variantele: fiecare
   varianta da o lista de fete in spatiu model, iar desenarea (rotatie,
   proiectie, lumina, sortare pe adancime) e comuna.

   Se strica astfel: o fata cu punctele in ordine inversa primeste normala pe
   dos si se stinge, sau sortarea pe adancime pune pavilionul peste coroana. */
(function (global) {
  'use strict';

  const norm = (p) => { const l = Math.hypot(p[0], p[1], p[2]) || 1; return [p[0] / l, p[1] / l, p[2] / l]; };

  function rotate3D(p, rotY, rotX) {
    const cy = Math.cos(rotY), sy = Math.sin(rotY);
    const x1 = p.x * cy - p.z * sy, z1 = p.x * sy + p.z * cy;
    const cx = Math.cos(rotX), sx = Math.sin(rotX);
    return { x: x1, y: p.y * cx - z1 * sx, z: p.y * sx + z1 * cx };
  }

  // ---- forme ------------------------------------------------------------

  /** Taietura de diamant cu sectiune hexagonala: masa, coroana, pavilion. */
  function diamantHexagonal(laturi) {
    const n = laturi || 6;
    const inel = (raza, y, rasucire) => {
      const p = [];
      for (let i = 0; i < n; i++) {
        const a = (i / n) * Math.PI * 2 + (rasucire || 0);
        p.push({ x: Math.cos(a) * raza, y, z: Math.sin(a) * raza });
      }
      return p;
    };
    const masa = inel(0.46, 0.34);
    const branta = inel(0.92, 0.06);
    const brantaJos = inel(0.92, -0.02);
    const varf = { x: 0, y: -1.02, z: 0 };
    const fete = [];
    fete.push({ pts: masa.slice(), tip: 'masa' });
    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      // coroana: trapez intre masa si branta
      fete.push({ pts: [masa[i], masa[j], branta[j], branta[i]], tip: 'coroana' });
      // branta: banda subtire pe ecuator
      fete.push({ pts: [branta[i], branta[j], brantaJos[j], brantaJos[i]], tip: 'branta' });
      // pavilion: triunghi spre varf
      fete.push({ pts: [brantaJos[i], brantaJos[j], varf], tip: 'pavilion' });
    }
    return fete;
  }

  /** Sfera acoperita cu hexagoane (dualul unui icosaedru subdivizat). */
  function sferaHexagonala(subdiviziuni) {
    const t = (1 + Math.sqrt(5)) / 2;
    let v = [[-1, t, 0], [1, t, 0], [-1, -t, 0], [1, -t, 0], [0, -1, t], [0, 1, t],
    [0, -1, -t], [0, 1, -t], [t, 0, -1], [t, 0, 1], [-t, 0, -1], [-t, 0, 1]].map(norm);
    let f = [[0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11], [1, 5, 9], [5, 11, 4], [11, 10, 2],
    [10, 7, 6], [7, 1, 8], [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9], [4, 9, 5],
    [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1]];

    const cheie = (p) => p.map(c => c.toFixed(5)).join(',');
    for (let s = 0; s < (subdiviziuni || 2); s++) {
      const index = new Map();
      v.forEach((p, i) => index.set(cheie(p), i));
      const adauga = (p) => {
        const k = cheie(p);
        if (!index.has(k)) { index.set(k, v.length); v.push(p); }
        return index.get(k);
      };
      const nou = [];
      for (const [a, b, c] of f) {
        const ab = adauga(norm([(v[a][0] + v[b][0]) / 2, (v[a][1] + v[b][1]) / 2, (v[a][2] + v[b][2]) / 2]));
        const bc = adauga(norm([(v[b][0] + v[c][0]) / 2, (v[b][1] + v[c][1]) / 2, (v[b][2] + v[c][2]) / 2]));
        const ca = adauga(norm([(v[c][0] + v[a][0]) / 2, (v[c][1] + v[a][1]) / 2, (v[c][2] + v[a][2]) / 2]));
        nou.push([a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]);
      }
      f = nou;
    }

    // dualul: fiecare varf devine un hexagon (pentagon in cele 12 colturi)
    const laVarf = new Map();
    f.forEach((tri, i) => tri.forEach(iv => {
      if (!laVarf.has(iv)) laVarf.set(iv, []);
      laVarf.get(iv).push(i);
    }));
    const centre = f.map(([a, b, c]) => norm([
      (v[a][0] + v[b][0] + v[c][0]) / 3, (v[a][1] + v[b][1] + v[c][1]) / 3, (v[a][2] + v[b][2] + v[c][2]) / 3]));

    const fete = [];
    for (const [iv, triunghiuri] of laVarf) {
      const n = v[iv];
      // baza tangenta, ca sa putem ordona centrele in jurul varfului
      const ref = Math.abs(n[1]) < 0.85 ? [0, 1, 0] : [1, 0, 0];
      let rx = ref[1] * n[2] - ref[2] * n[1], ry = ref[2] * n[0] - ref[0] * n[2], rz = ref[0] * n[1] - ref[1] * n[0];
      const rl = Math.hypot(rx, ry, rz) || 1; rx /= rl; ry /= rl; rz /= rl;
      const ux = n[1] * rz - n[2] * ry, uy = n[2] * rx - n[0] * rz, uz = n[0] * ry - n[1] * rx;
      const pts = triunghiuri.map(i => centre[i]).map(c => ({
        c, a: Math.atan2(c[0] * ux + c[1] * uy + c[2] * uz, c[0] * rx + c[1] * ry + c[2] * rz)
      })).sort((p, q) => p.a - q.a).map(p => p.c);
      // usor retrase spre centru, ca sa ramana o linie intre placi
      const k = 0.93;
      const cx = pts.reduce((s, p) => s + p[0], 0) / pts.length;
      const cy = pts.reduce((s, p) => s + p[1], 0) / pts.length;
      const cz = pts.reduce((s, p) => s + p[2], 0) / pts.length;
      fete.push({
        pts: pts.map(p => ({ x: cx + (p[0] - cx) * k, y: cy + (p[1] - cy) * k, z: cz + (p[2] - cz) * k })),
        tip: pts.length === 5 ? 'colt' : 'placa'
      });
    }
    return fete;
  }

  /** Cerc de placi hexagonale, usor bombat, cu un diamant in mijloc. */
  function cercDeHexagoane() {
    const fete = [];
    const R = 1.0, pas = 0.235, raza = pas * 0.80;
    const bomba = (d) => 0.14 * Math.cos(Math.min(1, d / R) * Math.PI * 0.5);
    for (let q = -6; q <= 6; q++) {
      for (let r = -6; r <= 6; r++) {
        const x = pas * 1.5 * q;
        const z = pas * Math.sqrt(3) * (r + q / 2);
        const d = Math.hypot(x, z);
        if (d > R || d < 0.44) continue;
        const y = bomba(d);
        const pts = [];
        for (let i = 0; i < 6; i++) {
          const a = (i / 6) * Math.PI * 2;
          const px = x + Math.cos(a) * raza, pz = z + Math.sin(a) * raza;
          pts.push({ x: px, y: bomba(Math.hypot(px, pz)), z: pz });
        }
        fete.push({ pts, tip: 'placa', inaltime: y });
      }
    }
    // diamantul din centru, mai mic, deasupra placilor
    for (const f of diamantHexagonal(6)) {
      fete.push({ pts: f.pts.map(p => ({ x: p.x * 0.46, y: p.y * 0.46 + 0.30, z: p.z * 0.46 })), tip: f.tip });
    }
    return fete;
  }


  /** Sfera hexagonala taiata in diagonala: jumatatea Before si jumatatea
      After, departate de-a lungul normalei planului de taiere. Fiecare fata
      stie din ce parte e; departarea si lumina se pun la desenare. */
  function douaEmisfere(subdiviziuni) {
    const n = [0.74, 0.67, 0.05];
    const l = Math.hypot(n[0], n[1], n[2]);
    const taiere = { x: n[0] / l, y: n[1] / l, z: n[2] / l };
    const fete = sferaHexagonala(subdiviziuni || 2).map(f => {
      const c = f.pts.reduce((s, p) => ({ x: s.x + p.x, y: s.y + p.y, z: s.z + p.z }), { x: 0, y: 0, z: 0 });
      const k = 1 / f.pts.length;
      const d = (c.x * k) * taiere.x + (c.y * k) * taiere.y + (c.z * k) * taiere.z;
      return { pts: f.pts, tip: f.tip, parte: d >= 0 ? 1 : -1, taiat: Math.abs(d) < 0.17 };
    });
    // cercul de taiere al fiecarei jumatati: marginea aprinsa a cupolei
    const ref = Math.abs(taiere.y) < 0.85 ? { x: 0, y: 1, z: 0 } : { x: 1, y: 0, z: 0 };
    let rx = ref.y * taiere.z - ref.z * taiere.y, ry = ref.z * taiere.x - ref.x * taiere.z, rz = ref.x * taiere.y - ref.y * taiere.x;
    const rl = Math.hypot(rx, ry, rz) || 1; rx /= rl; ry /= rl; rz /= rl;
    const ux = taiere.y * rz - taiere.z * ry, uy = taiere.z * rx - taiere.x * rz, uz = taiere.x * ry - taiere.y * rx;
    const inel = [];
    for (let i = 0; i < 48; i++) {
      const a = (i / 48) * Math.PI * 2;
      inel.push({ x: rx * Math.cos(a) + ux * Math.sin(a), y: ry * Math.cos(a) + uy * Math.sin(a), z: rz * Math.cos(a) + uz * Math.sin(a) });
    }
    return { fete, taiere, inel };
  }

  // ---- desenare ---------------------------------------------------------

  function culoare(fn, worldX, alfa, tip, luminos) {
    const lx = -0.46, ly = -0.72, lz = 0.52, ll = Math.hypot(lx, ly, lz);
    const ndotl = Math.max(0, fn.x * (lx / ll) + fn.y * (ly / ll) + fn.z * (lz / ll));
    const amestec = (Math.max(-1, Math.min(1, worldX * 1.15)) + 1) * 0.5;
    const sus = tip === 'masa' || tip === 'placa';
    let baza = luminos
      ? (sus ? 198 + ndotl * 54 : 126 + ndotl * 96)
      : (sus ? 162 + ndotl * 86 : 74 + ndotl * 124);
    if (tip === 'branta') baza *= 0.92;
    const r = Math.min(255, Math.round(baza + (1 - amestec) * 20 - amestec * 12));
    const g = Math.min(255, Math.round(baza + (1 - amestec) * 9 + amestec * 14));
    const b = Math.min(255, Math.round(baza + amestec * 30 - (1 - amestec) * 12));
    return { r, g, b, a: alfa, ndotl };
  }

  function creeaza(canvas, optiuni) {
    const o = optiuni || {};
    const ctx = canvas.getContext('2d');
    const emisfere = o.varianta === 'emisfere' ? douaEmisfere(o.subdiviziuni || 2) : null;
    const fete = emisfere ? emisfere.fete
      : o.varianta === 'sfera' ? sferaHexagonala(o.subdiviziuni || 2)
      : o.varianta === 'sticla' ? sferaHexagonala(o.subdiviziuni || 2)
        : o.varianta === 'cerc' ? cercDeHexagoane()
          : diamantHexagonal(6);
    const sticla = o.varianta === 'sticla';
    const stare = {
      fete, sticla, emisfere, luminos: !!o.luminos,
      rotY: 0, rotX: o.varianta === 'cerc' ? 0.52 : o.varianta === 'emisfere' ? 0.12 : 0.22,
      mouseX: 0, mouseY: 0, tintaX: 0, tintaY: 0, impuls: 0, unde: [],
      viteza: o.viteza === undefined ? 0.26 : o.viteza
    };

    function dimensiune() {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      const r = canvas.getBoundingClientRect();
      canvas.width = Math.floor(r.width * dpr);
      canvas.height = Math.floor(r.height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      stare.lat = r.width; stare.inalt = r.height;
    }

    /** Cele doua emisfere: desenare proprie (sticla aprinsa, muchii, puncte,
        inelul de taiere si unda care trece dinspre Before spre After). */
    function deseneazaEmisfere(t, cx, cy, R, rotY, rotX, puls) {
      const E = stare.emisfere, lum = stare.luminos;
      const departare = 0.26 + 0.10 * (0.5 + 0.5 * Math.sin(t * 0.5)) + puls * 5;
      const unda = -1.25 + 2.5 * ((t * 0.19) % 1);
      const CULORI = {
        '-1': { r: 255, g: 168, b: 56, h: [255, 228, 160] },   // Before, chihlimbar
        '1': { r: 46, g: 206, b: 232, h: [176, 248, 255] }     // After, turcoaz
      };
      ctx.save();
      ctx.globalCompositeOperation = lum ? 'source-over' : 'lighter';
      ctx.lineJoin = 'round';

      const muta = (p, parte) => ({
        x: p.x + E.taiere.x * departare * parte,
        y: p.y + E.taiere.y * departare * parte,
        z: p.z + E.taiere.z * departare * parte
      });
      const proiect = (p) => {
        const persp = 1 / (1 - p.z * 0.22);
        return { x: cx + p.x * R * persp, y: cy - p.y * R * persp };
      };

      // haloul fiecarei jumatati, inaintea fetelor
      for (const parte of [-1, 1]) {
        const c = proiect(rotate3D(muta({ x: E.taiere.x * 0.35 * parte, y: E.taiere.y * 0.35 * parte, z: E.taiere.z * 0.35 * parte }, parte), rotY, rotX));
        const C = CULORI[parte];
        const g = ctx.createRadialGradient(c.x, c.y, R * 0.05, c.x, c.y, R * 1.25);
        g.addColorStop(0, `rgba(${C.r},${C.g},${C.b},${lum ? 0.13 : 0.22})`);
        g.addColorStop(0.55, `rgba(${C.r},${C.g},${C.b},${lum ? 0.05 : 0.08})`);
        g.addColorStop(1, 'rgba(0,0,0,0)');
        ctx.fillStyle = g;
        ctx.beginPath(); ctx.arc(c.x, c.y, R * 1.25, 0, Math.PI * 2); ctx.fill();
      }

      const poligoane = [];
      for (const f of stare.fete) {
        const p3 = f.pts.map(p => rotate3D(muta(p, f.parte), rotY, rotX));
        const c = p3.reduce((a, p) => ({ x: a.x + p.x, y: a.y + p.y, z: a.z + p.z }), { x: 0, y: 0, z: 0 });
        c.x /= p3.length; c.y /= p3.length; c.z /= p3.length;
        const a = p3[0], b = p3[1], d = p3[2];
        let nx = (b.y - a.y) * (d.z - a.z) - (b.z - a.z) * (d.y - a.y);
        let ny = (b.z - a.z) * (d.x - a.x) - (b.x - a.x) * (d.z - a.z);
        let nz = (b.x - a.x) * (d.y - a.y) - (b.y - a.y) * (d.x - a.x);
        const nl = Math.hypot(nx, ny, nz) || 1; nx /= nl; ny /= nl; nz /= nl;
        if (nx * c.x + ny * c.y + nz * c.z < 0) { nx = -nx; ny = -ny; nz = -nz; }
        // pozitia fetei pe axa de taiere, pentru unda care trece prin sfera
        const pe = f.pts[0].x * E.taiere.x + f.pts[0].y * E.taiere.y + f.pts[0].z * E.taiere.z;
        const aprins = Math.exp(-Math.pow((pe - unda) / 0.2, 2));
        poligoane.push({ pr: p3.map(proiect), z: c.z, fata: nz > 0, parte: f.parte, taiat: f.taiat, aprins, lumina: Math.max(0, nz) });
      }
      poligoane.sort((p, q) => p.z - q.z);

      for (const p of poligoane) {
        const C = CULORI[p.parte];
        const fata = p.fata;
        const alfa = (fata ? 0.17 + p.lumina * 0.2 : 0.07) * (lum ? 1.5 : 1) + p.aprins * 0.3;
        ctx.beginPath();
        p.pr.forEach((pt, i) => i ? ctx.lineTo(pt.x, pt.y) : ctx.moveTo(pt.x, pt.y));
        ctx.closePath();
        ctx.fillStyle = `rgba(${C.r},${C.g},${C.b},${alfa.toFixed(3)})`;
        ctx.fill();
        const h = C.h;
        const tarie = (fata ? 0.34 + p.lumina * 0.42 : 0.12) + p.aprins * 0.55 + (p.taiat && fata ? 0.3 : 0);
        ctx.lineWidth = p.taiat && fata ? 1.25 : 0.85;
        ctx.strokeStyle = `rgba(${h[0]},${h[1]},${h[2]},${Math.min(1, tarie).toFixed(3)})`;
        ctx.stroke();
        // puncte in colturile placilor din fata, ca in imaginea de referinta
        if (fata && (p.lumina > 0.45 || p.aprins > 0.25)) {
          ctx.fillStyle = `rgba(${h[0]},${h[1]},${h[2]},${Math.min(1, 0.3 + p.aprins).toFixed(2)})`;
          for (const pt of p.pr) { ctx.beginPath(); ctx.arc(pt.x, pt.y, 1.35, 0, Math.PI * 2); ctx.fill(); }
        }
      }

      // bobite care trec prin taietura, dinspre Before spre After: diferenta
      // care pleaca dintr-o parte si ajunge in cealalta
      for (let k = 0; k < 14; k++) {
        const u = ((t * 0.26 + k / 14) % 1);
        const a = (k / 14) * Math.PI * 2 + t * 0.12;
        const raza = 0.30 + 0.46 * Math.sin(Math.PI * u);
        const baza = E.inel[Math.floor(((a / (Math.PI * 2)) % 1) * E.inel.length)];
        const parte = -1 + 2 * u;
        const p = rotate3D({
          x: baza.x * raza + E.taiere.x * departare * parte,
          y: baza.y * raza + E.taiere.y * departare * parte,
          z: baza.z * raza + E.taiere.z * departare * parte
        }, rotY, rotX);
        const pr = proiect(p);
        const m = u;
        const r = Math.round(255 + (46 - 255) * m), g2 = Math.round(168 + (206 - 168) * m), b = Math.round(56 + (232 - 56) * m);
        const al = Math.sin(Math.PI * u) * (lum ? 0.55 : 0.9);
        const gr = ctx.createRadialGradient(pr.x, pr.y, 0, pr.x, pr.y, 7);
        gr.addColorStop(0, `rgba(${r},${g2},${b},${al.toFixed(2)})`);
        gr.addColorStop(1, `rgba(${r},${g2},${b},0)`);
        ctx.fillStyle = gr;
        ctx.beginPath(); ctx.arc(pr.x, pr.y, 7, 0, Math.PI * 2); ctx.fill();
        ctx.fillStyle = `rgba(255,255,255,${(al * 0.7).toFixed(2)})`;
        ctx.beginPath(); ctx.arc(pr.x, pr.y, 1.5, 0, Math.PI * 2); ctx.fill();
      }

      // inelul de taiere: marginea aprinsa a fiecarei cupole
      for (const parte of [-1, 1]) {
        const C = CULORI[parte];
        const pr = E.inel.map(p => proiect(rotate3D(muta(p, parte), rotY, rotX)));
        ctx.beginPath();
        pr.forEach((pt, i) => i ? ctx.lineTo(pt.x, pt.y) : ctx.moveTo(pt.x, pt.y));
        ctx.closePath();
        const mij = pr.reduce((a, p) => ({ x: a.x + p.x / pr.length, y: a.y + p.y / pr.length }), { x: 0, y: 0 });
        const g = ctx.createRadialGradient(mij.x, mij.y, 1, mij.x, mij.y, R * 0.95);
        g.addColorStop(0, `rgba(${C.h[0]},${C.h[1]},${C.h[2]},${lum ? 0.10 : 0.16})`);
        g.addColorStop(1, `rgba(${C.r},${C.g},${C.b},0)`);
        ctx.fillStyle = g; ctx.fill();
        ctx.lineWidth = 2.1;
        ctx.strokeStyle = `rgba(${C.h[0]},${C.h[1]},${C.h[2]},${lum ? 0.5 : 0.78})`;
        ctx.stroke();
        ctx.lineWidth = 5.5;
        ctx.strokeStyle = `rgba(${C.r},${C.g},${C.b},${lum ? 0.12 : 0.2})`;
        ctx.stroke();
      }
      ctx.restore();
    }

    function deseneaza(t, dt) {
      const L = stare.lat, H = stare.inalt;
      ctx.clearRect(0, 0, L, H);
      stare.mouseX += (stare.tintaX - stare.mouseX) * 0.08;
      stare.mouseY += (stare.tintaY - stare.mouseY) * 0.08;
      stare.impuls *= 0.94;
      if (stare.emisfere) stare.rotY = Math.sin(t * 0.21) * 0.5 + stare.impuls * 0.2;
      else stare.rotY += dt * (stare.viteza + stare.impuls);
      const rotY = stare.rotY + stare.mouseX * 0.5;
      const rotX = (o.varianta === 'cerc' ? 0.52 : o.varianta === 'emisfere' ? 0.12 : 0.22) + stare.mouseY * 0.38 + Math.sin(t * 0.5) * 0.035;

      const cx = L * 0.5, cy = H * 0.5, R = Math.min(L, H) * 0.34;

      if (stare.emisfere) {
        let p = 0;
        for (let i = stare.unde.length - 1; i >= 0; i--) {
          const v = t - stare.unde[i];
          if (v > 1) { stare.unde.splice(i, 1); continue; }
          p += Math.sin(v * Math.PI) * (1 - v) * 0.1;
        }
        deseneazaEmisfere(t, cx, cy, R, rotY, rotX, p);
        return;
      }

      // aura
      const aura = ctx.createRadialGradient(cx, cy, R * 0.1, cx, cy, R * 1.75);
      aura.addColorStop(0.0, stare.luminos ? 'rgba(90,120,160,.13)' : 'rgba(148,185,224,.20)');
      aura.addColorStop(0.45, 'rgba(34,211,238,.09)');
      aura.addColorStop(0.75, 'rgba(251,191,36,.06)');
      aura.addColorStop(1.0, 'rgba(5,8,17,0)');
      ctx.fillStyle = aura;
      ctx.beginPath(); ctx.arc(cx, cy, R * 1.75, 0, Math.PI * 2); ctx.fill();

      let puls = 0;
      for (let i = stare.unde.length - 1; i >= 0; i--) {
        const v = t - stare.unde[i];
        if (v > 1) { stare.unde.splice(i, 1); continue; }
        puls += Math.sin(v * Math.PI) * (1 - v) * 0.1;
      }
      const scara = 1 + puls;

      const poligoane = [];
      for (const f of stare.fete) {
        const p3 = f.pts.map(p => rotate3D({ x: p.x * scara, y: p.y * scara, z: p.z * scara }, rotY, rotX));
        const c = p3.reduce((s, p) => ({ x: s.x + p.x, y: s.y + p.y, z: s.z + p.z }), { x: 0, y: 0, z: 0 });
        c.x /= p3.length; c.y /= p3.length; c.z /= p3.length;
        // normala din primele trei puncte
        const a = p3[0], b = p3[1], d = p3[2];
        let nx = (b.y - a.y) * (d.z - a.z) - (b.z - a.z) * (d.y - a.y);
        let ny = (b.z - a.z) * (d.x - a.x) - (b.x - a.x) * (d.z - a.z);
        let nz = (b.x - a.x) * (d.y - a.y) - (b.y - a.y) * (d.x - a.x);
        const nl = Math.hypot(nx, ny, nz) || 1; nx /= nl; ny /= nl; nz /= nl;
        if (nx * c.x + ny * c.y + nz * c.z < 0) { nx = -nx; ny = -ny; nz = -nz; }
        if (!stare.sticla && nz <= 0.02) continue;
        const pr = p3.map(p => {
          const persp = 1 / (1 - p.z * 0.22);
          return { x: cx + p.x * R * persp, y: cy - p.y * R * persp };
        });
        const alfa = stare.sticla
          ? (nz > 0 ? 0.30 : 0.13)
          : Math.max(0.30, Math.min(1, 0.62 + c.z * 0.4));
        poligoane.push({ pr, z: c.z, col: culoare({ x: nx, y: ny, z: nz }, c.x, alfa, f.tip, stare.luminos) });
      }
      poligoane.sort((p, q) => p.z - q.z);

      ctx.save();
      ctx.lineJoin = 'round';
      if (stare.sticla) ctx.globalCompositeOperation = stare.luminos ? 'source-over' : 'lighter';
      for (const p of poligoane) {
        ctx.beginPath();
        p.pr.forEach((pt, i) => i ? ctx.lineTo(pt.x, pt.y) : ctx.moveTo(pt.x, pt.y));
        ctx.closePath();
        const c = p.col;
        ctx.fillStyle = `rgba(${c.r},${c.g},${c.b},${c.a.toFixed(3)})`;
        ctx.fill();
        ctx.lineWidth = stare.sticla ? 0.8 : 0.65;
        ctx.strokeStyle = stare.sticla
          ? `rgba(190,235,255,${(0.16 + Math.max(0, p.z) * 0.4).toFixed(2)})`
          : (c.ndotl > 0.72
            ? `rgba(255,255,255,${(0.2 + c.ndotl * 0.4).toFixed(2)})`
            : `rgba(15,23,42,${(0.2 + Math.max(0, p.z) * 0.25).toFixed(2)})`);
        ctx.stroke();
      }
      ctx.restore();
    }

    // mouse: rotatie dupa pozitie, clic = puls
    canvas.addEventListener('click', () => { stare.unde.push(performance.now() * 0.001); stare.impuls = 1.3; });
    window.addEventListener('mousemove', (e) => {
      const r = canvas.getBoundingClientRect();
      stare.tintaX = Math.max(-1, Math.min(1, (e.clientX - (r.left + r.width / 2)) / Math.max(r.width * 1.3, 180)));
      stare.tintaY = Math.max(-1, Math.min(1, (e.clientY - (r.top + r.height / 2)) / Math.max(r.height * 1.3, 180)));
    });

    dimensiune();
    window.addEventListener('resize', dimensiune);
    return {
      deseneaza, dimensiune, stare,
      puls: () => { stare.unde.push(performance.now() * 0.001); stare.impuls = 1.3; },
      numarFete: fete.length
    };
  }

  global.GlobHex = { creeaza, diamantHexagonal, sferaHexagonala, cercDeHexagoane };
})(window);
