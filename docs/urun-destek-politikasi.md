# Ürün desteği — sınırlı pilot

14 Ekim 2026 kullanıcı kararı; K-224. Zee V1 öncesindedir. Destek modeli
**sınırlı pilot** olarak seçildi: kayıtlı ürünün doğrulanmış kullanımını,
güvenlik düzeltmelerini ve mevcut uyumluluk sözünü izler. Lisans seçimi
bekletildi; yeni lisans, LTS süresi, altı haftalık yayın takvimi veya genel
üretim SLA'sı ilan edilmedi.

## Destek kapsamı

| Hat | Pilot kapsamı | Sınır |
|---|---|---|
| `0.8.0-dev` | Kabul edilen exact commit ve kayıtlı ürün/proje fixture'ı | Aynı sürüm yazısını taşıyan başka commit aynı ikili/kabul değildir. |
| Önceki `v0.x` etiketleri | Tarihsel kaynak ve göç karşılaştırması | Sürekli yama veya LTS desteği ilan edilmedi. |
| Gelecek kararlı sürüm | Sürüm kapıları tamamlanınca ayrıca seçilecek | Takvim tarihi ve destek süresi henüz yok. |

Pilot kabul kaydı exact compiler SHA'sını, ürün pini/anonim provenance'ı,
platformu, çalışma kipini, olumlu/olumsuz testleri ve açık sınırları taşır.
Ürün sahibi canlı yükseltme, parola/veri göçü, yedek ve geri dönüş kabulünün
sahibidir. Bir compiler testi canlı ürün yükseltmesi veya fiziksel platform
kabulü yerine geçmez.

Yerel kaynak kabulü, uzak commit durumu, platform runtime, iki temiz klonlu
artefakt ve canlı staging sonuçları ayrı kaydedilir. Görünür CI başarı işareti
okunmamış adım günlüğü/artefakt içeriği veya Windows runtime kanıtı değildir.

## Güvenlik ve uyumlulukla yayın

- Güvenlik bildirimi [SECURITY.md](../SECURITY.md) içindeki özel pilot
  kanalını izler. İlk inceleme hedefi 3 iş günüdür; SLA değildir. Kritik/yüksek
  açık bulgu kapanmadan CI ve sürüm etiketi geçmez.
- Uyumluluk yüzeyi [K-167 politikası](uyumluluk-politikasi.md) ve
  [spec/27](../spec/27-uyumluluk-ve-surumleme.md) ile korunur. Yeni yüzey veya
  kaldırma, sürümlü envanter/deprecation kaydı ve kabul kanıtı ister.
- Core freeze ve [öncelikli backlog](oncelikli-backlog.md) geçerlidir.
  Pilot talebi tek başına yeni grammar veya geniş API gerekçesi değildir;
  gerçek ürün ihtiyacı, minimal reproducer ve test/karar zinciri gerekir.
- Yayımlanan ikili [iki temiz klon](tekrar-uretilebilir-surum.md),
  SBOM/provenance/imza ve [sürüm runbook'u](surum-runbook.md) kapılarını geçer.
  RC güvenlik/fuzz/Miri, fiziksel platform ve insan kapıları açıkken V1 kabulü
  verilmez.

## Erişim, lisans ve açık kanıt

Ana kaynak ve aynalar özeldir. GitHub yönetilen eski PR refs/cache teyidi
**GB-046/K-215 kapsamında açıktır**; [hazır destek taslağını](incelemeler/k215/github-destek-talebi.md)
kullanıcı iletecektir. Taslağın hazırlanması gönderim veya temizlik teyidi
değildir. Depoyu herkese açma ayrıca kullanıcı kararı ister.

Proje lisansı henüz seçilmedi; bağımlılık/font lisansları Zee kaynakları için
lisans tahsisi veya yeniden dağıtım hakkı sayılmaz. Resmî site kaynakları bu
sınırı anlatır; kaynak metni düzeltmek canlı siteyi dağıtmak değildir.

Gerçek insan formları, temiz fiziksel Windows/macOS ve minimum işletim sistemi
kanıtları mevcut olmadığından ilgili V1 kapıları açıktır. Freeze penceresi ve
diğer kapı durumu [V1 kapılarında](v1-surum-kapilari.md) izlenir; bu destek
kararı eksik kanıtları kapatmaz.

## Değişiklik ve bakım

Pilot kapsamı büyütülmeden önce ürün/provenance kaydı, testler, güvenlik ve
uyumluluk etkisi, kabul platformu ve geri dönüş yolu güncellenir. Destek
modeli, lisans veya düzenli yayın takvimi değişirse bu politika aynı değişiklikte
yenilenir; tamamlanmamış vaat tamamlanmış kapasite olarak yazılmaz.
