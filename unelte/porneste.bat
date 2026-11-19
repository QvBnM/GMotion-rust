@echo off
REM Porneste `gmotion-ui`, pe Windows. Implicit pe ecranul de conectare, la
REM serverele tale (teste reale, din 2026-11-19).
REM
REM   unelte\porneste.bat                       ecranul de conectare
REM   unelte\porneste.bat --tema luminos        la fel, tema luminoasa
REM   unelte\porneste.bat --sursa "c:\x.json"   pe un instantaneu al tau
REM   unelte\porneste.bat --previzualizare      instantaneul sintetic, fara server
REM
REM Caile cu spatii se pun in ghilimele. Argumentele se trimit mai departe la
REM aplicatie.
setlocal enabledelayedexpansion
cd /d "%~dp0.."

echo Compilez ^(prima data dureaza cateva minute^)...
REM profilul `proba` (rust\Cargo.toml): compileaza repede; release e pentru .exe-ul final
cargo build --profile proba --manifest-path rust\Cargo.toml -p gmotion-app || exit /b 1

echo.
echo Pornesc gmotion-ui %*
echo   F1 ajutor  -  F11 ecran complet  -  Ctrl+F cauta  -  Enter deschide obiectul
echo.
rust\target\proba\gmotion-ui.exe %*
set RC=!ERRORLEVEL!
if not "!RC!"=="0" (
  echo.
  echo Aplicatia s-a inchis cu codul !RC!. Mesajul de mai sus spune de ce.
  pause
)
exit /b !RC!
