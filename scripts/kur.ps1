# K-169/ADR-071, K-194: önce bütün ikilileri geçici alanda doğrula;
# yayınlama hatasında bu koşunun oluşturduğu dosyaları geri al.
param(
    [Parameter(Mandatory = $true)][string]$Artefakt,
    [string]$Hedef = "$env:LOCALAPPDATA\zee\bin",
    [string]$Veri = "$env:LOCALAPPDATA\zee"
)
$ErrorActionPreference = "Stop"
$ozetDosyasi = Join-Path $Artefakt "SHA256SUMS"
if (-not (Test-Path -LiteralPath $ozetDosyasi -PathType Leaf)) { throw "Artefaktta SHA256SUMS yok; doğrulanamayan ikili kurulmaz." }
if ($Hedef -match '[\t\r\n]' -or $Veri -match '[\t\r\n]') { throw "Kurulum yolu TSV kontrol karakteri içeremez." }
$gitSha = "-"
if (Test-Path -LiteralPath (Join-Path $Artefakt "GIT_SHA")) {
    $gitSha = (Get-Content -LiteralPath (Join-Path $Artefakt "GIT_SHA") -Raw -Encoding utf8).Trim()
    if ($gitSha -cnotmatch '\A[0-9a-f]{40}\z') { throw "GIT_SHA exact 40 küçük hex olmalı." }
}
$manifest = Join-Path $Veri "kurulum-v1.tsv"
if (Test-Path -LiteralPath $manifest) { throw "Önce var olan kurulumu kaldır: scripts/kaldir.ps1 -Veri $Veri" }
New-Item -ItemType Directory -Force -Path $Hedef, $Veri | Out-Null
$Hedef = (Resolve-Path -LiteralPath $Hedef).ProviderPath
$Veri = (Resolve-Path -LiteralPath $Veri).ProviderPath
if ($Hedef -match '[\t\r\n]' -or $Veri -match '[\t\r\n]') { throw "Kurulum yolu TSV kontrol karakteri içeremez." }
$manifest = Join-Path $Veri "kurulum-v1.tsv"
$hazirlik = Join-Path $Hedef (".zee-kurulum-" + [Guid]::NewGuid().ToString("N"))
$geciciManifest = Join-Path $Veri (".zee-kurulum-manifest-" + [Guid]::NewGuid().ToString("N"))
$kurulanYollar = [System.Collections.Generic.List[string]]::new()
$kurulacakAdlar = [System.Collections.Generic.List[string]]::new()
$gorulen = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
$tamamlandi = $false
try {
    New-Item -ItemType Directory -Path $hazirlik | Out-Null
    $satirlar = @("# zee-kurulum-1", "# dosya`tsha256`tkaynak_git_sha")
    foreach ($satir in Get-Content -LiteralPath $ozetDosyasi) {
        if ($satir.Trim() -eq "") { continue }
        $parcalar = $satir -split "  ", 2
        if ($parcalar.Length -ne 2) { throw "SHA256SUMS satırı geçersiz." }
        $beklenen = $parcalar[0]; $ad = $parcalar[1]
        if ($ad -notin @("dil", "dil.exe", "dillsp", "dillsp.exe")) { continue }
        if ($beklenen -cnotmatch '^[0-9a-f]{64}$') { throw "SHA256SUMS özeti geçersiz: $ad" }
        if (-not $gorulen.Add($ad)) { throw "SHA256SUMS içinde yinelenen ad: $ad" }
        $kaynak = Join-Path $Artefakt $ad
        if (-not (Test-Path -LiteralPath $kaynak -PathType Leaf)) { throw "Artefaktta $ad yok." }
        $hedefDosya = Join-Path $Hedef $ad
        if (Test-Path -LiteralPath $hedefDosya) { throw "$hedefDosya zaten var; kaldırmadan üzerine yazılmaz." }
        $hazirDosya = Join-Path $hazirlik $ad
        [System.IO.File]::Copy($kaynak, $hazirDosya, $false)
        $bulunan = (Get-FileHash -Algorithm SHA256 -LiteralPath $hazirDosya).Hash.ToLower()
        if ($bulunan -ne $beklenen) { throw "SHA-256 uyuşmuyor: $ad; kurulum iptal." }
        $kurulacakAdlar.Add($ad)
        $satirlar += "$hedefDosya`t$bulunan`t$gitSha"
    }
    if ($kurulacakAdlar.Count -eq 0) { throw "SHA256SUMS içinde dil/dillsp girdisi yok." }
    [System.IO.File]::WriteAllText($geciciManifest, ($satirlar -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
    foreach ($ad in $kurulacakAdlar) {
        $hedefDosya = Join-Path $Hedef $ad
        # Aynı hedef dosya sisteminde taşınır; File.Move var olan dosyayı ezmez.
        [System.IO.File]::Move((Join-Path $hazirlik $ad), $hedefDosya)
        $kurulanYollar.Add($hedefDosya)
    }
    [System.IO.File]::Move($geciciManifest, $manifest)
    $tamamlandi = $true
    Write-Output "Kuruldu ($($kurulanYollar.Count) ikili) -> $Hedef; manifest: $manifest"
} finally {
    if (-not $tamamlandi) {
        foreach ($dosya in $kurulanYollar) { Remove-Item -LiteralPath $dosya -Force }
    }
    if (Test-Path -LiteralPath $hazirlik) { Remove-Item -LiteralPath $hazirlik -Recurse -Force }
    if (Test-Path -LiteralPath $geciciManifest) { Remove-Item -LiteralPath $geciciManifest -Force }
}
