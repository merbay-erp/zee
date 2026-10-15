# ADR-145 — Artımlı editör eşitlemesi, ayrıştırma önbelleği ve lint

- **Durum:** S3 ve S4 kendi exact kaynaklarında dar kabul taşır; K-298 bildirim ömürlü dizin bakımının yeni kaynak ölçümü ve kabulü açık. Artımlı HIR/type, UI/gerçek ürün/SLA kabulü açık.
- İş: K-287; K-298 semantic davranışı koruyan performans bakımı; ürün talebi: zee-cekirdek C2/P19.
- İlişkili kararlar: ADR-019/035/069; RFC-0010; spec/30.

## Gereksinim ve karar

K-280 gerçek JetBrains tamamlama/kapanışını eski temiz kaynak üzerinde
kanıtladı. Bu karar yeni editör davranışını ayrıca sınar; o kanıt yeni kaynak
sürümüne otomatik taşınmaz. Editör `textDocumentSync: 2` ile UTF-16 aralıklarını
gönderir. Bir değişiklik paketinin bütün aralıkları özel aday üzerinde
doğrulanır; son aralık bozuksa önceki aralıklar da belgeye işlenmez.

`textDocument/formatting` mevcut kanonik biçimleyiciyi kullanır. Dosyaya yazmaz;
tam belge düzenlemesi döndürür. Bozuk/kapanmış belgeye biçim tahmini yapmaz.
`dil biçimle --denetle` davranışı değişmez.

## Yeniden kullanım ve sahiplik

Oturum `faz` sahibindedir. Lexer, parser'a bağımlı yapılmaz. Token LRU'su exact
köken ve kaynakla anahtarlanır, en çok 64 girdi ve hesaplanan 8 MiB token/anahtar
tahsis bütçesi tutar. Alınan tokenlar ayrı kopyadır. İthalatı olmayan yaprak
birimin ayrıştırılmış, henüz bağlanmamış AST'si ayrıca yeniden kullanılabilir:
64 girdi, toplam 1 MiB kaynak/köken ve 128.000 kaynak tokenı sınırı vardır.
AST sınırı toplam RSS veya exact heap baytı garantisi değildir.

Paket grafiği, kilit ve yükleyici her analizde yeniden doğrulanır. İthalatlı
birimde tohum işlem adları yeniden kurulur ve ayrıştırma yeniden yapılır.
Hoist, ad çözümü, türler ve HIR önbelleğe alınmaz. Aynı dosyanın değişen metni
eski ayrıştırma sonucunu kullanamaz. Belge kapanınca iki önbellek de bırakılır.

Diskten doğrulanan ana proje kaynakları, aynı gerçek dosya editörde açıksa
kaydedilmemiş metinle analiz edilir. Bir bağımlılık proje kökünün altında olsa
bile ana kaynak sayılmaz. Çelişen URI alias'larından rastgele metin seçilmez.
Paket kaynaklarını veya kilit hatasını açık editör metniyle örtmek mümkün
değildir. Başarılı açma/değiştirme ve kapatma bildirimleri mevcut sınırlı açık belge
envanterini yeniden analiz eder: değişen URI önce, diğerleri kararlı URI
sırasındadır. Böylece tüketici kökün tanısı yeni bir kök düzenlemesi beklemez.
Kalıcı ters grafik eklenmez. Bir bildirimdeki tanı gövdelerinin toplamı mevcut
8 MiB yanıt bütçesini aşarsa -32001 üretilir; kısmi başarılı yayın gösterilmez.

## İsteğe bağlı uyarılar

`dil denetle --lint` ve LSP `initializationOptions: {"lint": true}` mevcut
başarılı çözümleme üzerinde uyarı üretir. Uyarı derlemeyi başarısız yapmaz;
normal denetim aynı kalır. HIR'daki gerçek okumalar yeniden atamadan ayrılır.
Köken ve semantic kapsam farklı dosyalardaki aynı satır/adları ayırır.
Erişilemeyen kod yalnız kesin dönüş/sonlandırma veya literal boolean dalıyla
kanıtlanır. Yanıt göndermek akışı bitiren bir komut sayılmaz. Çağrının yanıt
etkisi bilinmiyorsa eksik yanıt uyarısı tahminle verilmez. İfade HIR
bağlarında herhangi bir kullanıcı işlem çağrısı varsa S048 program boyunca
bastırılır; bu muhafazakâr sınır bazı gerçek eksik yanıtları da kaçırabilir.

Gölgeleme mevcut dilde yapısal olarak yoktur (spec/03): dış ada atama aynı
SymbolId'yi değiştirir. Bunun için sahte bir gölgeleme uyarısı eklenmedi.
Yeni grammar veya desteklenen `dil::api::v1` export'u yoktur. Ortak, isteğe
bağlı rota kökeni `agac` sahibindedir; coverage ve lint birbirine bağımlı olmaz.

## Kanıt ve açık sınırlar

`lsp_artimli_testi`, `lint_testi` ve `faz::oturum::testler` değişiklik atomikliği,
UTF-16, cache kopya yalıtımı, kaydedilmemiş birim, disk/kilit değişimi ve lint
kimliği/origin ayrımını sınar. İlk 20.081 satırlık aday ölçümünde p95 233,586 ms
çıktı ve 200 ms hedefi karşılanmadı. Yaprak AST sonrası ikinci adayda p95
273,285 ms oldu; iki başarısız hedef de korunur. Asıl maliyet HIR sembol yazımı
listesindeki her ekleme için doğrusal üyelik aramasıydı. Aynı sıralı Vec çıktısını
koruyan ayrı HashSet üyeliği sonrası aday p95 19,963 ms, en yüksek 20,348 ms,
ilk analiz 32,076 ms oldu (5 ısınma, 20 örnek). Bu ölçümler kirli adaydan
gözlemdir; temiz exact commit benchmark tarihçesi değildir. Üretilen iş yükü
gerçek ürün korpusu veya bütün projeler için gecikme garantisi değildir.

HIR üyelik kümesi tek bağlama süresince yaşar; oturumda saklanmaz. Sıralı
sembol yazımı ve semantic kimlik çıktıları mevcut HIR/kimlik testleriyle korunur.
ADR-045/049 ölçüm politikası değişmez; kalıcı performans tabanı ancak temiz
exact kaynak, makine ve gerçek örnek dağılımı ile ayrıca yazılabilir.

Yeni kalıcı oturum nedeniyle temiz exact SHA'da 1800 saniye soak zorunludur.
Temiz kaynak RC fuzz/Miri, katman/SCC, lint, katalog/spec ve belge kapıları
tamamlanmadan bu iş tamamlandı sayılmaz. Eklenti yayın kanalı ve fiziksel
Windows kabulü ayrı açık sınırlardır.

Açık tüketici tanılarının otomatik yayını eklendikten sonraki kirli aday
release tekrarında aynı20.081 satır/40 birim ve5ısınma/20örnekle tek açık
belge p95 19,754 ms (max19,926; ilk29,627),41 açık belge p95 126,164 ms
(max130,027; tüm belgelerin sırayla ilk açılışı2712,895 ms) gözlendi.
Ölçüm taze grafik/kilit/yükleme/tür analizi ve tanı serileştirmesini içerir;
editör taşıması, UI ve lint açık kipi kapsamaz. Ham başarısız ilk adaylar
korundu; kalıcı performans tarihçesine kirli kaynak sonucu yazılmadı.

## S3 kanıtı

[İkinci dalga exact S3 kabulü](../docs/urunlesme-s3-kabul-20261015.md) bu kararla ilgili actual kohortu ve açık sınırları kaydeder. C1/P33 retleri ve ürün/insan/platform kapıları bu bağlantıyla kapanmaz.

K-297 ordinary LSP sorgu/yanıt UTF-16 düzeltmesi [ADR-153](153-lsp-utf16-konum-siniri.md) altında ayrıca izlenir; S3 ölçümleri yeni kaynağa otomatik taşınmaz.

## S4 kaynak kabulü

[S4](../docs/urunlesme-s4-kabul-20261015.md) yeni exact kaynağın dört stdio
ölçüm kohortunu ve 1800 saniyelik RSS kabulünü ayrı tutar. Taşıma ölçümü
UI gecikmesi veya adanmış koşucu performans tabanı değildir.

## K-298 — bildirim ömürlü açık metin dizini

S4'te lint açık/41 belge stdio kohortu p95 223,698 ms ile 200 ms hedefinin
üzerinde kaldı. K-298, mevcut açık belgelerin gerçek dosya yolu ile tutarlı
kaydedilmemiş metnini bir bildirim boyunca paylaşan ödünç metin dizini kurar.
Her tüketici analizi için aynı URI envanterini yeniden kanonikleştirme maliyeti azaltılır; kaynak metinleri ödünç alınır. Dizin bir sonraki bildirime saklanmaz;
kalıcı durum, yeni cache, artımlı HIR/type veya yeni compiler kapasitesi yoktur.

Aynı gerçek yola gelen alias metinleri çelişirse o yolun overlay'i kullanılamaz;
sonraki bir alias aynı metni getirse de çatışma bildirim içinde unutulmaz.
Diskten proje/paket/kilit doğrulaması korunur; kilitli paket metni açık belgeyle
örtülmez. Bildirimler arasında symlink hedefi yeniden çözülür. Tanı sırası,
span'ler, lint kipleri, yayın ve kaynak bütçeleri değişmez.

İki dar LSP testinin kapsamı üç URI alias'ından her birinin çatışması ve iki lint kipi
ile Unix'te symlink hedefinin sonraki bildirimde yeniden çözülmesidir. Testlerin
başarısı ve yeni exact kaynak stdio ölçümü ayrıca kaydedilecektir;
S4'ün eski ölçüm ve RC/soak sonuçları bu adaya taşınmaz. Hedef, aynı dört
stdio kohortunun 5 ısınma/20 ölçüm sözleşmesiyle karşılaştırılmasıdır; ölçüm
öncesinde 200 ms hedefi ya da tam P19 kapanışı ileri sürülmez.

Etki incelemesi: grammar, API, tanı, bağımlılık ve normatif spec davranışı
korunur; bu nedenle yeni RFC/ADR normu veya hata kataloğu girdisi gerekmez.
Production katman/çevrim, API ve uyumluluk kapıları, test faz sahipliği ve
belge tazeliği değişikliğin aynı diliminde doğrulanır. ADR-045/049 ile
`docs/performans-gecmisi-v2.tsv` etkisi incelenir; paylaşımlı stdio gözlemi
adanmış sabit taban veya hard eşik sayılmaz. Exact kaynak SHA'sı alındıktan
sonra semantic ve core-freeze kayıtları `maintenance` olarak bağlanır;
bugfix reproducer/fixed_by kaydı icat edilmez.
