# ADR-150 — K-292 native worker fault ve kontrollü resolver-call hazırlığı

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): iki native worker-loss/stale-gate vakası ile dar yerel kabul; kontrollü Python resolver native DNS iptali veya tam fault kabulü değildir.
P34 genel fault matrisi ile P13 whole-unit crash dar kanıtı ayrılır. Tek owned
native worker SIGKILL olduğunda diğer native worker'ın aynı retained Supervisor
owner'ıyla kapanması, foreign survival ve yeni liveness200 worker'ların eski true
gate'i ödünç alamaması gerçek native iki ayrı vaka olarak istenir.

Üretim lifecycle/native DNS değiştirilmez. Test-only Python resolver dependency
injection yalnız kendi fresh child'ında call noktasında stall oluşturur. DB helper'ın
gerçek timeout/cancel/selector-fault cleanup'ı böyle ölçülür; bu native DNS veya
UDP resolver kabulü değildir. Sistem DNS'inin native iptal sınırı açık kalır;
global resolver, preload, kernel, provider veya platform ayarı değiştirilmez.

[Rehber](../docs/service-fault-wave.md), actual kısa süreç hazırlıkları ve ayrı
native koşucu aynı dilimdedir. Unknown CLI path/argv sızdırmadan reddedilir.
GB-108, faz, backlog, plan ve uyumluluk kayıtları aynı dilimde güncellendi.
Compiler/grammar/API/cache veya soak ölçütü değişmez; full P34 ve live adoption
bu iki native vaka veya controlled stall sayılarıyla tamamlanmış gösterilemez.

## K-296 — deduplikasyonlu state olayı yerine gerçek liveness oracle

S2 release kabulünde worker-loss vakası geçti; stale-gate vakası aynı ready=false
durumunda yeni state olayı beklediği için kırıldı. Supervisor state'i yalnız
readiness değişince yayımlar; üretim davranışı korunur. Native koşucu iki worker'ın
Build kimliği doğrulanmış HTTP200 liveness'ını doğrudan owned Supervisor probe
ile gözler. Startup reddi, hiç ready=true olayı olmaması, eski gate bayt/mtime
değişmezliği ve bütün owned handle kapanışları birlikte zorunlu kalır. Yalnız
test oracle düzeltilmiştir; native DNS veya production lifecycle genişlemez.
S3 temiz native kabulünde iki vaka geçti.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

## K-299 — Kabul hazırlığı ve cleanup sırasında iptal

Üretim worker/Supervisor/lifecycle değişmez. Native kabul Campaign kurucusu child
başlatmaz; prepare ilk H.run/clean_exact öncesi handler kurar ve caller finally
kapsamındadır. Borrowed fixture kurucusu hata verirse yalnız kendi temporary'si
temizlenir. Cleanup iptali H.CANCELLED'i sıfırlamaz ve passed receipt'e dönüşmez;
sentinel ve worker grupları ilk Popen sahipliğiyle kapanır. Dış owner nested
cleanup için yeterli grace ayırır; PID keşfi/adoption yapılmaz.

Üç yeni vaka ile stdlib10 geçti. İlk C native kabulü, Cargo wrapper
`compiler/..` kökü nedeniyle Python Campaign kanonik yol kontrolünde, iç
proof oluşmadan reddedildi. Rust test wrapper `root()` yolunun
kanonikleştirilmesi takip adayındadır; Python sınırı ve worker davranışı
değişmez. Bu0,04 saniyelik libtest başarısızlığı üç saniyelik worker hazırlık
timeout’u değildir. GB-111 düşük/açık/K-299; yeni kaynakta kapılar ve native2
kabulü beklenir. [Kayıt](../docs/kabul-kosuculari-yasam-dongusu-20261016.md)
ayrı PDF9 başarısını ve birleşik failed sonucu korur. S3 native2 tarihseldir;
native DNS veya tam fault kabulü ileri sürülmez.