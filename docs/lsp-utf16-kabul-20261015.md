# K-297 LSP UTF-16 doğruluk dilimi — 15 Ekim 2026

Kaynak öncesi taban `cf2784e260c3b800e3d14482abc6891bcd1163a8`.
[ADR-153](../adr/153-lsp-utf16-konum-siniri.md) ve
[spec/30](../spec/30-editor-analizi-ve-lint.md) aynı değişikliktedir.
Kaynak/provenance SHA ve tam faz sonucu kapanışta buraya kaydedilir.

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
yenilendi; ilgili testler tekrar denetlenir. Son tam faz, exact kaynak
commit'ine bağlı regression manifesti eklendikten sonra kapanış kapısıdır.
Başarısız ilk koşu [kanıt arşivinde](incelemeler/k297/full-phase-before-provenance.md)
korunur. Rust/Cargo 1.93.1, Cargo jobs=1 ve RUST_TEST_THREADS=1 kullanıldı.

## Kanıt sınırı

Bu normal LSP correctness düzeltmesidir. S3 kaynak/RC/soak/performance/IDE
kampanyası kendi exact SHA'sında kalır; yeni kaynakta uzun kampanya veya
IDE kabulü iddia edilmez. Yeni cache, HIR/type artımlılığı, tanı kodu veya
hızlı düzeltme eklenmez. Tam P19 ve V1 açık kalır.

[S3 raporu](urunlesme-s3-kabul-20261015.md) tarihsel kaynak kanıtıdır.
[Kullanıcı girdileri](urunlesme-kullanici-girdileri.md) beş tam anonim
proje, insan çalışması, fiziksel cihaz, GitHub destek yanıtı ve ürün/staging
kabulünü ayrı listeler. Projeler henüz hazır değil, fiziksel cihaz yok.
C1/K-288 ve P33/K-293 önceki otomatik araç incelemesi retleri nedeniyle açık;
bu işler yeniden denenmedi ve kullanıcı veri eksikliği diye sınıflandırılmadı.
