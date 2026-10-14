# 14 Ekim 2026 onaylı ürünleşme uygulama kaydı

Kullanıcı A1–A6, B1–B9, C1–C6, D1–D4, E ve ZEE-001–018'in birlikte
değerlendirildiği planı 14 Ekim 2026'da onayladı. Exact ürün talebi
`cd5d12f0008620819e6167af5146e83503fd2b38`, başlangıç compiler ağacı
`be5699c3913fe66594fb9d9509253fd9a0b7f5b9`tir. Dış statik inceleme Cargo,
canlı ürün veya uzak CI yeniden kabulü değildir.

Makine okunur [39 işlik eşleme](urunlesme-plan-v1.tsv) bütün talep kimliklerini
korur. `P-*` plan kimliğidir; K kaydı yalnız uygulanmış/onaylı atomik dilime
ayrılır. Açık/ertelenmiş/harici kanıt satırı tamamlanmış özellik değildir.

## Kullanıcı kararları ve gerçek dış sınırlar

- Lisans seçimi bekletildi; teknik işlere devam edilir. Kaynak-erişilebilir,
  açık kaynak veya ticari lisans kullanıcı adına seçilmez.
- [Sınırlı pilot destek](urun-destek-politikasi.md) seçildi; güvenlik ve
  uyumluluk kapılarıyla sürüm. LTS veya sabit takvim sözü verilmedi.
- Kullanıcı gerçek K-161/K-162 oturumu, temiz fiziksel Windows/minimum OS
  cihazı ve GB-046 destek yanıtı bulunmadığını bildirdi. Geliştirici Mac'i
  fiziksel fakat temiz değildir; temiz makine kabulü yerine kullanılamaz.
- [GitHub destek taslağını](incelemeler/k215/github-destek-talebi.md) kullanıcı
  gönderecek. Bu çalışma destek mesajı gönderimi veya public açma değildir.
- Mevcut resmî marka/site hattı sürer. Kaynak metni düzeltmesi canlı deploy
  edilmiş sayılmaz; dağıtım ve görünürlük ayrı işlem yetkisi ister.

## Sıra ve yeniden açılmayan kapsam

1. P01 kaynak/ZIP/CI eşliği; P31 TLS/NULL ve P33/P34 kalan güvenlik kanıtı.
2. P03 bilinmeyen seçenek; P02 deterministik build kimliği; P05 uyumluluk
   işletimi. A6(2) genel config, küçük tanı düzeltmesine yüklenmez.
3. P08 anonim ürün korpusu; P15 parola rehash ve P16 form kabulü; P19 yalnız
   mevcut maliyeti gerçek temiz projede ölçme. Cache ölçülmüş ihtiyaca bağlıdır.
4. P29 platform kanıtı/P06 dağıtım; P12 göç/P13 işletim/P25 credential ve
   P14 draining/yeniden başlatma.
5. P09 kalıcı iş/P10 MIME; P11 PDF; P18–P23 ürün kompozisyon ve tooling.
6. P30 yalnız insan/freeze/platform/ürün/güvenlik birleşik kapıları sonrası.

B-001/B-002 insan hattı, K-174 14 Ekim–11 Kasım penceresi, K-192 cihaz
hattı ve GB-046 destek teyidi teknik testle ikame edilmez. P17 S3, P26
sanitizer, P35 yeni ergonomi grameri ve P38 backend varsayılan olarak
ertelenmiştir. Tek koşuda çalışma isteği kayıtsız veya sahte kapanış değildir.

K-218/K-219/K-220 dil makine kabulü korunur. K-206 bounded toplama ve
GB-041 tek-frame sınırı kapalıdır; global RSS garantisi değildir. K-213
birleşmesi canlı parola/veritabanı göçü sayılmaz. Rust compiler playground'u
Zee→WASM codegen değildir.

## Kaynak ve CI kanıtı

[Başlangıç eşleme JSON'u](incelemeler/urunlesme-20261014/source-ci.json)
1036 Git archive kaynağını ZIP ve dosya manifestiyle birebir karşılaştırır.
ZIP SHA-256 `1555749c7783e7b34a72b61b85985e83d1770494d0769a45ef4b768750664ede`;
ZIP comment exact başlangıç HEAD'idir. Kaynak eşliği Git archive'a aittir;
rapor hazırlanırken uygulama dalındaki çalışma ağacı kirli olabilir.

Gitsrv özel aynanın `master`ı exact be5699c olarak görüldü; #30 commit
durumu başarıdır. Job ayrıntısı 404, artefakt indirme kullanıcı tarafından
reddedilmiştir. #29 kaynak SHA'sı 3d7ff93, compiler kaynak commit'i
488dc7'dir; #29'dan be5699c'ye compiler tree aynı, helper/belge farkı vardır.
#29 veya visible #30 status yeni uygulama commit'lerinin CI kabulü değildir.
GitHub ana kaynak 404 ve Enterprise ayna SSO erişim sınırı ayrı açıktır;
silinme veya kaynak değiştirme sonucu çıkarılmaz. Aynaya push yapılmaz.

## Atomik kapanış

Her kaynak işi ilgili spec/RFC/ADR, olumlu/olumsuz test, tanı/regression,
faz/mimari/SCC/API/uyumluluk, tazelik ve güvenlik kayıtlarıyla birlikte ilerler.
Compiler source commit'inin exact semantic/core-freeze beyanı ve bugfix
`fixed_by` kaydı takip eden provenance commit'ine aittir. Kaynak şimdi-belge
sonra borcu bırakılmaz. [AGENTS.md](../AGENTS.md) ve [runbook](surum-runbook.md)
kapıları bağlayıcıdır. Cache/kalıcı durum ekleyen iş exact temiz kaynakta
1800 saniye soak; release exact ASan/Miri/imza/kurulum kabulü ister.

Normatif spec maddeleri exact test işlevine bağlanır; tasarım boşluğu test
eklenerek kapatılmaz. Tamamlandı durumu yalnız gerçek kabulden sonra yazılır;
yerel kabul, uzak CI, canlı ürün ve release birbirinin yerine geçmez.

## Yerel teknik kapanış kayıtları

[Temiz native checkpoint](incelemeler/urunlesme-20261014/native-7127c7d/README.md)
TLS 4/4, göç 1/1 ve rehash 7/7 gerçek PostgreSQL kabulünü taşır.
[K-226 iki-klon](incelemeler/urunlesme-20261014/build-99f31d7/README.md)
exact kaynakta eş hash üretmiştir; imzasız çıktı yayın kabulü değildir.

Mevcut primary GitHub HTTPS/connector ve mevcut SSH transport üzerinden
read-only kaynak okuma sonuçları 404/repository-not-found; Enterprise ayna
SSO sınırı 403'tür. Primary master yetkisi/SHA'sı bilinmiyor. Yerel onaylı
uygulama dalı uzak master değişikliği veya yeni exact CI kabulü sayılmaz.

Kullanıcı P08/A5 anonim beş tam site ve çekirdek korpusunun henüz hazır
olmadığını teyit etti; kapı açık tutulur ve mevcut teknik dilimler sürer.
Anonim küçük fixture sayısı tam ürün N/N kabulü değildir.

K-236/GB-060, P10 SMTP diliminde incelemeyle bulunan AUTH echo sızıntısını
sabit native/runtime C029 maskesine kapatır. Gerçek loopback AUTH535,
explicit URL hatası ve CLI testleri ile [SMTP regression kipi](semantic-regresyon-korpusu.md)
kullanılır. Kaynak SHA ve exact fixed_by takip provenance kaydında yazılır;
P09 kuyruk, canlı provider/ürün ve K-237 yanıt bütçesi ayrı kalır.
