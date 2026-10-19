# V1-P1-25 — üç platformda imzalı, tekrar üretilebilir sürüm adayı (19 Ekim 2026)

**K-317 / K-319 · ADR-156.** Açık depodaki `surum-adayi` iş akışı
`69a6d8fe8da23debe232426f82aa75305c81aa5f` kaynağında üç GitHub barındırmalı
platformda geçti (koşu `39228406948`, `workflow_dispatch`, `yayinla` etiket
olmadığı için atlandı). Aynı commit'te `dil` CI'ı (koşu `39225691714`) Linux,
macOS ve Windows'ta tam test, faz matrisi, clippy, wasm ve kanıt kapılarıyla
yeşildir.

Her platformda `scripts/surum-artefakti.sh` aynı commit'i iki bağımsız temiz
klonda derler; `dil`/`dillsp` SHA-256'ları eşit değilse artefakt üretmez.
Artefakt SPDX 3.0.1 SBOM, SLSA v1 provenance ve `zee-surum-imza-v1` Ed25519
imzası (`SHA256SUMS.zee-imza`) taşır. İmza resmî yayıncı anahtarıyladır
([yayinci-anahtari-v1.txt](yayinci-anahtari-v1.txt), kimlik
`sha256:50149cf7aa2d3cfff2e326413d704349ad1cd11c84cf4e31a1e428cec7268456`).

| Platform | `dil` SHA-256 | `dillsp` SHA-256 |
|---|---|---|
| linux-x86_64 | `0054bc7dc79602fe2264424b377cd985e6635002eca54a5b549132642add7153` | `d326931d02df06ffa8739bb64e20657d5f71169f3b54ba453200af078b5116d9` |
| macos-arm64 | `8c641e2eedf629edb7b1cc4b21a654392efb68a85e1688fd1eef741c755c06bd` | `6d9620922cf0ff70428f4b18e74f8b0db1346a9928ebbd64ad6ab77d2c79f622` |
| windows-x86_64 | `7396d50355d529e0e70e4c335114339215d57ab0e7ac210aaf7484e17d270c23` (`dil.exe`) | `5a0bc63d315474fb4b8d51028ab7676045cb6eeb1265c6c74428b3b982fb977a` (`dillsp.exe`) |

Üç artefakt da indirildikten sonra derleyicisiz doğrulandı:
`bash scripts/imza-dogrula.sh <klasör>` her biri için "İmza ve özetler geçerli:
resmî yayıncı" verdi (OpenSSL Ed25519 + `sha256sum -c`).

## Sınırlar

- Platformlar arası ikili eşitliği ve SLSA L2/L3 iddia edilmez; eşitlik her
  platformun kendi iki temiz klonu arasındadır.
- Barındırmalı CI makineleri fiziksel cihaz kabulü değildir: fiziksel
  Windows/minimum OS ve cihaz denemeleri P29'da açıktır.
- İlk gerçek yayın (etiket, GitHub Releases) P30'dadır; bu kayıt yayın değildir.
- Bu commit'te Windows ve wasm hedeflerinde 12 Ekim'den beri biriken
  regresyonlar giderildi (K-319: `zee.ayar` sürücü öneki, `@unix` faz
  seçicileri, Playground kapsam bağımlılığı); ayrıntı `kararlar/gunluk.md`.
