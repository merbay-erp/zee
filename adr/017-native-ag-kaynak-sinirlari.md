# ADR-017 — Native ağ I/O kaynak sınırları

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-105/K-127/K-130/K-131, B-025, V1-P0-15/V1-P0-29

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
3. yanıt yazımına da 10 saniyelik socket zaman aşımı uygular (K-372 ekiyle
   mutlak yazım bütçesine dönüştü, aşağıya bakın);
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
- Ortak `KaynakSinirlari` K-129'da, 64 süreç-geneli eşzamanlı bağlantı bütçesi
  K-130'da bağlandı. K-131 ağ timeout/header/body ile yerel HTTP
  süre/header/body değerlerinin sayısal sahipliğini aynı domain görünümlerine
  taşıdı.
  SSRF/hedef politikası ADR-031/spec-23'te; oturum ve LSP bütçeleri kendi
  ADR'lerinde kapanmıştır.
- Loopback, redirect, origin, DNS/IP ve varsayılan deadline testleri yeni
  istemci davranışını korur.

## K-372 eki — mutlak yanıt yazım bütçesi (23 Kasım 2026)

Madde 3'teki 10 saniyelik socket zaman aşımı syscall başınaydı: kısmi
ilerleme süreyi yeniliyordu ve Darwin'de tek bir yazım çağrısı ilerledikçe
süresini aşabiliyordu (10 sn zaman aşımıyla 28 sn süren tek `send` ölçüldü).
Yanıtını okumayan ya da her pencerede biraz okuyan istemci, okuma tarafındaki
mutlak bütçenin karşılığı olmadığından tek işçiyi uzun tutabiliyordu. Yerel
ölçüm (Darwin, tek işçi, 2 MiB yanıt): okumayan bir istemci işçiyi 25 sn,
üç istemci art arda 75 sn kilitledi; ~32 KiB/s okuyan istemci 41,5 sn tuttu;
Linux'ta çağrı başına süre azaldıkça döndüğünden tutma süresi yanıt/okuma
hızıyla sınırsız büyür. Bekleyen istekler 503 almaz, backlog'da gecikir; çok
işçide okumayan istemci sayısı işçi sayısına ulaşınca aynı tablo oluşur.
Kısa yanıtlar (çekirdek tamponuna sığan) etkilenmez.

Karar: yanıt yazımı yazım başlangıcından itibaren mutlak
`istek_okuma_saniyesi (10 sn) + yanıt baytı / yanit_yazim_alt_hizi
(128 KiB/s)` bütçesi taşır; ek olarak 10 sn ilerlemesiz kalan yazım kesilir
(eski davranışın sıkı hâli). Yazım nonblocking yapılır, `WouldBlock`
beklemesi 1–10 ms'lik uyku ile son tarihe kırpılır; socket zaman aşımına
güvenilmez. Aşımda bağlantı kapanır, mevcut "yanıt teslim edilemedi"
metriği artar (yeni metrik alanı yok), günlükte K-327 kopma satırı
`yanıt yazım bütçesi doldu` nedeniyle görünür ve işçi serbest kalır.

Hız seçimi: backend'in karşı ucu her zaman loopback reverse proxy'dir;
dürüst yavaş internet istemcisi ancak proxy yanıtı tamponlamıyorsa ve yanıt
proxy + çekirdek tamponlarını (loopback'te birkaç MiB) aştıysa backend'e
yansır. 128 KiB/s (~1 Mbit/s) ortalama, ilk 10 sn sonrası; 16 MiB metin üst
sınırında en çok 138 sn. Daha yavaş istemciye büyük yanıt veren site proxy
tamponlamasını açmalıdır (nginx varsayılanı açık). Değer
`VARSAYILAN_KAYNAK_SINIRLARI.http().yanit_yazim_alt_hizi()` sahipliğindedir.

Bilinçli sınır: bütçe bağlantı başınadır; tek işçide K bağlantı art arda
işçiyi en çok K × bütçe tutabilir (okuma tarafıyla aynı). Bağlantı başına
süre sınırı slowloris'i bitirmez, kaynak tüketimini sınırlar; internet
sınırındaki koruma proxy tamponlaması ve proxy zaman aşımlarıdır
(`docs/web-production-profili.md`). Kanıt:
`regression/security/yanit-yazim-butcesi.dil` (kip
`native-web-yazim-butcesi`; düzeltme öncesi Darwin'de işçi 34 sn tutuldu,
sonrası 25,5 sn), `web_profili_testleri` yazım birim vakaları, GB-150.
