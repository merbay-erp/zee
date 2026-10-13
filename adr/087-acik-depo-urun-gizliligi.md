# ADR-087: Açık depoda anonim ürün kanıtı (K-214)

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıt:** K-214

PR #5/#7, gerçek ürün talebinin kopyası ve kabul logları üzerinden müşteri ve
altyapı kimliğini ana kaynak hattına taşımıştı. Kullanıcı bugünkü geçmişin
anonimleştirilmesini, force push ve exact-SHA kayıtlarının yenilenmesini istedi.
Bu işlem dil davranışını genişletmez; kişisel/kurumsal kimliği ürün kanıtından ayırır.

Ürün adları Site A–F/ürün E olarak anılır. Gerçek ürün deposu, talep ve kaynak
özel ürün tarafında kalır; açık kanıt exact harici commit SHA'sı, küçültülmüş
reproducer ve anonim özet taşır. Yerel/sunucu yolu ve müşteri alan adı taşınmaz.
K-208/K-209/K-210–K-213 dogfood kayıtları ve freeze zinciri korunur.

`scripts/acik-depo-denetle.py` izlenen dosya adlarını, içerikleri, yeni dosyaları,
symlink hedeflerini, tabandan sonraki commit ağaçlarını ve mesajlarını denetler.
`docs/acik-depo-politikasi-v1.json` yalnız normalize edilmiş terim/yol hash'leri
taşır. Unicode biçim/harf değişimleri ve ayırıcılar sentetik testle kapsanır.
Hata çıktısı değer veya dosya adı yerine dosya SHA-256'sı ve satırı verir.
Eksik politika ve sığ/kayıp Git geçmişi başarısız olur. Sıkıştırılmış dosyanın
içini veya görseldeki yazıyı tarama iddiası yoktur; ürün kanıtı metin olarak tutulur.
Hash'ler sözlük saldırısına karşı sır saklama yöntemi değildir; plaintext listenin
yanlışlıkla açık belge/loga kopyalanmasını önler. Özel terim listesi açık depoda yoktur.

12c5921 tabanındaki önceki geçişler yalnız dosya+exact satır+terim fingerprint'i
ve tekrar sayısıyla kabul edilir. Kopyalama, değişiklik veya ek tekrar kabul edilmez.
Yeni istisna üretici komutu yoktur. Maskeli döküm kullanıcıya ayrı karar için
sunulur; eski geçişlerin tamamen temizlendiği iddia edilmez.

`git filter-repo` yeniden yazımında merge ebeveynleri ve vaka kimlikleri korunur;
kaynak beyanları ve regression `fixed_by`/`introduced_by` yeni SHA'ya bağlanır.
Ölçüm/ikili/provenance kayıtları özgün ölçülen kaynak SHA'sını korur; SHA eşlemesi
yeniden koşulmuş bir benchmark veya yeni ikili kabulü değildir. Kaynak yorumları
ve sentetik SMTP vektörleri anonimleştirilir, eski ürün davranışı testle korunur.

GitHub PR önbellek/ref'lerinin sunucu tarafı silinmesi destek talebi ister.
PR metinleri ve normal dal referansları temizlenir; PR refs/önbelleklerinin
temizliği teyit edilmeden tam silinme iddia edilmez. Enterprise yalnız aynı
kaynağın özel aynasıdır ve Actions kapalıdır. Depoyu yeniden açmak kullanıcı
onayı ister. Karar ve destek taslağı [K-214 kaydındadır](../docs/incelemeler/k214/README.md).

Kanıt: `compiler/tests/guvenlik_kapisi_testi.rs::acik_depo_kapisi_anonim_politikayi_ve_ci_kablosunu_korur`
ve `scripts/tests/acik-depo-test.py` olumlu/olumsuz tarihçe testleri.

Özel depo CI'ında self Git bağımlılığı aynı exact checkout nesnesinden gelir;
Cargo URI/rev değişmez, credential saklanmaz. Tam checkout zorunludur.
`ci-taban-kimligi.py` yalnız K-214 eski master ve iki birleşmiş dal ucu için kaydedilmiş yeni
ata SHA'yı doğrular; diğer tabanlar aynen geçer, genel force istisnası yoktur.
`acik-depo` workflow'u push/PR kaynağını güvenilir master denetleyicisiyle
yalnız veri olarak tarar; PR kodu çalıştırılmaz, token salt-okunurdur.
