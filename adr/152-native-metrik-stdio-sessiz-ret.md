# ADR-152 — Metric-only stdio alias için erken sessiz ret

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): 26 native stdio vakası, exact regression/fixed_by ve RC kanıtıyla dar yerel kabul; K-294/GB-110 bu kapsamda kapalı.

## Gerçek bulgu

K-284'ün güçlendirilmiş owned FD testi, eski metric-only profilinde stderr
ile aynı pipe'a işaret eden metric FD'nin exit2 ile reddedilirken61 bayt tanı
yazdığını gösterdi. Tanı generic'ti; özel kaynak/oturum sızıntısı iddia edilmez.
Buna rağmen kontrol/metadata kanalının yalnız protokol baytı taşıması gerekir.
Before candidate'ın gerçek binary hash/Build kimliği ve61bayt SHA kaydı dış
kanıtta korunur; dirty candidate exact temiz eski kaynak diye sunulmaz.

## Minimal düzeltme

Native listener/ready/activate ve metric FD rollerinin stdio alias'ları
herhangi bir CLI/config/source tanısından önce borrowed `fstat` ve mevcut
pipe-pair kimlik denetimiyle reddedilir. FD alınmaz, kapatılmaz veya başka
PID sorgulanmaz. Darwin'de mevcut own-process SDK pipe-peer yordamı aynen
paylaşılır; yeni C ABI veya platform vaadi yoktur.

Spaced seçenekler ve `name=value` ret biçimleri yalnız sessizlik amacıyla
incelenir; `=` typed profile kabulü yaratmaz. i32 taşmaları parse hatasıdır;
bozuk önceki seçenek daha sonraki geçerli alias'ı gizleyemez. Tarama kaynak
yolunda biter; kaynak sonrasındaki uygulama argv'si kontrol rolü sayılmaz.
Mevcut Profile extractor ile değer taşıyan seçenek sınıflaması ortaktır.

Alias olmayan olağan CLI hataları tanı üretmeye devam eder. Descriptor
validation/ownership kısıtları gevşetilmez. Valid designated nonblocking
metric writer ile gerçek Zee kaynağı çalıştırılması olumlu kontroldür.

## Kalıcı kanıt ve commit sırası

`native_metric_stdio_testi` gerçek own pipe/Popen/WNOWAIT ile26 cohort yürütür:
valid kaynak yürütümü, dörtFDrolü × iki stdio × üç bozuk/geçerli yazım ve
olağan CLI hata tanısı. Bu26cohort tek normal Cargo vakasıdır. Unix dışında
vaka görünür ignored'dır; zorla koşturulursa truthful panic verir.

`native-metric-stdio` semantic korpus kipi yalnız `cli`, exit2, boş çıktı ve
no-diagnostic/no-span bekler. Gerçek `.dil` kaynağı helper'a verilir; parser-only
replay değildir. Kaynak commit'ini izleyen commit exact `fixed_by` satırını,
minimal fixture'ı ve nonempty dedicated native korpus testini birlikte ekler.
İlk kaynakta placeholder SHA veya orphan fixture yoktur. Mevcut tüm korpus
vakaları/kimlikleri aynen korunur; aggregate portable vakaları çalıştırır.

K-257 eski alias tanı-oracle'ı bu düzeltmeyle yalnız alias dallarında sıfır
kontrol/diagnostic baytı ister. Diğer metrics8 kabul ölçütleri değiştirilmez.
Grammar, façade, normal uygulama stdout'u, hata kodları, performans eşikleri
ve public API değişmez. Yeni compiler bug manifest follow-up'ı olmadan iş
kapanmış sayılmaz.

Kaynak düzeltme SHA: `382beed1d01440f6f9df28b5ac7a111fbe1f75ee`.
`regression/v2.tsv` exact `fixed_by` ve gerçek minimal kaynak taşır; dedicated
native test boş manifestle geçemez. GB-110 exact regression/nonempty replay ve temiz S3 canonicalnative26/RC ile dar kapsamda kapandı.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.
