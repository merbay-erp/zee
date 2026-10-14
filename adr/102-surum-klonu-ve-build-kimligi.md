# ADR-102: Sürüm klonu ve derlenmiş build kimliği

- **Durum:** kabul
- **İş:** K-241 / GB-064
- **Tarih:** 2026-10-14
- İlişkili: [ADR-068](068-tekrar-uretilebilir-surum.md), [ADR-093](093-deterministik-build-kimligi.md)

## Sorun

Kanonik sürüm betiği Cargo logunu klon kökündeki `build.log` dosyasına
yönlendiriyordu. Kabuk bu dosyayı derlemeden önce yaratınca build.rs doğru
olarak kirli checkout gördü. İki aynı kirli ikili eş hash ürettiği halde
`dil sürüm --json` içinde `dirty: true` vardı. İlk/son caller temizliği ve
hash eşliği bu üretim kusurunu yakalamıyordu. Son a2b8557 ikilisini doğrudan
çalıştırma bunu doğruladı; eski hash kanıtı temiz derlenmiş kimlik kabulü
sayılmaz.

## Karar

Derleme logları ve native ortam raporları iki kaynak klonunun dışında,
üreticinin geçici kanıt dizininde tutulur. Klonun tamamı derleme öncesi ve
sonrası izlenen/izlenmeyen dosyalarla temiz olmalıdır. Her klonun gerçekten
derlediği `dil sürüm --json`, exact kaynak SHA'sı ve `dirty: false` için
denetlenir. Yanlış SHA, kirli kimlik, bozuk JSON veya sonradan değişmiş
checkout, eş hash olsa bile çıktı kümesi oluşturulmadan reddedilir.

Build.rs'nin tüm checkout'u inceleyen kirli-ağaç davranışı daraltılmaz;
ortamdan temiz kimlik dayatılmaz ve log dosyasına geniş ignore istisnası
eklenmez. Toolchain, remap, minimum native hedef, SBOM/provenance, imza ve
platform sözleşmeleri aynı kalır. `--kirli-kabul` yine yalnız caller'ın
HEAD'ini üretir; kaynak klonuna kirli kabul hakkı vermez.

## Kanıt ve sınır

`compiler/tests/tedarik_kapisi_testi.rs` mevcut gerçek shell koşucusunda
temiz kimlik, dirty JSON, yanlış SHA, bozuk JSON ve üretim sırasında kaynak
kirletme vakalarını yürütür. Final exact temiz kaynakta kanonik iki-klon
üretimi ayrıca gerçek ikililerle yinelenir. Önceki a2b8557 source ZIP'i ve
native testi tarihsel geçerliliğini korur; eski ikili hash'leri yeni temiz
kimlik kabulüne çevrilmez. Gerçek yayıncı anahtarı, fiziksel platform,
ürün, uzak CI ve RC/V1 kapıları bu dar araç düzeltmesinden kapanmaz.

Son [gerçek kanonik iki-klon kanıtı](../docs/incelemeler/urunlesme-20261014/final-2e970e4/build-proof.json)
`2e970e47d59e2408eff836cdacef9bd0fe0b3028` kaynağında a/b/çıktı
ikilisinin exact SHA ve `dirty: false` kimliğini, eş hash'leri ve native
hedefi doğrular. Kanıt JSON'undaki Rust dosya checksum'u için tarayıcı
istisnası açılmadı; alanlar `path` ve `sha256` olarak ayrıldı. Özgün eski
koşular korunur ve yeni SHA'ya aktarılmaz.
