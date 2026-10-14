# K-220 — Altı formun anonim entegrasyon kabulü

Ürünleşme talebinin B8 maddesi özel ürün commit'i
`cd5d12f0008620819e6167af5146e83503fd2b38` içindedir. Altı exact anonim
rota/dosya kökeni [mevcut talep tablosunda](talep.md) korunur.
[Kabul kaynağı](form-kabulu.dil), [mevcut helper'ı](turnstile.dil) birim
olarak kullanır. Yeni std, grammar veya compiler davranışı eklenmez.

CSRF ve native oran sınırı korunur. Beklenen konak güvenilir yapılandırmanın
sabitidir; formdan alınmaz. İstemci adresi yalnız native kanonik değerdir.
Eksik/boş jeton 403; yanlış jeton/konak/tip 403; eksik sır, timeout,
sağlayıcı arızası ve bozuk JSON kapalı 503'tür. Bunların hiçbirinde kayıt
ve e-posta etkisi gerçekleşmez. Bal küpü yanıltıcı başarılı yönlendirme
üretir, dış doğrulama veya kayıt yapmaz.

Geçerli doğrulamada fixture'ın anonim eylemi parametreli tek satır INSERT
işler. Ancak başarılı commit sonrası bildirim denenir ve 303 döner.
SQL/commit hatasında bildirim yapılmaz. COMMIT belirsizliği C027/503 ve
otomatik tekrar yoktur. SMTP hatasında zaten kaydedilmiş form kaybolmaz;
fixture başarı yönlendirmesini korur. Gönderim kuyruğu B1/B2'nin açık işidir.

[Ürün kabul testleri](../../../compiler/tests/zee_cekirdek_urun_kabulu_testi.rs)
gerçek handler/helper'ı sahte dış adaptör ile yürütür. Ağ gövdesi/timeout,
sır maskesi, altı rota, CSRF, bot yolları, DB ve e-posta etki sayıları
sınanır. Hermetik adaptör başarılı yönlendirmeyi `→ /...` işaretiyle
gösterir; bu hedefte gerçek soket/HTTP 303 taşıması ölçülmez. Sabit adresler
ve sırlar sentineldir. Cloudflare'e veya gerçek
SMTP/DB'ye bağlanılmaz. Ürünün beş site ağacı, gerçek schema/form doğrulaması,
staging ve canlı dağıtım kabulü bu fixture ile tamamlanmış sayılmaz.
