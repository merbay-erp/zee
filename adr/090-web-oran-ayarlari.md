# ADR-090 — Bellek oran ayarı ve korunmuş tahliye (K-219)

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıtlar:** RFC-0032, ADR-089, GB-052
- **Ürün:** zee-cekirdek, gerçek 100 kartlık liste yükü

## Gereksinim ve karar

Ürün incelemesi §3/2, sabit 100 istek/dakika ve 32768 anahtar dolunca yeni
adresin 503 almasını gösterdi. [Depo içi reproducer](../dogfood/zee-cekirdek/oran-yuk/README.md)
ve exact harici ürün kökeni K-218 ile korunur. Yeni grammar yerine yalnız
`--web-oran-deposu bellek` seçimine üç host ayarı eklenir:

| Bayrak | Değer | Varsayılan |
|---|---|---|
| `--web-oran-siniri` | 1..1000000, IP+uç nokta / 60 saniye | 100 |
| `--web-oran-kapasitesi` | 1..32768 canlı anahtar | 32768 |
| `--web-oran-doluluk` | `reddet` veya `en-eski` | `reddet` |

CLI seçenekleri tekil, doğrulanmış ve worker ömrü boyunca değişmezdir.
Web isteği, form, kaynak program veya ortam değişkeni bu ayarları değiştiremez.
Giriş 5/300 sn ve CSRF 60/60 sn eşikleri ayarlanmaz. Dosya backend'i ve
kalıcı durum biçimi değişmez. Desteklenen Rust facade api::v1 genişlemez.

## Tahliye güvenlik sınırı

Varsayılan `reddet`, K-218 davranışını korur. Explicit `en-eski`, kapasiteye
yeni anahtar girdiğinde yalnız uç nokta kayıtları arasındaki en eski pencereyi
(en küçük pencere sonu, eşitlikte hash sırası) çıkarır. Son erişim pencereyi
uzatmaz; LRU değildir. Yeni giriş/CSRF kaydı da yer açmak için bir uç nokta
kaydını çıkarabilir. Mevcut anahtar hiçbir zaman sırf artış için tahliye
başlatmaz. Giriş/CSRF canlı kayıtları hiçbir kipte tahliye edilmez; hepsi
korunmuşsa yeni anahtar 503 alır. Mevcut sayaçlar 429 üretmeye devam eder.

Tahliye edilmiş uç nokta tekrar gelirse yeni kota alır. Saldırgan adres/IP
çeşitlendirerek bu yerel uç nokta kotasını sıfırlayabilir. Bu explicit seçim
ortak proxy saldırı bütçesiyle kullanılmalıdır; kota aşımına mutlak koruma
sözü vermez. CLI başlangıçta bu sınırı açıkça bildirir. Giriş/CSRF anahtarları
aynı churn ile sıfırlanamaz; ayrı worker/restart sınırı ADR-089 ile aynıdır.

Kayıt, süre ve tahliye indeksleri aynı Mutex altında atomik güncellenir.
Toplam kayıt ve her indeks kapasiteyi aşmaz; yeni istek tüm haritayı taramaz.
Süre dolumu veya tahliye bütün indekslerden kaldırır. Bozuk/zehirli depo
fail-closed kalır; sınırsız kapasite, sessiz fallback ya da canlı auth
sayacı düşürme eklenmez.

## Kabul ve etki incelemesi

Olumlu/olumsuz CLI, sınır uçları, clone yarışı, süre temizliği ve 10000 anahtar
churn altında login/CSRF koruması birim testleri vardır. Gerçek worker testinde
aynı ürünün 100 kartlık sayfası kapasiteyi aşan adreslerle 200 verir ve aynı
IP/adres tekrarında seçilmiş kota 429 üretir. Tam faz, spec drift, tedarik,
uyumluluk, public API ve mimari/SCC kapıları birlikte çalıştırılır.

ADR-046: mevcut web_guvenligi sahibi ve bağımlılık yönü korunur; yeni kök
modül, dış bağımlılık veya temel→web ters kenarı yoktur. ADR-045/049:
performans tarihçesi ve sabit benchmark korpusu değişmez; yeni kapasite yükü
exact temiz kaynak ve çevreyle ayrı gözlemsel kanıt olur. ADR-069: ek tahliye
indeksi nedeniyle soak, her dönemde 32768 kayıt + 4096 tahliye ve sonraki
pencere dolumunu 1800 saniye çalıştırır. Kabul kanıtı
[K-219](../docs/incelemeler/k219/README.md) içinde kapanır. Yeni tanı kodu,
grammar ya da parola üretimi yoktur. Canlı dağıtımı ürün tarafı yapar.
