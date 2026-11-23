# Care e ultima versiune a proiectului (Windows / PowerShell). Vezi
# unelte/ultima-versiune.sh pentru motiv.
#
#   .\unelte\ultima-versiune.ps1
Set-Location (Join-Path $PSScriptRoot "..")
git fetch --prune --quiet origin
if ($LASTEXITCODE -ne 0) { Write-Host "ATENTIE: git fetch a esuat; lista poate fi veche." }

$curent = git rev-parse --abbrev-ref HEAD
Write-Host "Branch-uri pe origin, cel mai nou primul:"
git for-each-ref --sort=-committerdate --count=9 `
  --format='  %(committerdate:short) %(committerdate:relative)  %(refname:short)  %(subject)' `
  refs/remotes/origin | Where-Object { $_ -notmatch 'origin/HEAD' } | Select-Object -First 8

$celNou = git for-each-ref --sort=-committerdate --format='%(refname:short)' refs/remotes/origin |
  Where-Object { $_ -ne 'origin/HEAD' } | Select-Object -First 1
$numeNou = $celNou -replace '^origin/', ''
Write-Host ""
Write-Host "Ultima versiune: $numeNou"
Write-Host "Branch curent:   $curent"
if ($curent -eq $numeNou) {
  $inUrma = git rev-list --count "HEAD..$celNou"
  if ([int]$inUrma -gt 0) { Write-Host "ATENTIE: esti cu $inUrma commit-uri in urma: git pull origin $curent" }
  else { Write-Host "Esti pe branch-ul cel mai nou dupa data." }
} else {
  $lipsesc = git rev-list --count "HEAD..$celNou"
  Write-Host "ATENTIE: NU lucrezi pe ultima versiune. $numeNou are $lipsesc commit-uri care nu sunt aici."
  Write-Host "Ca sa treci pe ea:  git checkout $numeNou ; git pull origin $numeNou"
}

# lucru paralel: alte branch-uri cu commit-uri care nu sunt in cel mai nou
$paralel = $false
foreach ($b in (git for-each-ref --format='%(refname:short)' refs/remotes/origin | Where-Object { $_ -ne 'origin/HEAD' -and $_ -ne $celNou })) {
  $baza = git merge-base $celNou $b 2>$null
  $n = git rev-list --count "$celNou..$b"
  if ($baza -and [int]$n -gt 0) {
    Write-Host "LUCRU PARALEL: $($b -replace '^origin/', '') are $n commit-uri care nu sunt in $numeNou."
    $paralel = $true
  }
}
if ($paralel) {
  Write-Host ""
  Write-Host "ATENTIE: munca e impartita pe mai multe branch-uri neunite; cel mai nou dupa data NU le contine pe toate."
}
