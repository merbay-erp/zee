# urun-g — mini ERP dogfood ürünü (K-330)

Ürün G, zee ile yazılmış rol/sayfa/işlem yetkili bir mini ERP'dir. Gerçek
ürün özel depodadır; bu klasör derleyiciyi değiştiren ihtiyacın depo içi
kanıt aynasıdır.

| İhtiyaç | Üründeki yer | Yüzey |
|---|---|---|
| Gün boyu formla çalışan kullanıcının 30 dakikada girişe atılmaması | Bütün kimlikli sayfalar | `oturum_ömrü_saniye N olsun`, `oturum_kayar doğru olsun` (proje.dil) |
| Çıkışın oturumu hemen bitirmesi | `/cikis` rotası | mevcut `oturumu kapat` |

`dil denetle .` bu klasörde hermetik koşar. Kanıt: ADR-158,
`compiler/tests/web_cok_surec_testi.rs::kayar_oturum_iki_worker_restart_ve_cikista_dogru_davranir`,
K-330 günlük kaydı.
