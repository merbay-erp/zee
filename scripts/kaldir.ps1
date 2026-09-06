# K-169/ADR-071 Windows kaldırma: yalnız manifestte kayıtlı ve özeti değişmemiş
# dosyaları siler.
#   powershell -File scripts/kaldir.ps1 [-Veri $env:LOCALAPPDATA\zee] [-Zorla]
param(
    [string]$Veri = "$env:LOCALAPPDATA\zee",
    [switch]$Zorla
)
$ErrorActionPreference = "Stop"
$manifest = Join-Path $Veri "kurulum-v1.tsv"
if (-not (Test-Path $manifest)) { throw "Kurulum manifesti yok: $manifest (kurulu değil)." }
$satirlar = Get-Content $manifest
if ($satirlar[0] -ne "# zee-kurulum-1") { throw "Kurulum manifesti şeması bilinmiyor." }
# Önce bütün dosyalar doğrulanır; tek biri değişmişse hiçbiri silinmez.
$kayitlar = @()
foreach ($satir in $satirlar) {
    if ($satir -eq "" -or $satir.StartsWith("#")) { continue }
    $alanlar = $satir -split "`t"
    $kayitlar += ,@($alanlar[0], $alanlar[1])
    if (-not (Test-Path $alanlar[0])) { continue }
    $bulunan = (Get-FileHash -Algorithm SHA256 $alanlar[0]).Hash.ToLower()
    if ($bulunan -ne $alanlar[1] -and -not $Zorla) { throw "$($alanlar[0]) kurulumdan sonra değişmiş; hiçbir dosya silinmedi (-Zorla ile silinir)." }
}
$silinen = 0
foreach ($kayit in $kayitlar) {
    if (-not (Test-Path $kayit[0])) { Write-Output "Zaten yok: $($kayit[0])"; continue }
    Remove-Item $kayit[0]
    $silinen++
}
Remove-Item $manifest
Write-Output "Kaldırıldı ($silinen dosya); manifest silindi."
