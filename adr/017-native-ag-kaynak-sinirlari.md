# ADR-017 — Native ağ I/O kaynak sınırları

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-105/K-127, B-025, V1-P0-15/V1-P0-29

## Bağlam

Native bootstrap HTTP istemcisi yalnız çevreleyen `içinde` bloğu bir süre
verdiğinde socket zaman aşımı kuruyor, süre verilmezse sınırsız bekleyebiliyor
ve yanıtı sınırsız `Vec` içine okuyordu. Yerel HTTP sunucusunun başlık ve gövde
boyutu sınırları vardı; ancak tek bağlantı baytları yavaşça göndererek tek
iş parçacıklı adaptörü süresiz tutabiliyordu.

Boyut sınırı tek başına zaman, zaman aşımı tek başına bellek sınırı değildir.
Native ağ adaptörünün ikisini de varsayılan olarak taşıması gerekir.

## Karar

K-105 native HTTP istemcisi için şu kaynak sınırını kurdu; K-127 aynı sözü
battle-tested HTTPS backend'ine taşıdı:

1. kaynakta daha erken bir `içinde` son tarihi yoksa 30 saniyelik mutlak
   varsayılan son tarih kurar;
2. DNS çözümü, bağlantı, yazma ve yanıt okumaları aynı global bütçeye bağlıdır;
3. response header en çok 64 KiB, body 8 MiB toplam zarf içinde okunur;
4. redirect ve ortam proxy'si kapalıdır; DNS sonucu bağlantıdan önce
   RFC-0024 origin/IP politikasından geçer.

Native yerel HTTP sunucusu:

1. kabul anından başlayarak başlık ile ilan edilmiş gövdenin tamamını aynı
   10 saniyelik mutlak süre içinde ister;
2. her okumada kalan süreyi yeniden kurar ve aşımda 408 döndürür;
3. yanıt yazımına da 10 saniyelik socket zaman aşımı uygular;
4. var olan 16 KiB başlık ve 64 KiB gövde sınırlarını aynen korur.

Zee TLS gerçeklemeyecektir. K-127 native outbound için exact sabitlenmiş
`ureq 3.4.0` + rustls backend'ini seçti. Public internet HTTPS'tir; düz HTTP
yalnız explicit `yerel-ağ` ve exact target ile açılır. Üretim inbound sunucusu
TLS'yi loopback HTTPS reverse proxy'de sonlandırmaya devam eder.

## Platform sınırı

Resolver ureq'in global/resolve zaman bütçesini kullanır ve bağlantıda
kullanılacak soket adreslerini aynı zincire verir. Platform DNS çağrısının
işletim sistemi içi kesilebilirliği yine spec/09'un işbirlikli syscall
sınırındadır; dönüşten sonra private/metadata IP policy'yi geçmeden bağlantı
kurulmaz.

## Sonuçlar

- Deadline yazmayan başlangıç programı da sonsuz ağ beklemesi yapamaz.
- Yavaş gönderici yerel sunucuyu süresiz tek bağlantıda tutamaz.
- Büyük yanıt, bellek tahsisini 8 MiB üzerinde büyütemeden görünür hata olur.
- Ortak `KaynakSinirlari` ve eşzamanlı bağlantı bütçesi B-025'te açık kalır.
  SSRF/hedef politikası ADR-031/spec-23'te; oturum ve LSP bütçeleri kendi
  ADR'lerinde kapanmıştır.
- Loopback, redirect, origin, DNS/IP ve varsayılan deadline testleri yeni
  istemci davranışını korur.
