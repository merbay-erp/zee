# ADR-156 — Resmî yayın kanalı ve yayıncı anahtarı

- **Durum:** kabul edildi
- **Tarih:** 19 Ekim 2026
- **İlgili:** ADR-068, ADR-071, ADR-102, ADR-154, ADR-155, P24, P30, V1-P1-25

## Bağlam

`surum-artefakti` iki temiz klonda eş SHA-256, SBOM, provenance ve
`zee-surum-imza-v1` Ed25519 imzası üretebiliyordu; ancak güvenilen yayıncı
anahtarı, imzasız yayını reddeden bir kanal ve kullanıcının derleme yapmadan
imzayı doğrulayabileceği bir yol yoktu.

## Karar

- **Yayıncı anahtarı:** kimliği ve açık anahtarı [docs/yayinci-anahtari-v1.txt](../docs/yayinci-anahtari-v1.txt)
  içinde sabittir. Özel anahtar açık deponun `ZEE_SURUM_ANAHTARI` Actions
  sırrındadır; şifreli yedeği proje sahibindedir. Anahtar dönüşü yeni bir ADR ve
  yeni sabit dosyayla yapılır.
- **Kanal:** `v*` etiketi `surum-adayi` iş akışında Linux x86_64, macOS arm64 ve
  Windows x86_64 için GitHub'ın kendi makinelerinde iki temiz klonda derler ve
  imzalar. `yayinla` işi her platformun imzasını sabit anahtara karşı
  `scripts/imza-dogrula.sh` ile doğrular; imzasız ya da doğrulanamayan artefakt
  varsa yayın yapılmaz. Paketler ve `ZEE-PAKETLERI.sha256` GitHub Releases'e konur.
  Elle tetikleme derler ve imzalar, yayınlamaz.
- **Kullanıcı doğrulaması:** `bash scripts/imza-dogrula.sh <klasör>` yalnız
  OpenSSL 3+ ile alan ayrımlı (`zee-surum-v1\0`) Ed25519 imzasını ve ardından
  her dosyanın özetini denetler.

## Sonuçlar

P24'ün "güvenilir imza/yayın" kısmı kanal düzeyinde karşılanır; ilk gerçek
yayın P30 (RC) ile yapılır. Fiziksel platform kabulü (P29) bu kararla kapanmaz.
Kanıt: `compiler/tests/yayin_imzasi_testi.rs`.
