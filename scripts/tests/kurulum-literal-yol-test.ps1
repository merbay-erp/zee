# K-244/GB-067: yalnız sahipli geçici alanda PowerShell literal yol kampanyası.
# İkililer sentetik baytlardır; çalıştırılmaz. PowerShell 5.1 uyumlu sözdizimi.
param(
    [string]$SourceRoot = (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)),
    [string]$Report = ""
)
$ErrorActionPreference = "Stop"
$sandbox = Join-Path ([System.IO.Path]::GetTempPath()) ("zee-literal-yol-" + [Guid]::NewGuid().ToString("N"))
$cases = @()
$failure = $null
$cleaned = $false
$names = @("dil.exe", "dillsp.exe")

function Dogrula([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}
function Ozet([string]$Path) {
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLower()
}
function Hazirla([string]$CaseName, [string]$TargetName, [string]$ReceiptName, [bool]$Sibling) {
    $root = Join-Path $sandbox $CaseName
    $artifact = Join-Path $root "artifact"
    $target = Join-Path $root $TargetName
    $receipt = Join-Path $root $ReceiptName
    $foreign = Join-Path $root "bin1"
    $foreignReceipt = Join-Path $root "veri1"
    New-Item -ItemType Directory -Path $artifact | Out-Null
    $lines = @()
    foreach ($name in $names) {
        $source = Join-Path $artifact $name
        [System.IO.File]::WriteAllBytes($source, [System.Text.Encoding]::UTF8.GetBytes("synthetic non-executed K244 fixture: $name"))
        $lines += "$(Ozet $source)  $name"
    }
    [System.IO.File]::WriteAllText((Join-Path $artifact "SHA256SUMS"), ($lines -join "`n") + "`n", [System.Text.UTF8Encoding]::new($false))
    & (Join-Path $SourceRoot "scripts/kur.ps1") -Artefakt $artifact -Hedef $target -Veri $receipt | Out-Null
    $manifest = Join-Path $receipt "kurulum-v1.tsv"
    Dogrula (Test-Path -LiteralPath $manifest -PathType Leaf) "Kurulum gerçek manifesti yayımlamalı."
    if ($Sibling) {
        New-Item -ItemType Directory -Path $foreign, $foreignReceipt | Out-Null
        foreach ($name in $names) {
            [System.IO.File]::Copy((Join-Path $target $name), (Join-Path $foreign $name))
        }
        [System.IO.File]::Copy($manifest, (Join-Path $foreignReceipt "kurulum-v1.tsv"))
    }
    return [pscustomobject]@{ Target = $target; Receipt = $receipt; Manifest = $manifest; Artifact = $artifact; Foreign = $foreign; ForeignReceipt = $foreignReceipt; Sibling = $Sibling; ManifestHash = (Ozet $manifest) }
}
function YabanciKorundu($Fixture) {
    if (-not $Fixture.Sibling) { return }
    foreach ($name in $names) {
        $path = Join-Path $Fixture.Foreign $name
        Dogrula (Test-Path -LiteralPath $path -PathType Leaf) "Wildcard kardeş dosya silinemez."
        Dogrula ((Ozet $path) -eq (Ozet (Join-Path $Fixture.Artifact $name))) "Wildcard kardeş dosya değiştirilemez."
    }
    $path = Join-Path $Fixture.ForeignReceipt "kurulum-v1.tsv"
    Dogrula ((Ozet $path) -eq $Fixture.ManifestHash) "Wildcard kardeş manifest korunmalı."
}
function KuruluKorundu($Fixture, [bool]$Changed) {
    foreach ($name in $names) {
        $path = Join-Path $Fixture.Target $name
        Dogrula (Test-Path -LiteralPath $path -PathType Leaf) "Ret sırasında kurulu dosyaların tamamı korunmalı."
        if ($Changed -and $name -eq "dil.exe") {
            Dogrula ([System.IO.File]::ReadAllText($path) -eq "changed fixture") "Değişmiş dosya aynen korunmalı."
        } else {
            Dogrula ((Ozet $path) -eq (Ozet (Join-Path $Fixture.Artifact $name))) "Ret sırasında kurulu dosya değiştirilemez."
        }
    }
}
function Kaldirildi($Fixture) {
    foreach ($name in $names) {
        Dogrula (-not (Test-Path -LiteralPath (Join-Path $Fixture.Target $name))) "Manifestteki literal dosya kaldırılmalı."
    }
    Dogrula (-not (Test-Path -LiteralPath $Fixture.Manifest)) "Gerçek literal manifest kaldırılmalı."
    YabanciKorundu $Fixture
}
function RetBekle($Fixture, [string]$Expected) {
    $rejected = $false
    try {
        & (Join-Path $SourceRoot "scripts/kaldir.ps1") -Veri $Fixture.Receipt | Out-Null
    } catch {
        Dogrula ($_.Exception.Message.Contains($Expected)) "Beklenen kaldırma reddi yerine başka hata geldi."
        $rejected = $true
    }
    Dogrula $rejected "Olumsuz kaldırma vakası reddedilmeli."
}

try {
    foreach ($item in @(
        @("normal", "bin", "veri", $false),
        @("unicode", "Zee Türkçe Ω", "Kayıt Türkçe Ω", $false),
        @("bracket-sibling", "bin[1]", "veri[1]", $true)
    )) {
        $fixture = Hazirla $item[0] $item[1] $item[2] $item[3]
        & (Join-Path $SourceRoot "scripts/kaldir.ps1") -Veri $fixture.Receipt | Out-Null
        Kaldirildi $fixture
        $cases += [pscustomobject]@{ name = $item[0]; result = "gecti"; installed_remaining = 0; foreign_remaining = $(if ($item[3]) { 2 } else { 0 }); manifest_removed = $true }
    }

    $fixture = Hazirla "changed-hash" "bin[1]" "veri[1]" $true
    [System.IO.File]::WriteAllText((Join-Path $fixture.Target "dil.exe"), "changed fixture")
    RetBekle $fixture "kurulumdan sonra değişmiş"
    KuruluKorundu $fixture $true
    Dogrula ((Ozet $fixture.Manifest) -eq $fixture.ManifestHash) "Hash reddinde gerçek manifest korunmalı."
    YabanciKorundu $fixture
    $cases += [pscustomobject]@{ name = "changed-hash-no-delete-all"; result = "gecti"; installed_remaining = 2; foreign_remaining = 2; manifest_unchanged = $true }

    $fixture = Hazirla "force" "bin[1]" "veri[1]" $true
    [System.IO.File]::WriteAllText((Join-Path $fixture.Target "dil.exe"), "changed fixture")
    & (Join-Path $SourceRoot "scripts/kaldir.ps1") -Veri $fixture.Receipt -Zorla | Out-Null
    Kaldirildi $fixture
    $cases += [pscustomobject]@{ name = "force-literal-only"; result = "gecti"; installed_remaining = 0; foreign_remaining = 2; manifest_removed = $true }

    $fixture = Hazirla "missing-schema" "bin[1]" "veri[1]" $true
    [System.IO.File]::Delete($fixture.Manifest)
    RetBekle $fixture "Kurulum manifesti yok"
    KuruluKorundu $fixture $false
    YabanciKorundu $fixture
    [System.IO.File]::WriteAllText($fixture.Manifest, "# unknown-schema`n", [System.Text.UTF8Encoding]::new($false))
    $badHash = Ozet $fixture.Manifest
    RetBekle $fixture "Kurulum manifesti şeması bilinmiyor"
    KuruluKorundu $fixture $false
    Dogrula ((Ozet $fixture.Manifest) -eq $badHash) "Şema reddinde manifest korunmalı."
    YabanciKorundu $fixture
    $cases += [pscustomobject]@{ name = "missing-and-schema-fail-closed"; result = "gecti"; rejection_checks = 2; installed_remaining = 2; foreign_remaining = 2; manifest_unchanged = $true }
    Dogrula ($cases.Count -eq 6) "Literal kampanya altı vakanın tamamını yürütmeli."
} catch {
    $failure = $_.Exception.Message
    throw
} finally {
    if (Test-Path -LiteralPath $sandbox) { Remove-Item -LiteralPath $sandbox -Recurse -Force }
    $cleaned = -not (Test-Path -LiteralPath $sandbox)
    $proof = [pscustomobject]@{
        schema = "zee-kurulum-literal-yol-1"
        result = $(if ($null -eq $failure -and $cases.Count -eq 6 -and $cleaned) { "gecti" } else { "kaldi" })
        powershell_version = $PSVersionTable.PSVersion.ToString()
        powershell_edition = $PSVersionTable.PSEdition
        platform = [Environment]::OSVersion.Platform.ToString()
        case_count = $cases.Count
        cases = $cases
        owned_cleanup = $cleaned
        synthetic_binaries_executed = $false
        failure = $failure
    }
    $json = $proof | ConvertTo-Json -Depth 6
    if ($Report -ne "") { [System.IO.File]::WriteAllText($Report, $json + "`n", [System.Text.UTF8Encoding]::new($false)) }
    Write-Output $json
    if ($proof.result -ne "gecti" -and $null -eq $failure) { throw "Literal kampanya tamamlanmadı veya geçici alan temizlenmedi." }
}
