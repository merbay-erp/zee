# `.zep` v1 conformance ve saldırı korpusu

Bu rehber, zee kaynak paketinin farklı işletim sistemlerinde aynı byte'ı
üretme ve düşmanca yol girdilerini fail-closed reddetme kanıtını açıklar.
Normatif sözleşme [spec/18](../spec/18-paket-yayini.md), gerekçe RFC-0020 ve
ADR-028'dedir.

## Kanonik üretim hattı

1. Yalnız gerçek, sembolik bağ olmayan UTF-8 `.dil` dosyaları toplanır.
2. Her dosya sistemi yol bileşeni Unicode 17.0 UAX #15 NFC'ye çevrilir.
3. NFC sonrası aynı göreli yola inen iki girdi reddedilir.
4. Bileşenler yalnız ASCII `/` ile birleştirilir ve güvenlik profili uygulanır.
5. Yollar UTF-8 byte sırasıyla sıralanır; zaman, sahiplik, izin ve platform
   metadata'sı arşive girmez.
6. Tüketici gelen yolu yeniden yazmaz: zaten NFC değilse paketi reddeder.

Bu üretici/tüketici ayrımı önemlidir. Yerel dosya sisteminin NFD sunması
kanonik paketi değiştirmez; fakat ağdan gelen iki farklı byte gösterimi aynı
mantıksal yol sayılmaz.

## Tier-1 sabit fixture

`compiler/tests/fixtures/zep-kanonik-v1.hex`; `kaynak/çağrı.dil` içeren gerçek
bir proje ağacının tam `.zep` byte dizisidir. `tedarik_testi` bu ağacı yeniden
paketleyip fixture ile byte-byte karşılaştırır. Aynı test CI matrisindeki
Ubuntu, macOS ve Windows üzerinde koştuğu için yalnız yerel bir “iki kez aynı
çıktı” testi değildir.

Fixture yalnız bilinçli `.zep` wire-format değişikliğinde güncellenir. Önce
RFC-0020, ADR-028 ve spec/18 revize edilir; eski/yeni paketin uyumluluk etkisi
yazılır; ardından üç platformun tamamı yeşil görülür. Sırf test geçsin diye
beklenen hex'i yenilemek yasaktır.

## Kalıcı saldırı korpusu

`compiler/tests/fixtures/zep-saldiri-korpusu/yollar-v1.tsv` 80 vaka taşır:

- üst dizin, mutlak yol, ters bölü, boş/`.` bileşen, NUL ve denetim karakteri;
- NFD Türkçe yol;
- Unicode 17.0 UTS #39 `/`, `\\`, `.`, `:` benzerleri ve tam genişlikli
  eşleri;
- sıfır genişlikli ve çift yönlü metin biçim denetleyicileri.

Korpus kod noktalarını hex yazar; böylece görünmez saldırı karakterleri belge
incelemesini yanıltmaz. Test veri satırı sayısını da `80`e sabitler. Uzunluk,
dosya sayısı, yinelenen/sırasız yol, beklenmedik son ve UTF-8 gövde saldırıları
ayrı yapısal testlerde üretilir.

## Yerel doğrulama

```bash
cd compiler
cargo test --locked --test tedarik_testi
cargo test --locked --lib tedarik::testler
cargo clippy --all-targets -- -D warnings
cargo rustc --release --target wasm32-unknown-unknown --lib --crate-type cdylib -- -C link-arg=-zstack-size=16777216
```

Yeni Unicode sürümüne geçerken `unicode-normalization` veri sürümü, UTS #39
reddetme kümesi, korpus ve fixture aynı değişiklikte incelenir. Bu yükseltme
sıradan dependency bakımı değil, paket uyumluluk değişikliğidir.
