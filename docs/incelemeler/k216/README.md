# K-216 — Git2 / Gitsrv koşuları

Kullanıcı 13 Ekim 2026'da CI koşularını özel Git2/Gitsrv üzerinden istedi.
Ana kaynak merbay-erp/zee, Enterprise aynı kaynak aynasıdır; ikisinde
Actions kapalıdır. Git2 deposu gizli, Actions açık ve dört güncel Gitsrv
koşucusu başlangıçta çevrimiçi/boştaydı. Özel ürün ci/site-dil pini korunur.

## Kaynak ve kapılar

[ADR-088](../../../adr/088-git2-gitsrv-kosulari.md) ve
[Forgejo akışı](../../../.forgejo/workflows/zee-ci.yml) aynı kod/test/belge
dilimindedir. Push yalnız master/ci/zee-gitsrv, ayrıca manuel başlatma vardır.
PR kodu otomatik yürütülmez. Gerçek ref fetch adımının hermetik testi
ilgili tam geçmişi, exact SHA'yı, ilgisiz özel dal/etiket ret sınırını,
dolu çalışma dizini reddini ve kalıcı token yokluğunu kanıtlar.

Linux faz, kalite, kaynak korukları, gizlilik/sır, cargo-deny/compiler+fuzz,
offline vendor, gerçek WASM ve shared p50/p95 gözlemi çalışır. Hard performans
eşiği ve izlenen baseline değişikliği yoktur. Raporlar 90 günlük özel
artefakta gider; erişim adresleri, tokenler ve ham özel metadata buraya alınmaz.

## Yerel doğrulama

İlk K-216 macOS faz matrisi: **773/773 geçti**, 0 kalan, 0 atlanan;
faz süreleri toplamı 195.902 sn. Clippy/fmt, katalog/tazelik, kaynak korukları,
13 sentetik gizlilik testi, YAML parse ve tam kaynak/çalışma ağacı Gitleaks
geçti. 407 commit yaklaşık 34.23 MB ve çalışma ağacı yaklaşık 11.63 MB
sır taramasında sıfır bulgu verdi. İki yeni CI güven sınırı testi tedarik
fazının birincil sahibindedir; fixture ve türetilen belgeler günceldir.

İlk tam turda dört kanıt özeti testi, GB-048 sonrası bayat türetilen sayfa
nedeniyle reddedildi. Zee üreticisiyle sayfa yenilendi; Rust/Go/Zee bayt
eşdeğerliği ve ardından tam tur geçti. Hata geçmişi olumlu sonuç sayılmaz.

## Açık kanıt

İlk başarılı exact Gitsrv koşusu ve uzak sonuç kaydı beklenir. GitHub hesap engeli
çözülmüş veya eski GitHub checks'i yeşile dönmüş sayılmaz. Linux koşusu
Windows runtime testlerinin veya RC fuzz/Miri/kurulum kapılarının yerine
geçmez. GitHub PR refs/cache destek teyidi ve yeniden açılış onayı açıktır.

K-216: anonim CI eski ölçüm kaynak nesnesini geri getirmez. Tarihçedeki
özgün SHA/süre/RSS/LOC değiştirilmez; kaynak soyu incelenmiş K-214/K-215
exact eşlemesiyle ve eşlenmiş commit'in HEAD atası olmasıyla doğrulanır.
Kayıt dışı, bozuk, çelişkili veya HEAD dışı eşleme reddedilir. Bu yeni
kaynakta yeniden ölçüm değildir. Hermetik vaka eski nesne yokken olumlu
ve ilgisiz/bozuk kaynakta olumsuz sonuçları korur.

İlk Gitsrv koşusu #20 (run id 751), exact b3b96122e4bf364b82c3204d45052eac6a8acedc
kaynağında **770/771**: yalnız tarihsel kaynak nesnesi bulunamadığı için
karşılaştırma provenance testi kaldı. Kaynak/Rust/koruk adımları geçti;
gizlilik/tedarik/WASM/performance adımları bu başarısızlıktan sonra
çalışmadı. Faz artefaktı incelendi; bu koşu geçiş sayılmaz.

Tarihsel kaynak kontrolü düzeltildikten sonraki macOS çalışma ağacı
matrisi **774/774 geçti**, 0 kalan, 0 atlanan; faz süreleri toplamı
220.087 sn. Katalog/tazelik, clippy/fmt, 13 gizlilik ve 4 sır istisnası testi
geçti. Bu yerel sonuç uzak Linux tekrarının yerine yazılmaz.

İlk Git2 push'u sonrasında depo Actions alanı kapalı görüldü. Yeniden
açılıp manuel başlatılan #20 gerçekten Gitsrv'de çalıştı; alanın
kapanmasına yol açan sunucu tarafı süreç belirlenmedi. Kalıcı otomatik
push tetikleme kanıtı açıktır. 13 Ekim 2026 22:32–22:38 İstanbul aralığında
tarayıcı/API HTTPS erişimi zaman aşımı verdi; yeni uzak koşu bu erişim
olmadan başlatılmış veya geçmiş sayılamaz.
