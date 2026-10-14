# ADR-089 — Süreç başına bellek oran deposu (K-218)

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıtlar:** K-218, RFC-0031, GB-051
- **Ürün:** zee-cekirdek

## Gereksinim ve karar

Gerçek ürünün 100 kartlık anonim GET senaryosu dosya+fsync ortak oran
sayacı yüzünden worker sayısıyla ölçeklenmiyordu. Exact ürün kökeni ve
tekrar üretici [kanıt klasöründedir](../dogfood/zee-cekirdek/oran-yuk/README.md).
CLI `--web-proxy https://panel.example --web-oran-deposu bellek` açık seçimi
oran durumunu süreç başına Arc/Mutex deposuna ayırır. Varsayılan dosya
backend'i, ortak kota ve eski durum biçimi değişmez; PostgreSQL backend'i
gerçeklenmedi. Rust destek yüzeyi hâlâ yalnız api::v1'dir.

Aynı worker'ın clone'ları sayacı atomik paylaşır. Ayrı worker veya restart
ayrı kota oluşturur: toplam limit worker sayısıyla artabilir. Bu seçimi
kullanan ürün, giriş/CSRF dahil ortak saldırı bütçesini trusted proxy'de
uygulamalıdır. Yeni kip otomatik açılmaz. Bellek sayacı restartta kaybolur;
oturum/revoke/rotation/CSRF kaydı mevcut ortak dosyada kalır, cookie'li
istekler kalıcı durumu taze okur. Anonim cookie'siz istek oturum deposuna
dokunmaz; oturum temizliği okuma ve commit'te yapılır. Bozuk oturum kaydıyla
kimlikli işlem fail-closed kalır. Anonim salt GET'in bozuk oturum deposunu
okumaması, auth erişimi vermek anlamına gelmez.

## Boyut ve süre

Varsayılan 32.768 anahtar, mevcut endpoint/CSRF/giriş profilleri korunur. Canlı
kayıt tahliye edilmez; dolu depoda yalnız yeni anahtar reddedilir (503),
mevcut kota işlemeye devam eder. Son tarihe göre BTreeSet ve kayıt BTreeMap
birlikte kilitlenir; her istek tüm tabloyu taramaz. İndeks her canlı kayıt
başına tek giriş taşır; süre dolunca ikisinden de çıkarılır. Sistem saati
geri giderse aktif pencereler uzayabilir; kotalar sıfırlanmaz.

§3/2 ayarlanabilir kota/doluluk politikası [K-219 / ADR-090](090-web-oran-ayarlari.md)
ile ayrı dilimdedir. En eski canlı kotayı düşürmek saldırganın limitini
sıfırlayabilir; K-218 varsayılan seçimi bunu eklemez. K-219 yalnız explicit
uç nokta tahliyesi sunar; canlı giriş/CSRF sayaçları korunur.

## Kabul ve etki incelemesi

[spec/12](../spec/12-web-guvenlik-profili.md), [RFC-0031](../rfcs/0031-bellek-oran-deposu.md),
clone yarış testi, dosya değişmeme testi, süre/kapasite testi ve gerçek iki
süreç login/logout/restart testi birlikte kabul zinciridir. Tam faz, mimari
katman/SCC/public API/uyumluluk ve normatif drift kapıları gerekir.

Yeni uzun ömürlü tampon nedeniyle soak sürücüsü her dönemde tam 32.768
anahtar doldurup önceki pencereyi temizler; mevcut derleyici RSS ölçümü bu
tamponu da kapsar. ADR-069'un 1800 saniye exact kaynak kapısı yeniden koşulur.
ADR-045/049 ve performans tarihçesi incelendi: ürün yük senaryosu sabit
benchmark korpusunu değiştirmez, eski zaman/SHA sonuçları yeniden yazılmaz.
Yeni ölçümler temiz exact commit, gerçek sample ve çevreyle ayrı kanıtta
saklanır; shared CI hard eşik taşımaz. Katman sahibi web_guvenligi aynı
kalır, bin_soak yalnız web adaptörü API'sine ek bağımlılık kazanır; temel
katmana ters kenar yoktur (ADR-046). Yeni bağımlılık, grammar, tanı kodu veya
API facade export'u yoktur. Sunucu yükseltmesi ürün tarafındadır.
