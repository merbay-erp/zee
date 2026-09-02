# ADR-041 — Kritik işlev boyutu ve karmaşıklık eğilim kapısı

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-144, B-035

## Bağlam

Derleyici fazlarının dosya düzeyinde fiziksel sınırları vardır; buna rağmen tek
bir işlev, modül bütçesi içinde büyüyerek karar yoğunluğunu ve gözden geçirme
maliyetini artırabilir. Bütün işlevlere aynı mutlak satır ya da karmaşıklık
sayısını dayatmak da doğru değildir: bir AST/HIR ziyaretçisi ile küçük bir
adaptörün doğal şekli aynı değildir. Yalnız anlık üst sınır, borcun küçük
artışlarla birikmesini veya eşik altına saklanmasını da göstermez.

## Karar

1. Sabit Rust/Clippy araç zinciri, üretim `lib` ve `dil` ikilisindeki işlevleri
   `too_many_lines` ve `cognitive_complexity` ölçüleriyle tarar. Ölçüm eşikleri
   sıfıra çekilir; `--force-warn`, kaynak içi `allow` ile kapının atlanmasını
   önler. Test ve bakım ikilileri ürün tabanına girmez.
2. En az 80 satır veya bilişsel karmaşıklığı en az 12 olan işlev, gözden
   geçirilmiş sürümlü TSV tabanında izlenir. Bu değerler “iyi/kötü işlev” hükmü
   değil, yeni bir kritik işlevin sessizce doğmasını önleyen izleme girişidir.
3. Satır büyüme payı tabanın yüzde 10'u olup +8 ile +24 arasında; karmaşıklık
   payı yüzde 20 olup +2 ile +5 arasındadır. Pay aşımı otomatik refactor kararı
   vermez; CI'ı durdurur ve aynı incelemede işlevi bölmeyi veya gerekçeli yeni
   tabanı kabul etmeyi zorunlu kılar.
4. Taban küçülmeyi kendiliğinden kabul etmez. Böylece önce küçülüp sonra küçük
   adımlarla eski borca dönmek trendi sıfırlamaz. Silinen/yeniden adlandırılan,
   ölçülemeyen veya yeni kritikleşen işlev de açık inceleme ister.
5. Güncel taban→ölçüm farkı `docs/islev-egilimi.md` içinde deterministik üretilir.
   `--denetle` hem büyüme kuralını hem rapor tazeliğini fail-closed doğrular;
   Ubuntu CI kapıyı her push ve pull request'te çalıştırır.

## Reddedilen seçenekler

- **Her işlev için tek mutlak satır sınırı:** işlevin rolünü yok sayar ve
  mekanik bölmeyi iyi tasarım sanabilir.
- **Yalnız Clippy varsayılan uyarısı:** mevcut büyük işlevleri listeler ama
  incelenmiş başlangıca göre yönü, küçük birikimi ve yeni kritik işlevi tutmaz.
- **Yalnız dosya satır bütçesi:** sorumluluk başka işlevlerden tek mega işleve
  taşındığında toplam aynı kalabilir.
- **Raporu CI dışında üretmek:** ölçüm ve Markdown'ın koddan sonra bayat
  kalmasına izin verir.
- **İyileşmede tabanı otomatik düşürmek:** geçici küçülmeyi yeni borç alanına
  dönüştürür; yeni taban ancak bilinçli incelemeyle yazılır.

## Sonuçlar

- İlk K-144 incelemesi 48 üretim işlevini tabana aldı. En büyük gövdeler ve
  karar yoğunluğu artık dosya/satır yerine kararlı `yol::ad#sıra` kimliğiyle
  görünürdür; aynı adlı trait yöntemleri dosya içi sıra ile ayrılır.
- `islev_egilimi --rapor-yaz` yalnız güncel raporu yeniler.
  `--taban-yaz KAYIT` bilinçli inceleme kaydını ve yeni tabanı birlikte yazar;
  değişiklik normal kod incelemesinde görünür kalır.
- Bu bakım kapısı dil semantiğini, kullanıcı tanılarını veya normatif spec'i
  değiştirmez. Clippy metriğinin anlamı araç zinciri yükseltmesinde değişebilir;
  dolayısıyla Rust sürümü yükseltilirken taban ayrıca gözden geçirilmelidir.
- K-145/ADR-042 bütün kaynakları sabit `rustfmt` çıktısına taşıdığı için satır
  ölçüsü semantik değişiklik olmadan yeniden tabanlandı. İnceleme kaydı
  `K-145/ADR-041 kanonik rustfmt tabanı`, güncel kapsam 49 işlevdir.
