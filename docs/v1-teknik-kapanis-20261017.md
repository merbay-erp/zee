# V1 teknik kapanış — 17 Ekim 2026

Taban: `4f3caeeb7e44436f63777043da52473d02417ee0`.
Kullanıcı, kendisinin katılması/karar vermesi gereken işleri atlayıp teknik
işlerin sürdürülmesini istedi. Atlanan kapılar kapalı sayılmaz.

## Kapsam ve sıra

1. VPS3 canlı ürün envanteri ve exact ikili/source provenance farkı.
2. A03: mevcut UTF-16/NFC/morfolojili LSP davranışının birleşik regresyonları.
3. A04: aynı PostgreSQL backend'inde tenant bağlamının COMMIT/ROLLBACK temizliği.
4. Geri kalan teknik kapılar: mevcut kaynakta test/uyumluluk/dağıtım,
   yerel ürün korpusu, RC/fuzz/Miri/soak ve platform kanıtlarının gerçek sınırı.

## Kullanıcı/dış kanıt işleri — atlandı, açık

- 10 öğrenci/çocuk + 5 profesyonelle gerçek kullanılabilirlik oturumları.
- Hazır olmayan temiz fiziksel Windows ve minimum macOS cihazı.
- Lisans, kalıcı yayıncı güven anahtarı ve destek kapsamı kararı.
- GitHub özel destek yanıtı/public açma kararı; önceki güvenlik incelemesi
  engelleri yeni genel talep sayesinde aşılmış sayılmaz.
- 11 Kasım 2026'ya uzanan syntax freeze takvimi erkene alınmaz.

## Canlı ürün gerçeği

17 Ekim 2026 üretim sunucusu salt-okunur incelemesinde dört müşteri sitesinin ve
zeedili.com'un worker servisleri aktiftir. Dört proje bildirimi bulundu.
Bu ürünler uygulama yokluğu olarak yazılamaz. Anonim tam korpus ve exact
kaynak/ikili kabul kaydı farklı kanıtlardır.

- Ortak `dil` ikilisinde yeni `zee-build-v1` kaynak kimliği kaydı bulunmadı.
- Site başına `dil-<site>` bağlantıları be5699c ikilisine işaret eder; source SHA
  eşliği henüz doğrulanmadı.
- Canlı veri, oturum, sır veya uygulama servisi bu incelemede değiştirilmedi.

## Durum

K-312/A03: yeni dört birleşik LSP testi geçti. Mevcut LSP test grubunda
25 geçti / 0 kaldı / 1 tarihsel ölçüm ignored. LF/CRLF, iki emoji sonrası
Türkçe ekli rename, bağımsız UTF-16 edit uygulaması, NFD ad/kaynak reddi ve
surrogate ortası atomik paket reddi birlikte sınandı. Production kaynak,
grammar, tanı veya API değişmedi. Bu dört test gerçek IDE/ürün kabulü değildir.
A04 ve sonraki teknik kapılar yürütülüyor; sonuç olmadan kapanış yok.
V1 etiketi, kullanıcı/dış kapılar açıkken bu çalışmanın sonucu sayılmaz.

## K-312 kapı sonucu

Kanonik faz 1002 geçti / 0 kaldı / 25 atlandı. Cargo all-targets 1001 geçti / 0 kaldı / 25 atlandı; ayrı doctest 1/1 geçti.
Katalog+tazelik 7/7 ve rustfmt geçti. İlk özel target denemesi iki eksik
ortam önkoşulunda kaldı; kanonik dizinle tekrar geçti. Hatalı tur başarıya
katılmadı. [Makine kaydı](incelemeler/v1-teknik-20261017/k312/sonuc.json),
[faz sonucu](incelemeler/v1-teknik-20261017/k312/faz-sonucu.md).

## Mevcut ürün kaynakları — özel hazırlık kabulü

Beş site ve çekirdeğin özel kaynak kopyaları mevcut aday derleyicide temiz
denetlendi; toplam 248 hermetik test geçti (50+65+50+31+15+37).
Çekirdeğin katalogları mevcut resmi üreticiyle özel kopyada üretildi.
Bu ilk tur kirli test çalışma ağacından üretilmiş ikiliyle hazırlıktır;
temiz commit'li tekrar ayrı kaydedilecektir. Müşteri kaynakları depoya alınmadı.
Kullanıcının açık izniyle alınan dört canlı `proje.dil` SHA-256 özeti özel
kopyalarla eşleşti; bu bütün kaynak ağacının veya çalışan compiler SHA'sının
kanıtı değildir. Tam anonim P08 korpusu/ürün adoptionı kapanmış gösterilmez.
