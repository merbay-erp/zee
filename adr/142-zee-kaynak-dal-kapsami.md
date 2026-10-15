# ADR-142 — Opt-in Zee kaynak karar kapsamı

- **Durum:** [S3](../docs/urunlesme-s3-kabul-20261015.md) (`66e4149e53451111d153f47283c219079a8d1cab`): üç CLI/native kohort ile dar yerel kabul; üretici candidate etiketi korunur, temiz kaynak/kanonik ikili bağı dışarıdan doğrulandı; genel kapsama/ürün kabulü açık.
İş: K-284. Ürün: etkin `zee-cekirdek`; özgün C5 talebi
`cd5d12f0008620819e6167af5146e83503fd2b38` commit'indeki
`docs/dil-talepleri/2026-10-14-zee-urunlesme.md:163–167`.

## Gereksinim ve minimalite

Ürün login/logout ve panel testlerini `dil dene` içine almak ve gerçek Zee
kaynağının eksik dallarını görmek istiyor. Rust test kapsamı bu ihtiyacın kanıtı
değildir. Anonim depo içi reproducer
[ürün kökü](../dogfood/zee-cekirdek/test-kapsami/README.md) altındadır;
özel proje gövdeleri, hesapları ve hizmetleri kopyalanmaz.

## Karar

`dene` ve `çalıştır` açık `--kapsam /abs/fresh.json` seçeneğinde son derlenmiş
Program üzerinde karar envanteri kurar. Kapalı profilde kolektör oluşturulmaz;
probe yalnız thread-local kapalı durumunu kontrol eder, dosya/heap işi yapmaz.
Kaynak hash'leri ayrı yeniden okumadan, derleyicinin yüklediği aynı snapshot
stringlerinden çıkarılır. Ürün ve test gövdesi paydaları ayrıdır.

Profil `source-decision-outcomes-1`: her if yüklemi, göre karşılaştırması,
döngü gövdesi/normal tükenişi, açık süre bloğunun normal/timeout sonucu ve
kısa devrede gerçekten değerlendirilen her mantıksal operandın iki olası
sonucu sayılır. Çağrılmayan işlemler ve rotalar envanterdedir. İstisna türleri,
Rust/HTTP güvenlik kapılarının iç dalları ve statement kapsamı bu paydada yoktur.
Sıfır payda yüzde100 yerine `not-applicable` olur.

Kalıcı kimlik sıralı kaynak/owner/karar/sıra/sonuç kaydıdır; bellek adresi
rapora girmez. Süreç içi probe tablosu son AST'nin borçlanılmış düğümlerini
kullanır. Native rota ve prosedür yürütümü bu aynı düğümleri borçlanır.
Envanterden sonra başka AST klonunu yürütmek bilinmeyen probe ile C000 üretir;
bitmiş başarılı rapor verilmez. Klon kendi envanteriyle çalıştırılabilir.

## Sınırlar ve güvenlik

En fazla256 kaynak,16384 karar sonucu, derinlik256 ve2MiB rapor bütçesi
uygulamadan önce denetlenir. Sayaçlar taşmada doyar. Rapor yalnız mutlak,
kanonik sahipli parent altındaki yeni normal0600 dosyada oluşturulur;
var olan dosya/FIFO/symlink açılmaz. Başlangıç receipt'i `complete:false`
taşır; derleme/erken çıkış bunu başarıya çeviremez. Unix dışı CLI dosya
sahipliği profili açıkça reddedilir; kütüphane kolektörü portable'dır.

Kapsam çıktısı token, form gövdesi, oturum, SQL, DSN veya değer kaydetmez;
kaynak yolları/işlem adları görünür metadata'dır. Kullanıcı yolu kendi açık
tercihidir. Facade `api::v1`, grammar ve normal stdout/exit sözleşmesi değişmez.

## Kanıt ve etki

`kaynak_kapsami_testi` gerçek Zee yürütümü, kısa devre IO eşitliği,
çağrılmayan payda, thread/drop ve bilinmeyen klonu sınar. Native fixture
HTTP/session/CSRF ve iki gerçek login dalını sayar; guard'da reddedilen istek
Zee dalına vuruş yazamaz. İlk candidate `9214a3e` dirty=true kanıtı final
clean kaynak kabulü değildir. K-286 ayrı, additive native suite köprüsüdür;
PG vaka izolasyonu tüm HTTP isteklerini tek transaction'da rollback etmez.

Geçici, bounded ve test ömrüyle sınırlı durum eklenmiştir; ADR-069/soak etkisi
final kaynak kapılarında incelenir. Public facade genişlemesi ve kalıcı cache
yoktur. Katman yönü yorumlayıcı→test_kapsami→agac/tani; ters kenar gerekmez.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.
