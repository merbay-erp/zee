# ADR-153 — LSP sınırında UTF-16 konumları

- **Durum:** kabul; K-297 dar correctness dilimi exact source/fixed_by ve 996/1021 tam faz (0 hata, 25 ignored) ile kapalı.
- İş: K-297; P19 ordinary LSP correctness dilimi. İlgili: ADR-019/035/145, RFC-0010, spec/30.

## Bulgu ve karar

`isim "Ayşe" olsun` ve sonraki `"😀" ile isim yaz` satırında UTF-16
imleci 12, scalar indeksi 11'dir. Eski hover bu imleci boşlukta arıyor;
rename, ikinci satırın [9,13) UTF-16 aralığı yerine [8,12) yayımlıyordu.
İki emoji kullanılan testte yanlış [9,13), doğru [11,15) aralığıdır.
Kaynak öncesi testte hover ve rename hataları doğrulandı. İlk tanı testi
yanlış S015 beklentisiyle kırıldı; bu sonuç tanı konumunun kırmızı kanıtı
sayılmaz. Düzeltilen test, mevcut core tanısının kodunu ve scalar (1,10,5)
aralığını ayrı doğrular; LSP aynı adı [11,16) olarak yayımlar.

Compiler/HIR/Tani sütunları Unicode scalar kalır. Gelen hover, definition ve
rename imleçleri mevcut eşitlemenin UTF-16→byte doğrulamasını paylaşarak
scalar indekse çevrilir. Surrogate ortası ve belge dışı sorgu null döndürür.
Definition/rename/tanı çıkışları açık belgenin metninden UTF-16'ya çevrilir;
tanı aralığı satır sonuna kısaltılır. Reddedilmiş kaynağın sentetik S045
bildirimi metin konumu iddiası taşımayan mevcut [0,1) aralığını korur.
Birim tanıları önce açık belgedeki kullan satırına taşınır, sonra çevrilir.
CRLF eşitleme davranışı korunur. LF öncesinde olmayan CR içerik olarak
sayılır; mevcut S001 tanısı ve onu silen UTF-16 edit aralığı testlidir. Tek CR
satır ayırıcı desteği ve alternatif positionEncoding
pazarlığı bu düzeltmenin kapsamında değildir.

[LSP'nin resmi Position tanımı](https://github.com/microsoft/vscode-languageserver-node/blob/main/types/src/main.ts)
encoding belirtilmediğinde UTF-16 varsayımını açıklar. Yeni protokol
capability'si, dil/grammar özelliği veya public Rust export'u eklenmez.

## Kanıt ve etki incelemesi

`lsp_artimli_testi` LF/CRLF, çoklu emoji, geçersiz imleç, tanı başlangıç/uzunluğu,
core scalar konumunun korunması ve WorkspaceEdit'in bağımsız UTF-16 vektörüne
ve gerçek didChange yoluna uygulanmasını sınar. `lsp-utf16` minimal regression
kipi aynı kaynaktan hover, definition ve iki imleçten aynı rename sonucunu
üretir. Kaynak commit'inden sonra exact fixed_by manifesti ve semantic/core-freeze
beyanları eklenir; bu takip tamamlanmadan bug kapanmış sayılmaz.

Production modül/katman sahibi veya sahipler arası bağımlılık eklenmez;
ADR-046 ve katman fixture'ı değişmez. Yeni cache/state/tampon yoktur;
ADR-069 yeni soak gerektirmez. Satır taraması geçicidir, performans iddiası
ve sabit benchmark iş yükü değişmez. ADR-045/049, ölçüm defteri ve kalıcı
performans tarihçesi ilk doğruluk diliminde yeni exact ölçüm olmadığı için
korundu. İzleyen S4 gözlemi ayrı kabul raporundadır; sabit taban eklenmedi. S3'ün RC/soak/
performans/IDE sonuçları yeni kaynak commit'ine taşınmaz.

Tanı kimliği, önerisi, parser kurtarması, uyumluluk yüzeyi, bağımlılık ve
derleme bayrakları değişmez; hata kataloğu, ADR-064/066/068, SBOM/provenance
üretimi ve güvenlik bulgu kaydı değişiklik gerektirmez. Tanı kalitesi,
katalog, uyumluluk ve tedarik testleriyle mevcut sözleşmeler denetlenir.
Dogfood ürün/korpus kaydı ve çıktı sözleşmesi değişmez. Spec ve kaynak
beyan defterleri değiştiğinden kanıt özeti kanonik Zee ürünüyle yenilenir;
Rust/Go byte eşliği testleri aynı değerleri doğrular. RFC-0010'un editör niyeti
korunur; yeni RFC ve öğretici komut değişikliği gerekmez. README canlı
sayıları kanonik araçla yenilenir. C1/K-288 ve P33/K-293 yeniden denenmez;
artımlı HIR/type, gerçek ürün ve UI/taşıma/lint gecikme kabulü açık kalır.

Kaynak düzeltme SHA: `5b910fccad8e694fd26fabb58e1cc39adbfd65ea`.
Minimal `bmp-disi-konum` fixture/manifesti ve compiler/core-freeze beyanları
izleyen değişikliktedir; [kabul raporu](../docs/lsp-utf16-kabul-20261015.md)
ilk kapanışın tam faz başarısını ve o aşamada RC/soak/IDE yapılmadığı sınırını
kaydeder. İzleyen [S4 kabulü](../docs/urunlesme-s4-kabul-20261015.md) yeni exact
kaynakta bu kampanyaları ayrıca doğrular; S3 tarihsel kanıtını taşımaz.

## K-298 kaynağında ordinary UTF-16 tazeliği

[K-298 kabulü](../docs/urunlesme-k298-kabul-20261016.md) a57290f kaynağında 10 native stdio LF/CRLF
olumlu/olumsuz vakayı ve gerçek IDE hover/definition/rename/A001 aralıklarını
ayrıca doğruladı. K-297'nin özgün fixed_by kaydı değişmez. Yeni IDE iki normal
Quit/exit0/reap içerir; ayrı IDE shutdown trace'i yakalanmadı. Native stdio
protokol kapanışı ayrı kanıttır. Dört stdio p95 gözlemi UI/gecikme SLA veya
artımlı HIR/type kabulü değildir.
