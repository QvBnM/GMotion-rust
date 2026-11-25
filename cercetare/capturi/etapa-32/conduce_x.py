# Conduce un display X: x.py clic X Y | muta X Y | tasta NUME | scrie TEXT | dublu X Y | roata X Y N | trage X1 Y1 X2 Y2 | ferestre | muta_fereastra X Y
import sys, time
from Xlib import X, XK, display
from Xlib.ext import xtest
d = display.Display()
r = d.screen().root
def muta(x, y):
    xtest.fake_input(d, X.MotionNotify, x=int(x), y=int(y)); d.sync(); time.sleep(0.05)
def buton(b, jos):
    xtest.fake_input(d, X.ButtonPress if jos else X.ButtonRelease, b); d.sync(); time.sleep(0.05)
def tasta(nume):
    mods = []
    parti = nume.split('+')
    for m in parti[:-1]:
        mods.append({'ctrl':'Control_L','shift':'Shift_L','alt':'Alt_L'}[m])
    ks = XK.string_to_keysym(parti[-1])
    if ks == 0 and len(parti[-1]) == 1: ks = ord(parti[-1])
    kc = d.keysym_to_keycode(ks)
    for m in mods: xtest.fake_input(d, X.KeyPress, d.keysym_to_keycode(XK.string_to_keysym(m)))
    xtest.fake_input(d, X.KeyPress, kc); xtest.fake_input(d, X.KeyRelease, kc)
    for m in reversed(mods): xtest.fake_input(d, X.KeyRelease, d.keysym_to_keycode(XK.string_to_keysym(m)))
    d.sync(); time.sleep(0.05)
a = sys.argv[1:]
while a:
    c = a.pop(0)
    if c == 'clic': x, y = a.pop(0), a.pop(0); muta(x, y); buton(1, True); buton(1, False)
    elif c == 'dublu': x, y = a.pop(0), a.pop(0); muta(x, y); buton(1, True); buton(1, False); buton(1, True); buton(1, False)
    elif c == 'muta': muta(a.pop(0), a.pop(0))
    elif c == 'tasta': tasta(a.pop(0))
    elif c == 'scrie':
        for ch in a.pop(0): tasta('space' if ch == ' ' else ch)
    elif c == 'roata':
        x, y, n = a.pop(0), a.pop(0), int(a.pop(0)); muta(x, y)
        b = 5 if n > 0 else 4
        for _ in range(abs(n)): buton(b, True); buton(b, False)
    elif c == 'trage':
        x1, y1, x2, y2 = [int(a.pop(0)) for _ in range(4)]
        muta(x1, y1); buton(1, True)
        for i in range(1, 11): muta(x1 + (x2-x1)*i/10, y1 + (y2-y1)*i/10)
        buton(1, False)
    elif c == 'asteapta': time.sleep(float(a.pop(0)))
    elif c == 'marime':
        w_, h_ = int(a.pop(0)), int(a.pop(0))
        for w in r.query_tree().children:
            g = w.get_geometry()
            if g.width > 50 and w.get_attributes().map_state == 2:
                w.configure(x=0, y=0, width=w_, height=h_)
        d.sync(); time.sleep(0.8)
    elif c == 'ferestre':
        for w in r.query_tree().children:
            g = w.get_geometry(); n = w.get_wm_name()
            if g.width > 50: print(w.id, n, g.x, g.y, g.width, g.height, w.get_attributes().map_state)
