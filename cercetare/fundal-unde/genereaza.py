"""Fundalul cu unde si sigla gmotion: generatorul tuturor imaginilor.

Ruleaza din radacina depozitului:  python3 -I cercetare/fundal-unde/genereaza.py
Scrie:
  cercetare/fundal-unde/pornire-1366x768.png  cadrul plin (unde + sigla), pentru ecranul de pornire
  cercetare/fundal-unde/unde-1366x768.png     doar undele, intensitate plina
  cercetare/fundal-unde/pornire.gif           proba animatiei de pornire (jumatate de rezolutie)
  rust/fereastra/imagini/fundal-{intunecat,luminos}.png
                                              fundalul listei: stins, opac, amestecat deja cu
                                              culoarea de fundal a temei, marginile topite in ea
Totul e determinist (generator cu samanta fixa). Cere numpy si Pillow.
"""
import os
import numpy as np
from PIL import Image, ImageDraw, ImageFont

AICI = os.path.dirname(os.path.abspath(__file__))
RADACINA = os.path.dirname(os.path.dirname(AICI))
FONT = os.path.join(RADACINA, 'rust/fereastra/fonturi/Geist-600.ttf')
FUNDAL = {'intunecat': (0x0d, 0x11, 0x20), 'luminos': (0xf4, 0xf5, 0xf8)}
TEXT = {'intunecat': (238, 241, 255), 'luminos': (15, 17, 21)}
ACCENT = (255, 181, 71)
OPRIRI = [(0, (255, 120, 40)), (0.4, (255, 200, 60)), (0.7, (80, 230, 190)), (1, (40, 200, 230))]


def culori(t):
    """Culoarea undelor pe orizontala, 0..1 (portocaliu -> turcoaz)."""
    t = np.clip(t, 0, 1)
    out = np.zeros(t.shape + (3,), np.float32)
    for (a, ca), (b, cb) in zip(OPRIRI, OPRIRI[1:]):
        m = (t >= a) & (t <= b)
        u = ((t - a) / (b - a))[m][:, None]
        out[m] = np.array(ca) * (1 - u) + np.array(cb) * u
    return out / 255


def _cutie(a, r, ax):
    if r < 1:
        return a
    pad = [(0, 0)] * a.ndim
    pad[ax] = (r + 1, r)
    c = np.cumsum(np.pad(a, pad, mode='edge'), axis=ax, dtype=np.float64)
    n = a.shape[ax]
    return ((np.take(c, np.arange(2 * r + 1, 2 * r + 1 + n), axis=ax) - np.take(c, np.arange(n), axis=ax)) / (2 * r + 1)).astype(np.float32)


def estompa(a, sigma):
    """Aproape gaussiana: trei treceri de medie pe fereastra, pe x si pe y."""
    r = max(1, int(round(sigma * 0.93)))
    for _ in range(3):
        a = _cutie(_cutie(a, r, 0), r, 1)
    return a


def panglica(W, H, cy, amp, faza, n, samanta, ss=3):
    """Intensitatea unei panglici de `n` fire, desenate cu antialiere (puncte
    dese impartite biliniar pe o grila de ss x ss ori mai fina), fiecare fir
    cu luminozitatea lui, mai tare in miez si variind de-a lungul lui."""
    r = np.random.default_rng(samanta)
    k_h = H / 768
    acc = np.zeros((H * ss, W * ss), np.float32)
    xs = np.arange(-40 * ss, (W + 40) * ss, 0.7) / ss
    for k in range(n):
        u = k / (n - 1) - 0.5
        y = (cy + amp * np.sin(xs / (300 + u * 80) + faza + u * 1.4 + r.normal(0, 0.04)) * np.cos(xs / 610 + faza * 0.5)
             + u * 170 * k_h * np.sin(xs / 430 + faza + 1.3) * np.sin(xs / 1000 + faza)
             + 3 * np.sin(xs / 57 + k) * u)
        b = max(1 - abs(u) * 1.7, 0.08) ** 1.5 * r.uniform(0.5, 1.0)
        b = b * (0.55 + 0.45 * np.sin(xs / r.uniform(150, 400) + r.uniform(0, 6)) ** 2)
        X, Y = xs * ss, y * ss
        x0, y0 = np.floor(X).astype(int), np.floor(Y).astype(int)
        fx, fy = X - x0, Y - y0
        for dx, dy, w in ((0, 0, (1 - fx) * (1 - fy)), (1, 0, fx * (1 - fy)), (0, 1, (1 - fx) * fy), (1, 1, fx * fy)):
            xi, yi = x0 + dx, y0 + dy
            ok = (xi >= 0) & (xi < W * ss) & (yi >= 0) & (yi < H * ss)
            np.add.at(acc, (yi[ok], xi[ok]), (w * b)[ok] * 0.7)
    return acc.reshape(H, ss, W, ss).mean(axis=(1, 3))


def strat_unde(W, H, sus, jos, samanta=5):
    """Lumina undelor, particulelor si petelor (0..1, dupa o curba moale care
    nu arde albul), de adunat peste fundal."""
    rng = np.random.default_rng(samanta)
    k_h = H / 768
    c = culori(np.arange(W) / W)[None]
    lum = (panglica(W, H, jos, 85 * k_h, 0.3, 44, 1)[..., None] * 2.3
           + panglica(W, H, sus, 65 * k_h, 2.4, 36, 2)[..., None] * 1.8) * c
    # particule: cele mai multe pe langa unde, marimi variate, o parte defocalizate
    P = np.zeros((H, W, 3), np.float32)
    Pb = np.zeros((H, W, 3), np.float32)
    for _ in range(int(420 * W * H / (1366 * 768))):
        x = rng.uniform(0, W)
        if rng.random() < .6:
            y = rng.normal(jos, 70 * k_h)
        elif rng.random() < .6:
            y = rng.normal(sus, 60 * k_h)
        else:
            y = rng.uniform(0, H)
        xi, yi = int(x), int(y)
        if not (0 <= xi < W and 0 <= yi < H):
            continue
        cul = culori(np.array([x / W]))[0] * rng.uniform(0.3, 1.0) ** 2
        (Pb if rng.random() < 0.25 else P)[yi, xi] += cul * rng.uniform(0.6, 2.5)
    lum += estompa(P, 0.7) * 4 + estompa(Pb, 3) * 40
    # pete de lumina: discuri moi, cu marginea putin mai luminoasa
    yy, xx = np.mgrid[0:H, 0:W]
    for _ in range(11):
        x, y, r = rng.uniform(0, W), rng.uniform(0, H), rng.uniform(18, 55) * k_h
        d = np.hypot(xx - x, yy - y) / r
        disc = (np.clip(1 - d, 0, 1) ** 0.3 * (1 + 0.25 * np.exp(-((d - 0.9) / 0.08) ** 2)) * (d < 1)).astype(np.float32)
        lum += estompa(disc, 4)[..., None] * culori(np.array([x / W]))[0] * rng.uniform(0.06, 0.14)
    lum = lum + sum(w * estompa(lum, s) for s, w in ((2, 0.6), (8, 0.5), (30, 0.35)))
    return 1 - np.exp(-lum * 1.1)


def masti_sigla(W, H, cx, cy, marime):
    """Mastile (0..1) pentru „g", punct si „motion", ca in `Semn` din comune.slint."""
    F = ImageFont.truetype(FONT, marime)
    m = {k: Image.new('L', (W, H)) for k in ('g', 'punct', 'motion')}
    wg = ImageDraw.Draw(m['g']).textlength('g', font=F)
    wm = ImageDraw.Draw(m['motion']).textlength('motion', font=F)
    gol = marime * 0.47
    x0 = cx - (wg + gol + wm) / 2
    urcare = F.getmetrics()[0]
    y0 = cy - urcare * 0.72
    ImageDraw.Draw(m['g']).text((x0, y0), 'g', font=F, fill=255)
    ImageDraw.Draw(m['motion']).text((x0 + wg + gol, y0), 'motion', font=F, fill=255)
    r, px, py = marime * 0.15, x0 + wg + gol / 2, y0 + urcare * 0.72 + marime * 0.10
    ImageDraw.Draw(m['punct']).ellipse([px - r, py - r, px + r, py + r], fill=255)
    yy = np.mgrid[0:H, 0:W][0].astype(np.float32)
    u = np.clip((yy - y0 - marime * 0.25) / (marime * 0.9), 0, 1)[..., None]
    g_culoare = np.array([255, 140, 50], np.float32) * (1 - u) + np.array([60, 220, 200], np.float32) * u
    return {k: np.asarray(v, np.float32)[..., None] / 255 for k, v in m.items()}, g_culoare


def compune(tema, W, H, sus, jos, cx, cy, marime, putere_unde, putere_sigla, margine=0):
    baza = np.ones((H, W, 3), np.float32) * np.array(FUNDAL[tema], np.float32)
    lumina = strat_unde(W, H, sus, jos) * 255 * putere_unde
    if margine:  # marginile se topesc in fundal: fara cusatura cand fereastra e mai mare
        xx = np.arange(W, dtype=np.float32)
        yy = np.arange(H, dtype=np.float32)
        fx = np.clip(np.minimum(xx, W - 1 - xx) / margine, 0, 1)
        fy = np.clip(np.minimum(yy, H - 1 - yy) / margine, 0, 1)
        lumina *= (fy[:, None] * fx[None, :])[..., None]
    if tema == 'intunecat':
        img = baza + lumina
        img += np.random.default_rng(9).normal(0, 2.0, (H, W, 1)).astype(np.float32) * (1 if not margine else 0)
    else:  # pe alb, undele se pun ca cerneala, nu ca lumina
        a = np.clip(lumina.max(axis=2, keepdims=True) / 255, 0, 1)
        img = baza * (1 - a) + lumina / np.maximum(lumina.max(axis=2, keepdims=True), 1) * 200 * a
    if putere_sigla:
        m, g_culoare = masti_sigla(W, H, cx, cy, marime)
        if tema == 'intunecat':  # un halou moale, ca sigla sa stea in lumina undelor
            halou = estompa(np.maximum(m['g'], m['motion']), marime * 0.25)
            img = img + halou * np.array([150, 170, 210], np.float32) * 0.35 * putere_sigla
        for masca, c in ((m['g'], g_culoare), (m['punct'], np.array(ACCENT, np.float32)),
                         (m['motion'], np.array(TEXT[tema], np.float32))):
            a = masca * putere_sigla
            img = img * (1 - a) + c * a
    return np.clip(img, 0, 255).astype(np.uint8)


def main():
    # ecranul de pornire, ca in proba aprobata (Mihai, 2026-11-27)
    W, H = 1366, 768
    plin = compune('intunecat', W, H, H * 0.22, H * 0.80, W / 2, H / 2, 130, 1.0, 1.0)
    unde = compune('intunecat', W, H, H * 0.22, H * 0.80, W / 2, H / 2, 130, 1.0, 0)
    Image.fromarray(plin).save(os.path.join(AICI, 'pornire-1366x768.png'))
    Image.fromarray(unde).save(os.path.join(AICI, 'unde-1366x768.png'))
    gif(plin, unde)
    # fundalul listei: 1920x900, centrat sub zona de sus si desenat fara scalare;
    # la 1366x768 se vede mijlocul, ~1366x560. Sigla sta cu 240 px la stanga
    # mijlocului: acolo e mijlocul coloanei cu lista, langa panoul de detalii,
    # atat la 1366 cat si la 1920 de latime
    W, H = 1920, 900
    for tema in FUNDAL:
        img = compune(tema, W, H, H / 2 - 160, H / 2 + 175, W / 2 - 240, H / 2 + 5, 84,
                      0.45 if tema == 'intunecat' else 0.55,
                      0.22 if tema == 'intunecat' else 0.16, margine=160)
        Image.fromarray(img).save(os.path.join(RADACINA, f'rust/fereastra/imagini/fundal-{tema}.png'), optimize=True)


def gif(plin, unde):
    W, H = 683, 384
    mic = lambda a: np.asarray(Image.fromarray(a).resize((W, H), Image.LANCZOS), np.float32)
    B = np.ones((H, W, 3), np.float32) * np.array(FUNDAL['intunecat'], np.float32)
    U, L = mic(unde), mic(plin)
    xs = np.arange(W)[None, :, None].astype(np.float32)
    usor = lambda t: 1 - (1 - t) ** 3
    cadre, durate = [], []
    for i in range(14):  # undele se deseneaza de la stanga la dreapta
        t = usor((i + 1) / 14)
        m = np.clip((t * W * 1.2 - xs) / 60, 0, 1)
        cadre.append(B * (1 - m) + U * m); durate.append(45)
    for i in range(10):  # apare sigla
        t = usor((i + 1) / 10)
        cadre.append(U * (1 - t) + L * t); durate.append(45)
    cadre.append(L); durate.append(1500)
    for i in range(10):  # se stinge spre fundal
        t = usor((i + 1) / 10)
        cadre.append(L * (1 - t) + B * t); durate.append(40)
    cadre.append(B); durate.append(600)
    imgs = [Image.fromarray(np.clip(c, 0, 255).astype(np.uint8)) for c in cadre]
    imgs[0].save(os.path.join(AICI, 'pornire.gif'), save_all=True, append_images=imgs[1:], duration=durate, loop=0)


if __name__ == '__main__':
    main()
