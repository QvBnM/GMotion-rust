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
from PIL import Image, ImageDraw, ImageFilter, ImageFont

AICI = os.path.dirname(os.path.abspath(__file__))
RADACINA = os.path.dirname(os.path.dirname(AICI))
FONT = os.path.join(RADACINA, 'rust/fereastra/fonturi/Geist-600.ttf')
FUNDAL = {'intunecat': (0x0d, 0x11, 0x20), 'luminos': (0xf4, 0xf5, 0xf8)}
TEXT = {'intunecat': (238, 241, 255), 'luminos': (15, 17, 21)}
ACCENT = (255, 181, 71)
OPRIRI = [(0, (255, 120, 40)), (0.4, (255, 200, 60)), (0.7, (80, 230, 190)), (1, (40, 200, 230))]


def culoare(t):
    for (a, ca), (b, cb) in zip(OPRIRI, OPRIRI[1:]):
        if t <= b:
            u = (t - a) / (b - a)
            return tuple(int(ca[i] + (cb[i] - ca[i]) * u) for i in range(3))
    return OPRIRI[-1][1]


def panglica(W, H, cy, amp, faza, n, inversa=False, ss=2):
    """O panglica de `n` fire subtiri, culoarea dupa x (portocaliu -> turcoaz)."""
    L = Image.new('RGB', (W * ss, H * ss))
    d = ImageDraw.Draw(L)
    xs = np.linspace(-20, W + 20, 500)
    for k in range(n):
        u = k / (n - 1) - 0.5
        y = (cy + amp * np.sin(xs / (260 + u * 60) + faza + u * 1.6) * np.cos(xs / 530 + faza * 0.5)
             + u * 150 * np.sin(xs / 380 + faza + 1.3) * np.sin(xs / 900 + faza))
        pts = list(zip(xs * ss, y * ss))
        for i in range(len(pts) - 1):
            t = min(max(xs[i] / W, 0), 1)
            c = culoare(1 - t if inversa else t)
            f = 0.35 + 0.65 * (1 - abs(u) * 1.6)
            d.line([pts[i], pts[i + 1]], fill=tuple(int(v * f) for v in c), width=ss)
    return np.asarray(L.resize((W, H), Image.LANCZOS), np.float32)


def strat_unde(W, H, sus, jos, samanta=11):
    """Lumina undelor, particulelor si petelor, pe negru (se aduna peste fundal)."""
    rng = np.random.default_rng(samanta)
    acc = panglica(W, H, jos, 90, 0.3, 24) + 0.8 * panglica(W, H, sus, 70, 2.4, 22)
    halou = np.asarray(Image.fromarray(np.clip(acc, 0, 255).astype(np.uint8)).filter(ImageFilter.GaussianBlur(10)), np.float32)
    acc = acc + 1.4 * halou
    P = Image.new('RGB', (W, H))
    d = ImageDraw.Draw(P)
    for _ in range(int(260 * W * H / (1366 * 768))):
        x = rng.uniform(0, W)
        y = rng.normal(jos, 90) if rng.random() < .7 else rng.normal(sus, 70)
        r = rng.choice([1, 1, 1.5, 2])
        d.ellipse([x - r, y - r, x + r, y + r], fill=culoare(x / W))
    B = Image.new('RGB', (W, H))
    d = ImageDraw.Draw(B)
    for _ in range(14):
        x, y, r = rng.uniform(0, W), rng.uniform(0, H), rng.uniform(15, 45)
        d.ellipse([x - r, y - r, x + r, y + r], fill=tuple(int(v * .35) for v in culoare(x / W)))
    acc += 0.9 * np.asarray(P, np.float32) + np.asarray(B.filter(ImageFilter.GaussianBlur(9)), np.float32)
    return acc


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
    lumina = strat_unde(W, H, sus, jos) * putere_unde
    if margine:  # marginile se topesc in fundal: fara cusatura cand fereastra e mai mare
        xx = np.arange(W, dtype=np.float32)
        yy = np.arange(H, dtype=np.float32)
        fx = np.clip(np.minimum(xx, W - 1 - xx) / margine, 0, 1)
        fy = np.clip(np.minimum(yy, H - 1 - yy) / margine, 0, 1)
        lumina *= (fy[:, None] * fx[None, :])[..., None]
    if tema == 'intunecat':
        img = baza + lumina
    else:  # pe alb, undele se pun ca cerneala, nu ca lumina
        a = np.clip(lumina.max(axis=2, keepdims=True) / 255, 0, 1)
        img = baza * (1 - a) + lumina / np.maximum(lumina.max(axis=2, keepdims=True), 1) * 200 * a
    if putere_sigla:
        m, g_culoare = masti_sigla(W, H, cx, cy, marime)
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
                      0.45 if tema == 'intunecat' else 0.55, 0.35, margine=160)
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
