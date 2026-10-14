# zeedili.com — Zee'nin resmî sitesi

`zeedili.com`, Zee Türkçe Programlama Dili'nin çalışma alanı, öğrenme, teknik
yüzey, örnek, belge, manifesto, yol haritası, topluluk ve kurulum yüzeylerini
bir araya getiren resmî dil platformudur. HTML ve rotalar Zee ile üretilir;
gerçek derleyici WebAssembly olarak tarayıcıda çalışır. Nginx yalnız TLS,
statik dosya, güvenlik başlığı ve reverse-proxy sınırının sahibidir.

## Kimlik ve tasarım (K-203/F005)

Kurucu, mavi/lacivert paletle kurulan önceki iki tasarımı “yapay zekâ ürünü
olduğu anlaşılıyor” diye reddetti ve yeni ruby-lang.org ile TryRuby'yi örnek
gösterdi: kopya değil, benzeri. Site bu yüzden kendi kimliğinden yeniden
kuruldu:

- **Logo:** zee'nin resmî logosu (antrasit `#2b2f36` + altın `#d9b877→#9c7a3c`).
  `static/brand/zee-mark-koyu.svg` açık zeminde, `zee-mark-acik.svg` koyu
  zeminde kullanılır; favicon, uygulama ikonları ve sosyal kart aynı
  işaretten üretilir. ERP'nin mavi Zee kiti dil sitesinde kullanılmaz.
- **Palet:** kağıt `#f4efe3`, beyaz `#fffdf8`, antrasit mürekkep, logo altını;
  eylem rengi İznik çinisinden turkuaz `#0f7c79`, hata rengi mercan `#c0442d`.
  Tek tema vardır (açık); koyu tema ve tema düğmesi kaldırıldı.
- **Yazı:** Fraunces (başlık), Figtree (metin), JetBrains Mono (kod). Hepsi
  OFL lisanslı, `static/fonts/` altında aynı origin'den sunulur (lisans
  metinleri yanında); `font-src 'self'` CSP'si değişmedi, dış CDN yoktur.
- **Ana sayfa kurgusu (F008):** Ruby'ye özgü kurgu (ortada amblem + ışınlar
  + uçuşan kartlar, üç eşit kod kartı + hap düğme, dev ince kelimeler ve
  portreli konuşma balonları, dalga geçişleri) kopya izlenimi vermemek için
  bırakıldı. Hero solda başlık, sağda Zee'nin yüklem-sonlu yapısını gösteren
  “cümle çözümlemesi” kartıdır; Selçuklu yıldızı kartın köşesinde “zee”
  yazılı mühürdür. Örnekler tek bir “örnek defteri”nde sekmeli; kod ve gerçek
  çıktı yan yana. “Zee'nin dört sözü” çini karolardır; derleyici tanısı kendi
  bölümünde, manifesto koyu desenli şeritte. Bölüm geçişleri ince İznik
  bordürüdür.
- **Türk unsurları:** hero'da sekiz köşeli Selçuklu yıldızı (kurucu
  onayladı); üst çubuğun alt kenarında ince çini şeridi ve etkin menü
  öğesinin altında küçük altın yıldız; bölüm başlıklarında yıldızlı Selçuklu ayracı; kağıt bölümlerde
  yıldız ve baklava deseni; alt bilgide İznik çini
  bordürü (kobalt, turkuaz, mercan); madde işaretlerinde lale; kitle
  bölümünde lale, yıldız ve çini karo simgeleri (`static/brand/`). Örnek ve
  dersler Türk kültüründen beslenir: “Damlaya damlaya göl olur”, plaka
  kodları, Çalıkuşu, Maraş dondurması, çay; “Türkçe harfler” dersi `ı→I`,
  `i→İ` kuralını gösterir.
- **30 dakikada Zee (`/playground/`):** TryRuby'deki gibi solda ders metni,
  sağda editör/çıktı, altta geri–ileri gezinmesi olan 16 derslik akış.
  Dersler `kaynak/dersler.dil` içinde Zee tarafından `<template data-ders>`
  olarak HTML kaçışlı üretilir; girdi isteyen dersler girdisini taşır. Her ders
  tarayıcıdaki gerçek derleyicide beklenen çıktıyı verir (13. ders bilerek
  T002 gösterir). Paylaşım (`#kod=`), indirme, girdi ve tohum korunur;
  `#ders-N` ilgili dersi açar.
- `static/zee-motor.js` WASM yükleme/çalıştırma, yüklem-odaklı sözdizimi
  vurgusu (Zee'de anahtar kelime yoktur, cümlenin son kelimesi yüklemdir) ve
  tanı biçimlendirmesini tek yerde tutar. DOM'a yalnız `textContent` yazar.
- Dil turu, öğren, belgeler ve kurulum sayfaları kenar menülü belge
  düzenindedir; “Çalıştır” taşıyan her örnek gerçek derleyiciden geçer.
- Depo bağlantıları özel ana kaynağın varsayılan dalı `master`'a gider;
  `scripts/dogrula.sh` `blob/main` bağlantısını reddeder.
- Ana CSS/JS ve kimlik varlıkları sürümlü URL (`?v=k190a1`) ile çağrılır.

## Belgeler sitede (`/belge/`)

Depodaki Markdown belgeleri (docs, spec, RFC, ADR, editör) GitHub'a
yönlendirilmez; sitede kendi tasarımıyla okunur. Dönüştürücü de Zee ile
yazıldı (`kaynak/markdown.dil`): başlık, paragraf, madde/sıra listesi, kod
bloğu (Zee kodu renklendirilir), tablo, alıntı, yatay çizgi ve satır içi
`kod`, **kalın**, *eğik*, bağlantı. Bağlantılar katalogdaki belgeye
`/belge/<ad>/#çapa` olarak, diğer dosyalar GitHub'a gider; başlık kimlikleri
GitHub çapalarıyla uyumludur, `javascript:` bağlantıları etkisizdir.

```bash
dogfood/zeedili-com/scripts/belgeleri-uret.sh
```

Betik belgeleri `uretilen/` ara alanına kopyalar, her belge için
`kaynak/belge_uret.dil` üreticisini ayrı koşuda çalıştırır (Zee adım
bütçesi belge başınadır), `/belge/` dizin sayfasını üretir ve bütün iç
bağlantı ile çapaları denetler. Çıktı git dışıdır; `dagit.sh` her dağıtımda
yeniden üretir, Nginx `/belge/<ad>/` adreslerini statik olarak sunar.
204 belge yaklaşık 13 saniyede üretilir; 1.388 iç bağlantının hepsi sağlamdır.

## İçerik haritası

| Yol | Amaç |
|---|---|
| `/` | Dilin sözü, hedef kitlesi, kanıt kültürü ve yaşayan miras hikâyesi |
| `/playground/` | “30 dakikada Zee”: gerçek WASM derleyicili 16 derslik tarayıcı akışı |
| `/dil/` | Tür sistemi, runtime, web/veri ve araç zinciri |
| `/ogren/` | Çocuk, öğrenci ve profesyonel öğrenme yolları |
| `/ornekler/` | Gerçek Zee sözdizimiyle başlangıçtan web'e örnekler |
| `/belgeler/` | Öğretici belge, spec, RFC, ADR ve hata kataloğu |
| `/manifesto/` | Zee'nin kökeni ve değişmez ilkeleri |
| `/yol-haritasi/` | V1'e giden kanıt kapıları ve dürüst sınırlar |
| `/topluluk/` | Katkı, öğretim ve kanıt üretme çağrısı |
| `/indir/` | V1 öncesi kaynak kurulum yolu |
| `/belge/`, `/belge/<ad>/` | Depo belgeleri: Zee ile Markdown’dan üretilmiş statik sayfalar |
| `/canli`, `/hazir` | Worker liveness/readiness uçları |

## Mimari

```text
internet
   │ HTTPS + HTTP/2
   ▼
Nginx / sunucu A
   ├── /assets/*, robots.txt, sitemap.xml, webmanifest
   ├── canonical slash ve HEAD→GET upstream normalizasyonu
   ├── CSP/HSTS/COOP/nosniff/referrer/permissions başlıkları
   └── 127.0.0.1:8126
          ▼
       Zee --web-proxy worker
          └── HTML, rotalar, içerik, SEO metadata
```

Nginx içerik veya uygulama kararı üretmez. Zee worker dış ağa açılmaz;
production profili yalnız doğrulanmış `https://zeedili.com` origin'i ve
loopback proxy bağlantısını kabul eder. JSON-LD inline betiği deploy sırasında
exact SHA-256 CSP hash'ine çevrilir. Playground yalnız aynı origin'den
checksum kaydı doğrulanmış `dil.wasm` yükler. CSP dış scripti kapalı tutar ve
yalnız WebAssembly derlemesi için `wasm-unsafe-eval` açar; kullanıcı kaynağı
sunucuya gönderilmez.

## Çalışma alanı

`/playground/` bir kod görseli veya sunucuya kaynak gönderen sahte konsol
değildir. Compiler'ın C ABI v3 WebAssembly çıktısını doğrudan tarayıcıda
başlatır. Kaynak, girdi ve rastgele tohum aynı sekmede işlenir; çıktı ve kesin
Türkçe tanı yine aynı sekmede gösterilir. Yüzey şunları birlikte taşır:

- altı çalışan örnek, düzenlenebilir `.dil` editörü ve satır/sütun konumu;
- gerçek derle/çalıştır hattı, çıktı ve tanı panelleri;
- program girdileri ve deterministik `zee-io-1` tohumu;
- sıfırlama, kopyalama, `.dil` indirme ve URL hash'iyle yerel paylaşım;
- 8 MiB kaynak, 1 MiB/4.096 satır girdi bütçesi ve ABI bellek sahipliği;
- adım adım öğrenme rehberi ile belge/kurulum geçişleri.

WASM varlığı compiler kaynağından şu kapıyla yeniden üretilir:

```bash
dogfood/zeedili-com/scripts/playground-uret.sh
```

Betik kilitli release derlemesi yapar, `dil.wasm.sha256` kaydını yeniler ve
pointer/boy, UTF-8, sahiplik ile kaynak bütçelerini gerçek ABI üzerinden
doğrular.

## SEO ve erişilebilirlik sözleşmesi

- Her indexlenebilir sayfada farklı `title`, description ve canonical vardır.
- Open Graph/Twitter kartı 1200×630 PNG görsel kullanır.
- `SoftwareApplication` JSON-LD, `tr`/`x-default` hreflang, robots ve sitemap
  birlikte yayımlanır.
- Semantic başlık sırası, içeriğe geç bağlantısı, `aria-current`, canlı arama
  bölgesi, klavye menüsü ve `prefers-reduced-motion` desteği vardır.
- İçerik kullanıcı girdisinden üretilmez; kod örneği yardımcı işlemi yine de
  HTML kaçışını zorunlu tutar.
- Lisans alanı seçilmeden structured data içinde lisans iddiası yayımlanmaz.

## Yerel geliştirme

```bash
cd compiler
cargo build --release --locked --bin dil
cd ..
dogfood/zeedili-com/scripts/yerel-baslat.sh
```

Site `http://127.0.0.1:8125` adresinde açılır. Yerel Node ön kapısı yalnız
Nginx'in statik/proxy davranışını taklit eder; uygulama HTML'i Zee worker'dan
gelir. Toplu doğrulama:

```bash
dogfood/zeedili-com/scripts/dogrula.sh
```

## K-203 saha kanıtı ve açık sınırlar

- 597 satır gerçek Zee kaynağı, beş birim (ana, gorunum, sayfalar, dersler, markdown) ve 15/15 hermetik test vardır.
- K-203/F003: sitedeki 22 çalıştırılabilir örnek tarayıcıdaki gerçek
  derleyicide hatasız koştu; 29 GitHub bağlantısı 200 döndü; on sayfa
  390×844 mobilde yatay taşmasız, açık/koyu temada denetlendi.
- On indexlenebilir rota, iki sağlık rotası ve dokuz canonical slash
  yönlendirmesi gerçek HTTP üzerinden 200/308 ile doğrulandı.
- 1440×900 masaüstü ve 390×844 mobil görünüm, yatay taşma, menü davranışı
  ve açık/koyu tema tarayıcıda denetlendi.
- Resmî sembol/palet, çok boyutlu favicon, Apple touch icon, PWA 192/512
  ikonları ve dile özgü 1200×630 sosyal kart birlikte doğrulandı.
- K-203/F002: ilk teslim güçlü bir kurumsal anlatı sunsa da programlama dili
  platformu için “okumak” yeterli değildi. Mevcut ABI v3 yüzeyi kullanılarak
  compiler değiştirilmeden gerçek tarayıcı editörü, örnek seçici, girdi/tohum,
  çıktı/tanı, indirme ve yerel paylaşım çalışma alanı eklendi.
- K-203/F001: birimler arası dört argümanlı sayfa üretici çağrısı S015 verdi;
  compiler genişletilmedi. SEO açıklaması yol tablosuna alınarak dış çağrı üç
  argümana indirildi. Tek üründeki bu sürtünme yeni grammar gerekçesi değildir.
- Production worker HTML'i `text/html` ve güvenli varsayılan başlıklarla
  üretir. Deneysel localhost profili `text/plain` verdiği için yalnız yerel ön
  kapı geliştirme sırasında içerik türünü uyarlar; production Nginx worker'ın
  içerik türünü korur ve daha sıkı site CSP'sini tekilleştirir.
- Kaynak depo özeldir; lisans seçimi bekler. Kaynak alt bilgisi sınırlı depo
  erişimini açıkça söyler; site yeniden dağıtım lisansı vaat etmez. K-224
  kaynak düzeltmesi yerel kabul içindir; canlı dağıtım yapılmış sayılmaz.
  [Pilot destek politikası](../../docs/urun-destek-politikasi.md), güvenlik ve
  uyumluluk kapılarına bağlı destek kapsamını tanımlar.
- sunucu A worker ve dosya dağıtımı otomatikleştirilmiştir. 7 Ekim 2026'da
  https://zeedili.com public yayına girdi: Cloudflare proxy'li A kayıtları,
  DNS-01 Let's Encrypt sertifikası ve `scripts/canli-dogrula.sh` ile 14 uç
  200, canonical 308, www→apex 301, `application/wasm` ve CSP doğrulandı.

## Dağıtım

```bash
dogfood/zeedili-com/scripts/dagit.sh
```

Betik önce kaynak/test/statik kapılarını çalıştırır, dosyaları `sunucu A` staging
alanına taşır ve idempotent sunucu kurulumunu çağırır. `zeedili.service`
worker'ı `zeedili` sistem kullanıcısıyla ve yalnız `/opt/zeedili/.zee` kalıcı
web durumuna yazabilen sıkı systemd sandbox'ında koşar. Nginx yapılandırması
yalnız sertifika mevcutsa etkinleştirilir. Yerel geliştirme durumunu taşıyan
`.zee` klasörü staging aktarımından açıkça dışlanır; üretim durumu yalnız sunucu A
üzerinde doğar ve korunur. Staging alanında önceki koşudan kalmış dışlanan
durum da her aktarımda silinir.

### Public yayın topolojisi

sunucu A'ün (`[özel sunucu adresi]`) 80/443 portları sağlayıcı katmanında internete
kapalıdır; yalnız Cloudflare kenarı origin'e ulaşır. Bu yüzden:

- `zeedili.com` ve `www.zeedili.com` A kayıtları **Cloudflare proxy'li**
  (turuncu bulut) olmalıdır; “DNS only” kayıtla ne ziyaretçi ne ACME HTTP-01
  origin'e ulaşır. SSL/TLS modu **Full (strict)** önerilir.
- Sertifika HTTP-01 yerine hesabın sunucu A'teki certbot Cloudflare kimliğiyle
  (`/etc/letsencrypt/cloudflare-mustafaerbay.ini`, `ZEE_CF_KIMLIGI` ile
  değiştirilebilir) **DNS-01** ile alınır; otomatik yenileme aynı yoldan olur.
- Gerçek istemci IP'si Nginx'in http düzeyindeki Cloudflare `real_ip`
  yapılandırmasından gelir; worker'a giden `Forwarded for=` ziyaretçidir.
- Betikler DNS'i herkese açık çözücüye (`ZEE_DNS_COZUCU`, varsayılan
  `1.1.1.1`) sorar; kurumsal/yerel çözücüler yeni kaydı negatif önbellekte
  (SOA minimum 1800 sn) tutabilir.

İlk yayın ve sertifika yenileme:

```bash
dogfood/zeedili-com/scripts/yayina-al.sh
```

Bu betik site kapılarını çalıştırır, iki adın çözüldüğünü doğrular, DNS-01
sertifikasını alır, sıkı HTTPS vhost'unu etkinleştirir ve
`scripts/canli-dogrula.sh` ile public uçları Cloudflare üzerinden doğrular.
Sonraki içerik dağıtımları için `scripts/dagit.sh` yeterlidir; o da sonunda
aynı canlı doğrulamayı koşar.

## K-217 taşınabilir kurulum şablonu

Depodaki yollar örnek `/opt/zeedili` ve `/opt/zee/bin/dil` kurulumudur.
Ürün ekibi canlı envantere göre uyarlar; bu değişiklik canlı sunucuya
dağıtılmaz. `ZEE_SUNUCU` SSH bağlantısını; `ZEE_ORIGIN_IP` origin adresini;
`ZEE_CF_KIMLIGI` özel DNS-01 kimlik dosyasını verir. Bu değerler açık
kaynakta saklanmaz. Bash söz dizimi site doğrulamasının parçasıdır.
