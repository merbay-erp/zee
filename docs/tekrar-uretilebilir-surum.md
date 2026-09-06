# Tekrar üretilebilir sürüm rehberi

K-168/ADR-068'in işletim sözleşmesi. Betik `scripts/surum-artefakti.sh`,
araç `compiler/src/bin/surum_artefakti.rs`, workflow
`.github/workflows/surum-adayi.yml`.

## Yerel üretim

```bash
bash scripts/surum-artefakti.sh                      # temiz ağaç ister
bash scripts/surum-artefakti.sh --anahtar surum.zee-anahtar --cikti /tmp/surum
```

Betik iki bağımsız `git clone` ağacında aynı commit'i sabit toolchain,
`--remap-path-prefix` ve commit zamanı `SOURCE_DATE_EPOCH` ile derler; `dil` ve
`dillsp` SHA-256'ları eşit değilse hiçbir artefakt yazmaz. Çıktı:

| Dosya | İçerik |
|---|---|
| `dil`, `dillsp` | release ikilileri (ilk klondan) |
| `SHA256SUMS` | `<hex>  <ad>` satırları |
| `sbom.spdx.json` | SPDX 3.0.1 JSON-LD; kök paket, ikili dosyalar, `Cargo.lock` bağımlılıkları |
| `provenance.intoto.json` | in-toto/SLSA v1; commit, rustc, platform, iki klonun özeti, `ikiTemizKlonEsit` |
| `SHA256SUMS.zee-imza` | `zee-surum-imza-v1` Ed25519 imzası (anahtar verildiyse) |
| `GIT_SHA` | üretilen commit |

## Doğrulama

```bash
cd compiler
cargo run --locked --bin surum_artefakti -- dogrula \
  --dosya /tmp/surum/SHA256SUMS --imza /tmp/surum/SHA256SUMS.zee-imza \
  --kimlik sha256:<yayıncı-anahtar-kimliği>
sha256sum -c /tmp/surum/SHA256SUMS        # macOS: shasum -a 256 -c
```

## Anahtar

`dil anahtar üret surum.zee-anahtar` ile üretilen `zee-ed25519-private-v1`
dosyası kullanılır; CI'da `ZEE_SURUM_ANAHTARI` secret'ı bu dosyanın içeriğidir.
Açık anahtar kimliği (`sha256:…`) sürüm notunda ilan edilir.

## Sınır

Eşitlik aynı platform ve toolchain içindir; Linux/macOS/Windows artefaktları
ayrı ayrı çift-klon kanıtı taşır. SLSA L2/L3, hosted builder veya HSM imzası
iddia edilmez.
