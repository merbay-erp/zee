# K-297 LSP UTF-16 doğruluk dilimi — 15 Ekim 2026

Kaynak öncesi taban `cf2784e260c3b800e3d14482abc6891bcd1163a8`.
[ADR-153](../adr/153-lsp-utf16-konum-siniri.md) ve
[spec/30](../spec/30-editor-analizi-ve-lint.md) aynı değişikliktedir.
Kaynak düzeltme SHA: `5b910fccad8e694fd26fabb58e1cc39adbfd65ea`. Minimal
`regression/lsp/bmp-disi-konum.dil` vakası aynı SHA
`fixed_by` ile izleyen provenance değişikliğinde kayıtlıdır. Son tam faz
**996/1021 başarılı, 0 başarısız, 25 varsayılan dışı** sonuçla geçti.
K-297 dar correctness dilimi kapalıdır; tam P19/V1 kabulü değildir.

## Değişiklik

Emoji gibi BMP dışı karakterlerden sonra hover/definition/rename imleci
ve tanı/düzenleme aralığı UTF-16 olarak hesaplanır. İç compiler konumları
scalar kalır. LF/CRLF, çoklu emoji, geçersiz surrogate imleci, bağımsız
WorkspaceEdit uygulaması ve tek CR tanısı yedi yeni testle kapsanır.
Mevcut 39 LSP testi ve 20 artımlı LSP testi geçti. İlk kırmızı koşudaki
hover/rename başarısızlıkları korunur; yanlış tanı kodu beklentisi ayrı
belgelenmiştir. Kaynak hatası ile test beklentisi karıştırılmaz.

## İlk tam faz ve düzeltme

İlk tam faz 992/1021 başarılı, 4 başarısız, 25 varsayılan dışı vaka bildirdi.
Dört başarısızlığın tamamı yeni spec kayıtlarından sonra yenilenmemiş
`docs/kanit-ozeti.md` değerlerine bağlıydı: iki kanıt özeti ve iki Rust/Go
çıktı eşliği testi. Kaynak/worker testleri geçti. Özet kanonik Zee komutuyla
yenilendi; ilgili kanıt özeti/dogfood/Rust/Go testleri tekrar geçti. Son tam faz, exact kaynak
commit'ine bağlı regression manifesti eklendikten sonra kapanış kapısıdır.
Başarısız ilk koşu [kanıt arşivinde](incelemeler/k297/full-phase-before-provenance.md)
korunur. Rust/Cargo 1.93.1, Cargo jobs=1 ve RUST_TEST_THREADS=1 kullanıldı.

## Kapanış kanıtı

[Son tam faz raporu](incelemeler/k297/provenance/full-phase.md) 22 fazı,
[koşu kayıtları](incelemeler/k297/provenance/gates.json) hedefli ve tam
çalıştırmayı, [hash manifesti](incelemeler/k297/provenance/manifest.json)
arşiv bütünlüğünü gösterir. Compiler kaynakları ve testler kaynak commit'iyle
aynıdır; izleyen değişiklik fixture, provenance, üretilmiş özet ve kabul
belgeleridir. Önceki dört bayat özet hatası giderildi; worker dahil bütün
standart vakalar geçti. 25 ignored vaka bu yeni kaynakta ayrıca yürütülmedi.

Katalog/tazelik, spec drift, tanı kalitesi, fmt, clippy, katman/çevrim,
uyumluluk, public API, tedarik ve append-only conformance/regression/freeze
kapıları geçti. Zee/Rust/Go kanıt özeti çıktıları bayt bayt aynı kaldı.

## Kanıt sınırı

Bu normal LSP correctness düzeltmesidir. İlk kapanış uzun kampanya veya IDE
kabulü içermiyordu. İzleyen [S4 yerel kabulü](urunlesme-s4-kabul-20261015.md) exact
`4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2` üzerinde ayrı RC/soak, kanonik
macOS teslim/kurulum, on native stdio vakası ve gerçek IDE UTF-16 akışını
kaydeder. S3 sonuçları kendi kaynak kimliğinde kalır. Yeni cache, HIR/type
artımlılığı, tanı kodu veya hızlı düzeltme eklenmez; tam P19/V1 açıktır.

[S3 raporu](urunlesme-s3-kabul-20261015.md) tarihsel kaynak kanıtıdır.
[Kullanıcı girdileri](urunlesme-kullanici-girdileri.md) beş tam anonim
proje, insan çalışması, fiziksel cihaz, GitHub destek yanıtı ve ürün/staging
kabulünü ayrı listeler. Projeler henüz hazır değil, fiziksel cihaz yok.
C1/K-288 ve P33/K-293 önceki otomatik araç incelemesi retleri nedeniyle açık;
bu işler yeniden denenmedi ve kullanıcı veri eksikliği diye sınıflandırılmadı.
