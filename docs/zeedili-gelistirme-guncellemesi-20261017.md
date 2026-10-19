# Zee geliştirme güncellemesi — 17 Ekim 2026

Son sürüm etiketi **0.7.0**, geliştirme hattı **0.8.0-dev**. Bu sayfa
geliştirme hattındaki yenilikleri özetler; yayımlanmış bir V1 veya her
platformda gerçek ürün kabulü duyurusu değildir.

## Editör ve proje araçları

- LSP, emoji gibi UTF-16'da iki birim tutan karakterlerden sonra tanı,
  tanıma gitme ve yeniden adlandırma konumlarını doğru eşler. Türkçe
  morfoloji ve sembol kimliği korunur. Artımlı HIR/tür denetimi ile tüm
  gerçek istemci kabulü ayrı açık işlerdir.
- `zee.ayar` sürümlü çalışma ayarlarını projede tutar. `dil sürüm --json`
  derlemenin kaynak kimliğini gösterir. Yerel compiler seçici proje pinini
  doğrular; otomatik indirme veya genel sistem kurulumu değildir.
- `dil biçimle --denetle <dosya|proje>` dosyayı değiştirmeden biçim farkını
  bulur. Uygun biçim çıkış 0, fark çıkış 1 verir.
- Site ve panel başlangıçları depodaki `sablonlar/site` veya
  `sablonlar/panel` klasöründen yeni bir hedefe kopyalanır. Ardından
  `dil kilitle`, `dil denetle` ve `dil dene` kullanılır. Yeni bir şablon
  CLI bayrağı yoktur.

## Veri ve yerel uygulamalar

`dil göçür --durum [proje]` göç geçmişini, `dil göçür --kuru [proje]`
bekleyen planı veritabanına yazmadan gösterir. Uygulama için kullanılan
normal göç komutu ayrı kalır.

HTTP POST, kontrollü worker kapanışı, SQL işleri ve ekli SMTP e-posta
yerel araç zincirindedir. SQL COMMIT veya SMTP teslim sonucu belirsizse
otomatik tekrar ya da genel exactly-once garantisi verilmez. PostgreSQL,
sunucu dinleme, native PDF ve SMTP tarayıcı derslerinin yeteneği değildir;
uygun yerel çalışma ortamı ve izinler gerekir.

## Kabul durumu

Web ve veri araçlarının kabul kayıtları kendi kaynak kimlikleriyle
geçerlidir. Beş PostgreSQL test harness'inin temizlik takibi dar kapsamda
kapandı; farklı kaynakların test sayıları bir araya getirilmez. Bu sonuç,
bütün ürünlerin veya canlı e-posta sağlayıcılarının kabul edildiğini
göstermez.

39 maddelik ürünleşme planının **22 maddesi kısmi** kalıyor. Beş anonim tam
gerçek proje, 15 katılımcıyla kullanılabilirlik oturumları, temiz fiziksel
Windows ve desteklenen en eski macOS kabulü bekleniyor. Sözdizimi freeze
takvimi **14 Ekim–11 Kasım 2026** arasında sürüyor.

RC fuzz, Miri ve uzun süreli bellek gözlemleri belirli kaynaklarda geçti;
yeni kaynağa otomatik taşınmaz. Fiziksel platform, lisans ve ilk artefakt
yayını ayrı açık kapılardır. Native/WASM kod üretimi ve self-hosting bu
güncellemenin kapsamında değildir.

## Kaynak ve site kapsamı

Özet, `4f3caeeb7e44436f63777043da52473d02417ee0` geliştirme kaynağındaki
sürüm notları ve kabul planı incelenerek hazırlanmıştır. Belge SHA-256
özetleri `dogfood/zeedili-com/icerik-kaynagi.json` kaydındadır.

Site, ürün sunucusundaki dosyalarıyla eşleşen `site/zeedili-com` dalı üzerinde
güncellenir. Çini/altın tasarım, 16 ders, WASM ikilisi ve runtime izinleri
korunur. Bu içerik kaydı compiler binary'si veya canlı dağıtım kanıtı değildir.
