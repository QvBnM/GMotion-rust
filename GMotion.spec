# -*- mode: python ; coding: utf-8 -*-
#
# Fisier de build PyInstaller pentru GMotion.
# Produce un SINGUR .exe care porneste direct in fereastra aplicatiei
# (fara consola, fara prompt UAC).
#
# Compilare (pe Windows, in folderul aplicatiei):
#     pip install pyinstaller pyodbc pywebview
#     pyinstaller --clean GMotion.spec
# Rezultatul: dist\GMotion.exe

block_cipher = None

a = Analysis(
    ['app.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=['pyodbc', 'webview', 'webview.platforms.edgechromium', 'clr_loader'],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=['pandas', 'numpy', 'matplotlib', 'sqlalchemy'],  # nu sunt necesare -> .exe mai mic
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name='GMotion',                     # S-a schimbat numele pentru a genera GMotion.exe
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=False,                      # FARA fereastra de consola la pornire
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    uac_admin=False,                    # FARA prompt UAC la pornire
    manifest=None,
    icon="GMotion.ico",                 # Iconita aplicatiei tale
)