# RFC-0001 — Dil Manifestosu ve Tasarım İlkeleri

- **Durum:** kabul
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** tümünün çatısı
- **İlgili golden programlar:** tüm korpus (golden/01–30)

## Özet

Bu RFC, dilin değişmez ilkelerini bağlayıcı hale getirir. Buradaki maddeler
diğer bütün RFC'lerin anayasasıdır: bir öneri bu maddelerden biriyle çelişirse
öneri düzeltilir, ilke değil.

## Motivasyon

Türkçe konuşan bir çocuğun yabancı sözdizimi bariyerine takılmadan algoritmik
düşünceyle tanışması; aynı dilin yıllar sonra onu terk etmeye zorlamaması;
ekosistemin tek bir şirketin kapalı ürünü olmaması. Temel cümle:
**çocukların kullanabileceği bir dil yapacağız; çocuk dili yapmayacağız.**

## İlkeler (bağlayıcı)

1. **Türkçe-first.** Kaynak kodda İngilizce anahtar kelime zorunluluğu yoktur.
   Doğrulama: v0.1 kabul kriteri; korpusun hiçbir programında İngilizce
   anahtar kelime geçmez.
2. **Çeviri dili değil.** Dil, Türkçenin nesne→eylem akışını ve yüklem-sonlu
   yapısını izler. Gerçekleme kanıtı: ayrıştırıcı cümle türünü satırın SON
   kelimesinden belirler (`compiler/src/ayristirici.rs`) — İngilizce dillerin
   "ilk anahtar kelime" yaklaşımının aynadaki karşılığı.
3. **Deterministik.** Aynı geçerli kaynak tek AST ve tek semantik anlam üretir.
   Belirsizlik çözülemiyorsa bu bir derleme hatasıdır (örnek: A002 belirsiz ad).
   Rastgelelik, saat, dosya ve argümanlar yorumlayıcıda IO soyutlamasının
   arkasındadır; test koşuları bit-bit tekrarlanabilir. Gerçek koşuların bütün
   IO protokolü RFC-0022/spec-21 uyarınca sürümlü izlenip dış etkisiz replay
   edilebilir.
4. **AI semantiğin parçası değildir.** Programın anlamını yalnız sözcükleyici,
   ayrıştırıcı, tür denetçisi ve yorumlayıcı/derleyici belirler. Hiçbir aşama
   tahmin yapmaz; ad çözümleme dahi aday üretip TANIMLI adlarla eşleme yapar
   (RFC-0018, `zee-tr-1` morfoloji profili).
5. **Noktalama minimum, belirsizlik sıfır.** Kabul edilen noktalama: çift
   tırnak (metin), virgül (liste/argüman ayracı), `#` (yorum). Süslü parantez,
   noktalı virgül ve sembolik işleçler çekirdek yüzeyde yoktur
   (anti-örnekler A02, A03, A06).
6. **Statik tür güvenliği + yerel tür çıkarımı.** Yeni başlayan tür yazmaz;
   tür adları yalnız açık sınırlarda görünür (yapı alanları, ileride public API).
7. **Güvenli varsayımlar.** Null yerine Seçenek (`var/yok`), beklenen hatalar
   Sonuç (`dene/başarılıysa`), taşma denetimli aritmetik (C002), kaynakların
   kapsamla yönetimi hedefi.
8. **Hata mesajları Türkçe, öğretici ve eyleme dönüktür.** Her tanı: kod +
   açıklama + kaynak konumu + işaret + öneri (`compiler/src/tani.rs`).
   Doğrulama hataları beklenen/bulunan değerleri gösterir (D001).
9. **Çocuk dostudur ama oyuncak değildir.** Progressive disclosure: `"Dünyaya
   merhaba" yaz` tek satırdır; aynı dil yapılar, işlemler, testler taşır.
10. **Bağımsız çalışır.** Bootstrap derleyici sıfır dış bağımlılıdır; cloud,
    LLM ya da tek sağlayıcı gerektirmez; offline çalışır.
11. **Tooling ürünün parçasıdır.** `dil çalıştır/denetle/dene/biçimle` tek
    ikilide; biçimleyici idempotenttir ve tokenları değiştiremez (C011 güvencesi).
12. **Çekirdek küçük ve kararlı; kütüphane katmanı hızlı evrilir.**
13. **Self-hosting** uzun vadeli bağımsızlık hedefidir.

## Kontrollü Türkçe sınırı

v1 serbest doğal dil DEĞİLDİR. Desteklenen kalıplar ve ek biçimleri sürümlü
grammar'da sayılıdır; genişleme sezgiyle değil RFC ile olur (anti-örnek A04).

## Dört soru süzgeci (her özellik için zorunlu)

1. Türkçe doğal mı? 2. Deterministik mi? 3. Öğrenilebilir mi?
4. Profesyonel ölçekte savunulabilir mi? — Biri hayırsa özellik yeniden tasarlanır.

## Korpus etkisi

Yok — bu RFC mevcut korpusu tanımlar, değiştirmez.
