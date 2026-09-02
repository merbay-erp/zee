# ADR-031 — Merkezî yetkinlik politikası ve native outbound istemci

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **Revizyon:** 2 Eylül 2026 — K-139/ADR-036 inbound web proxy origin'i de
  aynı kanonik `AgHedefi` sahibine bağlandı.
- **İlgili kayıt:** K-127, K-139, B-023, B-024, B-049, B-052, V1-P0-29

## Bağlam

ADR-011 intrinsic kayıtlarının gereken yetkinliği tanımlamasını sağladı;
ancak bu bilgi izin kararı değildi. Çocuk modu ayrı bir `GuvenliIo` davranışı,
gerçek dosya ve ağ adaptörleri ayrı kontroller taşıyordu. Proje/paket bildirimi
hangi dış dünya yetkisini açtığını söylemiyor; native HTTP istemcisi TLS,
origin allowlist'i ve DNS sonrası SSRF sınırı taşımıyordu.

Bu parçalı yapı yeni bir intrinsic'in checker'da görünür olup runtime'da açık
kalmasına, paketin uygulama sahibinden habersiz yetki genişletmesine veya DNS
adının bağlantıda private/metadata IP'ye dönmesine karşı tek değişmez sunmuyordu.

## Karar

1. `YetkinlikPolitikasi`, bütün compile/runtime izinlerinin tek modelidir.
   Kararlı yetkinlik enum'u intrinsic modülünden genel `yetkinlik` modülüne
   taşınır; intrinsic kayıtları onu yeniden dışa aktararak API uyumunu korur.
2. Checker etki analizinden ayrı `cozumleyici/yetkinlik.rs` geçişinde ana/test
   ve bütün işlem gövdelerini tarar. Ayrım, mevcut etki katmanının 650 satır
   bütçesini ve tek sorumluluğunu korur.
3. Proje bildirimi explicit izin+origin üretir; paket politikası ana politikanın
   alt kümesi olmak zorundadır. P015 bildirim/grafik, T054 program kullanımı
   sınırıdır.
4. `PolitikaliIo`, eski `GuvenliIo` adını çocuk profili alias'ı olarak koruyan
   genel runtime enforcement point'tir. Gerçek IO da defense-in-depth kontrolü
   yapar.
5. Native HTTP(S) için elle protokol/TLS gerçeklemek yerine exact sabitlenmiş
   `ureq = 3.4.0`, yalnız rustls özelliğiyle kullanılır. Redirect ve ortam
   proxy'si kapalı; global/resolve timeout, header/body limitleri sabittir.
6. Ureq'in aynı bağlantıya verdiği çözülmüş soket listesi özel resolver'da
   filtrelenir. URL öncesi host kontrolü tek başına yeterli sayılmaz. Bir DNS
   cevabındaki tek yasak adres bütün cevabı reddeder. IANA public olmayan
   özel-kullanım ve IPv4-geçiş önekleri fail-closed sınıftadır.
7. Dosya kökü lexical ve erişim-anı canonical/symlink denetimi taşır. OS
   descriptor sandbox'ı ve paket başına süreç izolasyonu verilmiş söz değildir;
   bu sınır dokümanda açık tutulur.
8. K-139 ile `AgHedefi` yalnız outbound allowlist'in değil, production web
   proxy yapılandırması ile `Host`/`Forwarded host`/`Origin` doğrulamasının da
   tek kanonik origin değeridir. Ayrıştırıcı `yetkinlik/origin.rs` sahibinde
   kalır; CLI ikinci bir güvenlik parser'ı taşımaz.

## Reddedilen seçenekler

- Yalnız `GuvenliIo` kontrolü: normal proje ve embedding yollarını kapsamaz.
- Yalnız kaynakta sabit URL taramak: dinamik URL ve DNS rebinding'i kaçırır.
- Redirect başına yeniden denetim: karmaşık istemci davranışı yerine redirect
  tamamen kapatılarak yetki aktarımı yok edilir.
- Elle HTTPS/TLS: B-024'e ve bootstrap'ın küçük, denetlenebilir kripto yüzeyi
  ilkesine aykırıdır.
- Pakete kendi iznini doğrudan vermek: nihai yetki sahibi uygulama olmalıdır.

## Sonuçlar

B-023 ve B-049 kapanır; B-024 ilkesi gerçek istemciyle uygulanır. B-025'in
native ağ dilimi yeni backend'de de korunur. Yeni yetkinlik; enum yazımı,
manifest doğrulaması, compile çıkarımı, runtime kapısı, olumlu/olumsuz test ve
RFC/spec güncellemesi olmadan eklenemez. Registry taşıma/cache işi B-029'da,
dosya metadata sözleşmesi B-048/K-128/ADR-032'de ayrı kapatılmıştır. Inbound
proxy origin'i ve loopback peer güven sınırı B-052/K-139/ADR-036'da kapanır.
