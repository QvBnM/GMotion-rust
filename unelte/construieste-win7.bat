@echo off
REM Construieste gmotion.exe si gmotion-ui.exe pentru Windows vechi
REM (Windows 7 / Server 2008 R2 / Server 2012), ca proba. Se ruleaza pe
REM masina ta de lucru (Windows 10/11), nu pe serverul vechi; ce iese se
REM copiaza apoi acolo.
REM
REM   unelte\construieste-win7.bat
REM
REM Foloseste varianta Rust pentru Windows 7 (`x86_64-win7-windows-msvc`),
REM care cere compilatorul de proba (nightly) si sursele bibliotecii
REM standard; scriptul le instaleaza singur, o singura data. Runtime-ul C se
REM leaga static (`crt-static`), ca pe server sa nu trebuiasca instalat
REM Visual C++ Redistributable.
setlocal
cd /d "%~dp0.."

echo Instalez compilatorul nightly si sursele bibliotecii standard (o data)...
rustup toolchain install nightly --profile minimal --component rust-src || exit /b 1

echo.
echo Compilez pentru Windows 7+ ^(prima data dureaza 5-10 minute^)...
set RUSTFLAGS=-C target-feature=+crt-static
cargo +nightly build -Z build-std --target x86_64-win7-windows-msvc --release --manifest-path rust\Cargo.toml -p gmotion-cli -p gmotion-app || exit /b 1

if not exist proba-win7 mkdir proba-win7
copy /y rust\target\x86_64-win7-windows-msvc\release\gmotion.exe proba-win7\ >nul || exit /b 1
copy /y rust\target\x86_64-win7-windows-msvc\release\gmotion-ui.exe proba-win7\ >nul || exit /b 1

echo.
echo Gata. Copiaza pe serverul vechi cele doua fisiere din:
echo   %CD%\proba-win7\
echo si ruleaza acolo, intr-un cmd deschis in folderul lor:
echo   gmotion.exe --help
echo   gmotion.exe compara --server NUME_SERVER --referinta BAZA_A --baze BAZA_B --auth windows
echo   gmotion-ui.exe --previzualizare
