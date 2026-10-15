# S kanıt arşivi — yerel teknik kabul

Kaynak S=8b49113c413f62b35d4b251ed31068524d2fe26c; E yalnız belge/kanıt commit’idir. RC bundle manifest SHA256 `5e174eb9179e79e75b5127e12c1760b65d448704256ad69dbfa3abd5b343805c`; dış ham kapı makbuzu SHA256 `13e3130aba5b1c91b5f69d50a6429ab92622522812d4cdd2abe5613b09237c0e`. GB082 exact S yerel kapısı için kapalıdır. Kanonik paket ve aynı-S güvenlik kapısı kabul edildi. E bu kaynak için belge ve kanıt arşividir.

[Normalization](../NORMALIZATION.md), [raw/derived mapping](../DERIVED-METADATA-MAPPING.json) ve [v2 derivation manifest](../DERIVATION-MANIFEST.json) kardeş dosyalardır. Mapping yolları üst dizine göredir. .derived.json JSON nesnelerinin inverse eşliği parsed JSON içindir; ham byte eşliği değildir. Özgün hashli ham dosyalar dışarıda korunur; hash authentication değildir.

Linux full dış411 yaprak Git’te değildir:127 temsil=120raw+7reversible derived metadata. [Subset](linux/metadata/TRACKED-SUBSET.json) ve full external MANIFEST/SHA256SUMS bu ayrımı korur. Dış manifestlerin referenced raw/ikili dosyaları yalnız Git subset’ten doğrulanamaz.

HISTORICAL-V1-STAGE-MANIFEST.json yalnız özgün v1 hash kaydıdır; aktif inventory değildir, eksik raw adlarına işaret edebilir. DERIVATION-MANIFEST dışarıdaki frozen v2’nin özgün yol ve hashlerini belgeler; v2 draft belgeleri bu yayın önerisinde değiştirilmiş ve v1 STAGE manifesti tarihsel adla ayrılmıştır. Bu nedenle v2 manifesti bu yeni dizinin aktif tam envanteri sayılmaz; PUBLICATION-REPRESENTATION-MAPPING.json bu farkları açıklar. [Güncel arşiv manifesti](CURRENT-ARCHIVE-MANIFEST.json) mevcut dosyaları ve kardeş normalization kayıtlarını bağlar. HISTORICAL-PREPARATION-MANIFEST.json ile HISTORICAL-RC-PENDING.json yalnız önceki hazırlık kayıtlarıdır.

25cohort/163command/268indexedfile ile943/959 libtest/16ignored farklı run-local sayılardır, unique test toplamı yapılmaz. Timer inflight latch korunması runtime process cleanup arızası değildir. Tarihsel authoritylost ve failedexport kayıtları geriye dönük kapatılmaz.

RC paketinin tam 7483 dosyası ve fuzz korpusları dış kanıt dizininde korunur; bu arşiv seçili metadata ve günlükleri taşır. [Kapsam](rc/RAW-SCOPE.json), [sonuç](rc/FINAL-RESULT.json), [kök incelemesi](rc/root-independent-review.json) ve [makbuz türevi](rc/POST-GATE-RECEIPT.derived.json) birlikte okunur. İki yeni public kaynak-hash eşleşmesinin tersine çevrilebilir temsili [RC haritasındadır](rc/RC-DERIVATION-MAPPING.json). Hiçbir fuzz seed’i kaynak korpusuna alınmadı.

Owned Linux VM [kontrollü kapatıldı](linux/owned-VM-stop.json); disk ve manuel uzlaştırma kilitleri korundu. JetBrains tamamlama ve düzenli LSP kapanışı hâlâ doğrulanmamıştır.

Üç ham günlükteki özgün son boş satırlar korunur. Kök `.gitattributes` yalnız bu üç exact log yolu için `blank-at-eof` denetimini kapatır; günlük baytları ve hashleri değiştirilmez.
