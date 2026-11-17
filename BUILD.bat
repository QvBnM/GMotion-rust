@echo off
REM ============================================================
REM  Build GMotion -> un singur .exe (porneste direct, fara UAC)
REM  Ruleaza acest fisier pe o masina Windows cu Python 3.9+
REM  IMPORTANT: NU rula acest .bat ca Administrator! (doar dublu-click normal)
REM ============================================================

REM Muta-te in folderul unde se afla acest .bat (nu ramane in System32)
cd /d "%~dp0"

echo.
echo === GMotion - build .exe ===
echo Folder curent: %CD%
echo.

@echo off
REM Inchidem procesele anterioare ca sa nu blocheze suprascrierea fisierului .exe
taskkill /IM GMotion.exe /F >nul 2>&1
taskkill /IM msedgewebview2.exe /F >nul 2>&1
:wait
tasklist /FI "IMAGENAME eq GMotion.exe" | find /I "GMotion.exe" >nul && (timeout /t 1 >nul & goto wait)
rmdir /S /Q dist

REM Refuza rularea ca Administrator (PyInstaller nu compileaza din System32 / ca admin)
net session >nul 2>&1
if %errorlevel% == 0 (
    echo [EROARE] Acest build ruleaza ca ADMINISTRATOR.
    echo PyInstaller NU trebuie rulat ca admin.
    echo.
    echo Inchide aceasta fereastra si porneste BUILD.bat prin DUBLU-CLICK normal
    echo ^(fara click-dreapta -^> Run as administrator^).
    echo.
    echo Doar EXE-ul final ruleaza elevat, nu si compilarea.
    pause
    exit /b 1
)

REM 1. verifica Python
python --version >nul 2>&1
if errorlevel 1 (
    echo [EROARE] Python nu este instalat sau nu e in PATH.
    echo Descarca de la https://www.python.org/downloads/ si bifeaza "Add to PATH".
    pause
    exit /b 1
)

REM 2. instaleaza dependintele necesare pentru BUILD
echo Instalez dependintele de build ^(pyinstaller, pyodbc, pywebview^)...
python -m pip install --upgrade pip
python -m pip install pyinstaller pyodbc pywebview
if errorlevel 1 (
    echo [EROARE] Instalarea dependintelor a esuat.
    pause
    exit /b 1
)

REM 3. compileaza
echo.
echo Compilez GMotion.exe...
pyinstaller --clean GMotion.spec
if errorlevel 1 (
    echo [EROARE] Compilarea a esuat.
    pause
    exit /b 1
)

echo.
echo ============================================================
echo  GATA! Executabilul este in:  dist\GMotion.exe
echo  Copiaza acel fisier la client si ruleaza-l (dublu-click).
echo  Porneste direct in aplicatie - fara consola, fara prompt UAC.
echo ============================================================
echo.
pause