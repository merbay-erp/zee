# ADR-081 — Resmî site ve üretim sınırı

- **Durum:** kabul
- **Tarih:** 5 Ekim 2026
- **İlgili kayıt:** K-203, GB-049
- **Ürün:** `zeedili-com`

## Bağlam

Zee'nin resmî sitesi dilin anlamını, çalışan kapasitesini ve V1 öncesi açık
sınırlarını tek yüzeyde anlatmalıdır. Siteyi başka bir uygulama diliyle üretmek
ilk resmî ürün yüzeyinin kendi iddiasını zayıflatır; native worker'a TLS,
sertifika ve statik varlık sorumluluğu yüklemek ise mevcut production web
profilinin güven sınırını gereksiz büyütür.

Deneysel localhost profili HTML gövdesini `text/plain`, production proxy
profili ise `text/html` ve güvenli varsayılan başlıklarla sunar. Birimler arası
dört argümanlı sayfa yardımcı çağrısı K-203/F001'de S015 üretmiştir. Bunlar
site uğruna core freeze'i açmak için yeterli kanıt değildir.

## Karar

1. Rotalar, HTML, sayfa içeriği, canonical/OG/Twitter metadata ve JSON-LD Zee
   kaynaklarında üretilir.
2. Nginx yalnız TLS, HTTP/2, statik varlık, canonical slash/HEAD
   normalizasyonu, sıkı site güvenlik başlığı ve loopback reverse proxy
   sahibidir. Production worker'ın `text/html` başlığı korunur.
3. Zee production worker yalnız `--web-proxy https://zeedili.com` ile
   `127.0.0.1:8126` üzerinde çalışır; doğrudan internet listener'ı değildir.
   Yerel `.zee` çalışma durumu dağıtım staging alanına aktarılmaz; üretim
   durumu yalnız `/opt/zeedili/.zee` altında sunucu A'te oluşur.
4. CSP dışarıdan script/style kaynağı açmaz. Inline JSON-LD exact içerikten
   deploy anında SHA-256 hash'lenir; diğer JavaScript ayrı dosyadır.
5. SEO gerçeği sayfa testi + gerçek HTTP smoke + sitemap/robots/XML/JS
   doğrulamasıyla birlikte kapatılır. V1 olmayan kapasite tamamlanmış gibi
   sunulmaz; lisans seçilmeden structured data lisans iddiası taşımaz.
6. F001 mevcut dille üç argümanlı adaptör ve yol→description tablosuyla
   çözülür. İkinci bağımsız üründe tekrarlamadan compiler işi açılmaz.
7. Kurucunun sağladığı Zee kurumsal kimlik v2 paketinden master standalone
   sembol, `#1478FF/#00A8FF/#0B172A` paleti, Inter yönü ve aynı masterdan
   türetilmiş ikon ailesi kullanılır. `Zee ERP`
   yazı kilidi ile ERP sloganı dil ürününe taşınmaz; dil sitesinin yazı kilidi
   “zee / Türkçe programlama dili” olarak ayrı tutulur.
8. Ana sayfanın görsel dili stok veya üretken medya kullanmaz. Gerçek Zee kodu,
   derleme hattı, Türkçe morfoloji, öğrenme ölçeği ve kanıt mimarisi tam ekran
   HTML/CSS sahneleri olarak sunulur; görünüm ürün gerçeğinden türetilir.
9. Resmî platformun `/playground/` rotası compiler'ın var olan WASM C ABI v3
   yüzeyini aynı origin'den yükler. Kullanıcı kaynağı, girdileri ve tohumu
   sunucuya gönderilmez. Binary SHA-256 kaydı ve gerçek ABI denetimi dağıtım
   kapısındadır; CSP yalnız bu WebAssembly derlemesi için `wasm-unsafe-eval`
   açar, dış script ve bağlantı açmaz.

## Sonuç

Resmî site gerçekten Zee ile çalışan bir üründür; Nginx uygulama mantığının
sahibi değildir. TLS/statik/header işleri kanıtlanmış reverse-proxy sınırında
kalır. Domain DNS'i ve sertifika dış durumdur: worker sunucu A'e kurulabilir ama
public HTTPS ancak bu iki kanıt mevcutken canlı sayılır.

Kurumsal kimlik tek bir üst marka ailesi kurar fakat ürün sınırını silmez.
Resmî sembol ve renkler paylaşılır; ERP ile dilin adlandırma ve sloganları
ayrıdır. Sosyal kart da ERP kartını yeniden kullanmak yerine aynı kimlikle
dilin kendi vaadini taşır.

Yerel Node ön kapısı production bileşeni değildir; yalnız aynı statik/proxy
sınırını geliştirme ortamında taklit eder. Paket registry araması, kullanıcı
hesabı, sunucuda kod saklama/çalıştırma ve içerik yönetimi bu kararın kapsamında
değildir. Playground yerel ve hermetiktir; ağ, dosya ve production web
yetkinliği vermez.

## Revizyon — 7 Ekim 2026 (K-203/F003)

Kurucu, tam ekran editoryal/kurumsal ana sayfayı bir programlama dili sitesi
olarak kabul etmedi. Karar 8'in görsel dili şu biçimde değişti; karar 1–7 ve
9 (Zee/Nginx sınırı, CSP, kimlik, WASM kapısı) aynen geçerlidir:

- Site kod-öncelikli ve belge düzenli bir dil platformudur: açık zemin
  varsayılan, sistem tercihini izleyen ve kalıcı seçilebilen koyu tema, sistem
  yazı tipleri, dış font/CDN yok.
- Ana sayfanın merkezi dekoratif sahne değil, aynı WASM C ABI v3 ikilisiyle
  çalışan beş örnekli deneme kutusudur; derleyici ilk etkileşimde tembel
  yüklenir. Tanı bölümü gerçek `dil çalıştır` T002 çıktısını, araç zinciri
  bölümü gerçek `dil` alt komutlarını gösterir.
- Dil turu, öğrenme, belgeler ve kurulum sayfaları kenar menülü belge
  düzenindedir. Sitedeki her “Çalıştır” örneği gerçek derleyiciden geçer ve
  çalışma alanına URL parçasıyla taşınır; girdi isteyen ya da ağ isteyen
  örnek bu düğmeyi taşımaz.
- WASM yükleme/çalıştırma, yüklem-odaklı sözdizimi vurgusu ve tanı
  biçimlendirmesi tek `zee-motor.js` dosyasındadır; DOM'a yalnız `textContent`
  ile yazar, kaynak hiçbir koşulda HTML olarak yorumlanmaz.
- Depo bağlantıları public deponun varsayılan dalı `master`'a gider; önceki
  `blob/main` bağlantıları 404 veriyordu ve site doğrulama kapısı bunu artık
  reddeder.

- Yayın topolojisi: sunucu A'ün 80/443 portları yalnız Cloudflare kenarına
  açıktır. A kayıtları Cloudflare proxy'li tutulur, sertifika DNS-01 ile
  alınır ve yenilenir; HTTP-01 webroot akışı ve geçici ACME vhost'u
  kaldırıldı. Public uçlar `scripts/canli-dogrula.sh` ile doğrulanır. Site
  7 Ekim 2026'da https://zeedili.com adresinde yayına girdi.

JSON-LD içeriği değişmediği için deploy anındaki CSP hash'i aynıdır.

## Revizyon 2 — 12 Ekim 2026 (K-203/F005)

Kurucu mavi/lacivert paleti “yapay zekâ ürünü olduğu anlaşılıyor” diye
reddetti ve yeni ruby-lang.org ile TryRuby'yi kopyalanmadan örnek alınacak
yön olarak gösterdi. Karar 7 (kimlik) ve karar 8'in görsel dili değişti:

- Dil sitesi ERP'nin mavi Zee kitini değil, zee'nin resmî antrasit + altın
  logosunu kullanır. Favicon, uygulama ikonları ve sosyal kart bu işaretten
  türetilir. Palet kağıt/antrasit/altın; eylem için İznik turkuazı, hata için
  mercan. Tek açık tema vardır.
- Yazı tipleri (Fraunces, Figtree, JetBrains Mono) OFL lisanslı olarak aynı
  origin'den sunulur; mevcut `font-src 'self'` CSP'si yeterlidir, dış kaynak
  açılmaz.
- Ana sayfanın merkezi Selçuklu yıldızı içindeki logo ve kod kartlarıdır;
  `/playground/` TryRuby benzeri 15 derslik “30 dakikada Zee” akışına
  dönüştü. Ders içerikleri Zee birimi `kaynak/dersler.dil` içinde üretilir;
  her ders tarayıcıdaki gerçek derleyicide doğrulanır.

Karar 1–6 ve 9 (Zee/Nginx sınırı, CSP, WASM kapısı, yayın topolojisi) aynen
geçerlidir.

## Revizyon 3 — 13 Ekim 2026 (K-203/F006)

Kurucu, hero'daki sekiz köşeli Selçuklu yıldızını onayladı ve sitede bunun
gibi Türk unsurlarının bulunmasını istedi. Görsel dil; Selçuklu ayracı,
Kubadabad yıldız ve baklava deseni, İznik çini bordürü, lale ve çini karo
motifleriyle genişledi. Motifler depo içinde çizilmiş SVG'lerdir, aynı
origin'den sunulur; dış görsel ya da üretken görsel kullanılmaz. Örnek ve
derslerin verisi Türk kültüründen seçilir; her örnek gerçek derleyiciden
geçmeye devam eder.

## Revizyon 4 — 13 Ekim 2026 (K-203/F007)

Kurucu, belge bağlantılarının GitHub'a gitmesini istemedi; belgeler sitede
olmalı. Depo Markdown belgeleri dağıtım öncesi Zee ile yazılmış
dönüştürücüyle (`kaynak/markdown.dil`, `kaynak/belge_uret.dil`) site
sayfalarına çevrilir ve Nginx tarafından `/belge/<ad>/` altında statik
sunulur. HTML üretimi Zee'de kalır; Nginx yalnız dosya sunar. Üretilen
sayfalar aynı JSON-LD'yi taşıdığı için CSP hash'i değişmez. Üretim çıktısı
git dışıdır ve her dağıtımda yeniden üretilip iç bağlantıları denetlenir.

## Revizyon 5 — 13 Ekim 2026 (K-203/F008)

Kurucu mevcut görünümü beğendiğini, ancak ana sayfanın Ruby sitesine fazla
benzediğini ve kopya muamelesi görmek istemediğini söyledi. Palet, yazı
tipleri, Selçuklu yıldızı ve Türk motifleri korunarak Ruby'ye özgü kurgu
bırakıldı: hero “cümle çözümlemesi” kartı ve yıldız mühürle, örnekler sekmeli
örnek defteriyle, “Neden” bölümü çini karolarla, dalga geçişleri İznik
bordürüyle, ders sayfasının alt gezinmesi yıldızlı ders yoluyla değiştirildi. Sosyal
kart aynı sembol/paletle yeni görsel dile göre yeniden üretildi.
