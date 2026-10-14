# K-226 — build kimliği izolasyon kabulü

```bash
python3 scripts/tests/build-kimligi-test.py
```

Test gerçek `compiler/build.rs` ve `compiler/src/surum.rs` dosyalarını,
compiler Cargo sürümünü ve sabit Rust toolchain kimliğini kullanır. Bunları
geçici bir Git deposunun `compiler/` klasöründeki bağımlılıksız Cargo projesine
kopyalar. Kurulu toolchain `rustup which` ile bulunur; toolchain kurulmaz,
Cargo offline çalışır, kullanıcı Cargo ayarları ve ana compiler hedef dizini
kullanılmaz. Üretim kaynaklarına yazılmaz.

İki temiz yerel klon aynı actual fixture commit'inden bağımsız release
ikilileri üretir; kanonik klon yolları `--remap-path-prefix` ile eşlenir.
Kimlik JSON/metin ve ikili SHA-256 değerlerinin aynı olması gerekir. İkinci
gerçek Git commit'i, compiler kodunu değiştirmese de build kimliğini
yeniler; böylece Cargo'nun commit bağımlılığı da sınanır.

Exact SHA ve `SOURCE_DATE_EPOCH` uyuşmazlığı, geliştirme serisinin `stable`
iddiası, bilinmeyen kanal, kısa/büyük harfli SHA ve geçersiz epoch reddedilir.
Git'siz arşivde verilmemiş kaynak SHA/tarihi `unknown` kalır; duvar saati
derleme kimliği yapılmaz. Epoch 0, 2000/2024 artık günü, yıl sonu ve 9999
sınırı gerçek build scriptinden geçer. Kirli compiler kaynağı işaretlenir.

Bu izolasyon kabulü build kimliği mantığının ve küçük fixture ikilisinin
tekrar üretimini kanıtlar. Tam Zee compiler ikilisinin iki temiz klonlu
release/SBOM/provenance/imza kabulü [sürüm betiğiyle](../../tekrar-uretilebilir-surum.md)
temiz exact kaynak checkpoint'inde ayrıca yapılır. Fixture sonucu fiziksel
platform, signed release veya canlı ürün kabulü değildir.
