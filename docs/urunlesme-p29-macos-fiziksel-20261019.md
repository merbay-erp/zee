# P29 — macOS fiziksel cihaz kabulü (19 Ekim 2026)

**K-321 · P29 (macOS kısmı).** Proje sahibinin fiziksel Mac'inde, resmî imzalı
sürüm adayıyla ([V1-P1-25](urunlesme-p125-imzali-surum-20261019.md)) kullanıcı
yolu sınandı. Geliştirici araçlarının görünmediği temiz bir oturum kullanıldı:
`env -i`, boş `HOME`/`TMPDIR`, `PATH=/usr/bin:/bin:/usr/sbin:/sbin`.

| Alan | Değer |
|---|---|
| Platform ve mimari | macOS, arm64 (Apple Silicon, fiziksel) |
| İşletim sistemi | macOS 26.6.2 (25G83); sınanan en düşük sürüm iddiası budur |
| Temiz ortam | PATH'te `cargo`, `rustc`, `rustup`, `brew` yok; ayrı boş ev dizini |
| Yürütme | Proje sahibinin cihazında, sahipli oturum |
| Artefakt | `69a6d8fe8da23debe232426f82aa75305c81aa5f` macos-arm64; `dil` `8c641e2e…06bd`, `dillsp` `6d962092…f622` (`surum-adayi` koşu 39228406948) |
| Bağımlılıklar | Yalnız sistem: SystemConfiguration, CoreFoundation, Security, libiconv, libSystem |

Artefakt tarayıcıdan inmiş gibi `com.apple.quarantine` işaretiyle denendi.

| Adım | Komut | Sonuç |
|---|---|---|
| İmza | `bash scripts/imza-dogrula.sh <klasör>` | Geçti — resmî yayıncı `sha256:50149cf7…8456` |
| Kur | `bash scripts/kur.sh --artefakt <klasör> --hedef … --veri …` | Geçti — 2 ikili, manifest |
| Sürüm | `dil sürüm` | `dil 0.8.0-dev (69a6d8fe8da2, 2026-10-19, development)` |
| Program | `dil çalıştır merhaba.dil` (depo kökündeki örnek) | `Dünyaya merhaba`, çıkış 0 |
| LSP | `dillsp` stdio: initialize → didOpen → shutdown/exit | Yetenekler döndü, tanı yayımlandı (A001), çıkış 0 |
| Kaldır | `bash scripts/kaldir.sh --veri …` | 2 dosya ve manifest silindi |

## Denemede bulunan ve düzeltilen iki ürün hatası

1. **Gatekeeper:** Karantinalı, ad-hoc imzalı (Apple noter onaysız) ikili
   macOS'ta ilk çalıştırmada `Killed: 9` ile öldürülüyordu; tarayıcıdan indiren
   her Mac kullanıcısını etkiler. `kur.sh` artık SHA-256 doğrulanmış kopyadan
   yalnız `com.apple.quarantine` işaretini kaldırır. `kurulum_testi` macOS'ta
   karantinalı artefaktla kurup çalıştırır; düzeltme olmadan düşer.
2. **İmza aracı:** macOS'un `/usr/bin/openssl`'i LibreSSL 3.3.6'dır ve Ed25519
   doğrulamaz; betik bunu "imza geçersiz" diye raporluyordu. `imza-dogrula.sh`
   OpenSSL 3+'ı `ZEE_OPENSSL`, PATH ve Homebrew `openssl@3` yollarında arar;
   bulamazsa araç eksikliğini (çıkış 2) açıkça söyler. Homebrew'suz temiz bir
   Mac'te imza doğrulaması için `brew install openssl@3` gerekir; SHA-256
   denetimi `kur.sh` içinde her durumda yapılır.

## Sınırlar

- Apple Developer ID imzası ve noter onayı yoktur (ayrı karar: ücretli hesap).
- Daha eski macOS sürümleri sınanmadı; desteklenen en düşük sürüm iddiası
  26.6.2'dir.
- Fiziksel Windows kabulü açıktır (P29 Windows kısmı).
