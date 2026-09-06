# K-169/ADR-071 Windows kurulum: SHA256SUMS doğrulayarak dil.exe/dillsp.exe kopyalar
# ve kurulum manifestini yazar.
#   powershell -File scripts/kur.ps1 -Artefakt KLASÖR [-Hedef $env:LOCALAPPDATA\zee\bin] [-Veri $env:LOCALAPPDATA\zee]
param(
    [Parameter(Mandatory = $true)][string]$Artefakt,
    [string]$Hedef = "$env:LOCALAPPDATA\zee\bin",
    [string]$Veri = "$env:LOCALAPPDATA\zee"
)
$ErrorActionPreference = "Stop"
$ozetDosyasi = Join-Path $Artefakt "SHA256SUMS"
if (-not (Test-Path $ozetDosyasi)) { throw "Artefaktta SHA256SUMS yok; doğrulanamayan ikili kurulmaz." }
$manifest = Join-Path $Veri "kurulum-v1.tsv"
if (Test-Path $manifest) { throw "Önce var olan kurulumu kaldır: scripts/kaldir.ps1 -Veri $Veri" }
New-Item -ItemType Directory -Force -Path $Hedef, $Veri | Out-Null
$gitSha = if (Test-Path (Join-Path $Artefakt "GIT_SHA")) { (Get-Content (Join-Path $Artefakt "GIT_SHA")).Trim() } else { "-" }
$satirlar = @("# zee-kurulum-1", "# dosya`tsha256`tkaynak_git_sha")
$kurulan = 0
foreach ($satir in Get-Content $ozetDosyasi) {
    if ($satir.Trim() -eq "") { continue }
    $parcalar = $satir -split "  ", 2
    $beklenen = $parcalar[0]; $ad = $parcalar[1]
    if ($ad -notin @("dil", "dil.exe", "dillsp", "dillsp.exe")) { continue }
    $kaynak = Join-Path $Artefakt $ad
    if (-not (Test-Path $kaynak)) { throw "Artefaktta $ad yok." }
    $bulunan = (Get-FileHash -Algorithm SHA256 $kaynak).Hash.ToLower()
    if ($bulunan -ne $beklenen) { throw "SHA-256 uyuşmuyor: $ad; kurulum iptal." }
    $hedefDosya = Join-Path $Hedef $ad
    if (Test-Path $hedefDosya) { throw "$hedefDosya zaten var; kaldırmadan üzerine yazılmaz." }
    Copy-Item $kaynak $hedefDosya
    $satirlar += "$hedefDosya`t$bulunan`t$gitSha"
    $kurulan++
}
if ($kurulan -eq 0) { throw "SHA256SUMS içinde dil/dillsp girdisi yok." }
Set-Content -Path $manifest -Value ($satirlar -join "`n") -Encoding utf8 -NoNewline
Write-Output "Kuruldu ($kurulan ikili) -> $Hedef; manifest: $manifest"
