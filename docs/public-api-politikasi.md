# Rust public API ve SemVer politikası

Zee compiler crate'ini gömen uygulamalar için desteklenen sözleşme yalnız
`dil::api::v1` yoludur. Derleme, denetleme, çalıştırma, test, tanı, biçimleme
ve birim yükleme girişleri burada exact allowlist ile yayımlanır.

Crate kökündeki eski işlevler ve `#[doc(hidden)]` modüller bootstrap CLI,
entegrasyon testleri ve 1.0 öncesi geçiş için erişilebilir kalır; desteklenen
SemVer sözleşmesi değildir. Yeni tüketici bunlara bağlanmamalıdır. K-160
ownership ayrışması bu internal yolları saf model, çözüm, taşıma, doğrulama ve
yayın sahiplerine böldü; facade sözleşmesi değişmedi.

## Değişiklik kuralları

- `api::v1` öğesi silmek, yeniden adlandırmak veya imzasını uyumsuz değiştirmek
  breaking change'dir; V1 freeze sonrasında yalnız yeni major facade'da yapılır.
- Geriye uyumlu yeni işlev minor; düzeltme patch sürümüdür.
- Yeni public export bilinçli allowlist ve test değişikliği olmadan derlenemez.
- Internal modülün Rust görünürlüğü destek sözü değildir; rustdoc'ta gizlidir.
- `api::v2` gerekirse `v1` ile yan yana doğar; göç ve deprecation süresi sürüm
  notlarında ilan edilmeden `v1` kaldırılmaz.

`public_api_testi`, facade smoke kullanımını, exact export listesini ve kökte
`api` dışındaki her public modülün açıkça internal sınıflanmasını korur.
