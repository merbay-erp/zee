# RFC-0024 — Merkezî yetkinlik ve outbound ağ güvenliği

- **Durum:** geçici kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıtlar:** K-127, B-023, B-024, B-049, V1-P0-29
- **Gerçekleme:** `yetkinlik.rs`, `cozumleyici/yetkinlik.rs`,
  `yorumlayici/yetkinlik.rs`, `ag_istemcisi.rs`, `proje.rs`, `paket.rs`

## Özet

Zee'de ağ, dosya, sunucu, donanım, web oturumu ve kriptografi erişimi artık
dağınık adaptör tercihleri değildir. Kaynak ihtiyacı derleme öncesinde çıkarılır,
proje sahibinin `proje.dil` bildirimiyle karşılaştırılır ve aynı politika runtime
IO sınırında yeniden uygulanır. Paket kendi bildiriminde üst uygulamanın
vermediği bir yetkinliği veya outbound origin'i isteyemez.

Native outbound istemci elle yazılmış düz HTTP yerine tam origin allowlist'i,
DNS sonrası IP denetimi, kapalı redirect, sabit zaman/bellek bütçesi ve rustls
tabanlı HTTPS kullanır. Public proje ağı varsayılan olarak HTTPS'tir; loopback,
özel ağ ve düz HTTP ayrıca `yerel-ağ` onayı ister. Metadata/link-local hedefleri
yerel izinle bile açılamaz.

## 1. Proje bildirimi

İki isteğe bağlı alan vardır; yoklukları boş listeyle aynıdır:

```zee
yetkinlikler "dosya-okuma", "ağ" listesi olsun
ağ_hedefleri "https://api.example.com" listesi olsun
```

Kararlı yetkinlik yazımları `dosya-okuma`, `dosya-yazma`, `ağ`, `yerel-ağ`,
`ağ-sunucusu`, `donanım`, `web-oturumu`, `kriptografi`dir. Bilinmeyen veya
yinelenen değer P015'tir. `yerel-ağ`, `ağ`ı; `web-oturumu`, `ağ-sunucusu`nu
gerektirir. `ağ` ile boş olmayan `ağ_hedefleri` birlikte bulunur.

Ağ hedefi yol/sorgu değil yalnız tam şema+host+port origin'idir. Varsayılan
80/443 portu kanonik yazımda düşer; şema ve port yetkinliğin parçasıdır.
Düz `http://` hedefi yalnız `yerel-ağ` ayrıca açıkken bildirilebilir.

## 2. Profiller ve iki kapı

- Bir proje klasörü çalıştırılırken yalnız manifestteki izinler açılır ve dosya
  erişimi ana proje köküyle sınırlıdır.
- `--güvenli` çocuk profili yalnız proje-kökü dosya okuma/yazmayı açar; ağ,
  sunucu ve donanım fail-closed kalır.
- Bildirimsiz tek `.dil` geliştirici akışı geriye uyumluluk için dış dünya
  adaptörlerini açar; outbound yine yalnız public HTTPS'tir ve `yerel-ağ`
  kapalıdır.

Checker ana programı, testleri ve çağrılmasa bile bütün işlem/eylem gövdelerini
tarar. Eksik yetkinlik ya da sabit URL'nin allowlist dışı origin'i kesin ifade
aralığında T054'tür. Dinamik URL derlemede origin'e indirgenemese de runtime
kapısını geçemez. IO sarmalayıcısı ve gerçek native adaptör aynı politikayı
ikinci kez uygular; embedding host'u explicit politika seçebilir.

## 3. Outbound güvenlik profili

URL kullanıcı bilgisi, parça, ters bölü, denetim karakteri, bilinmeyen şema,
geçersiz DNS/IP veya port taşıyamaz. Native istek şu zinciri izler:

1. URL exact origin allowlist'ine bağlanır.
2. DNS çözümü istemcinin bağlantıda kullanacağı soket adreslerini üretir.
3. Çözülen adreslerin tamamı politikadan geçmeden bağlantı kurulmaz.
4. Public IP varsayılan kabul; loopback, RFC1918, CGNAT ve IPv6 ULA yalnız
   `yerel-ağ` ile kabul edilir. Link-local/metadata ile IANA public olmayan
   özel-kullanım veya IPv4-geçiş önekleri her profilde reddedilir. Karışık
   public/private DNS cevabı tümden reddedilir.
5. Ortam proxy'si kullanılmaz ve redirect sayısı sıfırdır; ikinci hedefe
   yetki taşınmaz.
6. Global/resolve zaman aşımı varsayılan 30 saniye; response header 64 KiB,
   body 8 MiB toplam zarf içinde sınırlıdır.
7. HTTPS, kilitli `ureq` 3.4.0 + rustls backend'iyle kurulur; Zee TLS
   algoritması veya sertifika doğrulayıcısı yazmaz.

## 4. Dosya ve paket sınırı

Proje dosya yolu göreli olmalı, `..` ile çıkmamalı ve erişim anındaki kanonik
hedef ana kökün altında kalmalıdır. Var olan dosya ya da yazılacak dosyanın
ebeveyni üzerinden sembolik bağ kaçışı reddedilir. Bu bir dil/runtime yetki
sınırıdır; kötü niyetli eşzamanlı yerel süreçlere karşı işletim sistemi
container/sandbox izolasyonu iddiası değildir.

Her yerel paket kendi ihtiyacını bildirir. Paket izinleri ve hedefleri ana
uygulama politikasının alt kümesi değilse grafik P015 ile daha derlemeden
reddedilir. Uygulamanın verdiği yetki süreç içinde ortak üst sınırdır; paket
başına ayrı süreç/tenant izolasyonu bu RFC'nin sözü değildir.

## Dört soru süzgeci

Doğal ✓ (izin proje sahibinin tek bildiriminde) · Deterministik ✓ (kararlı
isim/origin ve fail-closed sıra) · Öğrenilebilir ✓ (boş varsayılan, T054/P015
önerisi) · Savunulabilir ✓ (iki kapı, DNS sonrası IP, HTTPS, limitsiz redirect
yok, paket yükseltmesi yok).
