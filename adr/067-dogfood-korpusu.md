# ADR-067 — Yalnız dogfood boşluklarından büyüyen korpus

- **Durum:** kabul
- **Tarih:** 5 Eylül 2026
- **İlgili kayıt:** K-172, K-163, B-073, ADR-044, ADR-059, V1-P1-24

## Bağlam

K-163 dogfood'u on iki sürtünme (F007…F032) buldu; bir kısmı compiler
değişikliği açtı, çoğu mevcut dilde çözüldü. Çözümler ürün deposunda,
reproducer'lar `dogfood/catli-itwise-admin/*-gereksinimi.dil` altında kaldı;
fakat bu kaynakları derleyen ya da çalıştıran test yoktu. Bir grammar değişikliği
onları sessizce bozabilir, "ürün böyle yazdı" kanıtı çürüyebilirdi. Golden
korpus tasarım örneklerini, `regression/` düzeltilmiş bug'ları taşır; gerçek
ürün sürtünmesinden doğan başarı ve başarısızlık örneklerinin yeri yoktu.

## Karar

1. **Manifest.** `dogfood/korpus-v1.tsv` her vakayı ürün slug'ı, kaynak işi
   (`K-NNN` ya da `K-NNN/FNNN`), kip (`denetle|calistir`), beklenti
   (`basarili|basarisiz`), tanı kodu, beklenen çıktı, dosya yolu ve
   sürtünmeyi anlatan notla kaydeder.
2. **Kapsam kuralı.** `dogfood/` altındaki her `.dil` kaynağı (ürün
   `proje.dil` hariç) manifestte tam bir kez bulunur; dosya etkin ürün kökü
   altındadır, kaynak işi ve dogfood dilimi günlükte gerçektir. Korpusa yalnız
   gerçek ürün sürtünmesi girer; tasarım örneği golden'a, düzeltilmiş bug
   `regression/`e gider.
3. **Ürün politikasıyla koşma.** Her vaka ürünün `proje.dil` yetkinlik
   politikasıyla derlenir; `calistir` vakaları hermetik IO ile çalışır ve exact
   çıktı ya da tanı kodu karşılaştırılır. Kaynak kanonik biçimde ve sürtünmeyi
   anlatan başlık yorumuyla başlar.
4. **Çift yönlü büyüme.** Bir sürtünme hem mevcut dilde reddedilen biçimi
   (`basarisiz`) hem ürünün uyguladığı çözümü (`basarili`) taşıyabilir; böylece
   ileride açılacak bir dil değişikliği eski reddi ve çözümü birlikte görür.

## Reddedilen seçenekler

- **Dogfood kaynaklarını golden'a eklemek:** golden dil tasarımının
  öğretici örnekleridir; ürün sürtünmesi ve bilinçli reddedilen biçim orada
  anlam bulanıklığı yaratır.
- **Yalnız ürün deposundaki testlere güvenmek:** ürün deposu dış ve
  hareketlidir; compiler değişikliği onu CI'da göremez.
- **Gereksinim dosyalarını yalnız yapısal kontrol etmek:** derlenmeyen
  reproducer sahte kanıttır.

## Sonuçlar

- İlk taban 10 vaka: dört gereksinim aynası derlenir; F007 zincirli özellik
  S015 ve ara ad çözümü; F012 boş CSV başlığı C015 ve kanonik başlık çözümü;
  F030 eksik kaynak taşıma başarısız `Sonuç`; F032 aralık dışı durum S037.
- Aynı işte K-171 drift listesinden iki güvenlik test boşluğu (GB-020
  X-Zee-CSRF/çoklu Content-Type, GB-021 `__Host-` çerez nitelikleri) gerçek
  adaptör birim testleriyle kapandı.
- Yeni dogfood dilimi korpusa satır ekler; korpus dışı `.dil` CI'ı durdurur.
