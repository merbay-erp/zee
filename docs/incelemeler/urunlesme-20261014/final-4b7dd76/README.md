# S4 seçili kanıt arşivi

Test kaynağı `4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2`; takip commit belge/kanıt kaydıdır.
[Sonuç ve açık sınırlar](../../../urunlesme-s4-kabul-20261015.md).

[Ham kaynak/temsil haritası](RAW-SCOPE.json) seçili makbuzların dış
yolunu ve SHA-256 değerini verir. JSON makbuzları, açıkça işaretlenen
RC kapısı türetilmiş temsili dışında özgün byte kopyalarıdır. RC temsili
tek kaynak dosyası SHA-256 alanını ayrı bağlar; ters dönüşüm özgün byte
dizisini verir. Bu değişiklik tarama kuralı veya istisna eklemez;
ham metin/sürücüler UTF-8 metni, uzunluğu ve hashini taşıyan tersine
çevrilebilir JSON zarflarıdır. Sondaki boşluk/satırlar korunur. Tam
fuzz korpus/build/run/kapı günlükleri, ölçüm ham frame dosyaları, ikililer,
ZIP ve bundle dış kanıt kökünde korunur; bu Git altkümesinden bütün makbuz
referansları doğrulanamaz. Hash kimlik doğrulama değildir.

IDE JSON küratörlü araç/UI gözlemidir; tam ham trace veya yeni screenshot
dışa aktarımı sayılmaz. RC v2 süre ayrışması ve ilk Miri eksikliği başarısız
ön denemeler olarak korunur. S3 native/Linux/PG kabulü bu kaynağa taşınmaz.
