# S3 seçili kanıt arşivi

Test kaynağı `66e4149e53451111d153f47283c219079a8d1cab`; bu dosyaları ekleyen E belge/kanıt commit'idir. [Güncel teknik sonuç](../../../urunlesme-s3-kabul-20261015.md) gerçek kapsamı ve açık kapıları taşır. Public release, üretim imzası, deployment veya ürün adoptionı yapılmadı.

- [macOS native kabulü](mac-native/PROOF.json) ve [bağımsız denetim](mac-native/independent-review.json); [retained dedicated kohortlar](retained-native/PROOF.json).
- [Linux dispatcher4 türevi](linux/dispatcher/K282-proof.derived.json), [HTTP7 türevi](linux/http-retry/K283-proof.derived.json), [164 komut/167 ham dosya denetimi](linux/raw-review.json).
- [macOS delivery](mac-delivery/DELIVERY.json), [Linux delivery](linux/delivery/DELIVERY.json), [Mac kurulum8](mac-install/PROOF.json), [Linux kurulum8](linux/install/PROOF.json).
- [1822 kaynak dosyasının platformlar arası içerik eşliği](source-ZIP-content-comparison.json); ZIP DOS saatleri farklıdır, platformlar arası ZIP byte eşliği ileri sürülmez.
- [Gerçek JetBrains protokol gözlemi](jetbrains/SECOND-UI-OBSERVATIONS.json): completion111aday, S004→A012→[], formatting, shutdown/exit ve iki normal Quit exit0. JSON küratörlü tool gözlemidir; tam ham konsol veya yeni screenshot dışa aktarımı değildir. İlk trace-pending kayıt tarihsel olarak korunur.
- [1800s soak bağımsız medyan denetimi](soak/independent-review.json), [release1/41belge ölçümü](lsp/measurement/MEASUREMENT.json), [25 actual ignored owner eşliği](ignored/OWNER-REVIEW.json).
- [Dört ASan+Miri kampanyası](rc/campaign-proof.json), [aynı temiz S3 RC kapısı](rc/GATE-RECEIPT.json), [tam dış bundle kapsamı](rc/RAW-SCOPE.json).
- [Sahipli VM normal kapanışı](linux/end/VM-STOP.json) ve [son unit/latch denetimi](linux/end/GUEST-END.json); disk ve SQL unknown latch korundu.

Normal Cargo sayısı ile native alt senaryolar toplanmaz. Source-coverage üreticisinin candidate etiketi değiştirilmedi; clean S3/canonical binary bağı bağımsız denetlendi. Campaign üreticisinin release_gate_closed=false alanı kampanyanın tek başına sürüm kapısı olmamasını anlatır; sonraki same-S3 kapısının actual sonucu ayrı GATE-RECEIPT'tir.

## Temsil ve ham kapsam

[Kaynak/temsil haritası](RAW-REPRESENTATION-MAPPING.json) ham dış yol ve hash ile Git temsilini bağlar. Dört JSON'daki sekiz credential-baslat.py alanı sır değil, exact S3 Git blob SHA256'sıdır. Değişmeyen default Gitleaks politikası bu alanları işaretledi; sadece bu scalar alanlar index binding'e çevrildi. Tam inverse parsed JSON eşliği doğrulandı. Orijinal byte serileştirmesi dışarıda değişmeden durur; parsed eşlik byte eşliği değildir. Miri, LSP ve seçili test günlükleri ile K282 makbuzunun ham byte envelope'ları base64 ters çevrimiyle byte eşliğini korur. K282 makbuzundaki son boş satır özgün dosyada korunur; Git temsili biçim kapısı için envelope kullanır. Hash authentication değildir.

Tam fuzz corpus/build/run/gate logları, paket ikilileri/ZIP/bundle ve Linux'un164komutluk ham günlükleri Git altkümesinin dışında korunur. Dış manifestlerin bütün referred dosyaları yalnız bu Git altkümesinden doğrulanamaz. Kaynak seed korpusuna import ve uzak upload yapılmadı.

## Tarihsel başarısızlıklar

S2 daemon/oracle başarısızlığı ve erken sonlandırılan fuzz/soak yeni S3 kabulü sayılmaz. S3 ilk Linux birleşik controller'ı K282passed4 sonrasında K283case0 öncesi kendi expected failed-unit bayrağı nedeniyle durdu; yalnız issued unit reset edildi, unknown SQL latch değişmedi. Yeni HTTP7 ayrı source/binary pinleriyle geçti. İlk Linux delivery PATH/unzip, ilk Mac smoke params:null ve ek cross-platform ZIP-byte assert reddi ayrı makbuzlarda korunur. Bunlar başarıya çevrilmedi.

İlk aynı-S3 RC kapısı host boş alanı 10 GiB altına düşünce durdu; export/check geçti, tam güvenlik sonucu bilinmiyor ve ilk kayıt başarıya çevrilmedi. Yeni dış koşucuda kontrollü iptal testi geçti; geçerli bundle değişmeden tekrar kullanıldı. İkinci tam koşuda iki Cargo grubu başarısız oldu; seçili tekrarda listener geçti, worker grubunda farklı bir hazırlık vakası başarısız oldu. Kayıtlar başarıya çevrilmedi. Kullanıcı diğer oturumdaki temizliğin tamamlandığını bildirdikten sonraki seçili koşu ve seri tam v3 koşusu ayrı kayıtlardır. Son GATE-RECEIPT v3 koşusudur; kaynak, testler ve zaman sınırları değişmedi, tam koşunun test paralelliği bire indirildi. Zamanlama hassasiyetinin kök nedeni kanıtlanmış sayılmaz.

İlk JetBrains trace Apply vendor EDT hatası dış logda korundu; hata raporu gönderilmedi. İkinci normal oturumda trace açıktı. Dört IDE spelling Typo ile LSP boş tanısı ayrı gözlemlerdir. C1 özel quickfix ve P33 coordinated reproducer araç retleri açık kalır.

[Etkin dosya envanteri](CURRENT-ARCHIVE-MANIFEST.json) bu dizindeki temsil dosyalarını kapsar; kendisini hash tablosuna dahil etmez.
