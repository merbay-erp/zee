# ADR-118 — Sahipli native listener ve nesil hazırlığı

- **Durum:** kabul, sınırlı Unix host profili
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-259, GB-080

Temiz exact kaynak kabulü ayrı kampanyada kaydedilir; tam P14 benimsemesi ve genel sıfır hata sözü açık kalır.

Ürün kanıtı zee-cekirdek aktif kaydıdır. B6 talebinin external commit'i
cd5d12f0008620819e6167af5146e83503fd2b38, request SHA256
694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7;
etkilenen genel cekirdek/kaynak/ana.dil SHA256
f0a65a0b1272a717728db4050fd450fd85900434ed6d5309bdcb9bbf9f138619.
Özel ürün gövdesi, hizmet/nginx adı veya müşteri verisi buraya taşınmadı.

Mevcut proxy profili listener devri sağlamıyordu. En küçük ihtiyaç CLI/runtime
Unix adaptörüdür: owned host socket'i yaratır, child exact fd duplicate alır;
çekirdek grammar veya public facade genişletilmez. Host original socket'i tutar;
worker yalnız kendi duplicate'ini kapatır. Quiet preflight kaynak/config tanısından
önce STDIO ve role aliases'i reddeder. Darwin pipe endpoint inode'ları farklı
olduğu için own-PID SDK peerhandle çifti de karşılaştırılır; yabancı PID edinilmez.

Native ready pipe old worker HTTP cevabı değildir. Exact Popen PID, freshnonce,
build/source kimliği ve port doğrulanır; ayrı fresh generation-owned app receipt
ile birlikte exact ACTIVATE+EOF gelmeden accept başlamaz. Successful accept bu
profile'da admission'dır; TERM sonrasında stream mevcut sınırlarla tamamlanır.
Varsayılan direct bind K253 davranışını korur. WNOWAIT leader reservation grup
cleanup bitene kadar korunur; EOF yolu blocking diagnostic okumaz.

Aynı canonical root ortak atomic JSON quota/session/revoke durumunu korur.
Süreç belleği kota deposu inherited overlap profilinde reddedilir; reset gizlenmez.
Pilot aynı frozen application revision içindir; genel kod/resource deployment
atomicity ve live operational adoption açık kalır.

Kanıt: native listener yaşam döngüsü, quiet FD/config negatifleri, metrik kompozisyonu ve gerçek postaccept/SIGTERM race unit. Güncel exact sayılar Cargo wrapper ve temiz kampanya makbuzundadır.
Bin istek concurrency1/dört owned rollover; sentetik ayrı loopback istemcileriyle
sınır içi yükte1000HTTP200, no retry. Paylaşılan kota429 ve revoked-cookie401
restart/overlap boyunca korunur. GB-080 alias/erken reap/config kaynak seçimi bulguları kaynakta kapatıldı. Metrik pipe readerının önceden kapanması hizmeti durdurmaz; kontrol pipe’ları ise canlı peer ve exact rol ister. Credential launcher listener/ready/activate descriptorlarını devretmez; bu profil doğrudan sahipli native host ile sınırlıdır.

## İlk adayın geç EOF düzeltmesi

GB-080 incelemesi ilk commitlenmemiş adayda son poll turunda deadline sonrası EOF ile admission açılmasını da buldu. Kalan bütçeye göre poll ve exact frame/EOF sonrası deadline/sinyal kontrolü aynı özellik diliminde düzeltildi. Normal Cargo `native_listener_gec_eof_mutlak_butceden_sonra_admission_acmaz` gerçek owned worker üzerinde geç EOF, HTTP byte0, terminal0 ve parent listener korunmasını doğrular; native17/FD16 sayıları ayrı kalır.

Korpusun ürün kök politikası da `dosya-yazma` taşır: anonim B6 reproducer yalnız owned host tarafından verilen private generation receipt yoluna yazar. Alt proje bildirimi zaten bu yetkiyi taşıyordu; ürün kök aynası eksikliği ilk tam fazda T054 ile yakalandı ve aynı ilk özellik diliminde eşlendi. Bu kayıt gerçek ürün deposunu veya genel dış yol güven sınırını değiştirmez.
