# ADR-017 — Native ağ I/O kaynak sınırları

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-105, B-025, V1-P0-15

## Bağlam

Native bootstrap HTTP istemcisi yalnız çevreleyen `içinde` bloğu bir süre
verdiğinde socket zaman aşımı kuruyor, süre verilmezse sınırsız bekleyebiliyor
ve yanıtı sınırsız `Vec` içine okuyordu. Yerel HTTP sunucusunun başlık ve gövde
boyutu sınırları vardı; ancak tek bağlantı baytları yavaşça göndererek tek
iş parçacıklı adaptörü süresiz tutabiliyordu.

Boyut sınırı tek başına zaman, zaman aşımı tek başına bellek sınırı değildir.
Native ağ adaptörünün ikisini de varsayılan olarak taşıması gerekir.

## Karar

Native HTTP istemcisi:

1. kaynakta daha erken bir `içinde` son tarihi yoksa 30 saniyelik mutlak
   varsayılan son tarih kurar;
2. DNS sonrasındaki bütün adres denemeleri, istek yazımı ve yanıt okumalarını
   aynı kalan bütçeden besler;
3. her socket okumasından önce kalan mutlak süreyi yeniden hesaplar; aralıklı
   bayt gelişi son tarihi kaydıramaz;
4. başlıklar dahil en çok 8 MiB wire yanıtı belleğe alır.

Native yerel HTTP sunucusu:

1. kabul anından başlayarak başlık ile ilan edilmiş gövdenin tamamını aynı
   10 saniyelik mutlak süre içinde ister;
2. her okumada kalan süreyi yeniden kurar ve aşımda 408 döndürür;
3. yanıt yazımına da 10 saniyelik socket zaman aşımı uygular;
4. var olan 16 KiB başlık ve 64 KiB gövde sınırlarını aynen korur.

Zee TLS gerçeklemeyecektir. Native bootstrap istemcisi bugün yalnız `http://`
destekler; üretim sunucusu TLS'yi loopback HTTPS reverse proxy'de sonlandırır.
İstemci HTTPS backend'i B-024/B-049 altında battle-tested ve kilitli bir
bağımlılık seçimiyle ayrıca kararlaştırılacaktır.

## Platform sınırı

Sistem DNS çözümlemesi her platformda iptal edilebilir değildir ve
`ToSocketAddrs` dönüşüne kadar mutlak bütçeyi aşabilir. Dönüşten sonra süresi
dolmuş istek bağlantı kurmaz. Bu, spec/09'un işletim sistemi çağrıları için
ilan ettiği işbirlikli iptal sınırıdır; sınırsız socket beklemesine izin vermez.

## Sonuçlar

- Deadline yazmayan başlangıç programı da sonsuz ağ beklemesi yapamaz.
- Yavaş gönderici yerel sunucuyu süresiz tek bağlantıda tutamaz.
- Büyük yanıt, bellek tahsisini 8 MiB üzerinde büyütemeden görünür hata olur.
- Ortak `KaynakSinirlari`, eşzamanlı bağlantı bütçesi, SSRF/hedef politikası,
  sınırlı oturum deposu ve LSP girdisi bu ADR ile tamamlanmış sayılmaz.
- Üç yeni loopback testi mutlak son tarihi, yanıt sınırını ve deadline'sız
  istemcinin varsayılan süreyle başarılı küçük yanıtını korur.
