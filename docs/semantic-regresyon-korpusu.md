# Semantic regresyon korpusu

K-147/ADR-044, düzeltilmiş bir Zee bug'ını genel test toplamında kaybolan bir
anı olmaktan çıkarır. K-155/ADR-052 bunu exact düzeltme commit'i ve garanti
sürümüyle kaynak soy ağacına bağlar. Her kayıt “hangi bug, hangi commit, hangi
sürüm, hangi faz, hangi küçük kaynak ve hangi kesin gözlem?” sorularını
makine-okunur cevaplar.

## Yerleşim

```text
regression/
├── v2.tsv
├── parser/
├── checker/
├── hir/
├── runtime/
├── morphology/
├── concurrency/
├── security/
├── cli/
└── lsp/
```

Başlangıç tabanı yedi fazda 17 vaka taşır. Her `.dil` dosyası manifestte tam
bir kez bulunur; manifest dışı veya iki kez sahiplenilmiş kaynak test
kapısını kapatır. CI ayrıca taban Git revizyonundaki vaka kimliği, K-kimliği,
dosya yolu ve provenance zincirinin silinmesini ya da başka anlama taşınmasını reddeder.
Kaynak/beklenti bilinçli bir dil göçü nedeniyle değişirse ilgili RFC/spec ve
sürüm notu aynı committe güncellenir; vaka soy ağacı korunur.

## Manifest sözleşmesi

`regression/v2.tsv` on iki sekmeli alandan oluşur:

1. küçük ASCII kebab-case vaka kimliği;
2. düzeltmenin `kararlar/gunluk.md` içinde var olan `K-NNN` bug kimliği;
3. düzeltmeyi taşıyan tam 40 haneli `fixed_by` Git SHA;
4. kanıtlandıysa tam `introduced_by` SHA, bilinmiyorsa `-`;
5. güvencenin başladığı SemVer serisi (`0.8.0-dev`);
6. birincil faz;
7. çalışma kipi;
8. beklenen tanı kodu, başarı için `-`;
9. kesin `satır:sütun:uzunluk`, başarı için `-`;
10. beklenen exit;
11. `\\n` ile ayrılmış sıralı çıktı, çıktı yoksa `-`;
12. depo köküne göre `.dil` yolu.

Tanı ile span birlikte bulunur. Derleme, parser, policy veya runtime hatası
`exit=1`; normal runtime kendi `programı N ile bitir` değerini taşır. Başarılı
programda stdout satır sırası byte düzeyinde sabittir.

K-221 CLI vakası `cli` fazı ve `cli` kipindedir: ilk yorumdaki
`# cli-argumanlari: ` sonrasında boşlukla ayrılmış sabit argümanları gerçek
`dil` ikilisine verir. Fixture hermetik bilinmeyen seçenek reddini veya yerel kaynakta program
argümanlarının başarılı aktarılmasını çalıştırır; stdin, dış ağ, özel
dosya ya da kabuk yürütmez. Beklenti alanı
bu kipte varsayılan stderr satırlarıdır, stdout boş ve exit 2'dir. Explicit
`# cli-cikti: stdout` başarılı CLI vakalarında stdout satırlarını seçer;
startup metadata taşıyan stderr bu alt türde çıktı beklentisi değildir.
Alt tür aynı v2 manifestinde normal exit ve exact stdout satırlarını
korur; yeni compiler CLI seçeneği veya dil profili açmaz. CLI kullanım
hatası kaynak tanısı üretmediği için tanı/span alanı `-` kalır.

## Kipler

| Kip | Çalışan sınır |
|---|---|
| `parser` | Lexer ve fail-fast parser |
| `diagnostics` | Kurtarmalı parser + çoklu checker tanı yüzeyi; vaka tam bir tanı ister |
| `compile` | Resolver, checker ve typed-HIR üretimi |
| `hir` | Compile sonrası açık bağlı-HIR invariant denetimi |
| `run` | Invariantı geçmiş typed HIR ile runtime; stdout ve program exit'i korunur |
| `capability` | Compile sonrası kapalı yetkinlik politikası |
| `smtp` | Security fazında typed runtime → gerçek native SMTP; yalnız sabit loopback fixture |
| `lsp-utf16` | LSP mesaj işleyicisinde BMP dışı önek sonrası hover, definition ve iki imleçten aynı rename aralıkları; süreç/taşıma kabulü değildir |
| `lsp-uri` | LSP fazında gerçek stdio ikilisi, yüzde UTF-8 dosya URI'si ve hermetik disk birimi |

K-248 `lsp-uri`, iki satırlık kaynağı ve aynı dizindeki `yardimci.birim`
girdisini geçici Türkçe/emoji klasörüne kopyalar. Initialize, didOpen,
shutdown/exit çerçeveleri gerçek `dillsp` sürecine gider. Beklenti sıralı
tanı kodlarının kanonik JSON dizisi (`[]` başarı), gerçek exit ve boş
stderr'dir; manifest tanı/span alanları `-` kalır. URI echo ve üç tam
yanıt ayrıca doğrulanır; süreç 10 saniyede kapanmazsa öldürülüp beklenir.
Bu kip salt CLI derlemesiyle taklit edilmez.

## Yeni bug düzeltme protokolü

1. Arızayı en az satırlı, bağımsız `.dil` kaynağına indir.
2. Kaynağı sahibi olan faz klasörüne vaka kimliğiyle koy.
3. Davranış düzeltmesini ayrı, yeşil bir commit olarak al; tam SHA'yı kaydet.
4. `v2.tsv` satırında bug, `fixed_by`, kanıtlıysa `introduced_by`, garanti
   sürümü, faz, kip, tanı+span, exit ve çıktıyı kaydet.
5. Fixture/provenance commit'inde ilgili RFC/spec/ADR ve sürüm notunu güncelle.
6. Önce hedefli, sonra tam faz kapısını çalıştır:

```bash
cd compiler
cargo test --locked --test semantic_regresyon_korpusu_testi
cargo run --locked --bin faz_test_matrisi -- --denetle \
  --rapor target/faz-test-matrisi.md
bash ../scripts/semantic-regresyon-korugu.sh HEAD~1
```

Fixture en çok 4 KiB ve 32 dolu satırdır; CR, tab ve eksik son LF reddedilir.
Bu sınır bir kod golfü hedefi değil, tek arızayı açıklayan küçük örneği koruma
kapısıdır. Daha büyük senaryo golden veya conformance korpusuna aittir.

Manifest testi tam SHA biçimini, commit varlığını, `fixed_by`→HEAD ve varsa
`introduced_by`→`fixed_by` ata yönünü denetler. İlk 17 tarihsel vakada
minimal reproducer düzeltme anında mevcut olmadığı için introduced commit
tahmin edilmemiş, açıkça `-` bırakılmıştır. Sonradan bisect ile kanıtlanan
değer yalnız `-`→ata SHA yönünde bir kez eklenebilir. K-155A/ADR-053 ile koruk,
`1c73298…` sonrasındaki her `compiler/src` commit'ini mesajından bağımsız
olarak [semantic değişiklik beyanında](compiler-semantic-degisiklik-beyani.md)
tekil sınıflandırır. `semantic-bugfix` aynı SHA'yı `fixed_by` taşıyan fixture,
`semantic-change` normatif belge, `maintenance` ise açık gerekçe ister.
`semantic_regresyon_korugu_testi`, geçici gerçek Git deposunda masum görünen
başlığın beyansız reddini, exact beyan+fixture kabulünü ve provenance yeniden
yazım reddini uçtan uca çalıştırır; yalnız script metnine bakmakla yetinmez.
K-160A sonrasında ayrı [core freeze beyanı](core-freeze-beyani.md), semantic
değişikliğin gerçek dogfood/security/correctness provenance'ı olmadan sırf
normatif belge eklenerek geçmesini engeller.

## Korpusların rolleri

- `regression/`: geçmişte düzeltilen somut Zee bug'larının küçük yeniden
  üretimleri;
- `golden/`: öğretici, kullanıcı yüzeyli uçtan uca programlar;
- `conformance/`: başka gerçeklemelerin de tüketebileceği sürümlü davranış
  profilleri;
- `compiler/fuzz/corpus/`: hasım veya üretilmiş girdi başlangıç tohumları.

Bu alanlar birbirinin yerine geçmez; K-146 faz raporu aralarındaki bağı aynı
satırda görünür tutar.

### ürün E özelliklerinin ana kaynak hattı — K-213

[ADR-086](../adr/086-site-hatlarinin-birlestirilmesi.md) mevcut ürün E
parola üretimi/TOTP/SMTP, rota birimi, görünüm, görsel ve gizli değer
yüzeylerini K-208/K-209 bcrypt/IP ile aynı hatta toplar. Ürün kabulü
dosya kontrolüyle değil bütün proje `dil dene .` ile alınır. Ham ürün E
şablonu normal katalog üretimini ister. Exact source/CI kapanışı beklenir;
canlı dağıtım ve otomatik parola göçü bu işte yapılmaz.

K-213/ADR-086 çok-kaynaklı HIR vakasının aynı dizindeki `.birim` girdileri
hermetik birim yükleyici tarafından okunur. Birim kaynakları ağ/ürün deposuna
bağlı değildir. `trace` runtime kipi, kayıt → kanonik şema okuma → replay
zincirini aynı stdout/exit ve tam tüketimle doğrular.

`definition` HIR kipi, kullanıcı sembollerinin definition kaydını
her okuma/yazım SymbolId için ayrıca ister; görünüm LHS eksikliği yalnız
ifade invariantıyla yakalanamaz. K-213 vaka kimlikleri silinmez.

## SMTP security kipi (K-236/K-237/K-252)

`smtp`, `security` fazına aittir. Kaynakta tam bir `# smtp-vaka: auth-echo`
veya `# smtp-vaka: yanit-siniri` veya `# smtp-vaka: yavas-yanit` yorum satırı bulunur; duplicate/unknown
başlık reddedilir. [Hermetik adaptör](../compiler/tests/semantic_regresyon/smtp.rs)
serbest loopback portunda tek gerçek `eposta::gonder` çağrısı yapar; sabit
sentetik credential kullanır, dış SMTP URL’sini/ortamı çalıştırmaz. Manifest
beklentisi stdout ve exit’tir; yakalanan C029 Sonuç değeri kaynak tanısı
olmadığı için tanı/span `-` olabilir.

K-236 [AUTH echo reproducer’ı](../regression/security/smtp-auth-echo-sir.dil)
EHLO sonrası aldığı AUTH PLAIN payload’ını 535 ile echo eden gerçek peer’i
kullanır; sonraki AUTH/mail/data yokluğu ve sabit C029 maskesi denetlenir.
`yanit-siniri` fixture’ı 16385 bayt LF/EOF’siz greeting ve 50 ms yeni baytla
read timeout’ını canlı tutar; yalnız gerçek çağrı döndüğünde kapanır. Eski
sınırsız yardımcı 5 saniyelik fixture son tarihini kırar. K-237'nin zamansız
bayt/tampon sınırı beş actualhelper testinde ayrıca denetlenir; fixture zamanı
benchmark veya eski plain toplam oturum deadline garantisi değildir.
ToplayanIo yalnız diğer trait yöntemlerini taşır, SMTP başarısızlığı gerçek
native socket çağrısından gelir. Bu kip canlı sağlayıcı veya ürün kabulü değildir.

K-252 `yavas-yanit` fixture'ı 200ms native bütçede her40ms tek baytla
LF-siz yanıt verir; bir saniye toleransı, tek gerçek bağlantı ve C029
maskesini sınar. Eski idle-timeout yolunun gerçek3011ms önce-kaydı ve
çalışma ağacı201ms sonrası [yerel kanıttadır](incelemeler/urunlesme-20261014/k252-smtp/README.md).
Üç saniye bounded fixture cleanup'ı benchmark veya DNS iptali değildir.


K-259 için `security` fazındaki `native-listener-deadline` kipi gerçek sahip olunan
Unix listener/control pipe uçları ve native `dil` ikilisiyle geç EOF sınırını
çalıştırır. Yardımcıya `--fixture-source` ile manifestin minimal `.dil` kaynağı
verilir; parser-only taklit yoktur. Beklenen exit gerçek liderin WNOWAIT terminal
kodudur (`0`); bu kipte çıktı kanalı HTTP yanıt baytlarıdır ve boş olmalıdır.
Tek nesil, tüketilmiş prefix sonrası mutlak deadline üst sınırından geç EOF,
ebeveyn listener korunumu ve sahip olunan süreçlerin reap edilmesi ayrıca
doğrulanır. İlk henüz commitlenmemiş özellik adayındaki kusur için tarihsel
`introduced_by` uydurulmaz (`-`); `fixed_by` ilk düzeltilmiş özellik kaynak
commit'inin exact SHA'sıyla izleyen provenance commit'inde kaydedilir.
Bu destek production CLI/API eklemez; Unix dışı native kanıt üretilmiş sayılmaz.

Eski toplu semantic replay native-listener-deadline kayıtlarını çalıştırmaz;
diğer eski vakaları bütün platformlarda korur. Gerçek fixture ve exact SHA
kaydıyla aynı follow-up commit'inde eklenen ayrı native libtest vakası
Unix'te tüm native kayıtları seçer ve boş listeyi reddeder. Unix dışında bu
tek test `actual Unix owned FD capability unavailable` gerekçesiyle görünür
ignored durumundadır; eski toplu test ignored yapılmaz. `--ignored` ile
zorlandığında adaptör desteklenmeyen platformda panic eder; sahte başarı yoktur.

## Native metric stdio ret kipi (K-294)

`native-metric-stdio` yalnız `cli` fazında kullanılabilir. Manifest uygulama
exit2, boş çıktı ve tanı/span yokluğu ister. Kaydedilmiş minimal `.dil`,
`--fixture-source` ile gerçek owned FD helper'ına verilir ve aynı baytların
SHA256 kaydı doğrulanır. Olumlu metric FD ile kaynak gerçekten çalıştırılır;
stderr/stdout pipe alias'ları ise kaynak yürütülmeden sessiz reddedilir.
Ayrı CLI hata kontrolü olağan tanıların susturulmadığını doğrular.

Portable aggregate bu native kipi atlar. Exact kaynak düzeltme SHA'sını
izleyen gerçek fixture/manifest commit'i nonempty dedicated Unix testini
birlikte ekler; Windows için görünür ignored vaka vardır, native başarı
uydurulmaz. Bu ilk support commit'inde manifestte K-294 satırı/placeholder
ve regression altında sahipsiz `.dil` bulunmaz. Mevcut vaka kimlikleri ve
provenance alanları değiştirilmez.

K-294 takip kaydı artık `native-metric-stdio-alias` fixture ve exact
`fixed_by=382beed1d01440f6f9df28b5ac7a111fbe1f75ee` taşır.
Nonempty dedicated Unix libtest gerçek kaynağı ve observed exit2/boş kanal
sonucunu yürütür; kaynak commitinden sonraki provenance diliminde eklenmiştir.
