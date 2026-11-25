@echo off
REM Poarta, pe Windows. Vezi unelte/verifica.sh pentru varianta completa:
REM asta ruleaza doar partea de Rust plus harness-ul, fara verificarile scrise
REM in bash (directia dependentelor, sincronizarea CRATE.md).
REM Pentru poarta completa pe Windows: bash unelte/verifica.sh (Git Bash / WSL).
setlocal
cd /d "%~dp0.."
set RC=0

echo == format
cargo fmt --manifest-path rust\Cargo.toml --all -- --check || set RC=1

echo == clippy
cargo clippy --manifest-path rust\Cargo.toml --all-targets --all-features -- -D warnings || set RC=1

echo == teste
cargo test --manifest-path rust\Cargo.toml --all || set RC=1

REM Generatoarele **scriu** fixture-ul, deci rulate direct isi rescriu singure
REM asteptarea pe care regula 2 din CLAUDE.md o declara intangibila (poarta de
REM pe Linux a fost reparata la auditul din 2026-11-23; asta a rams cu
REM comportamentul vechi pana pe 2026-11-25). Aici fixture-ul se pune la loc si
REM se **compara**: o regenerare care l-ar schimba face poarta rosie.
echo == harness de referinta
REM Fara `&&` / `||` aici: un harness cazut ar arata ca "sarit: lipseste".
where node >nul 2>&1
if errorlevel 1 (echo   sarit: node lipseste) else (call :harness referinta\rezultate_js.json node referinta\genereaza.mjs)
where python >nul 2>&1
if errorlevel 1 (echo   sarit: python lipseste) else (call :harness referinta\rezultate_python.json python -I referinta\genereaza.py)

echo.
if "%RC%"=="0" (echo POARTA VERDE) else (echo POARTA ROSIE)
exit /b %RC%

:harness
set "FIXTURE=%~1"
shift
copy /y "%FIXTURE%" "%FIXTURE%.copie" >nul
%1 %2 %3 %4 %5 %6
if errorlevel 1 (
  copy /y "%FIXTURE%.copie" "%FIXTURE%" >nul
  del "%FIXTURE%.copie"
  set RC=1
  goto :eof
)
fc /b "%FIXTURE%.copie" "%FIXTURE%" >nul
if errorlevel 1 (
  copy /y "%FIXTURE%.copie" "%FIXTURE%" >nul
  echo   regenerarea ar schimba %FIXTURE%. Daca e intentionat, ruleaza
  echo   unelte/regenereaza-fixture.sh ca pas propriu si noteaza motivul in JURNAL.md
  set RC=1
) else (
  echo   ok    %FIXTURE% neschimbat
)
del "%FIXTURE%.copie"
goto :eof
