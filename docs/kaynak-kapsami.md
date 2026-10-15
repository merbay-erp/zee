# Zee kaynak karar kapsamı

K-284/C5 profili gerçek Zee karar sonuçlarını izler. Normal çalıştırma
etkilenmez. Unix'te sahibi olduğunuz kanonik bir parent altında **yeni** rapor
seçin:

```sh
dil dene --kapsam /absolute/owned/dene.json proje/
dil çalıştır --kapsam /absolute/owned/native.json proje/
```

`çalıştır` için seçenek kaynak yolundan önce gelir; kaynak sonrasındaki
argümanlar uygulamaya aittir. Rapor var olan dosyaya yazılmaz. Süreç zorla
sonlandırılırsa başlangıçtaki `complete:false` dosyası korunur. Native worker
normal TERM/drain sonrasında raporu tamamlar. Kaynak ve binary Build kimliği
rapordadır; dirty candidate final temiz kaynak kanıtı sayılmaz.

`product.payda`, `vurulan`, `eksik` gerçek ürün karar sonuçlarıdır. `tests`
test gövdelerini ayrı sayar. Her if yüklemi, göre kolu, döngü gövdesi/normal
bitişi, açık deadline normal/timeout ve değerlendirilen mantıksal operand iki
sonuç taşır. Çağrılmayan prosedürler/rotalar eksik kalır. Kısa devrenin
çalıştırmadığı operand vurulmuş sayılmaz. Sıfır payda `not-applicable` olur.
Bu profil Rust branch coverage, tüm exception yolları ya da statement
coverage iddiası değildir.

JSON `zee-source-coverage-1` her sonucun kaynak kökü, sahibi, gerçek satırı,
karar çeşidi, sırası, boolean sonucu ve vuruş sayısını taşır. Sütun konumu
uydurulmaz. Aynı yükleyici snapshot'larının SHA256 kayıtları kaynak kimliğini
bağlar. Token, PHC, session, form/HTTP gövdesi ve kullanıcı değerleri rapora
girmez; kaynak yolları ve işlem isimleri metadata olarak görünür.

Bütçe:256 kaynak,16384 sonuç, derinlik256,2MiB rapor. Aşımlar veya bilinmeyen
AST probe'u başarılı/eksiksiz rapor üretmez. Kapsam kolektörü test thread'ine
aittir; normal/native yürütüm aynı son AST'yi borçlanır. Windows CLI rapor
sahipliği henüz doğrulanmadığından seçenek açıkça reddedilir.

[ADR-142](../adr/142-zee-kaynak-dal-kapsami.md) gerekçe ve sınırları,
[anonim C5 reproducer](../dogfood/zee-cekirdek/test-kapsami/README.md) özgün
ürün talebinin exact provenance'ını açıklar. Native kit entegrasyonu K-286
altında ayrı doğrulanır; disposable PG vaka izolasyonu commit edilmiş HTTP
isteklerinin topluca rollback edildiği anlamına gelmez.
