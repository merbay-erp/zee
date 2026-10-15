# ADR-150 — K-292 native worker fault ve kontrollü resolver-call hazırlığı

- **Durum:** hazırlık7 ve dedicated native2 adayı; yeni native doğrulama bekleniyor.
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
