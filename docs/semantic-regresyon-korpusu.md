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

## Native web kopma kipi (K-327)

`security` fazındaki `native-web-kopma` kipi gerçek native `dil` worker'ını
`--web-proxy` profiliyle başlatır. Fixture'ın `# web-istegi:` başlığındaki
istek küçük alma tamponlu (SO_RCVBUF) bir socket'ten gönderilir, yanıtın ilk
baytları okunduktan sonra bağlantı SO_LINGER 0 ile RST olarak kesilir;
ardından `# web-sonraki:` isteği tam gönderilir. Exit, sonraki istekten sonra
worker hâlâ çalışıyorsa `0`, değilse sürecin çıkış kodudur; çıktı sonraki
isteğin `durum gövde` satırıdır. Düzeltme öncesi kaynakta worker C022 ile
`1` döndürür ve çıktı boştur. ToplayanIo socket kopmasını taklit edemediği
için toplu replay bu kipi atlar; ayrı native libtest vakası boş listeyi
reddeder ve Unix dışında görünür ignored durumundadır.

## Native dosya EXDEV kipi (K-329)

`runtime` fazındaki `native-dosya-exdev` kipi gerçek native `dil` sürecini
Cargo hedef klasöründe (disk) kurulan geçici bir projede çalıştırır. Projenin
`medya` klasörü `/dev/shm` (tmpfs) altındaki bir klasöre sembolik bağdır;
`.zee/yuklemeler/parca` → `medya/…` rename'i bu yüzden `EXDEV` verir. Bu,
systemd `ReadWritePaths=` ile ayrı bind mount edilen yolların root
gerektirmeyen eşdeğeridir. Harness önce iki yolun ayrı aygıtta olduğunu ve
düz rename'in errno 18 verdiğini doğrular; aksi hâlde kanıt üretilemediği için
kırılır. Çıktı programın stdout satırlarıdır; `taşındı` gözlenirse kaynağın
silindiği, hedefin tam içerik ve `0640` izinle yazıldığı ve klasörde geçici
dosya kalmadığı ayrıca denetlenir. Düzeltme öncesi kaynakta çıktı
`taşıma reddedildi`dir. ToplayanIo aygıt sınırını taklit edemediği için toplu
replay bu kipi atlar; ayrı native libtest vakası boş listeyi reddeder ve Linux
dışında görünür ignored durumundadır.

## Native dosya okuma penceresi kipi (K-342)

`runtime` fazındaki `native-dosya-okuma-penceresi` kipi gerçek native `dil`
okuyucusunu Cargo hedef klasöründe kurulan geçici bir projede çalıştırır ve
Windows `ReplaceFileW` penceresini deterministik kurar: program `başladı`
yazınca önce hedef yokken yazarın `.anlik.txt.zee-gecici-*` dosyası durur
(150 ms), sonra hedef paylaşımsız tutanakla açıkken `….zee-yedek` yedeği
durur (150 ms). Toplam pencere K-182'nin kısa bütçesini aşar. Çıktı programın
stdout satırlarıdır; düzeltme öncesi kaynakta ikinci satır `okunamadı: …`
olur. ToplayanIo dosya sistemi penceresini taklit edemediği için toplu replay
bu kipi atlar; ayrı native libtest vakası boş listeyi reddeder ve Windows
dışında (rename penceresiz) görünür ignored durumundadır.

Harness gerçek zee yazarı gibi pencere boyunca klasörün `.zee-yazma-kilidi`
dosyasını tekil kilitler (K-342 eki). Aynı fazdaki
`native-dosya-okuma-izsiz-penceresi` kipi aynı pencereyi geçici/yedek izi
bırakmadan kurar (iki değiştirmenin arası); ilk K-342 sürümünün iz tahmini
burada `okunamadı: …` verir, kilit bariyeri yeni bütünü okur.

## Native çöken değiştirme kipi (K-348)

`runtime` fazındaki `native-coken-degistirme` kipi, Windows CI teşhisinde
`ReplaceFileW` ortasında öldürülen yazarın bıraktığı durumu Cargo hedef
klasöründeki geçici projede aynı adlarla kurar: hedef yok, eski bütün
`.ortak.txt.zee-gecici-4242-1.zee-yedek`te, yayımlanmamış yeni içerik eş
geçicide. Gerçek native `dil` önce okur, sonra ekler, sonra yeniden okur.
Çıktı program satırları ve kalan `zee-gecici` iz sayısıdır (`iz <n>`);
düzeltme öncesi `okunamadı` ve dosya yalnız `b-1` olur. Kurtarma yalnız
Windows yazarında olduğundan kip Windows dışında görünür ignored durumundadır;
toplu replay bu kipi atlar.

## Native yedek artığı kipi (K-351)

`runtime` fazındaki `native-yedek-artigi` kipi, Windows'ta başarılı
`ReplaceFileW` sonrası silinemeyen yedeğin bıraktığı durumu kurar: hedef
`ortak.txt` güncel (`a-1`), eski içerik `.ortak.txt.zee-gecici-4242-1.zee-yedek`
artığında. Gerçek native `dil` okur ve ekler; harness kalan `zee-gecici` iz
sayısını (`iz <n>`) yazar, hedefi zee dışından siler ve programı yeniden
koşar; son satır dosyanın son içeriğidir (`son <satırlar|>`). Düzeltme öncesi
`iz 1`, ikinci koşu silinmiş içeriği (`eski`) okur ve dosya `eski|b-1` olur.
Yedek yalnız Windows yazarında oluştuğundan kip Windows dışında görünür
ignored durumundadır; toplu replay bu kipi atlar.


## Native web depo çekişmesi kipi (K-353)

`security` fazındaki `native-web-depo-cekismesi` kipi, `--web-proxy` profilli
gerçek native `dil` worker'ını ayağa kaldırır, `.zee/web-durumu-v1.json`
ortak deposunu ~1 MiB geçerli oran kaydıyla doldurur ve bir iş parçacığında
zee'nin klasör yazma kilidiyle (`kalici_dosya::atomik_yaz`) birkaç ms'de bir
yeniden yayımlar. Ardından üç istek gönderir; çıktı `başarılı <n>`, `503 <n>`
ve `diğer <n>` sayılarıdır. Düzeltme öncesi worker durumu kilit dışında okuyup
CAS ile yayımladığından her turu kaybeder ve `503 3` görülür; düzeltme
sonrası `başarılı 3`. Toplu replay bu kipi atlar.

## Native web oturum deposu kipi (K-370)

`security` fazındaki `native-web-oturum-deposu` kipi `--web-proxy` profilli
gerçek native `dil` worker'ını açar, ilk istekle `.zee/web-durumu-v1.json`
ortak deposunu kurdurur ve depoyu 4096 geçerli kimlikli oturumla zee'nin
klasör yazma kilidiyle yeniden yayımlar. Ardından oturumsuz `/form`
(`csrf belirteci`) ve `/` isteği gönderir; çıktı `form <durum> <gövde>` ve
`sonraki <durum> <gövde>` satırlarıdır. Düzeltme öncesi `form 500 istek
işlenemedi; sunucu hatası`; sonrası `form 503 web güvenlik durumu güvenle
kaydedilemedi`, worker iki durumda da `sonraki 200 tamam` yanıtlar. Toplu
replay bu kipi atlar.

## Native registry durum çekişmesi kipi (K-361)

`security` fazındaki `native-registry-durum-cekismesi` kipi `dil::registry`
istemcisini iki gerçek iş parçacığında aynı cache köküyle çalıştırır;
taşıyıcı imzalı metadata ve vakanın `.dil` kaynağından üretilen yayını
bellekten sunar. İlk istekte (ikisi de kalıcı durumu okuduktan sonra) ortak
engelde buluşturur, ikinci istemci targets isteğinde birincinin bitmesini
kanalla bekler; uyku ile eşitleme yoktur. Dört senaryo: aynı zincir, ileri,
geri ve aynı sürümde farklı içerik. Çıktı başarılı sayı, ikincinin döndürdüğü
targets sürümü ya da tanı kodu ve son kalıcı targets sürümüdür. Düzeltme
öncesi `aynı zincir başarılı 1`, `ileri P016 durum 3`, `geri P016 durum 4`,
`çelişki P016 durum 3`. Toplu replay bu kipi atlar.

## Native Unicode kimlik kipi (K-363)

`security` fazındaki `native-unicode-kimlik` kipi vakanın `# senaryo: <ad>`
başlığıyla tek arızayı seçer; büyük/küçük harf yalnız-farklı dosyalar ve
görünmez biçim karakterli adlar depoya güvenle alınamadığından senaryo verisi
`CARGO_TARGET_TMPDIR` altında çalışma anında üretilir. Senaryolar:
`paket-adi` (`UzakBagimlilik::ayristir`), `zep-harf-katlama` ve `zep-alm`
(ham `.zep` + `paket_arsivini_kur`; çıktı kurulum sonucu ve yayın adının
varlığı), `birim-harf-farki` (gerçek `dil çalıştır`; vaka kaynağı birimin
kendisidir, `Çay.dil`, NFD `çay.dil` ve tam ad olarak yazılır), `lsp-yeni-ad`
(`lsp::Sunucu` rename) ve `kabul-gerekcesi` (`RegistryCozumPolitikasi`).
Düzeltme öncesi macOS ölçümü: paket adı ve gerekçede her homoglif/bidi
`kabul`; `.zep` harf çakışması `dosya sistemi hatası` (klasör çakışmasında
yayın adı bozuk ağaçla `kurulum var` kalıyordu), ALM `kuruldu`; `Çay.dil`
`çıkış 0 merhaba`; LSP her adı `düzenleme 2`. Linux'ta öncesi harf çakışması
`kuruldu`, `Çay.dil` ve NFD birim `A010 bulunamadı` idi. Toplu replay bu kipi
atlar.

## Native işçi kabul/SIGTERM kipi (K-360)

`cli` fazındaki `native-isci-kabul-term` kipi yönetici rolünü harness'a
verir: ortak dinleyiciyi bağlar, tek birincil işçiyi gerçek grup ortamıyla
(`ZEE_WEB_ISCI_*`) başlatır. Isınma isteğinden sonra işçinin `dil` iş
parçacığı dinleyici `poll`'unda ptrace ile yakalanır, bağlantı gönderilir ve
iş parçacığı başarılı `accept4` dönüşünde durdurulur; sürece giden SIGTERM
`join` bekleyen ana iş parçacığında işlenip işleyici `futex`'e dönünce iş
parçacığı bırakılır. Uyku ile eşitleme yoktur. Üç tur; çıktı `korundu <n>`
ve `kayıp <n>`. Düzeltme öncesi `kayıp 3`, sonrası `korundu 3`. ptrace ve
`/proc` gerektiğinden kip Linux dışında görünür ignored durumundadır;
toplu replay bu kipi atlar.
## Native eşzamanlı taşıma kipi (K-347)

`runtime` fazındaki `native-eszamanli-tasima` kipi geçici bir projede bir
yazar programını (1000 satırı düz `ekle` ile `kaynak.txt`e yazar) ve regression
kaynağının iki kopyasını (ayrı hedef adlarıyla) yarıştırır. Taşıyıcı eylem
içinde kaynağa ekler, saf hesap döngüsüyle pencereyi açar, kaynağı hedefe
`atomik taşı`r ve başarısız döner. Taşıyıcı öldürülmez (geri alma ortasında
kesilen süreç satırı iki dosyada bırakabilirdi); C021 ile çıkınca harness
kalan hedefi kenara alıp yeniden başlatır. Çıktı bütün dosyalarda `b <satır>`,
birden çok dosyada görünen `kopya <adet>` ve `bozuk <adet>`; düzeltme öncesi
`b` 840–996'ya düşer. Tüm platformlarda koşar; toplu replay atlar.

## Native eşzamanlı paket ekleme kipi (K-347)

`cli` fazındaki `native-eszamanli-paket-ekle` kipi dört turda altı yerel
paketi (girişleri regression kaynağı) aynı uygulamaya eşzamanlı `dil ekle`
süreçleriyle ekler; "başka bir süreç" iletisiyle reddedilen komut yeniden
denenir. Çıktı başarı bildiren komutların kalıcı `bağımlılık <n>`, kaybolan
`kayıp <n>` ve son kilidin `dil çalıştır` doğrulamasıyla `kilit
geçerli|bozuk` satırlarıdır; düzeltme öncesi `bağımlılık 4`, `kayıp 20`.
Yeniden denenmeyen hata ve kilit doğrulama hatası stderr ile test çıktısına
yazılır. `fixed_by` Windows kaynak toplama düzeltmesidir (vaka ikisini birlikte
ister). Toplu replay atlar.

## Native eşzamanlı ekleme kipi (K-346)

`runtime` fazındaki `native-eszamanli-ekleme` kipi gerçek native `dil`
süreçlerini Cargo hedef klasöründe kurulan geçici bir projede yarıştırır:
harness'ın ürettiği iki yazar programı her biri 1000 satırı düz `ekle` ile
`ortak.txt` dosyasına yazarken iki iş parçacığı regression kaynağını (eylem
içinde ekleyip başarısız Sonuç döndürür) döngüde yeniden başlatır; eylem
araya giren yazarı gördüğünde C021 ile çıkar. Yazarlar bitince geri alanlar
öldürülür; son içerik resmî zee okuyucusuyla okunur (K-348: Windows’ta
yarım değiştirmede öldürülen geri alanın bıraktığı yedek görülür). Çıktı `b <satır>`, `c <satır>` ve `bozuk <adet>` (biçim dışı ya da
yarım son satır) satırlarıdır; düzeltme öncesi kaynakta yazar sayıları
1000'in altına düşer. Kip tüm platformlarda koşar; toplu replay bu kipi
atlar, ayrı native libtest vakası boş listeyi reddeder.

## Native web yükleme kipi (K-332)

`security` fazındaki `native-web-yukleme` kipi fixture'ı Cargo hedef
klasöründe geçici bir projeye kopyalar (worker kökü kaynağın klasörüdür; depo
ağacına `.zee` yazılmaz) ve gerçek native `dil` worker'ını güvenli proxy
profilinde başlatır. Harness `GET /form` ile oturum çerezi ve CSRF belirteci
alır, ardından 64 KiB'lık octet-stream gövdesiyle altı senaryo koşar: CSRF ve
oturumsuz (yalnız başlıklar, yazma yarısı kapalı), sahte oturum, var olmayan
adres, kısa gövde, dosyayı taşımayan rota ve dosyayı `kalici/` altına taşıyan
rota. Çıktı her senaryonun `durum ad` satırı, kalan `.parca` sayısı ve taşınan
dosyanın bayt sayısıdır; taşınan dosya gövdeyle bayt bayt karşılaştırılır.
Düzeltme öncesi kaynakta ilk senaryo `400` (gövde beklenir) ve `parça 5`
gözlenir. ToplayanIo gövdeyi diske akıtmadığı için toplu replay bu kipi
atlar; ayrı native libtest vakası boş listeyi reddeder ve Unix dışında
görünür ignored durumundadır.

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

## Native işçi grubu kipi (K-338)

`cli` fazındaki `native-isci-grubu` kipi gerçek native `dil` yöneticisini
`--web-proxy … --web-worker-port <boş kapı> --web-isci-sayisi 2` ile başlatır
(kaynağın kapısı yok sayılır) ve yönetici çıkana dek en çok 90 sn bekler.
Kaynak koşu başına Cargo hedef klasöründe geçici bir projeye kopyalanır
(K-341: birincil işçinin `.zee` süreç kilitleri depo ağacına yazılmaz,
eşzamanlı iki koşu birbirini canlı katılımcı görmez).
Exit yöneticinin çıkış kodudur; çıktı stderr'den sayılan iki satırdır:
`yuva 2 başlatma <n>` ve günlükteki `durma <n>`. Başlatma sayısı yeniden
başlatma satırlarının (`zee-web işçi 2 (pid …) çıktı`) bir fazlasıdır
(K-344): işçiler yöneticinin stderr'ini miras alır ve `eprintln!` bir
satırı birden çok write ile yazdığından yeni işçinin açılış çıktısı
yöneticinin "başladı" satırını bölebilir; yeniden başlatma satırı ise çıkan
işçi bittikten, yenisi başlamadan yazılır. Fixture'da ikincil yuva her
açılışta `programı 3 ile bitir` ile çıkar; düzeltme öncesi kaynakta çıktı
`yuva 2 başlatma 11` / `durma 10`, düzeltmeyle `10` / `10`'dur. Koşu
gecikme dizisi nedeniyle ≈28 sn sürer. Yönetici kendi süreç grubunda başlar;
çıkışından sonra grup öldürülür ve stderr borusu en çok 15 sn beklenir
(macOS'ta boru CLOEXEC'i atomik değildir, eşzamanlı spawn yazma ucunu
miras alabilir), böylece kip asılı kalamaz. ToplayanIo süreç yönetimini taklit
edemediği için toplu replay bu kipi atlar; ayrı native libtest vakası boş
listeyi reddeder ve Unix dışında görünür ignored durumundadır.

## Native işçi sağlık mirası kipi (K-345)

`cli` fazındaki `native-isci-saglik-mirasi` kipi kaynağı geçici bir projeye
kopyalar ve gerçek native `dil` yöneticisini `--web-isci-sayisi 32` ve
sağlık portuyla üç kez ard arda açar. Her grup başlarken sağlık uç noktası
altı iş parçacığıyla 4 sn yoklanır; ardından sağlıktaki işçi PID'lerinin
sağlık kapısında tuttuğu TCP uçları sayılır (macOS `lsof`, Linux
`/proc/<pid>/fd` + `/proc/net/tcp`). Çıktı `miras <n>` ve 2 sn içinde EOF
ile bitmeyen istek sayısı `asili <n>`; çıkış bir grup 32 canlı işçiye
ulaşmadıysa 1. Düzeltmeyle `miras 0`/`asili 0`. Düzeltme öncesi arıza
olasılıksaldır (macOS'ta grup başına ≈%20; üç grupta yakalama ≈%50);
deterministik kanıt `saglik_yaniti_kopya_acikken_de_baglantiyi_bitirir`
birim testidir. Linux'ta accept4 atomik olduğundan kip değişmez sonucu
doğrular. Toplu replay bu kipi atlar; Unix dışında görünür ignored.

## Native ayrı port birincillik kipi (K-341)

`cli` fazındaki `native-ayri-port-birincil` kipi kaynağı Cargo hedef
klasöründe geçici bir projeye kopyalar (`.zee` süreç kilitleri depo ağacına
yazılmaz) ve gerçek native `dil` worker'larını `--web-proxy … --web-worker-port
<boş kapı>` ile ayrı ayrı açar: süreç 1 ve 2 açılır, süreç 1 öldürülüp aynı
kapıda süreç 3 açılır. Her süreç dinlemeye geçtikten sonra `GET /rol` ile
sorulur. Exit 0, çıktı `1 <rol>`, `2 <rol>`, `3 <rol>` satırlarıdır.
Düzeltme öncesi kaynakta üçü de `birincil`, düzeltmeyle `birincil`,
`ikincil`, `ikincil`'dir. Kip Unix'e özgü çağrı kullanmaz ve her platformda
koşar; ToplayanIo süreç yönetimini taklit edemediği için toplu replay bu kipi
atlar, ayrı native libtest vakası boş listeyi reddeder.
