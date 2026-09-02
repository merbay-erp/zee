# ADR-025 — Tanı kimliği ve kod mezar taşı politikası

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-114, B-022, V1-P0-24

## Bağlam

Zee tanıları çocukların hata üzerinden öğrenme yüzeyi, editörlerin ise
makine-okunur karar API'sidir. Kaynak↔katalog birebirlik testi yeni ya da
kaldırılmış kodu yakalıyordu; fakat var olan bir `S004` gibi kodun katalog
özeti ve gerçek anlamı birlikte değiştirilirse iki taraf yine eşleşebilirdi.
Bu, bir editörün, ders içeriğinin veya otomasyonun aynı kodu sürümler arasında
başka bir olay sanmasına yol açardı.

Katalogdaki kullanılmayan A004 ve C014 yuvaları yalnız dipnotta yazılıydı.
S032 ise satır içi ayrılmış durumdaydı. Sessiz boşluklar, gelecekte yanlışlıkla
yeniden kullanılmaya karşı makinece yeterli koruma sağlamıyordu.

## Karar

Her derleyici tanısının kalıcı kimliği şu üçlüdür:

1. yayımlanmış `Ö###` kodu,
2. aile içinde benzersiz, küçük ASCII bir semantik anahtar,
3. katalogdaki kanonik “Ne oldu” özeti.

Şema-1 tabanı `compiler/tests/fixtures/tani-kimlikleri-v1.tsv` dosyasında
`durum, kod, kararlı_kimlik, kanonik_özet` alanlarıyla tutulur. Fixture kapısı
şunları birlikte zorlar:

- katalog ve fixture kod kümeleri birebirdir;
- kod, semantik anahtar ve durum tekildir;
- anahtar kod ailesiyle uyumludur;
- katalog özeti fixture'daki kanonik anlamla aynıdır;
- etkin/ayrılmış durumu sessizce değişemez.

Yeni bir semantik olay yeni kod alır. Eski olay kaldırılırsa kodu katalog ve
fixture'da `ayrilmis` mezar taşı olarak kalır; başka anlam için geri dönemez.
Özetin yalnız editoryal düzeltilmesi mümkündür, fakat fixture değişikliği aynı
commit'te açıkça görünür ve incelemede bunun anlam değişimi olmadığı
doğrulanır. Tüketicinin kod üzerinde dallanma kararını değiştirecek daraltma,
genişletme veya aile değişimi editoryal sayılmaz; yeni kod gerektirir.

Dinamik mesaj ayrıntıları, öneri metni, kaynak konumu ve işaret uzunluğu bu
kimliğin parçası değildir. Bunlar RFC-0010 sözleşmesine uyarak gelişebilir;
makine tüketicisi kararını kararlı kod üzerinden verir.

## Değişmezler

1. Yayımlanmış kod başka bir semantik olay için yeniden kullanılamaz.
2. Ayrılmış kod tekrar etkinleştirilemez.
3. S/A/T/C/D/P/Ç ailesi değiştirilemez.
4. Semantik anahtar benzersizdir ve küçük ASCII'dir.
5. Katalogdaki kanonik anlam fixture değişmeden değiştirilemez.
6. Yeni etkin tanı hem kaynak↔katalog hem katalog↔kimlik kapısından geçer.
7. Program değeri olan yapılandırılmış `Hata.kod` bu kayıtla karıştırılmaz;
   ADR yalnız derleyici/çalıştırıcı `Tani` kodlarını kapsar.

## Sonuçlar

- B-022 ve V1-P0-24 kapanır.
- A004, C014 ve S032 açık mezar taşıdır.
- Şema-1 tabanı 145 etkin ve 3 ayrılmış olmak üzere 148 kimlik taşır.
- K-127, bu append-only tabana P015 ve T054'ü ekledi; güncel şema-1 sayısı
  o adımda 147 etkin + 3 ayrılmıştır. K-129 aynı tabana S045 ve C023'ü ekledi;
  o adımda 149 etkin + 3 ayrılmıştır. K-130 değer/metin heap aşımı için C024'ü
  ekledi; o adımda 150 etkin + 3 ayrılmıştır. K-135 registry taşıma/cache/
  kalıcılık hatası P016'yı append-only ekledi; o adımda 151 etkin + 3
  ayrılmıştır. K-136 exact uzak bağımlılık bildirimi için P017'yi ekledi;
  güncel sayı 152 etkin + 3 ayrılmıştır. Eski kayıtların hiçbiri değişmedi.
- `katalog_testi.rs` kaynak↔katalog varlığını,
  `tani_kimligi_testi.rs` katalog↔sürüm kimliğini ayrı sorumluluklarla korur.
- Yeni tanı eklemek kasıtlı olarak kaynak, katalog, fixture ve ilgili davranış
  testini aynı atomik değişiklikte gerektirir.
