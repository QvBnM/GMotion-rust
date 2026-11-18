@echo off
REM Porneste `gmotion-ui`, pe Windows. Implicit intra direct in lista, pe
REM instantaneul sintetic de previzualizare: toate categoriile, toate
REM tipurile, patru baze, obiecte mari, cod scris dezordonat. Nu atinge
REM niciun server.
REM
REM   unelte\porneste.bat                       previzualizare, tema intunecata
REM   unelte\porneste.bat --tema luminos        previzualizare, tema luminoasa
REM   unelte\porneste.bat --server              ecranul de conectare, gol
REM   unelte\porneste.bat --sursa "c:\x.json"   pe un instantaneu al tau
REM
REM Caile cu spatii se pun in ghilimele. Restul argumentelor se trimit mai
REM departe la aplicatie.
setlocal enabledelayedexpansion
cd /d "%~dp0.."

REM instantaneul de previzualizare se commiteaza; daca lipseste (fisier
REM sters), se face acum
if not exist "referinta\instantanee\previzualizare.json" (
  where python >nul 2>&1 && (
    echo Generez instantaneul de previzualizare...
    python -I unelte\genereaza-previzualizare.py || exit /b 1
  ) || echo   sarit: python lipseste
)

echo Compilez ^(prima data dureaza cateva minute^)...
cargo build --release --manifest-path rust\Cargo.toml -p gmotion-app || exit /b 1

REM fara `--server` si fara `--sursa`, pornim in previzualizare
set "MOD=--previzualizare"
set "ARGS="
for %%A in (%*) do (
  if /i "%%~A"=="--server" (
    set "MOD="
  ) else (
    if /i "%%~A"=="--sursa" set "MOD="
    set "ARGS=!ARGS! "%%~A""
  )
)

echo.
echo Pornesc gmotion-ui !MOD!!ARGS!
echo   F1 ajutor  -  F11 ecran complet  -  Ctrl+F cauta  -  Enter deschide obiectul
echo.
rust\target\release\gmotion-ui.exe !MOD!!ARGS!
exit /b %ERRORLEVEL%
