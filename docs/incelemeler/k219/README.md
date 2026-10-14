# K-219 — Oran ayarı kabul kanıtı

Ürün talebi §3/2; [karar](../../../adr/090-web-oran-ayarlari.md).
Aynı exact ürün kökenli 100 kartlık kaynak kullanılır; müşteri/sunucu bilgisi
ve canlı ürün durumu kopyalanmaz. CLI, kota ve kapasite churn testleri ile
1800 sn soak ve Gitsrv kabulü bu sayfada gerçek sonuçla kapanacaktır.
Şu anda sonuç beklenir; performans tarihçesine yeni taban yazılmaz.

Varsayılan reddet; explicit en-eski yalnız uç nokta pencerelerini tahliye eder.
Tahliye edilen uç nokta tekrar gelirse kota sıfırlanabilir. Login/CSRF canlı
sayacı korunur; tüm anahtarlar korunmuşsa yeni anahtar 503 alır. Ayrı worker
ve restart kotası ortak değildir; proxy ortak saldırı bütçesi gerekir.
