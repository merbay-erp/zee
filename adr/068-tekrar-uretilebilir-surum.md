# ADR-068 — Tekrar üretilebilir sürüm artefaktı

- **Durum:** kabul
- **Tarih:** 5 Eylül 2026
- **İlgili kayıt:** K-168, ADR-006/038/048/066, V1-P1-25

## Bağlam

Zee paket yayını (`.zep`) K-094'ten beri byte-byte tekrar üretilebilir, imzalı
ve SBOM/provenance'lıdır; fakat derleyicinin kendisi için böyle bir söz yoktu.
`cargo build --release` çıktısının hangi commit'ten, hangi toolchain'le ve
hangi bağımlılıklarla üretildiği makine-okunur değildi; iki ayrı ortamın aynı
ikiliyi üretip üretmediği hiç ölçülmemişti. Üçüncü dış inceleme K-168 ile iki
temiz ortamda eş hash, SBOM, imza ve provenance istedi.

## Karar

1. **İki temiz klon kuralı.** `scripts/surum-artefakti.sh` aynı commit'i iki
   bağımsız `git clone` ağacına alır; her ikisini sabit `rust-toolchain.toml`
   ile, `--remap-path-prefix` (klon→`/zee`, Cargo home→`/cargo`,
   sysroot→`/rust`) ve commit zamanı `SOURCE_DATE_EPOCH` altında
   `cargo build --locked --release --bin dil --bin dillsp` ile derler. İki
   klonun SHA-256'sı eşit değilse artefakt üretilmez.
2. **Artefakt kümesi.** Çıktı klasörü `dil`, `dillsp`, `SHA256SUMS`,
   `sbom.spdx.json` (SPDX 3.0.1: kök `dil` paketi, her ikili `software_File`,
   `Cargo.lock`taki her bağımlılık `dependsOn` ilişkisiyle), 
   `provenance.intoto.json` (in-toto Statement v1 / SLSA provenance v1; commit,
   rustc, platform, `SOURCE_DATE_EPOCH`, iki klonun `SHA256SUMS` özeti ve
   `ikiTemizKlonEsit` alanı) ve `GIT_SHA` taşır.
3. **İmza.** Anahtar verildiğinde `surum_artefakti imzala`, `SHA256SUMS`
   üzerinde `zee-surum-imza-v1` belgesi üretir: `"zee-surum-v1\0" || dosya`
   girdisinin Ed25519 imzası, açık anahtar ve `sha256:` kimliği.
   `dogrula` dosya özetini, kimlik↔açık anahtar bağını ve imzayı birlikte
   doğrular; beklenen kimlik verilirse farklı anahtarı reddeder. Anahtar biçimi
   paket yayınının `zee-ed25519-private-v1` dosyasıdır.
4. **CI.** `surum-adayi` workflow'u `v*` etiketinde ve elle koşar: önce
   birleşik güvenlik kapısı (sürekli kip), sonra Ubuntu'da iki klon derlemesi;
   `ZEE_SURUM_ANAHTARI` secret'ı varsa imzalar. Artefakt 90 gün saklanır.
   Platformlar arası (Linux/macOS/Windows) eşitlik iddia edilmez; her platform
   kendi çift-klon kanıtını üretir.
5. **Determinizm sözü.** SBOM ve provenance aynı girdide byte-byte aynıdır;
   zaman yalnız commit zamanından türetilir. Üretici ikili bakım aracıdır ve
   `dil` diline dokunmaz.

## Reddedilen seçenekler

- **Tek derlemenin hash'ini yayımlamak:** tekrar üretilebilirlik iddiası
  ölçülmeden verilmiş olur.
- **Harici SBOM/imza aracı (`cargo-sbom`, `cosign`) kurmak:** tedarik yüzeyini
  büyütür; mevcut serde/sha2/ed25519 bağımlılıkları ve `.zep` zincirinin
  şeması yeter.
- **İmzayı CLI'a `dil imzala` olarak eklemek:** CORE FREEZE altında yeni
  kullanıcı komutu açar; sürüm imzalama bakım ikilisinin işidir.

## Sonuçlar

- İlk yerel kanıt (macOS arm64, rustc 1.93.1): iki temiz klonda `dil` ve
  `dillsp` eş SHA-256 verdi; ADR ve günlük exact özetleri taşır.
- Sürüm adayı akışı: `guvenlik-kapisi.sh --surum-adayi` → `v*` etiketi →
  `surum-adayi` workflow artefaktı → `SHA256SUMS` ve imza sürüm notuna
  eklenir. K-169 kurulum/kaldırma tatbikatı bu artefaktı tüketir.
- Cross-platform eşitlik, SLSA L2/L3 iddiası ve managed imza servisi vaat
  edilmez; bunlar açık sınırdır.
