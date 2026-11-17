@echo off
REM Poarta, pe Windows. Vezi unelte/verifica.sh pentru varianta completa:
REM asta ruleaza doar partea de Rust plus harness-ul, fara verificarile scrise
REM in bash (direcția dependentelor, sincronizarea CRATE.md).
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

echo == harness de referinta
where node >nul 2>&1 && (node referinta\genereaza.mjs || set RC=1) || echo   sarit: node lipseste
where python >nul 2>&1 && (python -I referinta\genereaza.py || set RC=1) || echo   sarit: python lipseste

echo.
if "%RC%"=="0" (echo POARTA VERDE) else (echo POARTA ROSIE)
exit /b %RC%
