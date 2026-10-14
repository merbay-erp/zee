# ADR-100 — SMTP yanıt okuma sınırı (K-237)

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-237, GB-061 (yüksek, kapalı)
- **Karar zinciri:** ADR-083, ADR-099, spec/23, ADR-045/049

## Bulgudan düzeltmeye

`Oturum::yanit` 16 KiB kontrolünden önce `read_line` ile LF/EOF’ye kadar
sınırsız String büyütüyordu; actualhelper 1 MiB girdinin tamamını tüketti.
İlk kaynak `cbacc4c09cf0534b7436f33d771a30ce687d510f`: dosyanın ilk eklenmesi,
blame ve helper’ın byte-byte eşliği aynı arızayı bağlar. Bu SHA güncel HEAD’in
atasıdır; K-213 merge’i kaynağı ana hatta taşır. K-232 MIME aynı helper’ı
kullanır; düzeltme MIME codec ve K-236 hata maskesinden ayrı security bugfix’tir.

16 KiB + 1 bayt kapasiteli tek yanıt String’i, her satırda kalan 16 KiB + 1 bayt `Take` ile
sınırlanır. Tek taşma baytı tüketilince UTF-8/read hata dönüşünden önce sınır
reddi gelir. Tam 16 KiB LF/EOF, geçerli tel baytları/kod/çok satırlı birleştirme
ve sonraki yanıt tamponu korunur; sınır içi bozuk UTF-8 okuma hatasıdır.
Varsayılan BufReader 8 KiB sabit ön okumayla 24576 bayt alabilir; payload tüketimi
16385’tir. TLS/kernel tamponları veya toplam RSS için 16 KiB iddiası yoktur.
Eski plain per-operation timeout ve MIME absolute deadline sınırı değişmez.
K-236 public/runtime hata maskesi ve retry yapılmaması korunur.

## Kanıt ve etki

Beş actualhelper testi EOF/LF-yok1MiB/sonsuz/tekbayt akışı, toplam ve tam
sınır, UTF-8, boş EOF/timeout ile tüketim/tampon/ön okuma değerlerini ayırır.
[Küçük regression](../regression/security/smtp-sinirsiz-yanit.dil) gerçek
loopback sender’ı `smtp` kipinde yürütür. Sunucu LF/EOF vermeden50 ms yeni
baytla read timeout’ını canlı tutar; çağrı dönerse kapanır. Eski gövde hem
1MiB byte assert’ini hem 5 saniyelik fixture deadline’ını kırar; restore pozitifi geçer.
Bu fixture zamanı benchmark değildir. Yerel archive/review sonucu temiz
exact commit, yeni uzak CI, RC/live provider/ürün/platform kabulü değildir.
Exact source bugfix SHA, `fixed_by` ve compiler/freeze beyanı takip kaydıdır.

Küçük yanıt da başlangıçta 16 KiB + 1 bayt ayırır; eski küçük/geometrik tahsis değişir.
Tampon çağrı/yanıt ömürlüdür; cache/kalıcı state, yeni grammar/API/dependency
ve mimari owner/edge yoktur. Sabit benchmark iş yükü SMTP taşımadığından
ADR-045/049 iş yükü/örnek semantiği ve immutable tarihçe korunur; p95/RSS
iyileşmesi veya yeni taban iddiası yoktur. Gelecek ölçüm temiz exact kaynak ve
tam çevre/sample/warmup provenance ister. Spec23 exact helper/native kanıtı,
GB061, semantic `smtp` sözleşmesi ve mevcut io/corpus faz sahipleri birlikte
güncellenir; tanı kodu, facade ve K-174 syntax fingerprint aynı kalır.
