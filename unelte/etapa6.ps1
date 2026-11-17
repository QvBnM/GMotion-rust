# Etapa 6: tot ce trebuie rulat pe Windows, intr-un singur loc.
#
# Rulare, din radacina repo-ului:
#   powershell -ExecutionPolicy Bypass -File unelte\etapa6.ps1
#
# Cere parola o data (fereastra Get-Credential), nu o scrie nicaieri.
# Tot ce apare pe ecran ajunge si in etapa6-rezultat.txt — acel fisier se
# trimite inapoi. Nu contine cod SQL, doar nume de obiecte si numaratori.
#
# Ce afla, in ordine:
#   1. versiuni (commit, Python, node)
#   2. binarul gmotion, reconstruit
#   3. servere si baze: autentificare SQL si integrata Windows
#   4. instantaneu proaspat (timp Rust) si paritatea logicii Python/Rust pe el
#   5. aplicatia Python cu pyodbc-ul ei pe aceleasi baze (timp Python) si
#      comparatia obiect cu obiect cu ce a adus tiberius
#   6. fixture-ul TESTSIMILITUDINE extras si verificat; ramane local pana
#      decide Mihai daca intra in repo

param(
  [string]$Server = 'TS2409\MIHAIG',
  [string]$Utilizator = 'sa',
  [string]$Referinta = 'CharismaERP_GetsBet',
  [string]$Tinta = 'CharismaERP_GetsBet_Copy'
)

$ErrorActionPreference = 'Continue'
Set-Location (Split-Path -Parent $PSScriptRoot)
$rezultat = Join-Path (Get-Location) 'etapa6-rezultat.txt'
$gm = 'rust\target\release\gmotion.exe'
$inst = 'rust\target\release\instantaneu.json'
$fixture = 'referinta\instantanee\testsimilitudine.json'

# pyodbc e necesar pentru pasul 5; se verifica inainte, nu la jumatate.
& py -3 -c "import pyodbc" 2>$null
if ($LASTEXITCODE -ne 0) {
  $r = Read-Host "pyodbc lipseste din Python. Il instalez acum cu pip? (d/n)"
  if ($r -eq 'd') { & py -3 -m pip install pyodbc }
}

# Compilarea INAINTE de parola: altfel scripturile de build ale tuturor
# dependentelor (build.rs) ar rula cu parola in variabilele de mediu.
$compilare = & cargo build --release --manifest-path rust\Cargo.toml -p gmotion-cli 2>&1 | ForEach-Object { "$_" } | Select-Object -Last 2
$codCompilare = $LASTEXITCODE

$c = Get-Credential -UserName $Utilizator -Message "Parola SQL Server pentru $Server"
if (-not $c) { Write-Host "Anulat."; exit 1 }
$env:GMOTION_PAROLA = $c.GetNetworkCredential().Password

function Pas([string]$titlu) { "`n==== $titlu" }

# Ruleaza o comanda externa, arata ieșirea ca text, codul de ieșire si durata.
function Ruleaza([scriptblock]$bloc) {
  $t = Get-Date
  & $bloc 2>&1 | ForEach-Object { "$_" }
  $cod = $LASTEXITCODE
  "-> cod iesire: $cod  ($([math]::Round(((Get-Date) - $t).TotalSeconds, 1)) s)"
}

& {
  Pas "1. versiuni"
  "commit: $(git rev-parse --short HEAD)"
  "python: $(& py -3 --version 2>&1)"
  "node:   $(& node --version 2>&1)"
  "data:   $(Get-Date -Format s)"

  Pas "2. binarul gmotion"
  $compilare
  "-> cod iesire: $codCompilare (compilat inainte de cererea parolei)"

  Pas "3a. servere in retea"
  Ruleaza { & $gm servere }
  Pas "3b. baze, autentificare SQL"
  Ruleaza { & $gm baze --server $Server --utilizator $Utilizator }
  Pas "3c. baze, autentificare integrata Windows"
  Ruleaza { & $gm baze --server $Server }

  Pas "4a. instantaneu proaspat (timp Rust: citire secventiala)"
  Ruleaza { & $gm instantaneu --server $Server --utilizator $Utilizator --baze "$Referinta,$Tinta" -o $inst --nota "etapa 6" }
  "marime: $([math]::Round((Get-Item $inst).Length / 1MB, 1)) MB"
  Pas "4b. paritate: logica Python v3.4 contra gmotion, pe instantaneu"
  Ruleaza { & py -3 -I referinta\paritate.py $inst --referinta $Referinta --gmotion $gm --json raport.json }

  Pas "5. aplicatia Python cu pyodbc (timp Python) contra obiectele aduse de tiberius"
  Ruleaza { & py -3 -I referinta\paritate_pyodbc.py $inst --server $Server --utilizator $Utilizator --referinta $Referinta }

  Pas "6. fixture TESTSIMILITUDINE (ramane local)"
  New-Item -ItemType Directory -Force (Split-Path $fixture) | Out-Null
  Ruleaza { & py -3 -I referinta\extrage.py $inst --filtru TESTSIMILITUDINE -o $fixture }
  if (Test-Path $fixture) {
    "marime: $([math]::Round((Get-Item $fixture).Length / 1KB, 1)) KB"
    Ruleaza { & py -3 -I referinta\paritate.py $fixture --referinta $Referinta --gmotion $gm }
  }

  Pas "gata"
  "Trimite fisierul: $rezultat"
} | Tee-Object -FilePath $rezultat

Remove-Item Env:GMOTION_PAROLA -ErrorAction SilentlyContinue
