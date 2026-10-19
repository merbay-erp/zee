# zee v1.0 sürüm kapıları

> `a57290f7a1c1fad07cda3f266b2a221f09aa6404` için [K-298 yerel kabulü](urunlesme-k298-kabul-20261016.md) dört stdio kohortunda gözlemsel p95 200 ms hedefini, macOS teslim/kurulum, ordinary UTF-16 ve dar IDE işlev kabulünü kaydeder. Aynı SHA'da ikinci RSS ve dört ASan/Miri3/tam RC kapısı geçti; K-298 dar kabulü kapalı, P19 kısmi kalır. Belge takip commit’i gerçek dış son kapı makbuzuna bağlıdır. [S4](urunlesme-s4-kabul-20261015.md) ve [S3](urunlesme-s3-kabul-20261015.md) kendi kaynaklarında tarihseldir; ürün, insan, fiziksel platform ve yayın kapıları açıktır.

Bu liste bir dilek listesi değildir: `1.0.0` etiketi bu kapılar kapatılmadan
atılmaz. Amaç yeni özellik sayısını büyütmek değil, çocuktan profesyonele aynı
dilin verdiği sözleri kanıtlamaktır. Bulgular 2 Eylül 2026'da derleyici
kaynakları, spec, RFC'ler ve otomatik test envanteri üzerinden yeniden
doğrulanmıştır. Kaynakta ilan edilen toplam README'de; platformda gerçekten
derlenen toplam ve faz dağılımı CI'ın faz matrisi artefaktındadır.

Durumlar: **KAPALI** = kanıtı var · **AÇIK** = v1 engeli · **KARAR** = önce
normatif seçim gerekir · **KAPSAM DIŞI** = v1'in açıkça vermediği söz.

K-191, V1-P1-25 provenance'ın araç zinciri seçimini exact kaynak commit'ine
bağladı; varsayılan Rust farklılığı giriş regresyonuyla korunur.
[Sistem bağımlılıkları ve backend açıkları](derleme-ve-bagimliliklar.md)
K-192/K-193 kapsamındadır; bu düzeltme yeni backend kapatmaz. K-192 kaynak
metadata alanlarını aynı kanonik resmî depo ve exact SHA'ya bağladı; uzak
erişim ve üç platform kanıtları ayrıca tamamlanır. Çalışma ortamı kapısı
checksum/SHA, doğrudan OS kitaplığı, Rust'sız PATH program ve LSP provasını
raporlar; rapor builder'da hiç Rust kurulmadığını iddia etmez.

K-194 kurulum hata geri alması Unix/Windows gerçek ret testleri ve
`38238657801` üç platform kur/çalıştır/kaldır tatbikatıyla kapandı
(ADR-071/GB-031). Ani güç kaybı
uzlaştırması veya dizin çapında atomiklik iddia edilmez. GB-032 kaynak
metadata manifest enjeksiyonu exact SHA ve TSV yol kontrolüyle kapandı.

## P0 — v1 etiketi öncesi zorunlu

| Kapı | Durum | Kaynakta görülen gerçek | Kapanma kanıtı |
|---|---|---|---|
| V1-P0-01 İşlem imzası çağrı sırasından bağımsızdır | **KAPALI (K-083/K-086/K-121)** | Public birim/paket işlemleri tam ve monomorfik imza taşır. Yerel başlangıç `<ad> al` işlemlerinde erişilebilir ana/test çağrı kısıtları kopya AST'de parametre konumu başına birleşir; asıl gövde ve HIR yalnız nihai imzayla denetlenir. | RFC-0006 + ADR-013 revizyonu + spec/04/10 + [çağrı çıkarım rehberi](cagri-cikarimi.md); açık gövde/paket/özyineleme kanıtlarına ek olarak dar↔geniş skaler, liste, çok parametre, iç içe grafik, iç blok kurtarması ve iki sıralı T017 altı regresyonda sabittir. Toplam 481 test yeşildir. |
| V1-P0-02 Web route ile uygulama eylemi ayrıdır | **KAPALI (K-087)** | Açık imzalı `eylem` HTTP etkisi taşıyamaz; aynı çağrı web/CLI/görev/test bağlamında kullanılabilir. GET/HEAD'in çağrı grafiğindeki dolaylı yazması T045, rota içi uygulama yazması T046'dır. Yöntemli rota 404/405, 64 KiB+100 alan 413 ve 30 saniye 504 üretir. Her eylem çalışma hatası/başarısız Sonuç için iç içe dosya savepoint'i taşır. | RFC-0015 geçici kabul + spec/11; doğrudan/dolaylı GET olumsuzları, POST→eylem zorunluluğu, CLI+web ortak eylem, 404/405/413 ve çok-dosyalı/nested rollback regresyonları. Süreç çökmesinde çok-dosyalı tek commit sözü verilmez; dosya başına K-084 geçerlidir. |
| V1-P0-03 Oturum ve çerez üretim güvenliği | **KAPALI (K-088/K-134/K-137/K-139/K-140/K-185)** | Native runtime 256 bit OS CSPRNG oturum+CSRF üretir, oturum kimliğinin yalnız SHA-256 özetini sunucuda tutar; Argon2id PHC doğrular. Giriş kimlik+CSRF'yi döndürür, 30 dakika sınırlar; logout/süre dolumu iptal eder. Roller sunucudadır; unsafe rota açık politika ve otomatik synchronizer CSRF ister. Production çerezi `__Host-`, Secure, HttpOnly, SameSite=Lax, Path=/ ve Max-Age taşır. K-134 session/cookie mutation'ını ilk yanıtla birlikte commit/rollback eder. K-137 production durumunu ve endpoint/CSRF/Argon2id oran pencerelerini süreçler arası kalıcı atomik depoda paylaşır. K-139 proxy origin'ini ortak `AgHedefi` kimliğine, dinleyiciyi açık loopback bind+peer değişmezine bağlar. K-140 HTTP framing'i kayıpsız byte parser'a taşır. K-185 doğrulanmış kullanıcı/rolü yalnız rota içi `oturum` sözlüğüne verir; istemci girdisi ve oturum sırları dışarıda kalır. | RFC-0017 + ADR-034/036/037/076 + spec/11/12; CSPRNG/Argon2id, rotation/revoke/expiry/rol, sahte iç oturum alanı, rota kapsamı, eksik-sahte-geçerli CSRF, 400/401/403/429/503, çerez nitelikleri, kanonik DNS/IPv6/port Host/tek-hop Forwarded/Origin ve CRLF/target/TE-CL/UTF-8 olumluları-olumsuzları; timeout/runtime/socket hatasında session+çerez+erken yanıt rollback'i. İki gerçek CLI worker'ı login, restart, çapraz logout ve ortak altıncı-deneme 429 akışını kanıtlar. `--web-proxy https://host` yalnız loopback HTTPS proxy zincirini ve N ayrı tek-worker süreci kabul eder; loopback dışı peer başlıklardan önce reddedilir. |
| V1-P0-04 Kalıcı durum atomik ve yarış güvenlidir | **KAPALI (K-084/K-128)** | Tek-dosya `yaz/ekle`, aynı klasörde temp+sync+atomik replace yapar; Unix/Windows işletim sistemi kilidi thread ve süreç yazarlarını sıralar. Okuyucu yalnız eski/yeni bütün sürümü görür; metadata koruması ayrı V1-P0-30 kapısındadır. | RFC-0016 + spec/08; replace hata enjeksiyonu eski veriyi korur, iki thread ve iki bağımsız CLI süreci satır kaybetmez, Drop'suz ani süreç sonu kilidi bırakır. Çok-kaynaklı uygulama transaction'ı V1-P0-02/RFC-0015 sınırındadır. |
| V1-P0-05 Deadline gerçekten iptal eder | **KAPALI (K-085/K-133/K-134)** | `IcindeBlogu` mutlak son tarihi sahipli Ç001 ile blok/işlem/döngü sınırlarına yayar. `bekle` kalan süreye kırpılır; HTTP aşamaları kalan tek bütçeyi alır. K-133 dış etkileri çağrının hemen önünde yeniden denetler ve görev HTTP öncesi sıra verdikten sonra kalan süreyi tazeler. K-134 web deadline'ında tamponlu yanıtla session/cookie mutation'ını geri alır. İç içe tarihlerde en erken sahip kazanır. | RFC-0011 + spec/09; geç ağ yanıtı çıktıya dönüşmez, uzun bekleme sonrası cümle çalışmaz, deadline dolmuş dosya/HTTP etkisi hiç başlamaz, web timeout'u erken başarı/oturum bırakmaz ve iç/dış `yetişmezse` sahipliği sanal saatle sabittir. Tek kesintisiz ifade/platform syscall sınırı normatif işbirlikli modeldir. |
| V1-P0-06 Normatif otorite tek ve izlenebilirdir | **KAPALI (K-081/K-118/K-144/K-145/K-146/K-147)** | Spec/RFC drift'i doğrulandı; bugünkü 91 numaralı RFC/ADR/spec belgesi makine-okunur durum+test yollarıyla birebir izlenir. README'nin canlı golden/test/tanı/RFC/ADR/spec sayıları depo ağacından üretilir; kritik işlev, 22 fazlı test sahipliği ve 17 vakalı bug korpusu ayrı kanonik kayıtlardır. | ADR-010/041/042/043/044 belge rollerini, atomik değişiklik sözleşmesini, işlev raporu tazeliğini, kanonik Rust biçimini, gerçek Cargo/libtest envanterinin tam sahipliğini ve bug→minimal kaynak→kesin gözlem bağını kurar; [depo bütünlüğü rehberi](depo-butunlugu.md), [faz matrisi](faz-test-matrisi.md), [semantic korpus rehberi](semantic-regresyon-korpusu.md), kanıt haritası ve üç platformlu CI bayat/sahipsiz/yinelenen kanıtı hata yapar. |
| V1-P0-07 İşlem çağrısı tek genel ve kullanıcıyla doğrulanmış yüzeydir | **KARAR — K-096 deney hazır** | A (`için`/`ile`) çalışan geçici yüzeydir; nihai V1 seçimi değildir. Serbest üretim, kör A/B/C kartları, sıra dengelemesi, yedi teknik bağlam ve alt grup eşikleri önden bağlandı. | [K-016 karar paketi](k016-cagri-karar-paketi.md) uyarınca gerçek 10 çocuk + 5 profesyonel formu; eşik geçen tek grammar; parser/formatter/LSP ile golden+anti-example kanıtı; RFC-0006/spec-02 ve migration'ın aynı atomik değişiklikte güncellenmesi. |
| V1-P0-08 İfade gramerinin büyüme önceliği açık ve fail-closed'dur | **KAPALI (K-097/K-119)** | Primary→erişim/postfix→çağrı→aritmetik→birleştirme→karşılaştırma→boolean katmanları ve tam bölge tüketimi normatiftir. Karışık `ve/veya` ya da tanımsız katman birleşimi sessiz AST üretmez; işlem-adı kuyruğu güçlü postfix'i gölgelemez, tam işlem adı sıfır-argüman çağrısını korur. Formatter satır/girinti yapısı dahil parser girdisini değiştiremez. | RFC-0021 + spec/20 + ADR-002 revizyonu; sekiz ifade conformance testi ve 33 golden programın dağınık-boşluk varyantında tam token izi + iki başarılı parser geçişi. Yeni ifade yüzeyi katman+çakışma+formatter/test protokolü olmadan eklenemez. |
| V1-P0-09 Alan adaptörleri core AST'yi şişirmez | **KAPALI (K-098)** | HTTP, sensör, CSRF ve parola yüzeyleri ayrı AST varyantları değil, kararlı kimlik ve sıralı argüman taşıyan tek `Intrinsic` düğümüdür. Tür imzası, gereken yetkinlik ve statik etki merkezi kayıttadır; kullanıcı kaynakları değişmemiştir. | ADR-011 + [intrinsic/yetkinlik rehberi](intrinsic-yetkinlik-modeli.md); kayıt tekilliği, dört lowering ve iki tür olumsuzu için yedi bağımsız test, mevcut alan davranış korpusu. Yeni intrinsic kayıt+lowering+tür+etki+runtime+olumlu/olumsuz kanıt olmadan eklenemez. |
| V1-P0-10 Derleyici handler büyümesi fiziksel faz ve işlev eğilimi sınırındadır | **KAPALI (K-099/K-144/K-145/K-146)** | Parser cümle/ifade, checker cümle/ifade/çağrı, runtime cümle/ifade modüllerine ayrıdır; kökler orkestrasyon ve ortak modellere sahiptir. Alt modüller yalnız üst fazına görünür. K-144/K-145, dosya bütçesi içinde büyüyebilen 49 kritik üretim işlevinin kanonik `rustfmt` satır ve bilişsel karmaşıklık yönünü izler; K-146 her testin birincil fazını ve aşağı akış blast-radius bağını görünür kılar. | ADR-012/041/042/043 + [faz rehberi](derleyici-faz-sinirlari.md) + [eğilim raporu](islev-egilimi.md) + [test matrisi](faz-test-matrisi.md); CI kanonik biçimi, kritik işlev trendini ve her platformun gerçek test envanterindeki eksik/yinelenen faz sahipliğini engeller. Mutlak tek işlev hard limiti yoktur; eşik tasarım incelemesini zorunlu kılar. |
| V1-P0-11 Checker semantik kuralları tek sahipli katmanlardadır | **KAPALI (K-100)** | Checker kökü yalnız geçiş sırası ve public yeniden dışa aktarımı taşır; tür, bağlam, sembol, akış, çağrı, sözleşme, etki/yetkinlik ve dönüş kuralları ayrı modüllerdedir. | ADR-013 + [checker katman rehberi](checker-katmanlari.md); beş kaynak-mimari testi kök bütçesini, katman sahipliğini ve public API'yi korur. Kaynak semantiği değişmeden toplam 398 test yeşildir. |
| V1-P0-12 Semantic identity depolama konumundan ayrıdır | **KAPALI (K-101)** | Yapı, işlem ve sembol bağları `YapiId`/`IslemId`/`SymbolId` taşır. `Tur::Yapi` vektör indeksi değildir; işlem imzası/özyineleme ID ile anahtarlanır ve çözülmüş AST kaynak adının yanında kimliği korur. | ADR-014 + [semantic kimlik rehberi](semantic-kimlik-modeli.md); yapı depolama ve işlem çağrı sırası tersleme, SymbolId bağı ve indeks-gerileme mimari testi. Toplam 402 test yeşildir. |
| V1-P0-13 Derleyici veri fazları kodda ayırt edilebilirdir | **KAPALI (K-102)** | Kaynak, token, parsed AST, bağlanmamış ve checker'dan geçmiş bağlı program ayrı Rust türleridir. Standart çalıştırma bağlı program ister; eski raw `Program` yalnız uyumluluk adaptörüdür. | ADR-015 + [faz modeli](derleyici-faz-modeli.md); parsed/bound bağ testi, eski API eşdeğerliği, standart-hat mimari testi ve yanlış geçişin derlenmediği compile-fail kanıtı. Toplam 406 test yeşildir. |
| V1-P0-14 Runtime typed HIR bağlarını tüketir | **KAPALI (K-103/K-104)** | Bağlı program her denetlenmiş ifade için `HirDugumId`, açık `Deger(Tur)`/`DegerDondurmez` türü ve semantic ID bağı taşıyan zorunlu `HirProgram` üretir. Standart runtime ve `dene` değişken/işlem/yapı kararlarını yalnız `HirBagi` ile yapar; kaynak AST tanı/v0 uyumluluğu için korunur. | ADR-016 + [typed HIR modeli](typed-hir-modeli.md); tür/SymbolId, dönüşsüz çağrı, işlem/yapı ID dizini, HIR'sız bağlı program, bozulan kaynak adına rağmen ID ile çalışma ve standard-hat mimari kanıtları. Toplam 413 test yeşildir. |
| V1-P0-15 Native ağ bekleme ve bellek sınırı | **KAPALI (K-105)** | Deadline yazılmasa da HTTP istemcisi 30 saniyelik mutlak bütçe ve başlıklar dahil 8 MiB wire yanıt sınırı taşır. Yerel sunucu başlık+gövdeyi kabulden başlayan 10 saniyede tamamlatır; her okuma yalnız kalan süreyi alır ve aşım 408'dir. | ADR-017 + spec/09/11/12; geçmiş mutlak tarihte hazır baytı bile okumama, yanıt boyut reddi ve deadline'sız küçük loopback yanıtı. Toplam 416 test yeşildir. DNS iptali platform sınırı; oturum/LSP sınırları ayrı açık kapıdır. |
| V1-P0-16 Process içi oturum belleği sınırlıdır | **KAPALI (K-106)** | Depo 4096 toplam/1024 anonim kayıtla sınırlı; süresi dolanlar önce silinir, en eski erişimli anonim deterministik tahliye edilir. Kimlikli kayıt rastgele düşürülmez; dolu kimlikli depo yeni girişi reddeder. 10/30 dakika ömür mutlaktır. | ADR-018 + spec/12; anonim LRU, anonimin girişe yer açması, kimlikli doluluk reddi ve erişimin expiry'yi kaydırmaması. Toplam 419 test yeşildir. Production çok süreçli ortak depo daha sonra K-137/ADR-034 ile kapandı. |
| V1-P0-17 LSP tek-girdi ve JSON-RPC sınırı kesindir | **KAPALI (K-107/K-129/K-138)** | `dillsp` 8 KiB başlık/8 MiB gövdeyi tahsis öncesi, tek `Content-Length` ile doğrular. JSON 128 iç içelik/100 bin düğümle sınırlı; Unicode, RFC 8259 sayı ve duplicate alan kuralları fail-closed'dur. Sayı float'a dönüşmeden kayıpsız lexeme taşır. K-129 ayrıca 256 açık belge/128 MiB toplam metin ve 8 MiB outbound sınırıdır. | ADR-019/033/035 + [LSP JSON-RPC profili](lsp-json-rpc-profili.md); framing, sayı differential, escaped duplicate, UTF-8, `-32700/-32600/-32601/-32602`, notification sessizliği ve toplam belge/yanıt regresyonları. Aşım durum değiştirmeden S045 veya `-32001` üretir; B-051 kapalıdır. |
| V1-P0-18 Semantic HIR düğümleri kaynak kökenlidir | **KAPALI (K-108/K-126)** | Her parser AST yaprak ve bileşik ifadesi ayrı, sıfır olamayan kesin `AstKaynakAraligi` taşır; checker bunu HIR'a birebir aktarır. Eksik/iç içe zarf ve AST↔HIR aralık uyuşmazlığı fail-closed'dur. | ADR-020 + ADR-030 + [typed HIR modeli](typed-hir-modeli.md); bileşik/yaprak kesinliği, spansiz AST reddi, tanı işareti, örtük çoğul kökeni, LSP kesin çağrı kuyruğu ve modül sahipliği kanıtları. Toplam 490 test yeşildir; B-050 kapandı. |
| V1-P0-19 Production doğrudan panic yüzeyi kapalıdır | **KAPALI (K-109)** | Lexer'dan CLI'a 46 `unwrap`/`expect`/açık panic noktası sonuç veya kodlu tanıya dönüştü. Dört production crate kökü test dışında `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!` ve `unimplemented!` kullanımını derleme hatası yapar; `SymbolId` yapay kapasite assertion'ı taşımaz. | ADR-021 + [production panic rehberi](production-panic-politikasi.md); sıfır konumlu elle kurulmuş AST'nin T016 üretimi ve dört crate lint sahipliği regresyonu. Clippy all-targets temiz, toplam 429 test yeşildir. Kullanıcı UTF-8 fuzz kanıtı B-015/K-110, malformed iç yapı kanıtı B-017/K-112 ile kapandı. |
| V1-P0-20 Lexer/parser kullanıcı UTF-8 girdisinde panic-free denetlenir | **KAPALI (K-110/K-156)** | `&str` libFuzzer hedefi başarılı lexer çıktısını normal ve hata-kurtarmalı parser yollarında yürütür. 64 KiB kampanya sınırı, beş saniye tek-girdi timeout'u, sekiz saldırı tohumu ve Zee mutation sözlüğü vardır; gecelik korpus cache ile büyür, K-156 coverage öğrenimini cache dışı provenance'lı artefakta da taşır. | ADR-022/055 + [fuzz rehberi](fuzzing.md); korpus replay, 4.096 deterministik UTF-8 üretimi ve 64 KiB sayı/virgül/girinti regresyonları. İlk smoke 1.048.287 girdiyi crash/panic olmadan tamamladı; toplam 432 test yeşildir. Malformed elle kurulmuş AST/HIR, B-017/K-112'nin ayrı kapısındadır. |
| V1-P0-21 Morfoloji üret→çöz ve belirsizlik değişmezleri sürekli denetlenir | **KAPALI (K-111)** | `zee-tr-1` bütün geçerli tek/iki katmanları 4.096 üretilmiş kökte geri çözer. 2.048 yüzey bütün yapısal kökleri kapsama alındığında çoklu aday A002'dir. NFC kabul edilir; ayrıştırılmış NFD kaynak S029'dur. Ayrı gecelik fuzz hedefi byte girdiden geçerli kök üretir. | RFC-0018 + spec/13 + [morfoloji doğrulama rehberi](morfoloji-dogrulama.md); üç stable regresyon ve 527.966 girdilik ilk mutation smoke'u ihlalsizdir. Toplam 435 test yeşildir; bütün dizilerin biçimsel ispatı iddia edilmez. |
| V1-P0-22 AST ve typed HIR faz sözleşmesi yürütülebilirdir | **KAPALI (K-112)** | Parser AST'si semantic bağ taşımaz. Bağlanmış programdaki her AST ifadesi benzersiz düğüm kimliği, açık tür, kaynak aralığı ve varyantla uyumlu semantic bağ taşıyan tam bir HIR kaydıyla eşleşir; yetim HIR kaydı reddedilir. | ADR-023 + [AST/HIR invariant rehberi](ast-hir-invariantleri.md); geçerli uçtan uca hat, parser semantic sızıntısı, imkânsız AST biçimleri ve eksik SymbolId/YapiId/IslemId regresyonları. Özellik→alan dönüşümündeki gerçek yetim kayıt bulunup düzeltildi; toplam 440 test yeşildir. |
| V1-P0-23 Parser çoklu tanıda kapsamı ve kardeşleri korur | **KAPALI (K-113)** | Hatalı cümle satır sonunda ve yalnız kendisine ait dengeli girinti gövdesinin sonunda senkronlanır. Sağlam kardeş ebeveyn blokta kalır, parser derinliği sonraki üst tanıma sızmaz. CLI/LSP birleşik tanıları kaynak sırasında ve belge başına en çok 20 kayıttır. | RFC-0010 §2.1 + ADR-024 + [parser kurtarma rehberi](parser-hata-kurtarma.md); iç kardeş, yapı alanı, derinlik, eşzamanlı görev, `göre` kolu, tanı bütçesi ve gerçek LSP sırası için yedi regresyon. Toplam 447 test yeşildir. |
| V1-P0-24 Tanı kodu sürümler arasında aynı olayı anlatır | **KAPALI (K-114)** | K-114 tabanından sonra eklenen P015/T054/S045/C023/C024/P016/P017 ile 152 etkin ve 3 ayrılmış kod; aile uyumlu tekil semantik anahtar, aktif/mezar taşı durumu ve kanonik katalog özetiyle şema-1 fixture'ına sabittir. Anlam değişikliği yeni kod ister; ayrılmış kod yeniden etkinleşemez. | RFC-0010 §2.2 + ADR-025 + [tanı kimliği rehberi](tani-kimligi.md); kaynak↔katalog ve katalog↔fixture kapıları bütün 155 kimliği iki bağımsız testte korur. K-136 exact uzak bildirim kimliğini append-only ekledi. |
| V1-P0-25 Gerçek IO koşusu dış etkisiz ve birebir yeniden oynatılabilir | **KAPALI (K-115)** | 27 gözlenebilir `GirdiCikti` işlemi sürümlü şema-1 izinde tek küresel sıra, işlem, argüman ve sonuç taşır. K-134'ün iki host-only request commit/rollback kancası bağımsız kullanıcı IO'su değildir ve iz olayı olmaz. Replay dış etkileri uygulamaz; ilk sıra/işlem/argüman farkında ve artan olayda fail-closed durur. 64 MiB/100.000 olay/4.096 alan sınırı yürütme öncesidir; parola/PHC yalnız parmak izi olur. | RFC-0022 + ADR-026 + spec/21 + [IO izi rehberi](io-izi.md); byte snapshot'ı, bütün gözlenebilir IO yüzeyli round-trip, sıra/argüman/tam tüketim, bozuk şema, sır sızıntısı, CLI kaynak farkı ve üzerine yazma reddi için yedi regresyon. Toplam 455 test yeşildir. |
| V1-P0-26 Tohum, saat ve hermetik IO gözlemleri sürümlüdür | **KAPALI (K-116)** | `zee-io-1` tohum karışımı+xorshift64* dizi vektörünü, yansız uçları-dahil tam i64 eşlemeyi, takvim/tekdüze saat ayrımını, geriye gitmeyen sanal beklemeyi ve FIFO/fallback+sahte dosya/ağ/sensör gözlemlerini bağlar. CLI/playground tek PRNG sahibidir; güvenlik belirteçleri OS CSPRNG'de kalır. | RFC-0023 + ADR-027 + spec/22 + [profil rehberi](deterministik-io-profili.md); profil/dizi snapshot'ı, tam i64, rastgele kuyruk sınırı, sanal saat ve bütünleşik fake-IO/playground için beş conformance testi. Toplam 460 test yeşildir. |
| V1-P0-27 LSP gezinme ve rename semantic kimliği izler | **KAPALI (K-120)** | Definition/rename geçerli belgede `SymbolId`/`IslemId`/`YapiId` typed-HIR bağını kullanır. Aynı yazımlı ayrı kapsamlar karışmaz; yeniden atamalar aynı sembolde kalır; morfoloji yalnız kimlik seçildikten sonra uygulanır. Hatalı/A002 belgede metin tahmini yoktur; dış birim tanımı için eksik tek-dosya rename üretilmez. | ADR-014/016 revizyonu + [semantic LSP rehberi](lsp-semantic-gezinme.md); HIR tanım/yazım/okuma dizini ve ayrı kapsam, yeniden atama, fail-closed, çok kelimeli işlem, yapı definition/rename ve çakışan dış-birim satırı dahil sekiz yeni regresyon. Toplam 475 test yeşildir; çok dosyalı WorkspaceEdit ve bütün AST'de kesin span ayrı kapılardır. |
| V1-P0-28 Ondalık binary float'a örtük dönüşmez | **KAPALI (K-125)** | Tek kesirli kullanıcı türü keyfî hassasiyetli Ondalık'tır; `Tur`/`Deger` envanterinde binary32/binary64 yoktur ve Stage 0 FFI yüzeyi sunmaz. Gelecekte C `float`/`double` köprüsü ancak deklarasyon+çağrıda görünür `kayıplı` işareti, `Sonuç` ve açık IEEE 754 semantiğiyle eklenebilir. | ADR-029 + RFC-0012 §2.1 + RFC-0013 + spec/16; enum envanteri, eski FFI taslağının reddi, exact `0,1+0,2=0,3` ve normatif metin tazeliği tek regresyonda korunur. FFI gerçeklenmesi olumlu/olumsuz platform korpusu olmadan açılamaz. Toplam 485 test yeşildir; B-012 kapandı. |
| V1-P0-29 Dış dünya yetkisi proje sahibinin açık ve savunmalı kararıdır | **KAPALI (K-127/K-139)** | Sekiz kararlı yetkinlik manifest→paket-alt-kümesi→compile→runtime boyunca tek politikadır. Ağ exact origin ve DNS sonrası IP'ye bağlanır; public internet HTTPS, private/loopback+düz HTTP ayrıca onaylı, metadata/link-local daima kapalıdır. Redirect/proxy kapalı; zaman/header/body sınırlıdır. Proje dosyası kök ve canonical symlink sınırı taşır. Aynı `AgHedefi` origin tipi production web proxy'nin inbound Host/Forwarded/Origin kapısında da kullanılır. | RFC-0024 + ADR-031/036 + spec/12/23 + [yetkinlik/ağ rehberi](yetkinlik-ve-ag-guvenligi.md); P015/T054 kimlikleri, manifest/paket yükseltme, ölü kod, kesin kaynak, origin, private/metadata/geçiş öneki DNS, symlink, redirect, body limiti ve inbound DNS/IPv6/port kanonikliği regresyonları. Exact `ureq 3.4.0` rustls backend'i kullanılır; Zee TLS yazmaz. Outbound ve web CLI ikinci bir origin parser'ıyla drift edemez. |
| V1-P0-30 Atomik replace erişim ve güvenlik metadata'sını sessizce düşürmez | **KAPALI (K-128)** | Linux/macOS mode+uid+gid'yi korur; Linux görünür ACL/security xattr'ını bütçeli descriptor kopyasıyla, macOS ACL+xattr'ı `fcopyfile` ile taşır. Windows DACL/security resource/named stream için hata-yoksaymasız `ReplaceFileW` ve kurtarma yedeği kullanır. Symlink, taşınamayan metadata ve Tier-1 dışı Unix replace'i fail-closed'dur. | RFC-0016 + ADR-032 + spec/08; macOS mode/owner/group, binary xattr ve gerçek ACL; Linux xattr; Windows named stream için platform-koşullu regresyonlar ve eski atomiklik paketi. Toplam envanter 506 test, 77 numaralı belgedir. |
| V1-P0-31 Kullanıcı girdisi host kaynağını sınırsız tüketemez | **KAPALI (K-105/K-129/K-130/K-131/K-132/K-140)** | Tek immutable profil kaynak/token/proje toplamı, C019 çağrı derinliği, adım/çıktı/koleksiyon/görev, 16 MiB bütçeli metin, yaklaşık 64 MiB saklanan değer tahsisi, 64 süreç-geneli bağlantı, bounded dosya, 16+64 KiB tahsis-sıralı byte HTTP ve LSP toplam bellek/outbound sınırlarını uygular. K-131 bütün eski domain sayılarının sahipliğini merkezde topladı; K-132 LSP JSON'unu append öncesi 8 MiB'ta keser; K-140 HTTP gövdesini doğrulanmış CL sonrasında ayırır. Aşım S045/C023/C024, C018/503, HTTP 413/431 veya protokol hatasıdır. | RFC-0025 + ADR-033/037 + spec/11/24; kaynak/heap/bağlantı/merkezî sahiplik, LSP sınırlı yazıcı, HTTP CRLF/TE-CL/exact gövde, taşma dönüşümü ve bütün outbound yolların geçerli JSON regresyonları. Güncel envanter 566 testtir. |
| V1-P0-32 Production sorumluluğu ve bağımlılık yönü incelemesiz değişemez | **KAPALI (K-149/K-150)** | 37 üst sahip on katmana atanır (K-167 `tedarik` cephesini kaldırdı); bütün açık iç kenarlar exact TSV tabanındadır. Yeni/kayıp sahip, yeni/kaldırılmış kenar ve izin dışı katman geçişi fail-closed'dur. Parser semantic/runtime'ı, checker runtime/adaptörü, runtime parser/checker/LSP/WASM'ı, LSP parser/checker/runtime'ı doğrudan bilmez. | ADR-046/047 + [katman rehberi](katman-mimarisi.md) + makine-okunur fixture; sahip/kenar birebirliği, lexical ayrım, kök takma-adı reddi ve yön matrisi korunur. SHA-256, takvim, semantic tür, tanı bütçesi ve binary yükleme sahipliği bağımsız alt sahiplerdedir. |
| V1-P0-33 Production dependency graph çevrim veya izin taşımaz | **KAPALI (K-150/K-160)** | Tanı/kaynak ve checker/HIR çevrimlerinden sonra son paket/registry/tedarik SCC'si de saf model, resolver, registry, verification ve yayın sahiplerine ayrıldı. Production SCC ve izin sayısı sıfırdır; yeni çevrim fail-closed'dur. | ADR-047/059 + `bagimlilik_cevrimi_testi` + boş izin fixture'ı; sahiplik testi modelin davranışsızlığını, verification'ın bağımsızlığını, registry devrini ve resolver'ın taşıma ayrıntısından uzaklığını korur. B-058/B-070 kapandı. |
| V1-P0-34 CI üçüncü taraf action kodu hareketli ref'ten çalışmaz | **KAPALI (K-151)** | Bütün workflow `uses:` değerleri, yerel action hariç incelenmiş 40 haneli commit SHA'dır. Action/sürüm/SHA/resmî kaynak kaydı workflow'larla birebirdir; haftalık Dependabot PR'ı otomatik birleşmez. Checkout kimliği kalıcı değildir, token yalnız `contents: read` taşır. | ADR-048 + [tedarik rehberi](tedarik-zinciri.md) + pin TSV'si; workflow dizin keşfi hareketli/kısa/kayıt dışı/farklı SHA'yı ve kullanılmayan pin kaydını reddeder. B-059 toplam 610 testte kapandı. |
| V1-P0-35 Core freeze gerçek ürün kanıtı olmadan açılamaz | **KAPALI (K-160A/B-073)** | Her yeni `compiler/src` commit'i semantic sınıfına ek exact freeze sınıfı taşır. Semantic feature yalnız dogfood/security/correctness olabilir. Dogfood kanıtı append-only kayıttaki etkin ürün ve exact provenance'a, gerçek K-işine, ürün kökü altındaki Zee dosyasına, reproducer'a, minimaliteye ve ilgili normatif karara bağlanır. | ADR-059 + [core freeze beyanı](core-freeze-beyani.md) + [dogfood ürün kaydı](dogfood-projeleri-v1.tsv) + `core_freeze_korugu_testi`; CI beyansız feature'ı, maintenance kaçışını, geçmiş satır yeniden yazımını, kayıtsız ürün/işi, kök dışı dosyayı ve alakasız kararı reddeder. B-071/B-073 kapandı; semantik ilgi insan review'undadır. |
| V1-P0-36 İlk gerçek Zee ürünü laboratuvar dışı kanıt üretir | **KAPALI (K-163)** | legacy panel/legacy Next.js yönetim paneli 1037 gerçek Zee LOC, 5 modül ve 12 hermetik testte auth/CSRF/içerik/ayar/sayfa, gerçek PostgreSQL, bağlantı/COMMIT arızası, binary medya saga'sı, production TLS/havuz ve liveness/readiness taşır. | F030 binary yaşam döngüsü, F031 pinned CA+bounded pool, F032 aynı worker'da DB-down readiness 503/liveness 200/DB dönüşü 200 ve 1000+ bakım skor kartını kapattı. Gerçek ürün LSP full-change p95 41,416 ms; değişiklik başına dosya medyanı 6'dan 5,5'e indi. Managed-provider rotasyonu/çok-worker toplam bütçe K-169/K-173; multipart/içerik güvenliği K-170 kapsamındadır, K-163'ü açık tutmaz. |

K-188/ADR-079, V1-P0-03'ün revoke kanıtını gerçek Zee ERP ihtiyacıyla
genişletir: kullanıcı oturumları ham bearer/CSRF açılmadan özet kimlik ve zaman
metadata'sıyla listelenir; seçili kimlik ortak kalıcı depoda transaction
kapsamında, worker ve restart sınırlarını aşarak iptal edilir.

## P1 — profesyonel kapasite kapıları

| Kapı | Durum | Kaynakta görülen gerçek | Kapanma kanıtı |
|---|---|---|---|
| V1-P1-01 Ondalık hassasiyeti dil semantiği mi profil sınırı mı? | **KAPALI (K-092)** | Ondalık tek keyfî hassasiyetli onluk türdür: katsayı ve ölçek makine kelimesiyle sınırlı değildir. Toplama/çıkarma/çarpma ve sonlu bölüm tam; yalnız sonsuz açılımlı bölüm 34 anlamlı haneye, yarımlar sıfırdan uzağa yuvarlanır. TamSayı i64 kimliğini korur; Ondalığa genişleme kayıpsızdır. | RFC-0013 revizyonu + spec/16; 9+ ve 30+ haneli sabitler, büyük katsayı, çok küçük değer, sonlu/sonsuz bölüm, negatif metin dönüşümü, JSON/para biçimi, karşılaştırma ve i64'e açık daraltma taşması regresyonları. S032 emekliye ayrıldı. |
| V1-P1-02 Morfoloji deterministik ve sürümlenebilirdir | **KAPALI (K-089/K-122/K-123)** | `zee-tr-1` profili soyut ekleri, yüzeyleri, iki katman sınırını ve kanonik üretimi tek modülde sabitler. Doğrudan eşleşme önce; sonra 0=A001, 1=çözüm, 2+=A002, heuristik yoktur. Profil SHA-256 semantic kayıtla immutable ve ikinci compiler için Rust'tan bağımsız veridir. | RFC-0018 + spec/13 + [profil uyumluluk](morfoloji-profil-uyumlulugu.md)/[conformance](morfoloji-conformance.md) rehberleri; 53.248 parmak-izi vektörü ve kök JSON Schema altındaki 27 çözüm/karar+21 üretim vakası. `proje.dil`/kilit profili pinler. Davranış, eski kayıt, schema veya conformance değişikliği test/CI koruğunu kırar; yalnız yeni sürümlü dosya yolu açıktır. Toplam 483 test yeşildir. |
| V1-P1-03 Structured concurrency adı runtime gerçeğini aşmaz | **KAPALI (K-090/K-124)** | `Eszamanli` görevleri dış ortam snapshot'ıyla kaydeder; `HepsiniBekle` kaynak sıralı tek-thread scheduler'da `bekle` noktalarında dönüşümlü ilerletir. `zee-esz-1`; çıktı/ortak IO sırası, sanal süre, sonuç bağları, sonlanma kodu ve iptal sonrası etki yokluğunu gözlenebilir uyumluluk yüzeyi yapar. | RFC-0011 + spec/14 + [eşzamanlılık conformance rehberi](eszamanlilik-conformance.md); 11 maddelik profil sözü ve 10 bağımsız kaynak+gözlem vakası snapshot, eşit/farklı uyanış, iç ağaç, ortak IO, hata/son tarih/çıkış iptali ve atomik eylemi kapsar. Çok çekirdek yalnız aynı gözlemleri veren iç optimizasyon olabilir. Genel Git-tarih koruğu eski profil verisini kilitler; toplam 484 test yeşildir. |
| V1-P1-04 Sonuç hata tarafı yapılandırılmıştır | **KAPALI (K-091)** | `Sonuç<T>` hata tarafı değişmez `Hata`dır: kararlı kod, Türkçe mesaj, `Seçenek<Hata>` neden zinciri ve sıra korumalı Metin sözlüğü verisi taşır. Kod `göre` ile eşlenir; neden güvenli daraltmayla açılır; tam yapı deterministik JSON olur. Eski metin hataları `GENEL` koduyla aynı insan çıktısını korur. | RFC-0008 §3 + spec/15; kod/mesaj/eşleme, iç içe neden, veri+JSON, yeniden yayma, yerleşik kodlar, S044/T052 olumsuzları ve eski kaynak regresyonları. |
| V1-P1-05 Gezmede yazma kullanıcı zihniyle doğrulanmıştır | **AÇIK — makine tarafı K-093 ile tamam** | RFC-0019/spec-17 derin değer kopyasını ve liste için değer-sonuç imlecini tanımlar: alan yazma ile yeniden bağlama aynı sıraya geri yansır; paylaşılan alias yoktur. Kaynak ekleme/silme/yeniden bağlama/aynı-kaynak iç içe gezme T053'tür. Alan/rebind, alias ve 1–24 uzunluk conformance kanıtı yeşildir. | `docs/usability-kiti.md` G1/G2/G3 kartlarıyla gerçek 10 öğrenci + 5 profesyonel sonucu. Önceden taahhütlü eşik sağlanmadan kapatılmaz. |
| V1-P1-06 Yerel modül/paket paylaşımı | **KAPALI (K-076–K-080)** | Kökenli birim/paket yükleme, doğrudan sınır, SHA-256 kilit, güvenli ekle/çıkar ve grafik görünümü çalışıyor. | P001–P010/A011 ve proje entegrasyon testleri; deterministik kilit. |
| V1-P1-07 Dağıtım/registry güven zinciri | **KAPALI (K-094/K-095/K-135/K-136)** | Deterministik yayın, sabit root→timestamp→snapshot→targets, yalnız HTTPS/statik taşıma, 64 ardışık root, CAS korumalı atomik monoton durum ve doğrulanmış salt-okunur cache'e K-136 exact `proje.dil`, `proje.kilit` v3 ve CLI bağını ekler. Normal derleme/LSP ağsızdır; yanked/kritik kabulü insan gerekçesiyle kilide yazılır. | Gerçek imzalı `.zep`/metadata uçtan uca testi çevrimiçi hazırlanmış proje-local cache'i ağsız grafiğe ve gerçek derlemeye bağlar; kök/yayıncı/dört hedef kimliği kilitlenir, bozuk kurulum P016 olur. Manifest P017 olumsuzları ve CLI seçenek regresyonları dahildir. |
| V1-P1-08 Kaynak paket kimliği platformdan ve Unicode gösteriminden bağımsızdır | **KAPALI (K-117)** | Dosya sistemi yolu NFC'ye kanoniklenir, normalizasyon çakışması reddedilir; gelen `.zep` yalnız zaten NFC ve Unicode 17.0 güvenlik kümesine uygun yolu kabul eder. Metadata yoktur ve UTF-8 byte sırası tektir. | RFC-0020 + ADR-028 + spec/18 + [conformance rehberi](zep-conformance.md); Türkçe Unicode adlı sabit `.zep` fixture'ı Linux/macOS/Windows CI'da aynı testtir. 80 kalıcı yol saldırısı ile sıra/tekillik/limit/fazladan byte yapısal olumsuzları fail-closed; toplam 463 test yeşildir. |
| V1-P1-09 Bootstrap bağımlılık zinciri denetlenmiş ve offline kurulabilirdir | **KAPALI (K-141)** | Compiler+fuzz grafikleri yalnız crates.io+lock SHA-256 kaynağından, sabit araçla güncel RustSec/lisans/ban/source politikasından geçer. Kilitsiz CI fallback yoktur. Zee ürün lisansı seçilene kadar iki crate yayımlanamaz. | ADR-038 + [tedarik rehberi](tedarik-zinciri.md); iki gerçek `cargo deny -D warnings` koşusu, dört statik regresyon ve iki kez aynı üretilen vendor manifestinden boş Cargo home ile compiler+fuzz offline derlemesi. Toplam 570 test yeşildir; B-054 kapandı. |
| V1-P1-10 Playground WASM C ABI'si hasım host girdisinde bellek güvenlidir | **KAPALI (K-142)** | `dil ABI v2` tabanı yalnız kayıtlı başlangıç pointer'ı+exact boyu kopyalar; strict UTF-8 ister. Sonuç türü ve toplamı kayıttadır; yanlış/çift bırakma sahipliği bozmaz. Aynı-page JavaScript'e sandbox sözü vermez. | ADR-039 + [güncel ABI rehberi](wasm-c-abi.md); null/iç/yabancı pointer, `usize::MAX`, invalid UTF-8, yaşam döngüsü ve 64 ardışık bozuk/geçerli native regresyon; gerçek wasm32 Node hostu, dört K-142 byte seed ve 1.745.134 çağrılık ayrı libFuzzer kampanyası. Toplam 575 testte B-055 kapandı; K-143 ABI v3 bütçe dışa aktarımlarını ekledi. |
| V1-P1-11 Playground girdisi tahsisten önce sınırlıdır | **KAPALI (K-143)** | ABI v3 merkezî profilden 8 MiB kaynak ve 1 MiB/4.096 satır soru limitini hosta bildirir. Rust byte sınırını sahipli kopyadan, satır sınırını `Vec<String>`den önce; tarayıcı UTF-8 byte dizisi ve WASM tahsisinden önce uygular. Sınırda kabul, bir fazlasında görünür fail-closed red vardır. | RFC-0025 + ADR-040 + spec/24 + [ABI v3 rehberi](wasm-c-abi.md); native sınır/bir-fazlası ve allocation-order regresyonları, gerçek wasm32 Node matrisi, kaynak+soru kipli beş kalıcı seed ve gecelik fuzzer. Toplam 579 test yeşildir; B-056 kapandı. |
| V1-P1-12 Performans eğilimi faz ve kullanıcı yüzeyi bazında gözlenebilir | **KAPALI (K-148/K-152)** | Release koşucusu parse, resolver/checker+HIR kanıtı, tam typed-HIR ön ucu, runtime başlangıcı, yürütme, LSP cold/open/change ve Unix tepe RSS'i için ham örnek, min/max ve p50/p95 üretir. K-152 her tarihçe satırını tam Git SHA, ayrı milestone, temiz Git ağacı, OS/CPU/RAM/Rust/release ve gerçek sample/warmup/örnekleme semantiğine bağladı. RSS 25 tur değil tek süreç-tepe görüntüsüdür. Shared CI hard gate değildir. | ADR-045/049 + [ölçüm rehberi](olcumler.md) + [sürümlü tarihçe](performans-gecmisi-v2.tsv); sekiz koşucu birim testi v2 provenance, yüzdelik, RSS ve eşik politikasını; mimari regresyon CI kablolamasını korur. K-148 sayıları exact `df737f…` commit'ine göçtü; B-040/B-060 kapalıdır. |
| V1-P1-13 Editör cold-start maliyeti kullanıcı süreci açısından ölçülür | **KAPALI (K-153)** | `lsp_engine_initialize` yalnız in-process maliyettir. `lsp_process_cold_start` gerçek dillsp spawn, stdio framing ve tam capabilities yanıtını kapsar; olmayan workspace-load metriği uydurulmaz. | ADR-050 + [ölçüm rehberi](olcumler.md) + gerçek ikili entegrasyon testi ve release CI kablosu. Exact temiz `a2693d6…` uygulama commit'inde 25 örnek process p50 1,557 ms/p95 1,997 ms; engine p50 542 ns/p95 625 ns'dir. B-061 kapalıdır. |
| V1-P1-14 LSP değişiklik maliyetinin proje boyutu kırılma noktası bilinir | **KAPALI (K-154)** | `--lsp-olcek`, 2k/5k/10k/20k tam-metin `didChange` p50/p95 eğrisini ve p95 250/500/1000 ms ilk aşımını raporlar. K-154 tarihsel ölçüm kaynağında invalidation tam belge+tam lexer/parser+resolver/checker+typed-HIR idi. K-287 token/yaprak AST yeniden kullanımını ekledi; S4 stdio gözlemi ayrı kayıttır, artımlı HIR/type açık kalır. | ADR-051 + [ölçüm rehberi](olcumler.md) + [sürümlü tarihçe](performans-gecmisi-v2.tsv) + koşucu birim/mimari CI regresyonu. Exact temiz `59580cd…` uygulama commit'indeki 25 örnekte p95 2k=167,281 ms, 5k=1.081,746 ms, 10k=4.669,372 ms, 20k=20.159,636 ms; üç eşik de ilk kez 5k'da aşılır. B-062 kapalıdır. |
| V1-P1-15 Düzeltilen compiler bug'ı kalıcı ve kaynaklanmış iz bırakır | **KAPALI (K-147/K-155/K-155A)** | 17 minimal fixture'ın v2 satırı K-kimliği ve gözlemin yanında exact `fixed_by`, kanıtlıysa `introduced_by` ve `guaranteed_since=0.8.0-dev` taşır. Bilinmeyen introduced commit tahmin edilmez. | ADR-044/052/053 + [semantic regresyon rehberi](semantic-regresyon-korpusu.md) + [compiler değişiklik beyanı](compiler-semantic-degisiklik-beyani.md) + veri-güdümlü iki korpus testi + geçici gerçek Git deposunda koruğun red/kabul/yeniden-yazım yollarını çalıştıran entegrasyon testi. V1 tabanı v2 göçünde okunur; sabit başlangıçtan sonraki her compiler kaynak commit'i mesajından bağımsız tekil beyan taşır ve bugfix exact `fixed_by` sahibi yeni fixture olmadan geçemez. B-039/B-063/B-064 kapalıdır. |
| V1-P1-16 Form kodlaması ham girdiyi sessizce değiştirmez | **KAPALI (K-176)** | Sorgu/form adı ve değeri tam `%xx` ve exact UTF-8 ister; bozuk girdi rota çalışmadan 400'dür. | ADR-054 + spec/11 + hedefli web testi + `web` kipli `bozuk-form-kodlamasi` fixture'ı. `fixed_by=32247c7…` exact uygulama commit'i compiler semantic bugfix beyanıyla eşleşir; B-065 kapalıdır. |
| V1-P1-17 Fuzz öğrenimi cache ömrüne bağlı değildir | **KAPALI (K-156)** | Dört hedefin koşu sonu coverage korpusu başarı/başarısızlıktan bağımsız 90 günlük artefakttır; manifest kaynak commit, run/attempt, sabit araçlar ve her seed'in SHA-256+yolunu taşır. Cache yalnız hızlandırmadır. | ADR-055 + [fuzz rehberi](fuzzing.md) + `fuzz_korpusu_testi`. Doğrulayıcı geçerli artefaktı kabul edip değiştirilmiş seed'i reddeder; kalıcı repoya terfi `cmin`, stable replay ve insan review'u ister. B-066 kapalıdır. |
| V1-P1-18 Release candidate uzun fuzz ve seçili Miri kapısından geçer | **KAPALI (K-157)** | Dört hedef aynı commit'te en az 30 dakika explicit AddressSanitizer ile koşar; haftalık/elle CI 60 dakika ister. Sonuç ve korpus provenance'lı 90 günlük artefakttır. | ADR-056 + [RC fuzz rehberi](fuzz-rc.md) + append-only tarihçe. İlk taban toplam 136.789.564 yürütme, sıfır crash/timeout/ASan bulgusu ve 3/3 Miri testidir; B-067 kapalıdır. |
| V1-P1-19 Test blast radius'u gerçek çapraz kapsama dayanır | **KAPALI (K-158)** | Her test tek bir birincil faza aittir; kritik seçiciler gerçekten yürüttükleri diğer fazları opsiyonel çoklu `ek_kapsam` ile bildirir. Metadata fail-closed doğrulanır. | ADR-057 + [faz matrisi](faz-test-matrisi.md); rapor birincil test, çapraz kapsayan exact seçici ve mimari aşağı akışı ayırır. B-068 kapalıdır. |
| V1-P1-20 Desteklenen Rust API internal yüzeyden ayrıdır | **KAPALI (K-159)** | Gömme sözleşmesi exact `dil::api::v1` allowlist'idir; kök legacy modüller internal sınıflanır. | ADR-058 + [public API politikası](public-api-politikasi.md) + `public_api_testi`; facade smoke, exact ihracat ve kök sızıntısı fail-closed korunur. B-069 kapalıdır. |
| V1-P1-21 Uyumluluk sözü ve deprecation süresi makinece yürütülür | **KAPALI (K-167)** | Kalıp kelimeleri, koşul yüklemleri, CLI komutları, `proje.kilit`/ZEP/yayın zarfı biçimleri, `zee-tr-1`/`zee-io-1`/`zee-esz-1` profilleri, WASM ABI v3, `api::v1` ve tanı şeması tek envanterde giriş sürümüyle donuktur. Kaldırma yalnız ardışık `DEP-NNN` kaydı, desteklenen yüzeyde bir alt sürüm serisi süre ve göç yoluyla olur; kaldırılan kelime mezar taşı kalır. Etiket sonrası çalışma ağacı `-dev` serisi taşır. | RFC-0028 + ADR-064 + spec/27 + [uyumluluk rehberi](uyumluluk-politikasi.md); `uyumluluk_testi` fixture↔kaynak birebirliğini, exact biçim/profil kayıtlarını, deprecation şema/süre kuralını, tanı mezar taşı çaprazını, kaldırılan `dil::tedarik` (DEP-004, B-072) yokluğunu ve sentetik bozuk girdi reddini korur. Edition sözdizimi açık RFC işidir. |
| V1-P1-22 Spec maddesi ile test kanıtı madde düzeyinde drift edemez | **KAPALI (K-171)** | Her ZORUNLU/ZORUNDA/YASAK/TANIMLI/AÇIK işaretli spec maddesi parmak-izi kimliğiyle exact `<dosya>::<işlev>` kanıtına ya da gerekçeli kısmi/açık durumuna bağlıdır. Metni değişen madde yeni kimlik alır; kayıtsız madde, spec'ten kaybolan kayıt, bayat özet, kayıp test işlevi ve bayat rapor fail-closed'dur. | ADR-065 + [drift rehberi](spec-drift.md) + [rapor](spec-drift-raporu.md); ilk taban 169 madde/151 kanıtlı/16 kısmi/2 açık. `spec_drift` birim testleri çıkarım, şema ve drift reddini; `mimari_sinir_testi` CI kablosunu korur. RFC/ADR belge düzeyi kanıt haritasında kalır. |
| V1-P1-23 Sürüm adayı açık kritik/yüksek güvenlik bulgusu taşımaz | **KAPALI (K-170)** | Ardışık `GB-NNN` bulgu kaydı kaynak, önem, yüzey, durum, kapanış K-işi ve karar belgesi taşır. Açık bulgu yalnız orta/düşük ve hedef işli; kapalı bulgu gerçek K-işine, kabul edilen sınır normatif spec/ADR'ye bağlı; kritik sınır kabul edilemez. Tek betik sürekli kipte cargo-deny+fuzz derlemesi+kayıt, sürüm adayı kipinde exact HEAD RC fuzz kanıtı, clippy, tam test ve korukları ister. | ADR-066 + [SECURITY.md](../SECURITY.md) + [rehber](guvenlik-surum-kapisi.md) + `guvenlik_kapisi_testi`; ilk taban 22 kayıt: 17 kapalı, 4 kabul, 1 açık orta (GB-022, K-173 hedefli), açık kritik/yüksek sıfır. `tedarik` workflow'u her push/PR/gecelik koşuda `--surekli` çalıştırır. |
| V1-P1-24 Gerçek ürün sürtünmesi kalıcı korpusta yaşar | **KAPALI (K-172)** | `dogfood/` altındaki her `.dil` kaynağı ürün slug'ı, K-işi/dogfood dilimi, kip, beklenti ve exact çıktı/tanıyla manifestte tam bir kez kayıtlıdır; ürün `proje.dil` politikasıyla derlenir, hermetik IO ile çalışır, kanonik biçimde ve sürtünme yorumuyla başlar. Korpus yalnız dogfood'dan büyür. | ADR-067 + [rehber](dogfood-korpusu.md) + `dogfood_korpusu_testi`; ilk taban 10 vaka (4 gereksinim aynası, F007/F012/F030/F032 ret+çözüm çiftleri). GB-020/GB-021 gerçek adaptör birim testleriyle aynı işte kapandı. |
| V1-P1-25 Sürüm ikilisi iki temiz ortamda eş hash, SBOM, imza ve provenance taşır | **ÜÇ PLATFORM İMZALI ADAY KANITLI (69a6d8f, K-317/K-319); FİZİKSEL CİHAZ P29 VE İLK YAYIN P30 AÇIK** | Betik aynı commit'i iki bağımsız git klonunda sabit toolchain, `--remap-path-prefix` ve commit zamanı `SOURCE_DATE_EPOCH` ile derler; klonlar önce/sonra temiz, gerçek ikili exact SHA/`dirty: false` olmalıdır. `dil`/`dillsp` SHA-256'ları eşit değilse artefakt yoktur. SPDX 3.0.1 SBOM `Cargo.lock` bağımlılıklarını, SLSA v1 provenance commit/rustc/platform ve iki klon eşitliğini, `zee-surum-imza-v1` Ed25519 imzayı deterministik taşır. | [K-241 / ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md) sonrası temiz 2e970e4 kaynağında [yerel macOS kanıtı](incelemeler/urunlesme-20261014/final-2e970e4/build-proof.json): a/b/çıktı exact SHA ve `dirty: false`, eş hash, SBOM/provenance; imzasızdır. Sonraki [tarihsel S3](urunlesme-s3-kabul-20261015.md) kaynağında macOS ve Linux iki temiz klonda eş CLI/LSP, SBOM/provenance ve kurulum kabulü ayrıca kaydedildi. [S4 kaynağında](urunlesme-s4-kabul-20261015.md) `4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2` için macOS iki temiz klon ve sekiz kurulum vakası ayrıca geçti. S3 Linux sonucu S4'te yeniden koşulmuş sayılmaz; fiziksel Windows/minimum OS ve gerçek yayıncı imzası açıktır. K-168/ADR-068 tarihsel kanıtı ve a2b8557 dirty kimlik reddi korunur. **19.10.2026:** [üç platform imzalı aday](urunlesme-p125-imzali-surum-20261019.md) `69a6d8f` kaynağında Linux/macOS/Windows iki temiz klon eş hash, SBOM/provenance ve resmî yayıncı imzasıyla geçti; imza derleyicisiz OpenSSL ile doğrulandı. [Rehber](tekrar-uretilebilir-surum.md), `surum_artefakti`, `tedarik_kapisi_testi` ve `surum-adayi` kablosu; platformlar arası eşitlik ve SLSA L2/L3 iddia edilmez. |
| V1-P1-26 Uzun compiler/LSP oturumu kaynak sızdırmaz | **KAPALI (K-173)** | `soak` koşucusu bütün golden korpusunu derleyip sabit programı yürütürken gerçek `dillsp` sürecine içeriği değişen 2.000 satırlık `didChange` döngüsü sürer; her 5 sn iki sürecin RSS'i örneklenir. Isınma penceresi atılır; ısınma sonrası ilk/son pencere medyanı büyümesi hem %10 hem 32 MiB'ı aşarsa kapı kalır. Tarihçe yalnız temiz ağaçtan exact SHA, platform, rustc, süre, dönem ve medyanlarla yazılır. | ADR-069 + [rehber](soak.md) + [tarihçe](soak-gecmisi-v1.tsv) + `soak` birim testleri (ısınma atımı, çift eşik, provenance satırı) + `mimari_sinir_testi` kablosu + haftalık `soak` workflow'u. İlk 30 dakikalık taban exact SHA ile tarihçededir. Mutlak bellek bütçesi spec/24'ün işidir; Windows'ta `ps` olmadığı için koşu Unix Tier-1'dedir. |
| V1-P1-27 En sık acemi hatalarında tanı işaret, öneri ve gürültü ölçülüdür | **KAPALI (K-166)** | Golden korpusuna 11 deterministik acemi hatası uygulanır; (operatör, kod) sınıfı başına ilk tanının işareti ±1 satır ≥ %90, öneri %100, gürültü medyan ≤ 2 tepe ≤ 4. En az 3 vakalı sınıflar frekansla en çok 50 taneye kadar kapıdadır; “en sık” insan verisi gelene kadar korpus frekansıdır. | ADR-070 + [rapor](tani-kalitesi.md) + `tani_kalitesi` birim testleri + `parser_kurtarma_testi` (S007 önerisi, gövde dışı parametre S004, A001/A007 tek tanı) + regresyon vakaları. İlk taban 244 vaka/14 sınıf: düzeltmelerden önce 6 ihlal, sonra sıfır ihlal ve sıfır istisna. |
| V1-P1-28 Kurulum, kaldırma ve sürüm günü tatbikatlıdır | **KAPALI (K-169)** | Kurulum yalnız `SHA256SUMS` doğrulanan `dil`/`dillsp` için; var olan dosya ezilmez, manifest varken ikinci kurulum reddedilir. Kaldırma yalnız manifestteki, özeti değişmemiş dosyaları siler. Runbook sürüm gününün bağlayıcı sırasıdır. | ADR-071 + [runbook](surum-runbook.md) + `kurulum_testi` (gerçek ikililerle kur→sürüm→kaldır, ikinci kurulum ve oynanmış artefakt reddi, Unix) + `kurulum-tatbikati` workflow'u (Ubuntu/macOS/Windows, etikette). Yerel macOS tatbikatı geçti; Linux/Windows koşusu workflow kanıtıdır. |
| V1-P1-29 İkinci gerçek ürün farklı iş yükünde dilde yazılmıştır | **KAPALI (K-164)** | `dogfood/kanit-ozeti` toplu metin/veri işleme aracıdır (TSV → Markdown; web/DB/ağ yok): 10 birim, 19 birim testi, `docs/kanit-ozeti.md` çıktısı. On altı sürtünme kaydedildi; elmas birim içe alımı ve birim kökenli tanı compiler düzeltmesi oldu, kalanı dilde çözülüp korpusa ret/çözüm çifti olarak girdi. | ADR-072 + `kanit_ozeti_testi` (hermetik koşu, bayt bayt tazelik, determinizm, bağımsız sayım) + `dogfood_korpusu_testi` `proje` kipi + `birim_testi`/`lsp_testi` (elmas, köken) + CI yeniden üretim/diff adımı. |
| V1-P1-30 Aynı araç Zee, Rust ve Go ile veri temelli karşılaştırılmıştır | **KAPALI (K-165)** | Üç gerçekleme aynı sayfayı bayt bayt üretir; kaynak satırı, test sayısı, derleme/çalışma süresi, tepe RSS ve sürtünme sayısı exact SHA tarihçesindedir. Zee'nin kaybettiği yerler (yorumlayıcı süresi, sıra erişimi/sekme kaçışı/morfoloji kısıtları) belgede açıkça listelenir. | ADR-073 + `dogfood_karsilastirma_testi` (Rust/Go bayt bayt eşdeğerlik, tarihçe SHA ve yeniden hesaplanan satır/test sayıları) + `scripts/dogfood-karsilastirma.sh` + [veri sayfası](dogfood-karsilastirma.md). |

K-203/ADR-081 resmî siteyi gerçek Zee ürünü olarak ekosisteme kazandırır;
ancak yeni bir V1 kabul kapısı değildir. Gerçek WASM derleyicili yerel
playground vardır ve site 7 Ekim 2026'dan beri public yayındadır; paket
araması açıkken site V1 RC'yi kapatmış gösterilemez.

## Uygulama sırası

1. Güvenli olmayan çalışan web/eşzamanlılık yüzeylerinin opt-in sınırını koru
   (K-082); son tarih iptali artık K-085 ile ayrı ve tanımlıdır.
2. K-083/K-086 public işlem sözleşmesini kitaplık, paket ve gelecekteki eylem
   API'lerinin değişmez tabanı olarak koru.
3. K-087 eylem sınırını, K-088 oturum/CSRF/proxy profilini, K-134
   session/cookie/yanıt request transaction'ını ve K-137 ortak kalıcı
   oturum/rate-limit + tek-worker/N süreç modelini koru. Çok-dosyalı çökme
   atomikliği ve çok-hostlu harici web deposu verilmiş söz değildir.
4. K-090, K-085 deadline çekirdeğinin üstüne deterministik scheduler,
   sözcüksel sahiplik ve kardeş iptalini koydu; K-124 gözlenebilir sonucu
   `zee-esz-1` bağımsız profiline kilitledi. K-091 Sonuç'un hata tarafını
   kodlu, zincirli ve geriye uyumlu Hata değerine dönüştürdü. K-092 Ondalık
   kapasitesini keyfî katsayı + açık 34 haneli sonsuz bölüm kuralıyla kapattı.
5. K-089 `zee-tr-1` morfolojisini property kanıtıyla dondurdu; K-122 semantic
   parmak izi ve tarih koruğuyla aynı kimlik altında değişikliği kapattı;
   K-123 ikinci compiler için bağımsız JSON korpusunu kurdu. K-093 gezmenin
   değer/reference kararını RFC-0019/spec-17, T053 ve property korpusuyla
   makine tarafında tamamladı; yalnız gerçek usability formları bekleniyor.
6. K-094 yayın çekirdeğini Ed25519 + deterministik `.zep` + SPDX/SLSA zinciriyle
   kurdu. K-095 root/çevrimiçi rol metadata doğrulamasını ve targets
   politikalarını saldırı matrisiyle kapattı. K-135 limitli HTTPS taşıma,
   kalıcı durum ve doğrulanmış cache/offline katmanını ekledi. K-136 exact
   CLI/manifest/kilit v3 ve atomik kaynak kurulumunu uçtan uca bağlayarak
   V1-P1-07'yi kapattı. K-141/ADR-038 sabit cargo-deny advisory/lisans/ban/
   source taramasını, kilitli CI'ı ve iki üretimli boş-cache offline vendor
   kanıtını ekleyerek V1-P1-09/B-054'ü kapattı.
   K-142/ADR-039 kayıtlı exact pointer/uzunluk, strict UTF-8, sorgulanabilir
   sonuç ömrü ve hasım host fuzz hattıyla V1-P1-10/B-055'i kapattı.
7. K-096 çağrı deneyi gerçek insan verisini bekler; V1-P0-07 bu kanıt olmadan
   kapanmaz. K-097 expression grammar katmanlarını dondurdu ve V1-P0-08'i
   kapattı. K-098 alan yüzeylerini generic intrinsic/yetkinlik modeline taşıdı
   ve V1-P0-09'u kapattı. K-099 fiziksel faz sınırlarını kurup V1-P0-10'u
   kapattı. K-100 checker semantik katmanlarını tek sahipli hale getirip
   V1-P0-11'i kapattı. K-101 semantic identity'yi depolama konumundan ayırıp
   V1-P0-12'yi kapattı. K-102 gerçek veri fazlarını türleyip V1-P0-13'ü
   kapattı. K-103 typed HIR üretim çekirdeğini, K-104 bağlı runtime tüketimini
   tamamlayıp V1-P0-14'ü kapattı.
8. K-105/ADR-017 native ağın sınırsız bekleme ve bellek yollarını, K-106/
   ADR-018 process içi oturum kotasını ve mutlak ömrü, K-107/ADR-019 LSP
   tek-girdi sınırını kapattı. K-108/ADR-020 her semantic HIR ifadesine
   zorunlu kaynak aralığı ekleyip V1-P0-18'i, K-109/ADR-021 production
   doğrudan panic yüzeyini kapatıp V1-P0-19'u, K-110/ADR-022 lexer/parser fuzz
   hattını kurup V1-P0-20'yi tamamladı. K-111 morfoloji property/fuzz hattıyla
   V1-P0-21'i, K-112 AST/HIR invariant kapısıyla V1-P0-22'yi, K-113 parser
   kurtarma politikasıyla V1-P0-23'ü ve K-114 tanı kimliği fixture'ıyla
   V1-P0-24'ü kapattı. K-115/RFC-0022 bütün runtime IO sınırını sürümlü,
   kanonik ve dış etkisiz replay protokolüne bağlayıp V1-P0-25'i kapattı.
   K-116/RFC-0023 tohum, sanal saat ve hermetik adaptör davranışını
   `zee-io-1` profiline sabitleyip V1-P0-26'yı kapattı.
   K-117/ADR-028 NFC yol kanonikliğini, Tier-1 `.zep` fixture'ını ve Unicode
   saldırı korpusunu bağlayıp V1-P1-08'i kapattı.
   K-125/ADR-029 Ondalık'ın binary float'a örtük FFI eşlemesini yasaklayıp
   V1-P0-28'i kapattı; FFI'nın kendisi Faz 4/5 taslağı olarak kaldı.
   K-126/ADR-030 bütün AST ifadelerini kesin kaynaklandırıp K-108'in
   V1-P0-18 sözünü tanı ve LSP tüketimine kadar tamamladı; B-050 kapandı.
   K-127/RFC-0024/ADR-031 proje/paket dış dünya yetkilerini compile+runtime
   kapısına, native outbound'u HTTPS+origin+DNS/IP sınırına bağlayıp
   V1-P0-29 ile B-023/B-049'u kapattı.
   K-138/ADR-035 LSP JSON-RPC sayı, duplicate alan, zarf ve standart hata
   davranışını protokol-kesin yapıp V1-P0-17'nin B-051 açıklığını kapattı.
   K-139/ADR-036 outbound ve inbound proxy origin'ini tek `AgHedefi` kimliğinde
   birleştirip loopback bind+peer değişmezini kanıtladı; B-052 kapandı.
   K-140/ADR-037 HTTP framing'i byte parser ve ayrı libFuzzer hedefiyle
   sabitleyip B-053'ü kapattı.
9. Yeni dil özelliğinden önce bağlayıcı sıra
   [öncelikli backlog](oncelikli-backlog.md) içindeki P0 compiler omurgasıdır.

Her kapının kapanışı: karar + spec + olumlu/olumsuz test + sürüm notu. Yalnız
“kod çalışıyor” işareti v1 kanıtı değildir (ADR-010).

K-195 Windows statik CRT profili ve çift-klon dağıtım kablosu hazırdır; exact `c7e3d04` kaynağının üç platform tatbikatı geçti ve K-195 kapandı. K-192 temiz Windows/macOS ve minimum OS/linker-SDK kanıtı açıktır.

K-195 ilk Windows çift-klon koşusu eş-hash kapısında kaldı; artefakt
üretilmedi. Yeni MSVC linker tarifi `/Brepro` ve provenance
`targetRustflags` alanını birlikte günceller; exact tekrar kanıtı gerekir.
Eş olmayan byte'lar ayrı `yayinlanamaz-surum-farki` tanısında saklanır,
sürüm/kurulum kümesine girmez. Benchmark tarihçesine yeni satır yazılmadı.

K-195 kapanma: `c7e3d04adf27905ade8b622d24bd76762123d2dc` kaynağındaki
[38244549913 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38244549913)
üç platformda geçti. Windows iki temiz klonda eş hash, dinamik CRT importu
yok ve kur/program/LSP/kaldır başarılı. İndirilen byte'lar metadata ile
ayrıca eşleştirildi; ham rapor/provenance kalıcı fixture'dadır. İlk Windows
reddi korunur. K-192 temiz Windows/macOS, minimum OS/linker-SDK kanıtı
ayrı açık kalır; native Zee backend ve self-hosting tamamlanmış değildir.

K-196 linker/SDK kayıt dilimi exact üç platform kanıtıyla kapandı: release iki klonun gerçek link
komutunu, program path/hash/version ve SDK referanslarını provenance'a
bağlar. Unix `-v`, MSVC `/VERBOSE:LIB` tarifte görünür; Python 3 build kanıtı
aracıdır, runtime bağımlılığı değildir. Eksik kanıt yayımlanmadan reddedilir.
Exact b153064 kaynağı/run 38252942210 üç platformda geçti; minimum OS/temiz Windows-macOS ve
bütün native C araçlarının kaydı tamamlanmış gösterilmez. Performans
baseline'ı/tarihçe satırı üretilmedi; ADR-045/049 kuralları korunur.

K-192 ek kanıt: exact `e0f5363` Linux ikilileri AlmaLinux 9.8 minimal
userland/glibc 2.34 üzerinde ağsız, read-only, ayrıcalıksız program ve LSP
provasını geçti; Rust paketleri/PATH araçları yok. [Ham rapor](temiz-almalinux-runtime-kaniti-2026-10-12.json).
Bu seçili amd64 emülasyon provası tüm dil IO davranışları, minimum kernel
ve temiz Windows/macOS garantisi değildir; K-192 kısmen açık kalır.

K-197 temiz Windows runtime provası exact kanıtıyla kapandı; exact release kaynağı/hash, immutable Nano Server image,
ContainerUser/network none/read-only bind ile program ve LSP denenir.
Nano Server program/LSP sonucu geçti; masaüstü/minimum OS ve bütün IO
desteği ayrı açıktır. Release/prova kaynak SHA'ları ayrı kaydedilir.
ADR-068/GB-034; spec/RFC/tanı/API/production Rust katmanları ve benchmark
tarihçesi değişmez.

K-197 kapanma: release c7e3d04, prova 27c0fa9/run 38253777857/job
114826470199. [Ham rapor](temiz-windows-runtime-kaniti-2026-10-12.json),
[defter](temiz-windows-platform-kaniti-v1.tsv); Nano Server 10.0.26100.33438
program/LSP geçti. Kurulum/kaldırma, desktop/minimum OS ve bütün IO ayrı.

K-196 kapanma: b153064/run 38252942210; iki klon eş hash ve gerçek
kur/program/LSP/kaldır üç platformda geçti. GCC 13.3.0, Apple Clang 21.0.0
+ SDK 26.5, MSVC FileVersion 14.51.36260.0 + SDK 10.0.26100.0 dört
link kaydında doğrulandı. Ham runtime/provenance ve artifact hash'leri
[platform defterinde](derleme-platform-kaniti-v1.tsv) korunur. K-192 minimum OS, bütün native C
araçları ve fiziksel temiz makine kanıtı ayrıca açıktır.

K-198 temiz Server Core kurulum/kaldırma dilimi exact kanıtla kapandı; exact source/
byte/script hash'leri, Türkçe boşluklu ContainerUser dizini, manifest
path/hash/source, program/LSP ve kaldırma yokluk kanıtı istenir. Gerçek
Windows koşusu geçti; desktop/minimum OS/fiziksel macOS ve bütün IO
tamamlanmış gösterilmez. ADR-068/GB-035. Spec/RFC/tanı/API, production
Rust katmanları, release dosya kümesi ve benchmark tarihçesi değişmez.

K-198 kapanma: release b153064/prova 33176cf/run 38255788989/job
114833388588; stock Server Core 10.0.26100.33438'de Rust kurulmadan
Türkçe dizine kur/program/LSP/kaldır geçti. [Ham rapor](temiz-windows-kurulum-kaniti-2026-10-12.json)
ve [kimlik/hash defteri](temiz-windows-platform-kaniti-v1.tsv) kalıcıdır.
ContainerUser/network none/read-only bind; host kernel, fiziksel/desktop
minimum OS ve tüm IO kapsamı ayrı açıktır.

K-199 Cargo compiler/wrapper pin düzeltmesi üç platform exact kanıtıyla kapandı: helper
exact kurulu rustc executable path/version'ını Cargo RUSTC'ye bağlar ve
wrapper/config seçimini boş değerle kapatır. Önce helper 1.93.1 bildirip
Cargo RUSTC ile 1.97.1 çağırabiliyordu. Resmî release/offline girişi
ambient compiler/wrapper seçimini korumaz; custom geliştirme wrapper
kendi Cargo çağrısıyla kullanılır. Gerçek küçük Cargo poisoned ENV/config
provası exact 22ea2b8/run 38259341995 üç platformda geçti. ADR-068/GB-036; dil spec/RFC/
tanı/API/production Rust kenarı ve release dosya kümesi değişmez.

K-199 kapanma: 22ea2b8/run 38259341995; Linux/macOS/Windows gerçek
Cargo/build-script/ikili Rust 1.93.1, empty wrappers ve temiz source
kanıtı [kalıcı defterde](rustc-pin-platform-kaniti-v1.tsv). Helper/prova/pin
hash'leri kaynakla ayrıca eşleşir. Tam Zee release ve bütün Cargo
profil/linker/native C ayarları bu dar probe'un kapsamı değildir.

K-200 macOS C/Rust deployment uyum kapısı kapalı: final minos 11.0
bildiren eski derlemede C nesnesi 27.0 idi. Resmî release exact Rust
default deployment'ını C/Rust'a ortak export eder; iki klon native .o
başlık/hash kanıtı provenance'a girer. Yerel sonrası 24/24 nesne 11.0
ve program/LSP geçti. Exact kaynak `44fba404c79762aff1698a07978c79ea069a59df`,
run 38263264133 üç platform iki-klon/kurulum/program/LSP/kaldırma kapısını
geçti; macOS a/b klonlarında ayrı ayrı 24 nesnenin minos değeri 11.0. Gerçek
macOS 11/minimum OS/tüm native alt süreç desteği değildir. ADR-068/GB-037;
spec/RFC/tanı/API/production Rust katmanı ve release dosya kümesi değişmez.

K-201 native C/archiver driver çağrı kaydı hazırdır; iki klonda compile ve
archive zorunlu, paket/argüman/executable path/hash ve sürüm sorgusu sonucu
provenance'a eklenir. Exact kaynak 25b6882/run 38349601833 üç platform release kanıtı geçti.
pkg-config/vcpkg, driver iç alt süreçleri ve minimum OS K-192'de ayrıca
açık kalır. Ayrıntı [derleme rehberinde](derleme-ve-bagimliliklar.md).

K-202: ana GitHub/CI deposu `merbay-erp/zee`; `itwise-bilisim/zee` yalnız
ayna ve repository Actions kapalıdır. SBOM/provenance yeni ana kaynak
URI kullanır; eski CI/fixture kanıtları tarihsel bağlantılarıyla korunur.

K-204: sır taramasının dört kamuya açık belge yanlış pozitifine yalnız
immutable fingerprint/kaynak satırı hash doğrulaması uygulandı; yeni
anahtar exit 1/REDACTED testi, default kurallar ve iki tarama korunur.
K-202 beyan sonrası türetilen kanıt özeti yenilendi.

K-207 başarılı ad çözümlemesini hash üyeliğine taşıdı; full ad listesi
yalnız A001 önerisinde üretilir, A002 ve exact-ad kararı değişmez. Oracle
26.624 karar, golden/morfoloji/LSP ve mimari kapıları geçti; exact önce/
sonra performans kanıtı açık. K-206 PG sonucu bellek deneyi GB-041 olarak
orta/açık kaydedildi; 16 MiB sonuç bütün driver belleği tavanı değildir.

K-207 ilk exact ölçümü: 29be85b → 94de420 aynı M4/Rust/release/25+2
koşulunda 20k p95 22.169,906 → 2.961,638 ms. Raw sample ve tarihçe
[ölçümlerde](olcumler.md). Kapsam maliyeti ve 1 sn hedefi açık;
incremental parser/cache veya genel platform SLA eklenmedi.


K-207 kapsam dilimi kapalı: temiz exact 9e80b69 release/25 örnek/2 ısınma
20k LSP p95 165,137 ms; aynı exact kaynakta 1800 sn/264.343 döngü soak
geçti. Derleyici RSS medyanı 8.144→8.160 KiB, LSP 19.088→17.568 KiB.
Ham örnek/çevre ve tarihçe [ölçümlerde](olcumler.md). 728/728 faz,
26.624 ad oracle ve 4.096 kapsam işlemi oracle karşılaştırması geçti.
K-207 KISMEN; full didChange ve incremental/cache kapasitesi açıktır.

K-206 native okuma akış toplamasına geçti: sonuç sınırından sonra çıktı
biriktirilmez; borrowed metin kopyalanmadan denetlenir. Hata önceliği ve
mevcut read retry/transaction/API korunur. Tek wire satırı/driver RSS
bütçesi GB-041'de açık; exact native kabul ve RSS kaydı aşağıdaki kapanışta saklıdır.

K-206 akış toplama dilimi kapalı: source f2ba654, temiz beyan/ölçüm
6463a7f; 731/731 faz ve gerçek native oracle/transaction kabulü geçti.
64×1 MiB sonuçta native süreç RSS 128,77→23,81 MiB, tek 64 MiB satırda
136,77→72,72 MiB; üç vakada sonraki sorgu başarılıdır. Bir örnek/sıfır
ısınma gözlemi, genel RSS garantisi değildir. [Ham kanıt](incelemeler/k206/README.md).
**K-206 KISMEN:** tek wire satırı/driver sınırı ve GB-041 açık kalır.

K-206 tedarik hazırlığı: upstream protokol Rust src byte'ları değişmeden
local patch kimliği compiler/fuzz lock ve doğrulanmış SPDX kaynak/tree
hash ile kaydedildi. GB-042 kapalı; GB-041/erken wire sınırı açık.
Offline vendor ve iki-clone release kabulü kapanış öncesi beklenir.

K-206 path patch iki-clone kabulünde reddedildi; yalnız kaynak kimliği
exact ana repo Git SHA pinine çevrildi. Protokol davranışı değişmedi.
SBOM yerel + Git blob tree hash eşitliği ve exact lock source ister;
v2 kayıt etkin, v1 tarihsel. Yeniden offline/release kanıtı beklenir.

K-206 dependency kimliği dilimi kapalı: source 9dae3e6, temiz beyan/
ikili üretimi ab8e97e. 735/735 faz, git pin/blob/tree SBOM, offline vendor
163 paket ve iki temiz klonda eş dil/dillsp SHA-256 geçti. Yerel macOS
kabulüdür; [ham kanıt](incelemeler/k206/source-identity/README.md).
Path kaynaklı başarısız iki-clone koşusu da saklanır. Protokol src byte'ları
upstream ile aynıdır; GB-041/32 MiB erken wire guard hâlâ açık kalır.

K-206 erken header kabul tasarımı: iki ayrı backend parser'ı kapsayan
typed 32 MiB aday yama bağımsız geçici crate'te 34/34 testle doğrulandı.
Aktif dependency pin değişmedi; üretimde mesaj sınırı etkin değildir.
Transaction durum temizliği, read retry engeli ve exact immutable kaynak
entegrasyonu açık; GB-041 kapatılmadı. [Kabul tasarımı](incelemeler/k206/frame-guard/README.md).

K-206 aday protokol kabulü clean c68af0f üzerinde kapandı: iki parser'da
34/34 bağımsız lib test, native 5 vaka, typed io cause ve kopan transaction
yazısının kalıcı olmaması doğrulandı. Ham JSON/log [kayıttadır](incelemeler/k206/frame-guard/README.md).
Bu yalnız adayın kabulüdür; aktif pin, Zee session/retry/RLS cleanup
entegrasyonu ve GB-041 açık kalır. Compiler fazı 735/735 geçti.

K-206 wire entegrasyonu: compiler/fuzz protokolü exact 1ffdb584 .2 kaynağına
pinlidir. İki parser'da 32 MiB length reddi, typed C025 verisi, read retry
engeli ve geçersiz transaction/request-local cleanup uygulanır. Gönderilmiş
COMMIT sonrası bağlantı kaybı C027 olarak korunur. Native loopback kabulü
okuma, büyük Notice üreten write, nested/RLS ve deferred COMMIT'i kapsar.
GB-041 bu kaynakta uygulandı; exact temiz kaynak, vendor/iki-clone ve
sürekli kapı kabulü henüz beklenir. Global RSS sözü verilmez.

K-206 wire security dilimi kapandı: source 7836b73, clean beyan/kanıt
a4e1045; native 13 vaka, 736/736 faz, offline vendor 163 ve iki-clone
release geçti. GB-041 kapalı; global RSS garantisi ve sürekli geniş
fault-proxy matrisi değildir. Exact ham kanıt
[buradadır](incelemeler/k206/frame-guard/README.md).

## K-208/K-209 — Zee çekirdek legacy giriş ve audit IP

Gerçek ürün talebi ve exact köken dogfood/zee-cekirdek altında korunur.
K-208 bcrypt $2a/$2b/$2y yalnız doğrulama (maliyet 4–14, 72 UTF-8 byte
sınırı), K-209 trusted `istemci adresi` Metin ifadesidir. Node vektör,
real worker proxy/socket/IPv6/spoofed XFF, CLI boş değer ve replay testleri
mevcuttur. RFC-0030 geçici kabul/ADR-082; yeni özet Argon2id kalır.
Ürün upgrade/migration/audit saklama işini kendi tarafında yapar.
K-174 syntax gözlem penceresi gerçek K-209 grammar değişikliği nedeniyle
2026-10-13–2026-11-10 olarak yeniden başladı; B-001/B-002 açık.
Tam faz, source beyanı ve vendor/release kapanışı beklenir.

K-208/K-209 kapandı: feature source 2c355ef, clean beyan/ikili kanıt
daaa3ea. 742/742 faz, Node crypto ve gerçek HTTP/CLI/trace kabulü,
167 paket offline vendor ve iki-clone binary eşitliği geçti.
[Ham kayıt](incelemeler/k208/README.md) yerel macOS kanıtıdır; V1
usability/RC/signed release ve sunucu dağıtımı değildir.

### ürün E özelliklerinin ana kaynak hattı — K-213

[ADR-086](../adr/086-site-hatlarinin-birlestirilmesi.md) mevcut ürün E
parola üretimi/TOTP/SMTP, rota birimi, görünüm, görsel ve gizli değer
yüzeylerini K-208/K-209 bcrypt/IP ile aynı hatta toplar. Ürün kabulü
dosya kontrolüyle değil bütün proje `dil dene .` ile alınır. Ham ürün E
şablonu normal katalog üretimini ister. Exact source/CI kapanışı beklenir;
canlı dağıtım ve otomatik parola göçü bu işte yapılmaz.

K-213 clean source kabulü: source e09493e, beyan/ikili/test 0531432;
769/769 faz, beş bütün proje 242/242, 187 paket offline vendor ve
macOS arm64 iki-clone eşliği geçti. Exact [kayıt](incelemeler/k213/README.md).
İlk Linux CI'ın test Git kimliği bağımlılığı düzeltildi; son uzak CI
kapanışı beklenir. Canlı dağıtım ve otomatik parola göçü açık kalır.

K-213 compiler gerçekleme/test/belge dilimi kapandı. Eski ürün E
sunucu hattı ile master aynı kaynak geçmişinde; ürün CI/sunucu pininin
bu kaynağa geçirilmesi ve canlı giriş/IP kabulü ürün tarafının işidir.
Otomatik bcrypt→Argon2 ürün göçü henüz uygulanmadı.

### K-214: Anonim açık depo kanıtı

Müşteri ve altyapı kimliği anonimleştirildi; yeni içerik hash politikasıyla
CI içinde denetlenir. Eski geçişler K-214 sırasında maskeli dökümle ayrı
karar bekledi; K-215 için kullanıcı temizliği onayladı. Tarihsel ölçümler
eski kaynak kimliklerini korur; yeniden açılış ve
GitHub PR önbelleği temizliği ayrıca onay/teyit ister. [Kayıt](incelemeler/k214/README.md).

### K-215: Önceki ürün geçişleri ve bağımsız yerel yama

K-214'te ayrı bırakılan geçişler kullanıcı onayıyla anonim kaynak soy ağacına
taşınır; eski fingerprint istisnaları kaldırılır ve ilk commit dahil tüm ata
geçmişi denetlenir. Compiler/fuzz PostgreSQL yaması path bağımlılığıdır;
derleme self Git deposu veya credential gerektirmez. SBOM gerçek build
commit'ini ve yerel paket baytlarını kayıtlı SHA-256 ile doğrular.
Tarihi ölçümler yeni kaynakta koşulmuş sayılmaz. GitHub PR refs/cache teyidi
ve yeniden açılış onayı açıktır; depo gizli kalır.

K-215 yerel makine doğrulaması tamam: 771/771 test, bütün yayımlanmış
ref geçmişinde sıfır gizlilik eşleşmesi ve iki temiz klon ikili eşitliği.
Güncel master uzak CI'ında altı iş hesap ödemesi/harcama limiti nedeniyle
başlamadı; test adımı sıfır. Linux/Windows ve uzak kapı kanıtı açık kalır.
[Hesap engeli kaydı](incelemeler/k215/ci-hesap-engeli.json) ve GitHub destek
teyidi tamamlanmadan yeniden açılış hazırlanmış sayılmaz.

### K-216: Git2 ve Gitsrv üzerinde koşular

Kullanıcı CI koşularını özel Git2/Gitsrv sistemine taşıdı. Ana GitHub kaynağı
merbay-erp/zee, Enterprise yalnız aynadır; ikisinde Actions kapalıdır.
Forgejo akışı tam Linux faz/kalite, gizlilik/sır, tedarik/offline vendor ve
WASM kapılarını exact seçili kaynakta çalıştırır; ilgisiz özel dal/etiket
almaz, PR kodu otomatik çalıştırılmaz. Uzak sonuç doğrulanınca kaydedilir;
Windows runtime kanıtı ve GitHub destek teyidi açık kalır.
[Karar](../adr/088-git2-gitsrv-kosulari.md).

## K-217 — Tek merkez ve dal incelemesi

Ürün talebi §2: site temiz içerik, issue formları, özel Forgejo MSVC
çapraz akışı ve incelenmiş action pinleri ana hatta uygulanır. Snapshot
compiler/CLI genişlemesi core freeze kanıt zinciri olmadığı için kapatılır.
Dal yedekleri özel; uzak silme ve CI teyidi henüz açıktır.
[Dal kararları](incelemeler/k217/README.md). Dil/spec/tanı/API değişmez.

### K-218 — Bellek oran backend'i

[ADR-089](../adr/089-bellek-oran-deposu.md) ile `--web-oran-deposu bellek`
güvenli proxy'de süreç başına oran sayacı seçer; varsayılan ortak dosya kotası
korunur. Oturum/revoke kalıcı paylaşılır. Çok worker ve restart ortak kota
vermez; proxy'de ortak limit gerekir. PostgreSQL ve §3/2 kota ayarı/tahliye
açıktır. Exact 1800 sn soak ve gerçek ürün yük kabulü kapanış kanıtına eklenir;
yalnız yerel ilk testlerle performans/sürüm hazır ilan edilmez.

K-217/K-218 kapanış (14.10.2026): §2 dalları temiz içerik/gerekçeli kapatma
ve exact lease silme ile tek master'da toplandı. Süreç başına bellek oran
seçimi gerçek ürün yükünde 54→2638 tek-worker ve 54→5079 iki-worker
istek/sn gözlemiyle, exact 1800 sn soak %0 RSS büyümesi ve Gitsrv #24
başarılı sürekli akışıyla kabul edildi. Kaynak 4e351d34, exact beyan 5a3cb270,
uzak kanıt 4077efe28; [tam kanıt](incelemeler/k218/README.md). Varsayılan
ortak kota ve kalıcı oturum paylaşımı korunur; explicit bellek seçiminde
proxy ortak saldırı bütçesi gerekir. Windows runtime/RC/usability ve
§3/2–5 açık kalır; bu değişiklik V1 hazır veya canlı yükseltme değildir.

### K-219 — Bellek oran ayarları

[ADR-090](../adr/090-web-oran-ayarlari.md) / RFC-0032 ile yalnız explicit
bellek backend’i için `--web-oran-siniri`, `--web-oran-kapasitesi` ve
`--web-oran-doluluk` eklenir. Varsayılan 100/60 sn, 32768 ve reddet;
en-eski yalnız uç nokta pencerelerini çıkarır, login/CSRF canlı sayaçları
korunur. Tahliye ile uç nokta kotası sıfırlanabilir; proxy ortak saldırı
bütçesi gerekir. Dosya backend’i ve kalıcı oturum paylaşımı değişmez.
Kod/test/spec aynı dilimdedir; exact 1800 sn soak, 786/786 yerel faz, ürün
kapasite yükü ve Gitsrv #26 kabulü [kanıt sayfasında](incelemeler/k219/README.md)
tamamlandı. Ürün sunucusu yükseltilmedi.

### K-220 — Ürün HTTP POST ve başlıklar

[ADR-091](../adr/091-http-post-baslik-ve-gizli-ortam.md) / RFC-0033: native
HTTPS GET/POST, başlık ve metin gövde, açık 1..30000 ms süre; Metin sözlüğü
Sonucu (`durum`, `gövde`). Ortam sırları yalnız explicit ana proje listesiyle
okunur; yeni sır taşıyan işlem iz kaydında/replay’de kapalıdır. Kanonik HTTP
JSON helper bool/metin ayrımını korur. Eski GET ve Rust facade değişmez;
WASM yeni native adaptörü açmaz. Ürün Turnstile helper ve yerel TLS testleri
aynı dilimde; 801/801 yerel faz ve Gitsrv #29 [kabulü](incelemeler/k220/README.md) geçti.
Sunucu/ürün rotası ve otomatik kayıt/e-posta entegrasyonu ürün tarafındadır.

K-220 / K-174: exact gerçek ürün talebiyle üç contextual kalıp eklendi; syntax gözlem penceresi 14.10.2026–11.11.2026 olarak bilinçli yeniden başladı. Core freeze ve insan/RC kapıları sürer; V1 tamamlandı olarak gösterilmez.

### K-221 ürünleşme CLI kabulü

Bilinmeyen seçenek kaynak keşfinden önce exit 2 üretir; program argümanları
korunur. CLI regression ve exact compiler/core-freeze beyanı toplu
kapanış ister. Bu değişiklik insan, fiziksel platform veya RC kabulü değildir.

### Ürünleşme dalgası K-222–K-225

GB-022 dedicated hermetik dört testle kapandı; normal faz dört ignored
vakayı ayrı sayar. Yeni uzak source CI kabulü ayrıca gerekir. K-225 beş
doğrudan spec testini tamamlar; insan/fiziksel/minOS/RC sınırları kapatılmaz.

K-226 build JSON'u exact kaynak tanımlamayı sağlar; dirty/unknown kimlik
release kabulü değildir. CLI/LSP/SBOM/worker eşliği ve iki-clone kapıları
ayrı sınanır; kanal etiketi kapı geçişini ikame etmez.

K-229 source biçim kapısı yazmadan denetlenebilir; sekiz CLI kabulü default
writer ve readonly/mtime sınırlarını korur. V1 insan/RC kapıları ayrı açık.

K-228 dedicated native göç testi normal fazda ignored ayrı raporlanır;
no-DDL/no-DML/rol/checksum/rollback explicit1/1 kapısı CI'a bağlanır.
Live ürün, fiziksel platform veya RC kabulü yerine geçmez.

K-223 native rehash 7/7 yerel kabulü CAS row-lock, dış eylem rollback ve
commit öncesi görünürlüğü kapsar. Uzak CI/ürün staging ve COMMIT yanıt
kaybı ayrı açıktır; insan ve RC kanıtını ikame etmez.

K-234 config profile/capability mevcut host davranışını sürümlü taşır.
17 yerel test eski-reader/önceki küçük sürüm, ürün ve fiziksel Windows
rollback kabulü değildir. K-174 kalıp/koşul/komut fingerprint değişmez.

K-232 tek ekli contextual kalıbı exact B2 ürün gerekçesiyle açar; K-174
14 Ekim–11 Kasım syntax penceresi aynı gün restart edildi. Native/test
kabulü live M365/Gmail/katalog/queue ve fiziksel Windows/RC kabulü değildir.
Inherited SMTP güvenlik kapanışları K-236/K-237 ayrı izlenir.

K-237/GB-061 yüksek SMTP yanıt-tahsis bulgusu beş actualhelper/native LF-yok corpus ve eski gövdeyle ret/düzeltme geri konunca kabul kabulüyle kapalıdır. K-236 maskesi ve mevcut 16 KiB profil korunur. P34 global RSS/full fault, P10 live/queue, fiziksel platform, RC ve yeni exact uzak CI kapıları bu yerel kanıttan kapanmaz.

K-238/GB-062 rustls advisory patch'i compiler/fuzz kilitlerinde 0.23.46
ortak checksumuyla dar tedarik kapanışıdır; manifest/production source ve
advisory ignore kümesi değişmez. [ADR-101](../adr/101-rustls-advisory-yamasi.md)
standart ring feature-tree ile dört upstream tetik koşulunu ayırır. İki
deny, offline vendor ve yerel HTTPS/tedarik kapısı yeni lock için kaydedilir;
eski #29/#30 ya da eski iki-klon kanıtı yeni CI/release/platform kabulü değildir.

K-239/GB-063 özel inceleme URL’lerini yalnız izole çalışma geçmişinden
çıkarır; GitHub yönetilen refs/cache destek teyidini veya public yayın
kapısını kapatmaz. Tarihsel checkpoint kanıtları yeni SHA’ya taşınmaz.

[Son yerel teknik teslim](urunlesme-sonucu-20261014.md) temiz
`2e970e47d59e2408eff836cdacef9bd0fe0b3028` kaynağında 887/893 faz
(0 başarısız, 6 dedicated atlama), ayrı TLS/bütçe 5/5, göç 1/1, CAS 7/7,
sürekli güvenlik/sır ve yerel macOS kanonik iki-klon kabulünü taşır.
[K-241 / ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md) sonrası
[gerçek a/b/çıktı kimlikleri](incelemeler/urunlesme-20261014/final-2e970e4/build-proof.json)
exact SHA/`dirty: false` ile doğrulandı. a2b8557 [kirli kimlik reddi](incelemeler/urunlesme-20261014/final-a2b8557/build-identity-recheck.json)
ve e9145844 ara raporları tarihsel kalır; checksum JSON biçimi düzeltmesi
yeni sır istisnası değildir, eski koşular yeni HEAD'e taşınmaz. Atlanan altı vaka
dedicated koşuda yürütülmüştür; yeni uzak CI, temiz fiziksel OS, tam ürün
ve insan kabulü çıkarılmaz. Yeni exact Linux/Windows üretimi, ASan/Miri RC,
imza ve yayın kapıları açıktır; takip belge commit'i test kaynağı değildir.

### K-242 / K-243 — Yerel kanıt ve kaynak aracı devamı

[Devam raporu](urunlesme-devam-20261014.md) K-242 on bir native senaryoyu
(yedi TLS, dört owned loopback fault/kontrol) ayrı exact kaynak kanıtıyla
izler. C027 ve otomatik tekrar etmeme korunur; canlı login/ürün/provider,
RSS/full fault, fiziksel platform ve RC kabulü buradan çıkarılmaz.
K-243 kaynak ZIP tek snapshot, UTF8 ve canonical çıktı sınırını gerçek
Git testleriyle kapatır. Python3 stdlib araç bağımlılığıdır; compiler
semantiği, spec normları, public facade ve core freeze aynı kalır.
39 maddelik ürünleşme planının tam kabulü ve bekleyen dış kanıtlar açıktır.

K-244/GB-067 literal kaldırma yerel resmî geçici PowerShell macOS
koşusunda önce/sonra ve altı senaryo kanıtlıdır. Receipt/checksum/Zorla
aynı kalır; Windows Cargo dalı ve tatbikat aynı native fixture'a bağlıdır.
Yeni uzak CI, fiziksel Windows ve minimum OS bu yerel sonuçtan kapanmaz.

Temiz `316a313ed6798d084450097d79281dce7f3fc82d` devam kabulü
888/894 faz (0 başarısız, 6 dedicated atlama), ayrıca native 5 TLS/1 göç/11
parola-COMMIT ve Mac PowerShell6/6, sürekli güvenlik/sır ve exact1146
kaynak ZIP eşliğiyle tamamlandı. [Devam raporu](urunlesme-devam-20261014.md)
kapıları aynı SHA'ya bağlar; takip belge commit'i yeni teknik checkpoint
değildir. 2e970e4 iki-klon ikilileri tarihsel kalır; full ürün/insan/platform,
uzak CI/lisans/signed release ve RC sınırları açıktır.

K-248/ADR-108 gerçek LSP yüzde UTF-8 dosya yolu arızasını minimal byte
çözümü ve native stdio birim regresyonuyla giderir; exact source/provenance
ve temiz c67a69e faz kabulü tamamlandı. Yeni core özelliği veya platform kabulü yoktur.

K-245/P25 credential host pilotu (ADR-105/GB-068) mevcut env izinlerini
koruyan POSIX dosya köprüsüdür;14 actual vaka native allowlist/SMTP dahil
yürütülür. Temiz c67a69e host14 kabulü tamamlandı; Windows/systemd/canlı
ürün ve file-native sır inheritance sınırı tam P25 olarak kapanmaz.

K-246/P13 (ADR-106/GB-069) worker host identity+ayrı readiness ve sahipli
cleanup pilotudur; public launcher cleanup bug'ı temiz c67a69e üzerinde 25-vaka kabulüyle kapandı.
Tam request drain/reload/metrics ve P14 socket devir açıktır.
K-247/P09 (ADR-107/GB-070) dar idempotent SQL queue helper'ıdır;11 native
PG crash/COMMIT observer ve16 validator temiz c67a69e kabulündedir. Canlı ürün adoptionı, SMTPretry ve full P08 açık kalır.

K-249/P21 (ADR-109/GB-071) tek anonim TR–EN article fragmenti, plain
Metin slotları ve sabit sözlüktür. Dört kaynak testi, yedi native CLI
snapshot/XSS/ret senaryosu mevcut grammar/API ile temiz c67a69e kabulündedir. Genel layout/i18n ve gerçek ürün/P08 kabulü açıktır.

K-245–K-249 [temiz pilot kabulü](urunlesme-pilot-k245-k249.md)
`c67a69e6ed9c24c4a540a55eeaaa986a348a2de3` üzerinde 895/901 faz,
0 başarısız/6 dedicated atlama; ayrıca 28 PostgreSQL, credential14,
supervisor25 ve iki Python sürümünde TR–EN7+4 ile kanıtlıdır. İki temiz
klondan Mac ARM64 ikilileri eş ve dirty=false; yeni sır istisnası yoktur.
Lisans/insan/fiziksel/tam ürün/uzak CI/RC kapıları ve genel P09/P13/P14/
P21/P25 kapsamı açık kalır. Takip belge commit'i test kaynağı değildir.

### K-252 süre correctness dilimi

Plain SMTP slow-trickle arızası için mutlak bütçe ve minimal security regresyonu eklendi ([ADR-110](../adr/110-smtp-mutlak-sure-butcesi.md)). Ortak exact kaynak ve RC kabulü ayrıca gereklidir; eski c67a69e kanıtı yeni kaynak kabulü sayılmaz.

## K-250 — PostgreSQL statement fault kanıtı

[ADR-111](../adr/111-postgresql-statement-fault-kaniti.md): gerçek 40P01 ve
40001, no-retry sayaçları, outer rollback onayı ve bağımsız durable observer
iki native vakayla sınanır. Dedicated TLS kapısı yedi vakadır; COMMIT
belirsizliği ADR-103/C027 olarak kalır. Çalışma ağacı native7/strict9 geçti;
temiz exact ortak kabul ayrıca gereklidir; provider/full fault matrisi açık.

## K-251 — DB ve göç hazırlık probu

[ADR-112](../adr/112-db-goc-hazirlik-probu.md), mevcut CLI ile uygulama
sahipli tek seferlik DB+migration gate üretir. Native13/strict10 yerel aday
kabulü vardır; compiler özelliği yoktur. DB/worker nesli değişince yeniden
prob gerekir; continuous/TTL, request metrics/drain/config reload ve P14
socket devri açık kalır. Gerçek ürün/platform adoptionı ve exact ortak temiz
kabul ayrıca gereklidir.

## K-253 — Native Unix graceful shutdown

[ADR-113](../adr/113-native-web-sigterm-drain.md), gerçek B5 ürün ihtiyacını
anonim native listener reproducer'ına bağlar. Yeni grammar/CLI/API yoktur;
SIGTERM mevcut admitted handler'ı bitirir, yeni admission durur ve idle
listener kapanır. Supervisor default5s korunur, max60s/explicit55s fırsatı
vardır; dış grace sonunda KILL/cleanup sürer. Native4/guard3 ve eski host25
stage kanıtıdır; exact kaynak/beyan ve final soak1800s kabulü ayrıca gerekir.
Bu semantic-change/dogfood-change'dir; bug fixed_by kaydı icat edilmez.
Metrics, Windows signal drain, tüm backlog/P14 socket/overlap ve adoption açıktır.

## K-255 — P19 mevcut LSP ölçümü

**Yerel ölçüm dilimi kapalı.** Temiz021bff4 release25örnek/2ısınma
LSP2k–20k p95 7.490/18.227/53.784/177.321ms;
[actual raw proof](incelemeler/urunlesme-20261014/k255-lsp-olcum/measurement.json)
ve immutable tarihçe eklendi. Komşu yerel işler varken gözlemseldir, hard eşik
ve yeni baseline yoktur. Gerçek ürün200ms/cache/lint/JetBrains ve P19bütünü açık.

## K-254 — Ayrı SQL kira sahibi heartbeat

[ADR-114](../adr/114-ayri-sql-kira-heartbeat.md): mevcut SQL-only job satırı
handler'da kilitliyken ayrı lease satırı yenilenir. Lease30s/toplam120s/24renew;
immutable0002 geçişi, old-generation fencing ve mixedv1/v2 ret sınırı vardır.
Dedicated21 native+24validator stage kabulündedir; final exactclean ayrı.
Dispatcher/cron/SMTP non-idempotent etkiler ve gerçek katalog adoptionı açık.

## K-256 — TLS1.1 capable-positive kontrol

[ADR-115](../adr/115-tls11-capable-positive-kontrol.md): owned verifiedTLS1.1
positive connection+server gözlemi sonrasında native ret, dedicated8/strict14.
Yeni compiler özelliği/tanı yoktur; legacy cipher policy test contextiyle
sınırlıdır. Provider/full matrix/Windows/uzakCI ve final exactclean kabul ayrı.


## 15 Ekim 2026 temiz teknik kontrol noktası

[K-250–K-256 kontrol noktası](urunlesme-kontrol-noktasi-20261015.md)
exact temiz d42f147 üzerinde faz904/913 (0 başarısız/9 dedicated atlama),
ayrı PostgreSQL41, DB hazırlık13, native drain4, credential14, worker25 ve
iki Python sürümünde TR–EN7+4 kabulünü kaydeder. K-252 exact regression ve
beyan zinciriyle kapalıdır. K-253 mevcut admitted handler'ı SIGTERM sonrası
bitirir; final1800s soak hâlâ gereklidir. K-254 uzun handler heartbeat ve
K-256 verifiedTLS1.1 capable-positive ret kanıtı artık çalışma ağacı
adayı değil, bu exact kaynakta yerel kabuldür. Metrics/socket/dispatcher,
gerçek ürün, insan/platform ve RC/release kapıları açık kalır. Kaynak
kimliği daha sonraki belge veya özellik commit'lerine taşınmaz.

## K-262 — Mevcut web oran CLI argüman kayması

Temiz17b preexisting CLI bug kanıtlıdır: mevcut dört değerli oran seçeneği kaynak öncesi tüketilmediğinde APP_ARG yerine bellek görünür. K-262 dar kaynak öncesi skip onarımı, kaynak sonrası oran argümanları ve eski globalproxy sınırını korur. Yeni flag/grammar/export yoktur. Dar bugfix KAPALIDIR: kaynak commit `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179`, olumlu/olumsuz gerçek CLI testleri, ADR-121 ve `regression/v2.tsv` içindeki `web-oran-program-argumani` exact fixed_by kaydı semantic-bugfix/core-freeze correctness zincirini tamamlar. Final exact RC ve birleşik özellik kabul kapıları AÇIKTIR; bu kayıt sürüm veya ürün adoption kapanışı değildir.

## K-257–K-260 — Yerel aday kapasitesi, ürün kabulünden ayrı

Native request/PG pool metrikleri (dedicated8), SQL-only finite dispatcher
(native11), inherited TCP listener ve nesil hazırlığı (native17/quietFD16),
credential→tek metric FD köprüsü (7+2) aynı ürünleşme dalgasının dar kapasiteleridir.
ADR-116–119 sınırlı kabul durumundadır. Listener17 içinde gerçek argv oracle,
bozuk `zee.ayar`, metric seçenek sırası ve kapanmış metric reader kompozisyonu
vardır; FD16 erken stdio/control/metric/config aliases'i reddeder. GB-078–081
sınırları kaynak+test+kararla birlikte değerlendirilir.

Bu dalga yerel çalışma ağacı/adayı kabulüdür. Listener17/FD16, metrik8 dedicated,
credential7 model/subprocess+2 native ve dispatcher11 sayıları ayrı kampanyalara
aittir; Cargo/libtest toplamına eklenmez. Listener birleşim kanıtı pinlenmiş
`2caa26e55e50bf35b4220e4f6d16ac7273f337159243b6e97c8f2443581adbf8` ikilisinin
actual BuildJSON'unda `17b29da1b8321fe6dcc4d34bd2027c4c969bf406` tabanı ve
`dirty=true` taşır. Bu kimlik temiz kaynak kabulü değildir. Final exact temiz
ortak faz/native, 900 sn host tatbikatı, 1800 sn soak, dört hedef explicit ASAN
RC/Miri, imza/kurulum ve fiziksel platform kanıtı henüz bu yeni kaynağa alınmadı.
Önceki d42f147 temiz kabulü kendi tarihsel kaynağında kalır.

Native listener host tek kendi loopback TCP LISTEN socket'ini tutar; proxy,
SO_REUSEPORT veya yabancı PID/port benimsemesi değildir. Native ready channel
eski worker HTTP200'ünden ayrı, fresh generation receipt de ortak eski dosyadan
ayrıdır.1000 istek concurrency1/dört owned rollover ve ayrı sentetik loopback
istemcileriyle sınır içi kabul gördü; quota raise/reset/retry yapılmadı. Bu genel
zero5xx/refusal SLA değildir. Kalıcı `.zee/web-durumu-v1.json` kota ve revoke
paylaşımını korur; süreç belleği kota overlap profilinde reddedilir.

P09 gerçek catalog/SMTP effect adoptionı; P13 continuous/TTL/config reload ve
uzun işletim; P14 farklı kod/resource revision dağıtımı/live/platform; P25
file-native secret ve listener credential kompozisyonu açıktır.39 plan satırı,
insan/harici CI/fiziksel/RC kapıları ve frozen syntax takvimi kapanmış gösterilmez.
[Dispatcher](job-dispatcher.md) ve [listener](native-listener.md) rehberleri bu
kapsamın kullanım ve ret sınırlarını verir. K262 ayrı correctness dalgasıdır.

Güncel ek sınır: K263 owned-host cleanup dar düzeltmesi ac077e31 ile commitlendi; yukarıdaki yerel
aday makbuzları birleşik finalclean özellik kabulü değildir.
K262 dar source/argv düzeltmesi 7c56518c ve exact provenance4d8edeb2 ile kapalıdır; ortak özellik kabulü açıktır.

Kanonik güvenlik sırası: atomik K-263 owned-host cleanup düzeltmesi GB-077 ile `ac077e31e761e401eedc5140a8035fd02f2a19ba` kaynak commitinde tamamlandı; K-262 dar CLI bugfix `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179` ve exact provenance `4d8edeb2a9f4453d2f28481ecd550bf8ba5f2032` ile kapandı. Ortak temiz özellik kabulü ayrıca beklenmektedir. Kaynak `54a8d6dc0efb060ae5644b37546da40ad03ee035` ile commitlenen özellik dilimlerinin kayıtları K-257/GB-078, K-258/GB-079, K-259/GB-080 ve K-260/GB-081’dir; ADR-116–119 numaraları korunur. Önceki immutable dış-stage makbuzlarının geçici GB etiketleri tarihsel kanıttır, kanonik yeni kayıt kimliği değildir. Finalclean/900sn host/1800sn soak/RC kabulü bu revizyonda ileri sürülmez.

## K-263 — Sahipli supervisor EPERM geçişi

K-263/ADR122/GB077 dar host correctness adayıdır: gerçek sahipli worker EPERM/WNOWAITNone geçişi artık tek mutlakgrace deadline içinde heldleader üzerinde gözlenir. Persistent ambiguity/returncode-set/ECHILD failclosed, canlı descendant temizliği ve foreigngroup sınırı korunur. Ayrı8 lifecycle testi +Cargo wrapper1 mevcutworker25 sayısını değiştirmez. Compiler source/API/grammar, performans/soak ve K262/featurewave kapsamı değişmez. Dar host source commit ac077e31e761e401eedc5140a8035fd02f2a19ba ve 905/914 faz kapısı tamamlandı; birleşik özellik finalclean gate AÇIKTIR; tamamlanmış release/adoption iddiası yoktur.

## K-264 — Kritik işlevlerde dar sorumluluk incelemesi

[ADR-123](../adr/123-kritik-islev-secici-taban-incelemesi.md) dört yeni kayıt ve yalnız govde tabanını gerekçeli kabul eder. Bütün diğer satır değerleri/eski kimlik/global paylar korunur; gerçek islev_egilimi denetimi64 kayıtla geçmiştir. Kaynak davranışı değişmez; fullclean/RC/soak/ürün kapıları ayrı açıktır. Mevcut olmayan parser/MIME kapsamı varmış gibi gösterilmez.

K-259 ilk adayın geç EOF admission arızası GB-080 içinde aynı ilk özellik diliminde düzeltildi: kalan poll bütçesi ve EOF sonrası deadline/sinyal kontrolü; ayrı normal Cargo gerçek worker1 HTTP byte0/terminal0/owned cleanup doğruladı. Native17 ve quietFD16 ayrı sayılardır; finalclean RC/soak kabulü açık kalır.

K-257–260 birleşim adayında seri tam faz 924/936 geçti, başarısız0/atlanan12; kaynak+belge93 dosya koşu boyunca aynı hashte kaldı. Ayrı fmt/clippy, katalog1+tazelik6, spec214 ve işlev64 kapıları geçti; sırtaraması/sürekli güvenlik temizdir. İlk başarısız koşu (paralel Cargo binary-pin değişimi ve korpus ürün politikası receipt yetkisi eksikliği) dış immutable kayıtta korunur. Bu sonuç finalclean, dedicated DB/native, 900sn host,1800sn soak veya RC kabulü değildir.

K-257–260 kaynak commit `54a8d6dc0efb060ae5644b37546da40ad03ee035` öncesi tam faz926/938 geçti, başarısız0/atlanan12; dondurulan97 kaynak+belge aynı kaldı. K-259 minimal `native-listener-late-eof` vakası introduced_by=- ve exact fixed_by bu SHA ile takip provenance dalgasında kaydedilir; gerçek Unix replay ayrı nonempty testtedir, Unix dışı native vaka açıkça ignored kalır. Semantic-change/dogfood-change beyanları K-259/ADR-118 ve gerçek B5/B6 provenance zincirine bağlanır. Bu kaynak/provenance kapanışı finalclean/native900/soak1800/RC veya ürün adoption kapanışı değildir.

Provenance adayının ilk tam fazı test-only çıktı adı yarışını yakaladı: aynı PID ve eş SystemTime nanos ile paralel native/preflight aynı fresh dizini seçti; 16-vaka makbuzu mevcutken native17 reddedildi. Wrapper yoluna exact vaka rolü eklendi; üretim compiler/host davranışı değişmez. Başarısız926/939 koşu ayrı dış makbuzda korunur, geçer kabul edilmez.

Son kapsam incelemesi semantic regression grubuna yalnız `proje` ekler: actual DIRECTORY çalıştırması ProjeGrafigi çözümü/kilit/giriş/politikayı yürütür. Deadline sonrası client TCP baytları native HTTP parser tarafından kabul edilmediğinden `http` eklenmez. Vaka rolüyle ayrılan output düzeltmesi sonrası927/939 geçti, başarısız0/atlanan12; son `proje` kapsam metadatasıyla tam kapı da 927/939 geçti, başarısız0/atlanan12.

## K-265 — Exact kaynak dış RC kanıtı

[ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md) sourceS/historyE self-reference doğruluk boşluğunu explicit dış immutable paketle ele alır; kaynak eşdeğerliği/bypass yoktur. İzole25 sentetik protokol+9 kısa ownedprocess test dilimi, tam4ASAN2Miri/rawhash/initialsourcecorpus/erkenHEADpin/streamingdizinbütçesi negatiflerini kapsar. K-265 kaynak commit’i `1367ce1a7d0281d8a7f3d9307021afbc452da15f` ile kayıtlıdır: kaynak öncesi tam faz928/940 geçti, başarısız0/atlanan12; son EOF boşluğu düzeltmesi sonrası25 protokol testi yeniden geçti. Final gerçek cleanS uzunASAN/Miri kampanyası,900/1800/release/ürün adoption kapıları açıktır. Kullanıcı AGENTS değiştirilmez, exactsourceS ve append-only arşivE ayrımı açık tutulur.

## K-267 — Typed credential/native listener bileşimi

[ADR-125](../adr/125-credential-native-listener-kompozisyonu.md) mevcut D3/P25 ve B5/B6 ürün ihtiyacını dar Unix host profilinde birleştirir. Credential okumadan all-or-none FD/rol/socket/port/argv doğrulaması, native child tarafından devralınmayan ayrı startup FIFO ile PID/generation/binary kimliği ve varsayılan noinherit korunur. İki normal Cargo vakası model/subprocess5 ile gerçek native crypto/overlap/metrics/admission/cleanup5 kampanyalarını ayırır; alt-senaryolar eşsiz toplam diye sayılmaz. İzole dirty d53 kısa aday kanıtı final temiz kaynak kabulü değildir. P25 bütünü, Windows/systemd/file-native sır aktarımı, canlı benimseme ve 900/1800/RC kapıları açıktır. GB-083 bilinçli capability sınırını kaydeder; yeni grammar/compiler/public Rust API eklenmez.

K-267 doğrulama bakımı: macOS nativeTLS platform hazırlığı, mevcut SMTP IO
iptal testinin ölçülen kısmından önce tamamlanır; 60 ms IO bütçesi, iki saniye
takılma koruması ve sessiz peer değişmez. Production davranışı, tanı, test
sayısı ve normatif sözleşme değişmedi. Önceki iki tam matrisin aynı SMTP
korumasında kalması korunur; K-267 son tam faz koşusu 930/942 geçti, 0 kaldı, 12 atlandı; koşu boyunca 40 kaynak dosyası değişmedi. Final temiz kaynak ve uzun kabul kampanyaları ayrı kalır.

## K-268–K-270 kanıt sınırı

K-268/ADR-126/GB-084 sonlu timer37 hazırlığı; K-269/ADR-127/GB-085 kopyalanabilir site/panel2 mevcut CLI/HTTP pilotu; K-270/ADR-128/GB-086 yerel import/proje launcher14 ve korunmuş selector25. Yeni üç normal Cargo vakası gerçek alt-senaryolarıyla ayrı sayılır. Native Linux/systemd/SQL timer, gerçek kullanıcı/live ürün, production publisher/platform ve final temiz kaynak kampanyaları açık; P09/P23/P07 kısmi kalır. Yeni compiler/src/grammar/facade/dependency veya spec normatif madde yoktur; spec214 ve önceki source54/c86 maintenance/provenance, K267 kayıtları korunur.

[Timer](../docs/sonlu-dispatcher-timer.md), [başlangıç](../sablonlar/README.md), [yerel araç](../docs/derleyici-yerel.md).

K-268–270 ortak çalışma ağacı kabulü: dondurulan 41 dosya koşu boyunca
değişmeden tam faz matrisi **933/945 geçti, 0 başarısız, 12 explicit ignored**
sonucunu verdi; faz süreleri toplamı 376.425 saniyedir. Timer37, kopyalanan
site/panel2 ve yerel araç14 (eski seçici25 korunarak) normal Cargo kapısındadır.
Fmt/clippy/işlev64/spec214/mimari/SCC/API/uyumluluk/katalog/tazelik/tedarik,
sürekli güvenlik, kanonik sır taraması ve semantic/core-freeze korukları geçti.
Bu kayıt temiz exact kaynak/native Linux/systemd, 900/1800 saniye, ASAN/Miri,
kurulum veya sürüm kabulü değildir; ilgili açık sınırlar korunur.

## K-271–K-274 dar gate kapsamı

Dört wrapper grubunun birincil fazı sırasıyla proje/web_guvenlik/proje/web_guvenlik'tir.
K-271 normal20 hazırlık Zee pipeline çalıştırmaz; actual4 PG16.15 cohort'u explicit
ignored gate'tir. K-272 normal gate new9HTTP/5Zee ve legacy7HTTP/4Zee'yi aynı private
binary/source snapshot'ıyla korur; formatter çağrısı gerçek ast ek kapsamıdır.
K-273 normal23 CLI/localpackage/route ve kontrollü journal/fault gate'tir; SQL
arşivi execute edilmediği için DB lifecycle kabulü çıkarılmaz. K-274 normal
models16/nativeTTL-config5 ve explicit ignored PG15 (eski13 dahil) ayrıdır.

Toplam7 yeni libtest vakası5 normal/2 ignored olarak bağlandı; global sayılar
repository generatorlarından alınır, nested/repeated native cohortlar unique
vaka toplamı diye eklenmez. Önceki exact K-268–270/RC/source-archive kanıtları
kendi SHA'sında kalır. Yeni final exact clean common kabul, 900s host,1800s soak,
4ASAN+Miri RC, gerçek platform/live ürün/P08 ve release/install kanıtı hâlâ ayrı
kapılardır; source/hash/postcleanup/Build doğrulaması atlanamaz.

## K-275 — PDF teknik kapasite ve açık kabul

K-275/ADR-133/GB-091 sabit owned PDF host profili: mevcut Zee JSON örneği, exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0, stdlib8 normal ve hostbackend8 explicitignored ayrımı. Sabit kaynak/font/notice, bounded job/output ve nofollow roller korunur. Finalclean S native emitter+backend8 ortak kohortu ayrı zorunludur; tarihsel dirty kanıt temiz S sayılmaz. P11/B3 Zee içinden invocation/resmi signedplugin, ürün/adoption/P08/platform ve proje lisansı açıktır; full900/1800/RC/soak/kurulum kabulü ileri sürülmez. Bağımsız source+asset-notice incelemesi, root tam fazı ve exactS kohortu tamamlanmadan final kabul yazılmaz.

### K-276–278: Linux taşıyıcı hazırlığı, native dış etki ve editör şablonu

K-276/[ADR-134](../adr/134-linux-ozel-paket-tasiyicisi.md) özel inert Linux taşıyıcı hazırlığıdır: normal wrapper
12 preflight +9 carrier model/negatifi yürütür. ELF header doğrulaması Linux
ABI/çalıştırma kanıtı değildir. İki temiz klon kanonik teslimi, final kaynak S
üzerinde gerçek .deb build/install/remove/user-prefix activation/container
kabulü ve yayıncı/lisans/imza/apt repository ayrı açıktır. Namespace/image
manifest önkoşulu tamamlanmış Zee container kabulüne dönüştürülmez.

K-277/[ADR-135](../adr/135-sabit-native-dis-etki-tuketicisi.md), mevcut SQL-only dispatcher'dan ayrı fixed owned
loopback native SQL→MIME SMTP→ack profilidir. Effect-start latch bilinen committed
olmadan SMTP başlatılmaz; owner/token/generation/lease korunur. Yalnız kanıtlı
pre-effect SQL reddi, sıfır send invocation ile toplam3/1–2sn backoff yapabilir.
C029/DATA/ackCOMMIT belirsizliği guard+exit3/manual incelemedir; SMTP otomatik
replay/exactly-once sözü yoktur. Bilinen completion sonrası görünür guard unlink
fakat final fsync hatası ayrı exit4'tür; olmayan guard yeni admission'ı engelliyor
denmez. Normal hazırlık8 ve explicit ignored native11 ayrı kapılardır. Kaynak
harness yalnız dört pinli fixture dosyasını kopyalar; ek SQL/.dil/FIFO alınmaz.
Tarihsel dirty aday kanıtı finalclean değildir. Gerçek B1/B2 katalog, sağlayıcı
Gmail/M365, K-275 PDF üretimiyle ürün entegrasyonu ve tam P09/P08 açıktır.

K-278/[ADR-136](../adr/136-jetbrains-yerel-lsp-sablonu.md) mevcut K-270 launcher'ını kullanan POSIX LSP4IJ veri
şablonudur; TextMate mevcut VS Code bundle'ından gelir. Npm bağımlılığı olmayan
elle yazılmış VS Code istemcisi için owned `dillsp` mutlak yolu açıkça seçilir;
IDE macro/tokenizer/kurulum/etkileşim/Windows kabulü henüz açık kalır. Şablon
installer/download/yeni grammar/capability/cache veya gerçek IDE kabulü eklemez.
[PDF/SMTP'den ayrı editör rehberi](../editors/jetbrains/README.md) bu sınırı korur.

### K-279: gerçek önceki küçük sürüm korpusu ve açık göç

[ADR-137](../adr/137-onceki-kucuk-surum-korpusu.md) gerçek v0.7.0 exact
`d08a947713ddc38d2163360c17d07fbffcb1213d` kaynağındaki33 golden girdisini
değişmeyen blob/SHA ile korur.32 directcheck eşliği ve15 saf runtime çıktısı
gerçek eski/yeni CLI'da sınandı; birim örneği K-086 publictype sözleşmesi için
yalnız birimde iki annotation satırıyla göç ister. Ham33 kaynak gizlice
yenilenmez. Normal iki Rust vakası49 modernCLI altkontrol+5 erken bootstrap
reddini yürütür;54 libtest veya gerçek anonim ürün/P08 kanıtı sayılmaz.

Eski CLI config capability JSON sunmadığı için mevcut BuildJSON/selector
reddi korunur; yeni `zee.ayar` için otomatik legacy rollback açılmaz. Native
çalışma altkümesi effect-free'dir; ağ/web/dosya/saat/rastgele/async/hardware
runtime örnekleri çalıştırılmaz. Sabit33 kaynakta provenance olarak duran bu örnekler
compiler bugfix/dogfood veya minOS/platform kabulü değildir.

Linux'ta e76 retained leader cleanup ölçümü komut başına yaklaşık3.001s'dir.
52 sınırlı komutun yaklaşık156s toplam bakım yükü için yalnız test harness
aggregate bütçesi120→240s seçildi; her native çağrı8s, owned cleanup/deadline/
capture, kaynak/output oracles ve production SLA/perf eşikleri değişmedi.
Bağımsız Mac49+5 replay ayrı; gerçek tam Linux ve temiz finalS kabulü ayrıca
gerekir. Tam P04/P05, ürün/kanal/platform/adoption açık kalır. [Rehber](onceki-kucuk-surum-korpusu.md).

K-271–K-279 commit öncesi birleşik kapısı (15 Ekim 2026): tam faz matrisi
**943/959 geçti, 0 başarısız, 16 explicit ignored**; faz süreleri toplamı
429.100 saniyedir. Koşu boyunca 1.477 kaynak/belge/fixture dosyası aynı hash'te
kaldı. Fmt, tüm hedeflerde Clippy, işlev64, spec214, katalog/tazelik,
katman/SCC/public API/uyumluluk/tedarik, gerçek Zee–Rust–Go çıktı eşliği,
sürekli güvenlik, semantic/core-freeze ve kanonik sır taraması geçti.
Üç yeni Rust test sarmalayıcısındaki sabit platform assertion'ı açık Unix
koşullu gövde ve desteklenmeyen platform panic'iyle kanonik biçime taşındı;
Unix vakaları ve üretim compiler davranışı değişmedi. İlk Clippy retleri
ayrı dış kayıtta korunur. Normal fazdaki16 dedicated atlama temiz kaynak
kampanyasında ayrıca yürütülmelidir; bu sonuç final temiz kaynak/native,
900 saniye işletim, 1800 saniye soak, ASan/Miri, paket/kurulum veya gerçek
ürün/platform/IDE/insan kabulü değildir. Yeni compiler/src, grammar, spec
normu, public Rust API veya production bağımlılık değişikliği yoktur.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

[15 Ekim S raporu](urunlesme-final-kabul-20261015.md)943/959normalfaz,16exactdedicatedowner/25kohort,900/1800sn ve yerelmacOS/Linux teslim/kurulumu aynıS ASAN/Miri/export/securitygate kanıtıyla birleştirir. Temiz S üzerinde dört ASAN hedefi ayrı ayrı1801sn geçti; lexer_parser7.117.460, morfoloji8.265.144, http_istegi11.194.063 ve wasm_abi9.011.299 yürütme kaydedildi. İki seçili Miri grubunda toplam3test(ondalık2,zaman1) geçti. Kanonik export/check ve aynı S güvenlik kapısı510.198758sn exit0 ile geçti; kapının118 Cargo/libtest grubunda943passed/0failed/16ignored vardır. GB082 yalnız bu exactS yerel protokol ve kanıt kapısı için kapalıdır; E kaynakS gibi kabul edilmez. Bu S kabulü E exactHEAD kabulü, signedpublication veya bütün V1 kapılarının kapanması değildir. P08/insan/fizikselplatform/dışdestek/syntax takvimi açıktır.

[RC sonuç makbuzu](incelemeler/urunlesme-20261014/final-8b49113/rc/FINAL-RESULT.json), [aynı-S kapı makbuzunun türetilmiş temsili](incelemeler/urunlesme-20261014/final-8b49113/rc/POST-GATE-RECEIPT.derived.json), [bağımsız kök incelemesi](incelemeler/urunlesme-20261014/final-8b49113/rc/root-independent-review.json).


### K-280: JetBrains tamamlama ve normal çıkış

15 Ekim 2026 owned IntelliJ IDEA2025.3.1/LSP4IJ0.21.0 POSIX profilinde
mevcut S8b49113 launcher ile gerçek completion isteği7, `sayi 7 ols` →
`sayi 7 olsun` uygulaması ve ardından boş LSP tanısı doğrulandı. Trace ayarı
uygulanırken shutdown5/yanıt/exit ayrıca görüldü. Normal Quit/Confirm Exit
sonrasında original süreç sahibi exit0 ve kapalı/toplanmış grubu kaydetti;
zorunlu sonlandırma gerekmedi. Son Quit protokol konsolu dışa aktarılmadı;
popup görünürlüğü ayrıca kanıtlanmadı. Önceki retler korunur. Dar IDE
etkileşim eksikliği kapalı; Windows/eklenti yayın kanalı/incremental/lint/
20k satır200ms ve tam P19 açık. [ADR-138](../adr/138-jetbrains-tamamlama-ve-duzenli-kapanis.md),
[kabul kaydı](incelemeler/urunlesme-20261014/k280-jetbrains/ACCEPTANCE.json).
Yeni kaynak dalgası bu S kanıtını kendi derleyici kabulü sayamaz.

## K-281–K-294 yeni teknik aday dalgası

Güncel kaynak/aday test/eksik kabul ayrımı [ikinci dalga kaydındadır](urunlesme-ikinci-dalga-20261015.md).
İmzalı PDF hostu, sürekli hizmetler, SQL modül yaşam döngüsü, kaynak kapsamı,
`dene` HTTP/DB köprüsü, editör eşitleme/cache/lint, TR–EN bileşimi ve içerik/medya
başlangıcı eklendi. Önceki S1/E1 kabulü bu yeni kaynağa aktarılmaz. K-294
regression follow-up ve temiz exact normal/dedicated/RC/Miri/soak kapıları
bitmeden dalga tamamlandı sayılmaz; C1/P33 araç retleri açık tutulur.

K-294 kaynak/provenance takibi: `382beed1d01440f6f9df28b5ac7a111fbe1f75ee`
exact `fixed_by` ile `native-metric-stdio-alias` minimal regresyonuna
bağlandı. Gerçek owned host ile nonempty native korpus testi dahil dokuz
semantic regresyon testi geçti; compiler/core-freeze beyanları kaydedildi.
Bu paragraf S3 öncesi takip kanıtıdır. GB-110 kapanışı ve actual temiz
RC/Miri/soak/native/JetBrains zinciri [S3 teknik kabulü](urunlesme-s3-kabul-20261015.md) raporundadır.

## K-295/K-296 — S2 kabulünün ardından yeni kaynak gereği

S2 `e35b617095e6a9d00915794d6adee90842238a5f` final kabul değildir.
macOS iki-klon paketi, canonical stdio probe'ları, TR/EN, signed-PDF9, SQL9,
starter14, kapsam, bridge4/19HTTP+negatif6 ve mixed-revision9 kanıtı korunur.
Linux dispatcher0/4 ve fault stale-gate oracle başarısızlığı K-295/K-296 olarak
ayrıldı. İlk worker-loss geçti; her iki başarısız native koşunun cleanup'ı temizdir.
S2 ASan/soak kampanyaları sahipli cancel-file ile erken kapatılmış, outer
handle'lar reaped olarak kaydedilmiştir; 30 dakika/1800 saniye kabulü sayılmaz.
Bu bölüm S3 öncesi aday kaydını korur; iki düzeltmenin actual temiz
ortak/native/RC/Miri/soak ve JetBrains kabulü [S3 teknik kabulü](urunlesme-s3-kabul-20261015.md) raporundadır. Compiler/grammar/tanı/yüzey/API/katman/bağımlılık
değişmedi; C1/P33 araç retleri ve insan/platform/ürün kabulü açık kalır.

## S3 — ikinci dalga temiz kaynak kabulü

Kaynak `66e4149e53451111d153f47283c219079a8d1cab` üzerinde ikinci dalganın native senaryoları, macOS/Linux için iki temiz klondan teslim ve sekizer kurulum testi, gerçek JetBrains tamamlama/biçimlendirme/tanı/kapanış akışı, 1.800 saniyelik soak, 1 ve 41 açık belge ölçümü, dört ASan hedefi ve seçili Miri testleri geçti. Kanonik kanıt dışa aktarımı ve denetimi ile aynı kaynakta güvenlik kapısı geçti: tam Cargo koşusunda 989 başarılı, 0 başarısız, 25 varsayılan dışı vaka. Bu 25 vaka ayrıca çalıştırılan kampanyalara eşlendi; alt senaryolar Cargo toplamına eklenmez. GB-110 exact regression/native26/RC zinciriyle kapalıdır.

[S3 teknik kabulü](urunlesme-s3-kabul-20261015.md), sonuçları ve önceki başarısız denemeleri ayrı kaydeder. E commitleri belge ve kanıtları kaydeder. C1/K-288 ve P33/K-293 araç retleri; artımlı HIR/type, gerçek ürün/provenance/adoption, insan/takvim, fiziksel Windows/minOS ve yayın/lisans/destek kapıları açıktır. Bu kayıt tam V1 ya da public signed release kabulü değildir.

## K-297 — ordinary LSP UTF-16 konum düzeltmesi

S3 raporundaki BMP dışı karakter sonrası hover/definition/rename ve tanı
konumu sınırı [ADR-153](../adr/153-lsp-utf16-konum-siniri.md) ile ele alındı.
İç kaynak aralıkları korunur, dönüşüm yalnız LSP sınırındadır. Yeni
olumlu/olumsuz LF/CRLF testleri ve minimal lsp-utf16 regression kipi eklendi;
exact kaynak `5b910fccad8e694fd26fabb58e1cc39adbfd65ea` ve `bmp-disi-konum`
fixed_by kaydıyla dar K-297 kapandı. Son tam faz 996 başarılı, 0 başarısız,
25 varsayılan dışı vaka; [kabul kaydı](lsp-utf16-kabul-20261015.md). Yeni cache,
grammar veya feature yoktur. S3 kanıtı tarihsel kaynağında kalır. İzleyen
[S4 yerel kabulü](urunlesme-s4-kabul-20261015.md) aynı yeni kaynakta ayrı RC/soak/IDE ve stdio
gecikme gözlemini kaydeder; artımlı HIR/type, gerçek ürün/UI/SLA ve tam P19/V1 açıktır.

Dış girdiler [kullanıcı teslim listesinde](urunlesme-kullanici-girdileri.md)
ayrıldı. Beş tam proje hazır değil, fiziksel cihaz yok; bu kapılar bekliyor.

## K-299 — Kabul koşucusu kapanış kapısı

Stdlib PDF9/service10 gerçek kısa koşuları geçti. Kaynak `8178920e03fed782a817d1864b8280fcb80812a3`,
tree `1eccf96d1a407880bf161c55bdd55c4d9a144866` için dar sonuç: temiz C2 native2 iki Cargo sahibiyle geçti: signed PDF9 ve service-fault2; alt vakalar Cargo owner sayısına eklenmez;
C2 öncesi gate4: 13 adım geçti; tam faz 998/1023 geçti, 0 başarısız, 25 atlanan C2 öncesi security2 --surekli geçti; 16 seçili libtest tam faz toplamına eklenmez GB-111: kapalı; toplu commit öncesi son belge kapısı gerekli.
Erken iptal, kısmi keygen/fixture ve nested owner kapanışı yalnız kabul betiğinde
düzeltildi. [Actual kayıt ve kapanış koşulları](kabul-kosuculari-yasam-dongusu-20261016.md)
stdlib hazırlığı, C2 native kabulü, kaynak öncesi kapıları ve bağımsız incelemeyi
ayırır; ilk iki başarısız kapı korunur. Yeni Cargo owner yoktur.
K-298 kaynağının RC/soak/IDE/stdio kanıtı K-299 HEAD kabulü değildir; eski S3
native kabulü tarihsel kalır. P11/P34/P19 ve tam V1 bu dilimle kapanmaz.

## K-300 — Mixed-revision kabul koşucusu doğruluğu

**Dar kaynak/native kabulü KAPALI (GB-112):** canonical Rust wrapper kökü,
erken handler, kısmi setup/host sahipliği ve cleanup iptalinin `passed`
sonucuna dönüşmemesi [ADR-149](../adr/149-mixed-revision-owned-wave.md)
revizyonuyla doğrulandı. Hazırlık13 ve C3 öncesi kaynak kapısı13/tam faz998/0/25
geçti; Clippy/sürekli güvenlik dahil, ilk ADR metadata kapı başarısızlığı ayrı
korunur. Temiz C3 `549dbe66a775db17875b47c67948245a6a8fbf60` üzerinde
mevcut tek Cargo sahibi/native9, dört dalga/160 HTTP200; sıfır
5xx/retry/body mismatch/failure ve özgün owner exit0/reaped kabulü geçti.
Son11 belge kapısının geçmesi takip commit'inin önkoşuludur.
Hazırlık alt vakaları yeni Cargo sahibi değildir; üretim compiler/host
değişmez. Tarihsel S3 ve ayrı C2/S kabulleri, RC/soak/IDE yeni kaynağa
aktarılmaz; P14 ürün/şema/SLA ve platform sınırları açık kalır.
[Koşucu rehberi](mixed-revision-wave.md).

## K-301 — Worker inflight test kanıtı

**Dar test doğruluğu tamamlandı.** Handler giriş işareti ve EOF/reset sonuna
kadar sınırlı yanıt kontrolü mevcut 0,2 saniye host grace/üç saniye handler
fixture'ında doğrulandı. V3 normal `web-oturumu`/CSRF GET/çerez/form POST
akışını kullanır; `herkese açık` POST rotasında dosya yazımı `eylem` içindedir.
CSRF atlanmaz. Worker3 tek Cargo sahibi 1/0/0 (78,28 saniye), wrapper'ın
25/0/0 doğrulaması ve özgün controller exit0/reap ile geçti. V3 kontrollü
10 senaryo/20 yöntem karşılaştırması ayrı mock kanıtıdır; HTTP/CSRF
helper'ının gerçek kabulü worker3 koşusudur.

İlk GET fixture'ının T045 reddi ve worker1 başarısızlığı korunur. V2 denetimi
geçti, worker2 yalnız inflight vakada başarısız oldu (25 test/1 hata);
assertion ayrıntısı yoktur. Eksik CSRF/çerez kaynak incelemesidir; gözlenmiş
HTTP403 veya işaret yokluğu değildir. İki başarısız koşu kendi sonucu ile korunur.

Varsayılan fixture AST'si ve 25 mevcut test metodu korundu; test eklenmedi/
taşınmadı, yeni Cargo sahibi yoktur. K-300'ün tam faz998/0/25 sonucu bu
çalışmaya aktarılmaz. Son 11 belge/güvenlik kapısının geçmesi kayıt commit'inin
önkoşuludur. Üretim davranışı ve genel P34/native DNS/tam fault/drain/yayın
sınırları değişmez. [Worker rehberi](worker-supervisor.md).

## K-302 — PG/TLS/göç yerel kabulü

Temiz `e6322dbc0424a54f777a7b14f65d78712cfced1a` üzerinde TLS8/göç1 geçti; mevcut 25 explicit ignored
vakanın dokuzu bu kaynakta ayrıca doğrulandı. Tam faz/ASan/soak sonucu yeni
kaynağa aktarılmaz; uzak CI, ürün/provider ve P36 kalan kapsamı açık.
Son belge/güvenlik kapısı kayıt commit’inin önkoşuludur; sonuç ortak raporun dış makbuzundan izlenir.
[Makbuz ve kapsam](urunlesme-k298-kabul-20261016.md).

## K-303 — Tek pool sahibinin yerel kabulü

Temiz `47812bfe8312d5a5a439931bf49a079a7b0b22db` üzerinde mevcut pool
Cargo sahibi 1 passed / 0 failed / 0 ignored verdi; 16 ham metrik anlık
görüntüsü, tek timeout sayacı ve normal PG SIGINT/exit0/reap doğrulandı.
Özel geçici kök kaldırıldı; kaynak başlangıç/son kimliği ve 12 pin eşleşti.
Mevcut 25 ignored vakanın yalnız biri bu kaynakta ayrıca yürütüldü; test
sayısı değişmedi. K-302’nin C5 TLS8/göç1 kaydı ayrı kaynaktadır. Native
metrik tüketicisinin sekiz vakası, genel P36 ve uzak CI/ürün/provider
kapanışı ileri sürülmez. Son 11 belge/güvenlik kapısının geçmesi kayıt
commit’inin önkoşuludur; [gerçek sonuç ve kapsam](urunlesme-k298-kabul-20261016.md).


## K-304 — Owned web finalizasyonu dar kabulü

**KAPALI (GB-113, dar kapsam):** cleanup/proof/son need sırasında gözlenen
iptal/deadline başarısız makbuz ve nonzero çıkışta korunur. İlk altı karşı
örnek ve V1 son need iki hatası tarihçede kalır; V2 sekiz olumsuz alt vaka
ve normal kontrolü iki mock metotta geçti. Mock, OS sinyal/PG/HTTP kabulü değildir.
Tam22 hazırlık ve kaynak öncesi15 kapı: tam faz 998/1023 geçti, 0 başarısız, 25 atlanan; Clippy ve sürekli güvenlik dahil 15 adım 1030,946 sn.
Temiz `abdc5cf36b4d55021df5edad08cee3a7ad953ebd` direct native4: tek Cargo sahibi 1 başarılı/0 başarısız/0 atlanan, dört vaka, 19 HTTP ve 6 SQL gözlemi; iki ayrı PG kümesi.
Özgün dış controller exit0/reaped, 22,077 sn. Dört vakada worker/PG grupları ve son command grupları boş; üç dış komut exit0/reaped. Özel PG dizini kaldırılması veya çocuk başına normal PG çıkışı bağımsız gözlem olarak ileri sürülmez.

Son 11 belge/güvenlik kapısı kayıt commit önkoşuludur; gerçek sonuç E/k304-final-gates-1/PROOF.json ve k304-final-gates-controller-1.json ile izlenir. Python alt vakaları yeni Cargo sahibi
değildir; ayrı `dene`/source-coverage ve genel P22/P36/ürün/platform kabulü
açık kalır. S/S3 ve K-298 RC/soak/IDE kanıtı aktarılmaz.
[Kit rehberi](owned-web-test-kiti.md), [exact makbuzlar](urunlesme-k298-kabul-20261016.md).


## K-305 — Bridge negatif koşucusu ve ayrı kaynak kapsamı kabulü

**KAPALI — GB-114 dar test-harness kapsamıyla kapalı.** İlk child öncesi
flag-only handler, original-owned bekleme/cleanup ve final proof sırasında
gözlenen iptal başarısız sonuçta korunur. Selector init tek başına leak kanıtı
değildir; production Rust/kit/Supervisor davranışı değişmez.

Hazırlık kaydı: Kontrollü hazırlıkta 15 Python metodu ve 27 senaryo geçti; eski sürümde 17 yapısal eksik, iki cleanup hatasında eksik makbuz ve bir terminal süre aşımında yanlış passed ayrıldı. Özgün dış owner eski exit1 ve yeni exit0 ile reaped; gerçek çocuk, OS sinyal teslimi, PG veya ağ kabulü değildir. `K305-PREPARATION.json`
SHA-256: `e445af8d0a82448488bd72d47e1360bba990751db0354f91061a93603c4918b1`.

Mevcut normal/ignored Cargo sahipleri korunur. Kaynak15 kapı:
15/15 geçti; faz 998/1023 başarılı, 0 başarısız ve 25 ignored; faz komutu 839,411 sn, toplam 1045,134 sn; Clippy ve sürekli güvenlik geçti, 194 kaynak pini korundu, özgün dış owner exit0/reaped. Temiz `40788a3d0a9f0d43083130bb0f062871861787db`, tree `9c1ea8dffa61d91f5946eff32a9c4cf1c124759e`,
üzerinde ayrı bridge Cargo1 ile unit ürün paydası14/dört native vaka/altı
admission negatifi geçti: Beş kaynak kapsamı raporu ham edge kayıtlarından yeniden hesaplandı: unit ürün 4/14; site_snapshot 0/12, panel_session_csrf 8/16, db_committed_and_request_rollback 3/6 ve db_fresh_case_isolation 0/6. Her raporda test paydası0 uygulanamaz; complete=true rapor bütünlüğüdür, yüzde100 kapsam değildir ve farklı raporların paydaları toplanmaz.
Mevcut tek Cargo sahibi 1 başarılı/0 başarısız/0 ignored (4,63 sn); dört native senaryo, 19 HTTP isteği, altı SQL gözlemi ve iki ayrı fresh PG kümesi doğrulandı; native sürücü 20,169 sn, dış owner 21,076 sn. Altı gerçek CLI admission negatifi geçti; installed=true, cancelled=false ve owned_groups_closed=true. Handler restoration ayrı kontrollü hazırlık kanıtıdır; gerçek OS iptal teslimi gözlemi ileri sürülmez.
Özgün Popen/WNOWAIT süreç sahipliği ve boş worker/PG/komut grubu makbuzları doğrulandı; dış owner exit0/reaped. Her PG child için bağımsız normal SIGINT/exit veya private runtime dizini kaldırma gözlemi ileri sürülmez; 205 kaynak ve 17 runtime pini korundu Bağımsız inceleme yedi özgün makbuzu, ham günlükleri, 12 public dosyayı ve beş kapsam raporunu doğruladı; E/k305-final-audits-1/INDEPENDENT-REVIEW.json SHA256=421facd52cc777ae0d34a480a3e971b2df6a6c63789eba64b05bcfb40d622d6e

Son 11 belge/güvenlik kapısı kayıt commit önkoşuludur; gerçek sonuç E/k305-final-gates-1/PROOF.json ve E/k305-final-gates-controller-1.json ile izlenir. Bu metin kapı sonucu yerine geçmez.
Bu kayıt yeni kaynağın actual bridge sonucudur; S3/K-304 direct kit veya
RC/soak/IDE aktarımı değildir. Unit/worker paydaları birleştirilmez; genel
P22/P36, ürün/platform ve C1/P33 kapsamları [köprü rehberinde](dene-native-web-koprusu.md)
ayrı tutulur. [Ortak rapor](urunlesme-k298-kabul-20261016.md) exact makbuzları taşır.

## K-306 — ayrı kaynak, Linux ve belge kanıtı

Kaynak doğrulaması, ikinci denemenin geçen 12 komutu ile yalnız EOF boşluk düzeltmesinden sonra yeniden geçen beş komuta dayanır; 15 kapının bütünü yeniden çalışmış gösterilmez. Temiz `7d5e4b0` kaynak commit’i ve exact ağaç kayıtlıdır. Kontrollü hazırlık Campaign16 ve PG18 yöntem/31 senaryoda geçti. Dış sahiplik hazırlığında core17, CLI9 ve runner5 modeli geçti; bu sayılar Cargo/native vaka toplamına eklenmez.

Gerçek inert VM kabulünde normal komut ve EOF olumsuz yolu doğrulandı. EOF iç adımı `failed/control-eof` kaldı; özgün çocuk kapanıp reap edildi. VM normal SIGINT/exit0 ile kapandı, fallback kullanılmadı ve özel host runtime silindi. Bu iki inert prob, Linux native4+7 kabulünün yerine geçmez.

Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Yalnız bu exact kaynakta2/25 mevcut ignoredCargo sahibi ve dispatcher4+HTTP7native geçti;tamfaz998/0/25 ayrı;kalan23ignoredCargo-vakasıbuAttempt8kampanyasındaçalışmadı. Native11 iç vaka Cargo toplamına eklenmez; C12 kontrollü modelleri yeni Rust/native toplamına katılmaz. Tarihsel RC/soak/IDE, fiziksel platform ve ürün kabulü bu kaynağa aktarılmaz. Foreground PostgreSQL normal SIGINT/exit0/reaped; iki kabul rolünün sahipli unit/path temizliği, manager running son durumu, guest/host bütün özgün sahiplerin reap sonucu ve VM normal exit0 kapanışı doğrulandı; fallback kullanılmadı, host runtime kaldırıldı. Gerçek source15/clean commit ve attempt8 source/native/owner/raw zinciri bağımsız review ile issues=[] olarak doğrulandı; frozen kabulün exact11 öncül kayıt ve public artifact seti eşleşti. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir.

GB-115 kapali; Tarihsel S3 ile K-298–K-305 sonuçları kendi kaynaklarında kalır; farklı SHA sonuçları toplanmaz. P09/P13/P36 kısmi, P37 sürekli; ürün/sağlayıcı benimsenmesi, genel exactly-once ve fiziksel platform U2 açık kalır. C1/P33 reddedilmiş kapsamı yeniden denenmez. İlk iki failed kaynak kapısı ve EOF olumsuz adımı başarıya çevrilmez. [ortak kabul raporu](urunlesme-k298-kabul-20261016.md).

## K-307 — SQL kabul harness’i ve foreground sahibi: dar kapsamda kapalı

Fresh exact ikinci kaynakta mevcut tek ignored Cargo sahibi1 başarılı/0 başarısız/0 ignored ve dokuz SQL native vaka geçti; frozen kaynak/native/özgün owner/raw zinciri bağımsız doğrulandı.

Tarihsel ilk sticky dilimde baseline10, ayrı inert13 ve birleşik23 geçti; exact AST fakeTERM/fakecluster.close karşı örneği tek beklenen failure/exit1/reaped kaldı. Gerçek OS/PG sızıntısı gözlemi değildir. Foreground takip2 standalone23 ve birleşik ordinary24 de kendi özgün owner0/reaped/errors[]/cancelled[] makbuzlarıyla geçti. Birleşik24 önceki23+tek nested foreground23 hazırlık yöntemidir;47, yeni Cargo sahibi veya native vaka toplamı değildir.

**İlk kaynak diliminin gerçek ayrı kanıtı:** Tam15 kaynak kapısı ve tam faz998/0/25 geçti; temiz kaynak `216d7e1a2d3012d8bef7f55030d42d83045b3fff`, tree `002ca4938767884b12b0cb395e94142059eedfbd`. Kaynakgate1/commit1 ve özgün owner makbuzları kendi kaynaklarında korunur; bu ilk kaynak kabulü SQL foreground takip2 native9 veya ikinci kaynak kapısı yerine geçmez.

Beş test-harness kaynağı SQL tüketicisini `OwnedLocalCluster` foreground özgün Popen sahibi ve `SqlPgCampaign` adapter’ına bağlar. Ayrı yapışkan iptal ledger’ı, cleanup sırasında H maskesi/garantili restore, korumalı handler kurulumu, tüm handler restore, ExitStack sonrası passed ve publication iptal/deadline/failure kontrolleri korunur. Cleanup90 kabul600 sınırını uzatmaz. Readiness/data-directory/version pinleri ve özgün probe/PG nesneleri korunur; belirsiz sahip veya private root kimliği değişiminde kök silinmez ve sonradan benimsenmez. Dokuz SQL oracle, pending/unknown no-replay ve DB+FS atomik değil sınırı aynı kalır.

`ZEE_MODULE_SQL_NATIVE_OUT` yalnız internal ignored testin taze mutlak çıktı yolu seçimidir; yokluğunda mevcut0700 geçici dizin varsayılanı korunur. Ürün CLI/API kapasitesi açılmaz.

**Yeni exact ikinci kaynak:** `9df1b4abb52372dce86c92255392032b6fa08574`; tree `e7de0cfc2dce9e9c271979cc963c9629c49263e8`. Yeni exact ikinci kaynak tam15 kapısı geçti (1118.889 sn); gerçek tam faz998/1023 geçti,0 başarısız,25 ignored. İlk216d7 kaynak15 sonucu bu sonuca eklenmez. Temiz ikinci kaynak 9df1b4abb52372dce86c92255392032b6fa08574, tree e7de0cfc2dce9e9c271979cc963c9629c49263e8; sourcegate2 tested17 overlay ve sourcecommit2 özgün owner makbuzları eşleşti; kaynak344/production175 korundu.

**Fresh SQL native kabulü:** Fresh exact ikinci kaynakta mevcut tek ignored Cargo sahibi1 başarılı/0 başarısız/0 ignored ve dokuz SQL native vaka geçti; frozen kaynak/native/özgün owner/raw zinciri bağımsız doğrulandı. SQL/HTTP/DDL/down-up/pending/unknown no-replay oracleları değişmedi. İki genel şema, parametreli CRUD/hostile veri, bağımsız readonly gözlem, HTTP/host kilidi, gerçek down-archive-up, kalıcı pending/reconcile ve collision rollback dokuz mevcut vakada doğrulandı; DB+FS atomikliği iddiası yok. Gerçek Rust sürüm günlüğü macOS aarch64-apple-darwin/rustc1.93.1; aynı foreground PG16.15 profili doğrulandı. Aynı kaynak release ikili/typed build JSON ve explicit Cargo/Rust/Python/git4 tool pini, public PG/runtime17 pini başlangıç/son denetiminde korundu. Foreground PostgreSQL aynı özgün Popen nesnesiyle normal SIGINT/observed_exit=exit=0/reaped; readiness kimliği ve tüm özgün probe sahipleri kapandı, fallback yok, private kök kaldırıldı. Native sürücü 60.761 sn, harness 3.672 sn; özgün dış owner0/reaped/errors[]/cancelled[]. Root native audit ve bağımsız peer issues=[]; frozen exact9 öncül kayıt ve sıralı public artifact seti gerçek hash/baytlarla eşleşti.

Yalnız yeni exact ikinci kaynakta mevcut tek ignored Cargo sahibi/SQL9 dar kabulü sayılabilir; diğer24/25 ignored Cargo bu kampanyada çalışmadı. Python kontrollü yöntemler ve farklı SHA’ların sonuçları Rust/native toplamına eklenmez. Diğer dört PG tüketicisi açık; SQL dar diliminin kabulü tüm beş PG sahibinin kapanışı değildir. P20 kısmi ve39 plan statüsü korunur; ürün/adoption/platform ve C1/K288/P33/K293 retleri açık kalır. RC/soak/IDE/diğer fiziksel platform/ürün sonuçları bu kaynağa aktarılmaz.

Bilinçli etki incelemesi: production Rust175/bağımlılık/katman, grammar/API/ABI, normatif spec/RFC, kullanıcı tanısı/katalog kodu, sabit performans iş yükü veya compiler/LSP kalıcı durum değişmez.

**Kapanışın koşulu:** K307/GB116 dar kapanışı yalnız exact belge overlay son11 kapısı ve sonraki tested-overlay temiz kayıt commit’i ile iki özgün owner makbuzuna bağlıdır; bu metin kendi başına kapı/commit kanıtı değildir. Dar kapanış yalnız bu exact belge overlay’inin son11 kapısı ve o test edilmiş overlay’i kaydeden sonraki temiz commit/özgün owner makbuzlarıyla geçerlidir. Bu metin kendi commit SHA’sını üretmez veya kendi başına kapı/commit kanıtı sayılmaz.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-2/PROOF.json` | `374b2c08411e4a56950b03c8f927d3c8002743920bfb0862aa1776de84af5a86` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-controller-2.json` | `9656b67c69241742721c657f2a1ed0c92d0ba7604057b896245293b97e4f4251` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-2.json` | `877b4ba036b9d35a4f0275b2709925e4a6841682e2bd7c6c67087835435eb226` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-controller-2.json` | `e647bf72ae26de16717a6b8aa5d1c71ff56e4282c3aa8c4da11d19c3c5364584` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-1/PROOF.json` | `b62fd8cf03bfd4d14fb0453e06ef1a1f0c83abddbf94464928acb0b34b13b98b` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-controller-1.json` | `4496e9ea5bf4812782d756f7f5ce80c86944c7c44a84b97f08879a0e7a62c012` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-native-sql9-1/native/proof.json` | `3dac8e6964993a3b4dcbe8ef36e6aae8ad3832be2e7424b735a143898abece32` |
| root_native_audit | `/private/tmp/zee-k300/K307-NATIVE-ROOT-AUDIT-1.json` | `de1632e288fa29940c29ad3e9bd785ad9013abf44f511448dd5aa31605a263cd` |
| native_peer_review | `/private/tmp/zee-k300/K307-NATIVE-PEER-REVIEW-1.json` | `1020a23986c7a998acedaf99d0efa6289bc0a395b8fa7ce81e00f019ff5ff961` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-frozen-acceptance-1.json` | `e542370e8d951ec0921e268412aeee459eca371645d4d69bca4ed9819293fb1a` |
| source_pins | `/private/tmp/zee-k300/k307-source-followup-pins.json` | `42cab50fc8be5935dc2ac71c115b7e7ef8a011dc5f67887543a6881ca6c97aad` |
| template_record_application_ready_documents | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-DOCS-APPLICATION-READY-1.json` | `ba40001b8cd92e20a2026c980b080fc74d0a995c12187148263a1720b4a65b55` |
| template_record_application17 | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-APPLICATION-1.json` | `8cee65e19ae75e8057755690ed9b58178818fc7e52381c51378de1c93c1abcde` |
| template_record_application_independent | `/private/tmp/zee-k300/K307-FOREGROUND-FOLLOWUP-APPLICATION-INDEPENDENT-REVIEW-1.json` | `8c353ca282c03e5c1af32d0718d8b685390a0d5a70139901fd11c70934370628` |
| template_record_source1_commit_root_audit | `/private/tmp/zee-k300/K307-SOURCE-COMMIT-ROOT-AUDIT-1.json` | `f8397d4f59a1c72f1d963b0bd3b7d1e0cf4f1e6eb7ac0e8f0df73ceac38971fb` |
| template_record_source1_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-1.json` | `7cd3434d0de144d09249128262025ad082a0c3d46228b175fc64c0ba597a5b86` |
| template_record_source1_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-1/PROOF.json` | `1db8c354449b0d96fa0209afd3b8761f00b142c31b6f082b6c86e1ec73d266a1` |
| template_record_first_source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-commit-controller-1.json` | `6536aa77e7b1f65a2a817de4ef61761c270f5149b6ff3b9c3a0ef4de298205c9` |
| template_record_first_source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k307-source-gates-controller-1.json` | `20e9306f6251a5a938a2990002877ee12f0bf8b20024e84087f06b5bdc4a35a6` |

**Kalan yerel işler:** Backlog bağımlılık sırasındaki diğer dört PG tüketicisinin özgün foreground sahiplik/iptal kabulü ve ayrı açık teknik işler sürer; genel P20, ürün/adoption, fiziksel platform ve dış girdiler açık kalır.

## K-311 — effect harness lifecycle: dar kapsamda kapalı

K-311 / GB-120 dar kapsamda kapalı: effect native kabul harness’inin kalan özgün kaynak cleanup denemeleri, ilk hata kimliği, yapışkan iptal/publication ve plaintext PG sahipliği dar dilimi. Exact kaynak ea535fad3dbd4beb2dd7cd7e3b217c041686e89b tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/effect11/60need frozen kabulü; özgün plaintext PG/readiness/retained cleanup kanıtlandı, tüm inner host0 iddiası yok.

Child-free1 expectedFAIL, eski baseline8, ayrı lifecycle27/PG18, aynı yöntemleri içeren selected45 ve eski8+tek wrapper ordinary9 kendi kontrollü kayıtlarında korunur; nested sayılar Cargo/native toplamı değildir. Native11/60need/source14/plaintext loopback196608/no-replay korunur; bu teknik kabul ürün/provider exactly-once sözü değildir.

Bu dar effect kapanışı gerçekten tamamlanınca beş-harness PG takip kuyruğunda açık tüketici kalmaz; genelP09 ve39 plan statüsü, ürün/sağlayıcı/insan/fiziksel platform/dış girdiler değişmez. Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her kabul kendi exact başarılı deneme kayıtlarıyla izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belge kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler. Ayrıntılar [ADR-135](../adr/135-sabit-native-dis-etki-tuketicisi.md) ve [ortak raporda](urunlesme-k298-kabul-20261016.md).

## K-310 — metrics harness cleanup ve özgün PG/FD sahibi: dar kapsamda kapalı

K-310 / GB-119 dar kapsamda kapalı: Metrics native kabul harness cleanup/ilk hata, yapışkan iptal/publication ve özgün PG/worker/FD sahipliği dar dilimi. Exact kaynak 1130569460203049374e16146a955e208da9dd16 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta metrics8/29 require ve iki ayrı ignored Cargo sahibi frozen kabulü; özgün PG/readiness/FD/cleanup ve beş direct worker/two alias2 doğrulandı. Baseline15, child-free tek beklenen failure ve kontrollü standalone52/combined16 ayrı kapsamda korunur. Yalnız bu dar metrics kabulü gerçekten tamamlandığında effect tek açık PG tüketicisi kalır; genelP13 kısmi ve39plan statüsü mevcut değerlerindedir.

**Gerçek kaynak kabulü:** Exact kaynak 1130569460203049374e16146a955e208da9dd16, tree 98898300e2a5e1ecf8c77c88171dd12c3b69a2e0; tam15 kaynak kapısı 1220.731sn ve temiz tested-source commit/özgün owners doğrulandı. Aynı kaynak koşusunun tam faz998/1023 sonucu0 başarısız/25ignored ve22faz olarak frozen validator ile doğrulandı. K310 source4+OPENdocs11 atomik uygulama zinciriyle bağlıdır. Baseline15, standalone52(genericPG18+metrics34), ordinary16(eski15+tek52wrapper) ve native-driver pure28 ayrı kontrollü hazırlıklardır; Cargo/native toplamına eklenmez. Proposal1→2 yalnız Rust biçim değişikliğidir; Python model baytları değişmedi. Tarihsel ilk kaynak15 koşusu ilk12 komutu geçti; faz998/1023,0 başarısız/25ignored ayrı kayıttadır.13.diff-check native harness line540 dört trailing space nedeniyle exit2 verdi;14.secret-scan/15.security-continuous çalışmadı, özgün dış owner exit1/reaped oldu. Bu başarısız koşu kaynak kabulü değildir. Proposal2→3 yalnız bu dört whitespace baytını kaldırdı; native harness fullAST aynı, diğer iki Python/Rust/production175/shared helper değişmedi. P3 controlled modelleri ve başarılı yeni kaynak15/cleancommit yalnız kendi actual makbuzlarıyla değerlendirilir; önceki başarısız deneme kabul diye taşınmaz. İkinci kaynak15 denemesi ilk13 komutu geçti; phase22/998/1023/0failed/25ignored,clippy ve diff-check0 tarihsel ayrı sonuçlardır.14.secret-scan1 nedeniyle koşu başarısızdı,15.security-continuous çalışmadı; özgün dış owner1/reaped oldu. Public16 dosyalık redakte teşhisteki tek generic-api-key816 literal gerçek scripts/native-metrics.py public SHA256 kimliğiyle eşti, credential değildi. Proposal3→4 yalnız preflight named METRICS_SOURCE_SHA globali ve aynı selected hash verifier referansıdır; literalpublichash, native b5fd, diğer kaynaklar/production175/H/PG6e5 değişmez. P4 standalone52/.395sn ve combined16/.441sn özgün owner0/reaped kontrollü sonuçları actualpeer3 kayıtlarıyla ayrıdır, Cargo/native toplamı değildir. Erken default8.30.1/emptyignore taraması16proposed publicdosyada0bulgu verdi; taranan report öncekiNULLDRAFTnote idi, FILLEDnote/fullR/source15 kabulü değildir. Başarılı freshsource3 yalnız kendi tam15/cleancommit/owner makbuzlarıyla değerlendirilir; başarısız source1/source2 kabul diye taşınmaz.

**Gerçek metrics8 ve distinct nested libpool kabulü:** Yalnız bu exact kaynakta canonical ignored metrics Cargo sahibi ve onun çağırdığı mevcut exact ignored release lib pool Cargo sahibi geçti: iki ayrı Cargo sahibi, sekiz metrics gözlemi/29 require; native sürücü 159.862sn. Kaynak350/production175, consumed15+iki unchanged shared helper, iç11 sahipli komut/20 açık safe log ve public raw envanter frozen/root/peer ile bağlandı. Nested release pool120s ve TLSv1.2/local-loopback trust/sslmode=require+CA profili korundu. Aynı kaynak envanterindeki25ignored içinden bu iki sahibi çalıştı; kalan23 bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback ve özgün readiness probe kapanışı doğrulandı. Beş doğrudan worker özgün owner ile reaped; alias ve stderr-alias için beklenen exit2 doğrulandı, tüm worker/host için normal0 sonucu türetilmedi. FD/component kapanışı, private root kaldırılması, sticky cancellation ve ayrı tek-seferlik cleanup90 sınırı doğrulandı; toplam600 ve nested120 sınırları genişletilmedi. Private credential/CA/key/DSN/PHC/env/argv bytes veya hash’leri public kanıt değildir. Metrics mevcut public kapsamı beş direct native worker içerir; bunların ikisi alias beklenen ret yoludur; alias retleri dahil bütün worker/host/component için normal0 sonucu varsayılmaz. Özgün owner/reaped/PG/readiness/FD oracleları yalnız gerçek public makbuzlarının kapsamıyla değerlendirilir.

**Tarihsel kontrollü hazırlık:** Gerçek child-free exact finally karşı örneği1test/0.003sn tek beklenen failure/exit1/reaped olarak doğrulandı; static14b25 ve actualpeer8401 kayıtları exacteski8cb/finallyb757 baytlarına bağlıdır. Yalnız owner.close fake olayı görüldü; gerçek OS/FD/PG sızıntısı veya disktePASS gözlenmedi. Mevcut değişmeyen ordinary metrics baseline15 gerçek15test/0.002sn PASS/exit0/reaped/errors[]/cancelled[] verdi. Bu eski baseline kabulüdür; ayrı focused52/combined16 kontrollü yöntemler, kaynak15 veya metrics8/nestedlib native sonucu değildir. Standalone preflight52 yöntemi (korunan genericPG18+metrics34) gerçek0.424sn PASS/exit0/reaped/errors[]/cancelled[] verdi; özgün watcher0.526857375sn, ilk koşu. Bu kontrollü fake-boundary model sonucudur; gerçek PG veya metrics8/nestedlib native kabulü değildir. Birleşik ordinary16 yöntemi (korunan15+tek nested52 preflight wrapper) gerçek0.540sn PASS/exit0/reaped/errors[]/cancelled[] verdi; özgün watcher0.601194875sn, ilk koşu. İçteki52 yeniden toplamlanıp68Cargo/native testi sayılmaz; mevcut Rust ordinary sahibi aynı kalır. Dört kaynak/test dosyası P2 hazırlığında pinlidir: scripts/tests/native-metrics-test.py SHA256=ac4c84f9a2f9948883715f4c96c0bda70f5a02d6194e3bd493e270a66d8159a0 bytes=36531; scripts/tests/native-metrics-model-test.py SHA256=a7233ef5c711fdfb46697a584f663d41473709e311e360ba0c8dc25e99c1d1a3 bytes=5329; scripts/tests/metrics-pg-owner-preflight-test.py SHA256=0db506b67b448b34fcbf990f1fb8c3396e906940e11b65e3f6011abb3183ebcf bytes=52402; compiler/tests/native_metrik_host_testi.rs SHA256=0a8277ceb3908dfd19b50cef7a4140de078adc424c1e33ccd728ce1027f1fc99 bytes=4933. P1 staticpeer e537f73d ve actualmodelpeer 7f418546997cef115a41c75387b4f1957bb432a9c88fd43199029b70e0a56cbc; P2 yalnız Rust biçim delta review dda334664c824bff97336239dee51fb4936bbd91c42927db5090bed3eae1e95c. Kaynak/test/bu OPEN belgeler aynı atomik uygulama dilimidir; bu pinler tek başına kaynak15, temiz commit veya native kabulü değildir. Dört kaynak/test dosyası ve52/16 kontrollü sonuç pinlidir; source pin toplamı yalnız ayrıca doğrulanmış tam manifestten bağlanır, burada tahmin edilmez. Mevcut15 ordinary yöntemi ve mevcut Rust Cargo sahibi korunur; tek yeni nested preflight wrapper ile Rust beklentisi gerçek combined16 sonucuna bağlıdır. Gerçek değişmeyen baseline15/0.002sn PASS/exit0/reaped ayrı kayıttadır; standalone52/combined16 kontrollü kabulü veya native sonucu ile toplanmaz. Python model yöntemleri native vaka veya Cargo sahibi toplamına eklenmez.

Child-free exact eski finally fake owner.close karşı örneği tek beklenen failure/exit1/reaped olarak tarihsel kalır: kalan fake child/FD/PG/proof callback denemeleri atlanmıştı; gerçek OS/FD/PG sızıntısı veya diskte kalmış PASS gözlemi değildi. Değişmeyen ordinary baseline15/.002sn, standalone52=genericPG18+metrics34/.424sn ve combined16=korunan15+tek nested52 wrapper/.540sn ayrı kapsamlarıdır. Bunlar68 Cargo/native test değildir. P2 yalnız Rust biçim değişikliğidir; Python3 P1 ile bitexact, actual model kanıtı aynıdır.

Mevcut metrics8 gözlemi ve29require assertion;200/200/429/404/426/200 response/DB oracleı, actual4lease/5.waiter/timeout/recovery, bounded backpressure/noSIGPIPE, stdout/stderr alias sessiz ret, iki worker rollup ve credential designated metadata kompozisyonu korunur. Metrics TLSv1.2 sunucu/local+loopback-trust HBA/kendi CA ve client sslmode=require profili korunur; job35sn/scopedCOMMIT veya effect plaintext196608 profili taşınmaz. Shared native-pg-owner.py exact6e5f helper değişmez.

Outer mevcut ignored `native_metrik_http_pg_ve_fd_sahipligi` ile nested mevcut ignored `postgresql::havuz::testler::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar` aynı actual source üzerinde iki distinct Cargo sahibidir;8gözlem8Cargo testi değildir. Nested release `--lib/--exact/--ignored` seçicisinin120sn bütçesi korunur. Yalnız gerçek aynı-kaynak ignored envanteri25 ve iki sahibin ayrı başarılı sonuçları doğrulandığında2/25 ve kalan23bu kampanyada çalışmadı denebilir; başka kaynak/faz sonuçları toplanmaz. Prebuild/no-run native kabulü değildir.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-3/PROOF.json` | `dc35cc4d5631aade4c08d8c6ca8e7008cabc666cd073a841e5d6e210ecb6ae81` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-controller-3.json` | `7b0062fd897f8429077116d769b7346f5a986c47cb85313a58b762b1a9c6ceff` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-commit-1.json` | `25da4ddfd0ca81e82f1c5fd4c7fd69d96ab19f9e7120b4230b7970b4ca81349d` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-commit-controller-1.json` | `47bb522fa812a9077d6a4c3ea99225b16a1a6ed48c049c18c2c3ef641dd26219` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-native-metrics8-1/PROOF.json` | `2021432ed37dac8f01f1a2a5ae9a3e41a7da1ddb623d081c762ed3bb3ef0064b` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-native-metrics8-controller-1.json` | `69ab8a04a3993e61faa63a433c9f36683d996dfa111a0b638f33644cc6be09fc` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-native-metrics8-1/native/proof.json` | `5a5231335ee084f9ea39bc3a32b91027ed50889ff3574b58a16e9f3402fe8e94` |
| root_native_audit | `/private/tmp/zee-k300/K310-NATIVE-ROOT-AUDIT-1.json` | `8fd5d7e89398830f8f374c20c36763ce9f11d85934e8711c824a23903719fef1` |
| native_peer_review | `/private/tmp/zee-k300/K310-NATIVE-PEER-REVIEW-1.json` | `f6992c6a02820299d5f11c495859f1b186c79d6455b8f6ae8652444f54c13c99` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-frozen-acceptance-1.json` | `2874488d10c7b6777d1261345996eb4e5547bc3da911f8dda65413de24ef1820` |
| source_pins | `/private/tmp/zee-k300/k310-source-pins-v3.json` | `8fd3c5f281eb8e7f347d74040a07a3bc642185816458a43eabe5b31dbb8399ba` |
| closure_template | `/private/tmp/zee-k300/K310-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `99540f4b2aefe94f10e5199c690bc1da0aa646971e6d6b5dcf7bdf7d3892a176` |
| closure_template_review | `/private/tmp/zee-k300/K310-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `f44e16a514962ed3b78563ff895cb16dd79d707a0e8172260931138c1a808bd5` |
| K310-METRICS-PROPOSAL2-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-PROPOSAL2-INDEPENDENT-STATIC-REVIEW-1.json` | `dda334664c824bff97336239dee51fb4936bbd91c42927db5090bed3eae1e95c` |
| K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-1.json` | `7f418546997cef115a41c75387b4f1957bb432a9c88fd43199029b70e0a56cbc` |
| METRICS-NATIVE-CLEANUP-ACTUAL-COUNTEREXAMPLE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/METRICS-NATIVE-CLEANUP-ACTUAL-COUNTEREXAMPLE-INDEPENDENT-REVIEW-1.json` | `8401a089c646f72b53c6c591f035b6d69bb7225f389552d59ae8ee1556d22ae1` |
| K310-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `57b58279f1a29c35faa431eb8b797227b5b8b075a0b59fc3dd01f0c0d3d79329` |
| K310-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `a658e5a764a1caf0d2f9c1cb9017f89c23e9d940f1d6753fcd693bf1a4741ffc` |
| K310-METRICS-PROPOSAL3-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-PROPOSAL3-INDEPENDENT-STATIC-REVIEW-1.json` | `59e7f4e8570b24c2d42fd05a30baa1c81b0c5319e54a899a9c3a12a0ca1f0399` |
| K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-2.json | `/private/tmp/zee-k300/K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-2.json` | `c9cca289dc45b62ac2e1daed5da9e1d7cf3992346874abb7ce614c868dc6b664` |
| K310-P3-ACTUAL-MODELS-ROOT-REVIEW-1.json | `/private/tmp/zee-k300/K310-P3-ACTUAL-MODELS-ROOT-REVIEW-1.json` | `6f824c817464d9220e85965a8dd552c62c59eeae69202343552d0bcd160935a7` |
| K310-SOURCE15-ATTEMPT1-ROOT-FAILURE-AUDIT-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT1-ROOT-FAILURE-AUDIT-1.json` | `841d1b2eabbe7190eabdb0f58dae1ef05af5fb8e0d4db905edbdaf94f6bcabed` |
| K310-SOURCE15-ATTEMPT1-FAILURE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT1-FAILURE-INDEPENDENT-REVIEW-1.json` | `61e70c39523d33d94a8494b858162ab26015aeae29c1fcc03a3fbbb5cf6f8dab` |
| K310-FORMAT-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-FORMAT-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json` | `45bfbeb76b117f96216497fc06ed1d38c5e4649b33fb7c8a70fe9e84ba1ae430` |
| K310-FORMAT-FOLLOWUP-APPLICATION-1.json | `/private/tmp/zee-k300/K310-FORMAT-FOLLOWUP-APPLICATION-1.json` | `a2c8254541f0dee63b32ecb6b015503a81e2b6f749ff477ad14ab6d1c5e7289d` |
| K310-FORMAT-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-FORMAT-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `f350605fd3de178679675c2dbfb1fc72f3800628752ee930d9396d463449b214` |
| k310-metrics-proposal-4/PREPARATION.json | `/private/tmp/zee-k300/k310-metrics-proposal-4/PREPARATION.json` | `a6ffdebd90d9f2edc7b8b1a16cedf41425cb2f99fb2a197217d36a8f5d739dec` |
| K310-METRICS-PROPOSAL4-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-METRICS-PROPOSAL4-INDEPENDENT-STATIC-REVIEW-1.json` | `5b9576f7a48e1a1adc20129fba7aa491d6ffb625753bc34e2d6cc6320f86fdbf` |
| K310-MODEL-FIXTURE-PREPARATION-3.json | `/private/tmp/zee-k300/K310-MODEL-FIXTURE-PREPARATION-3.json` | `b625c646bc8bdd429d9d06cc1989a58050a78bb0110fbb598526c11347bef57f` |
| K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-3.json | `/private/tmp/zee-k300/K310-METRICS-MODELS-ACTUAL-INDEPENDENT-REVIEW-3.json` | `2f8e05bcf0c5fed4441cc9700445fa37894d043e79bdf679bdef6b6d923d7b5f` |
| K310-P4-EARLY-REDACTED-SCAN-1.json | `/private/tmp/zee-k300/K310-P4-EARLY-REDACTED-SCAN-1.json` | `e0a985151ee0891f0f4afb5885d77bfd5ffdbc64cf04d02386e01a104b2efd49` |
| K310-SOURCE-ATTEMPT2-PUBLIC-HASH-FOLLOWUP-DOCUMENT-FILLED-PREPARATION-2.json | `/private/tmp/zee-k300/K310-SOURCE-ATTEMPT2-PUBLIC-HASH-FOLLOWUP-DOCUMENT-FILLED-PREPARATION-2.json` | `2edffa553e26b9f71f8cee30691901b40b473374eef72abd499a9db25ae7d137` |
| K310-PUBLIC-HASH-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json | `/private/tmp/zee-k300/K310-PUBLIC-HASH-FOLLOWUP-SOURCE-TOOLS-DOCUMENT-CAS-INDEPENDENT-STATIC-REVIEW-1.json` | `09a117ead29ccd53b4b65bbae0f212800d105bb0e39ee6092abd662e0d916a80` |
| K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-1.json | `/private/tmp/zee-k300/K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-1.json` | `c7040a7157dfe007fa6daef0bbe0d2b4440d8505f3b5b3ce4019407ca70e5a49` |
| K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-PUBLIC-HASH-FOLLOWUP-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `64698561b962971d15d9755c00c1bacf8d578096ffe670e3fb89aab20944d044` |
| K310-SOURCE15-ATTEMPT2-ROOT-FAILURE-AUDIT-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT2-ROOT-FAILURE-AUDIT-1.json` | `b2bff5608af534f36b29ca865854f57a56abba2cd98b48b7376872f4a262923e` |
| K310-SOURCE15-ATTEMPT2-FAILURE-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-SOURCE15-ATTEMPT2-FAILURE-INDEPENDENT-REVIEW-1.json` | `5ec4d3f3b78e948ae9e061b3802a646be36bbab711d68fa9c50bed2eaaf067e7` |
| K310-EARLY-SOURCE-PREFLIGHT-ROOT-AUDIT-1.json | `/private/tmp/zee-k300/K310-EARLY-SOURCE-PREFLIGHT-ROOT-AUDIT-1.json` | `83fc9f318014ce0c76ee81ee68fc0c7c57f53890f3255e4a151be95b45e662c1` |
| K310-EARLY-SOURCE-PREFLIGHT-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K310-EARLY-SOURCE-PREFLIGHT-ACTUAL-INDEPENDENT-REVIEW-1.json` | `7b9f7e7d2925eb3e9fc386f33cf00fbf053e9ff3991e47bb4f173dcb4264e478` |
| k310-public-hash-followup-controller-1.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-public-hash-followup-controller-1.json` | `2c5f28d75d75380707d0ac86e789df080d15331178c919fd5b6578916ffea634` |
| k310-source-attempt2-failed-PROOF.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-2/PROOF.json` | `e20d431d6befffe569e3aeb4b14fae41d585eac425c3add8bd95e720d015f0e3` |
| k310-source-gates-controller-2.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-source-gates-controller-2.json` | `15c379b19dd921174c9bf6a19f60d4dc7ec92902d71b5941363d052aa3958697` |
| k310-early-source-preflight-1-PROOF.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-early-source-preflight-1/PROOF.json` | `d4d62003530b9bf03464e14d306b5f88b7841e175cea56e1f6e6e644c20b476f` |
| k310-early-source-preflight-controller-1.json | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k310-early-source-preflight-controller-1.json` | `b9a4cd385609f7fed41325e096f0a80e18d6b2af03fa4a94486fa63d58815d50` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K310-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `a658e5a764a1caf0d2f9c1cb9017f89c23e9d940f1d6753fcd693bf1a4741ffc` |

Bu yalnız dar metrics dilimidir; gerçek kapanıştan sonra effect tek açık PG tüketicisi ve diğer açık teknik işler sürer.39plan statüsü korunur; P09/P13/P20/P23/P36 kısmi/P37sürekli, GB046/109 açık, GB115–118 kendi exact kaynaklarında kapalı geçmişi ve ürün/adoption/provider/fiziksel platform/dış proje/RCsoakIDE/C1K288P33K293 ret sınırları değişmez.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans ve compiler/LSP kalıcı durum değişmez. Ürün komutları/protokol/counter/göç yolu aynı olduğundan docs/native-metrics.md, ADR119/152 ve mevcut kaynak-pinli rehberler bilinçli unchanged tutulur; ADR116 dar harness sınırını taşır. PHC/parola/cookie/request/cert/key/DSN/credential özel baytları veya hash’leri public kayda alınmaz. Yalnız explicit public artifact role envanteri bağlanır; private runtime taranmaz, eski PID/kök/port benimsenmez.

## K-309 — job dispatcher harness cleanup ve özgün PG sahibi: dar kapsamda kapalı

K-309 / GB-118 dar kapsamda kapalı: Job SQL dispatcher native kabul harness cleanup/sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/job11 frozen kabulü; özgün PG/readiness/cleanup ve yalnız ilk vaka iki host0 gözlemi doğrulandı. Baseline2, child-free tek beklenen failure ve kontrollü standalone44/ordinary3 ayrı kapsamda korunur. Diğer metrics/effect iki PG tüketicisi açık; genel P09 kısmi ve39plan statüsü mevcut değerlerindedir.

**Gerçek kaynak kabulü:** Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96, tree 1f055acaa194c9eb47c51f25c74f85cf8803b273; tam15 kaynak kapısı 1248.429sn ve temiz tested-source commit/özgün owners doğrulandı. Tam faz998/1023 geçti,0 başarısız,25ignored kendi kaynak koşusudur. K309 source4+OPENdocs11 gerçek atomik uygulama ve bağımsız makbuzlarıyla bağlıdır. Standalone44(genericPG18+job26), ordinary3(oldmask2+tek44wrapper), baseline2 ve native-driver pure15 hazırlığı ayrı kapsamlardır; Cargo/native toplamına eklenmez. İlk combined3 fixture1 eksik üç public import bağımlılığı nedeniyle2maskfailure/exit1/reaped kaldı; fixture2 kopyaları tamamlandı, Proposal3 baytları değişmedi; kaynak/native kusuru olarak gösterilmez.

**Gerçek native11 kabulü:** Yalnız bu exact kaynakta canonical tek ignored Cargo sahibi/mevcut job11 geçti; native sürücü 111.174sn. Kaynak348/production175,11case/32assertion/35s handler ve bağımsız heartbeat, TLSrequire ile scoped plaintextCOMMIT/no-replay ayrımı, fixture20 ve dokuz safe tool/raw kaydı frozen/root/peer ile bağlandı. PHC bahanesiyle gizli/atlanan komut boşluğu kabul edilmez; kalan24 ignored Cargo vakası bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback ve readiness özgün probe kapanışı doğrulandı; private runtime kaldırıldı, özgün dış owner0/reaped/errors[]/cancelled[] ve sahipli cleanup sınırları korundu. Host normal0/reaped/stderr0 makbuzları yalnız ilk vakadaki iki canonical-host-actor gözlemidir; tüm worker/host/component için bağımsız normal0 sonucu değildir. Public host0/reaped/stderr0 makbuzları yalnız ilk vakadaki iki canonical-host-actor gözlemini kapsar; tüm worker/host/component için bağımsız normal0 sonucu iddia edilmez. Diğer özgün sahipler için kapanış/reaped oracleları kendi kapsamındadır.

**Tarihsel kontrollü hazırlık:** Proposal3 dört harness/test kaynağı bağımsız static incelemede doğrulandı: korumalı handler kurulumu/restore, sticky iptal/ilk hata, bağımsız cleanup ve publication, özgün foreground PG/readiness adapter’ı. Mevcut11native vaka/32require assertion/35s heartbeat döngüsü ve TLSrequire/scopedCOMMIT ayrımı korundu. Bu kaynak/test/OPEN belge dilimi aynı atomik uygulama kapsamındadır; tam kaynak veya native kabul sonucu değildir. Gerçek standalone preflight44 (genericPG18+job-specific26) PASS/exit0/reaped, unittest0.331sn; yalnız inert sahiplik/cleanup/iptal/publication modelleridir, gerçek PG/native11 değildir. Değişmeyen mask baseline2/.099sn ayrı tarihsel kabul olarak korunur. Gerçek fixture2 combined ordinary3 PASS/exit0/reaped, unittest0.511sn: eski mask2+tek nested44 wrapper. İlk fixture1 koşusu3test/2maskfailure/exit1/reaped kaldı; root üç public production import bağımlılığını fixture’a ekledi, Proposal3 baytları değişmedi. Bu eksik fixture hazırlığı kaynak/native kusuru veya OS/PG sızıntısı değildir.44/3 sayıları toplanıp Cargo/native vaka yapılmaz. Exact Proposal3 `/private/tmp/zee-k300/k309-job-proposal-3/PREPARATION.json` SHA256 `2096a3744a2989830b6833748b7382dfd9f290a77fcd793f0139a2df250e6cbe`; static peer `29eed668c09d318f258d78a167df9525c6cc50a9af41161465ac475d91ee3070`; actual controlled peer `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559`; fixture2 `a7c866f4914d02db9b17b4b118a66bee6e9d78c48ac4b16844c84c3b33b8e309`. Dört source pini: `scripts/tests/job-dispatcher-native-test.py`=`ef6a3ab8f990c3148a09d3234fe8b8075b9fd12083742059139df2599be5f49d`, `scripts/tests/job-dispatcher-mask-test.py`=`92357774ae9a09d55a366d844a33711564df5a924c237bdb9051135529a4b21a`, `scripts/tests/job-pg-owner-preflight-test.py`=`158af9ca7f2a97d4499c36c3af5ed5befc73380195da98bdf30d1a76b80d630f`, `compiler/tests/job_dispatcher_testi.rs`=`bda79d9307573127ee62eff6931b5dad363bbf4f2b6c3c73052d00c5f8d5a3d1`. Baseline/odaklı saf modeller/ordinary wrapper/native11 ayrı kanıt kapsamlarıdır; Python yöntemleri Cargo veya native vaka toplamına eklenmez.

Child-free exact AST fake probe.close karşı örneği tek beklenen failure/exit1/reaped olarak tarihsel kalır: cluster.close ve proof.write callback denemeleri atlanmıştı; gerçek OS/PG sızıntısı veya diskte kalmış PASS gözlemi değildi. Baseline mask2 ayrı kabul olarak korunur.

Mevcut11 case kimliği/32require/35s sabit handler, en az6bağımsız heartbeat ve eski30s süresinin aşılması korunur. CLI TLSrequire/ownedCA sınırı ile yalnız owned loopback host-component COMMIT fault fixture’ındaki plaintext profil ayrıdır; unknown COMMIT/sinyal/timeout replay/fail yazımı üretmez. Fixture20 ve privacy-safe public tool kaydı sırası/tam envanter kontrolü ayrı exact source/native sözleşmesine bağlıdır; PHC gizliliği gerekçesiyle gizli/atlanan komut boşluğu veya ham credential verisi kabul edilmez.

Yalnız bu exact kaynakta mevcut tek ignored Cargo sahibi/job11 sonucu kullanılabilir; kalan24 ignored Cargo vakası bu kampanyada çalışmış sayılmaz. K306 dispatcher4/HTTP7, K307 SQL9, K308 starter14 ve tarihsel job11 sonuçları kendi kaynaklarında kalır.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-gates-1/PROOF.json` | `0391ffe6ed48eb5cfa4bb87cb4eb3520a4cdb0b4fdb40bfc43cdeacf95a71d3f` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-gates-controller-1.json` | `2b719cbf8871dd4adbaf6f68256718b985843334b7368ea6ee277b8159aeeec6` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-commit-1.json` | `d6a0fc6a1dbf6011901434eedc7f375e470bf5ecb5033f5222f2cd558f39d8b9` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-source-commit-controller-1.json` | `9b85a6a205d8aa03d5b566b0f787fdb5365dda87bd9f42819a5c0b626e932614` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-1/PROOF.json` | `c1a13da0dee5cdd49b1924ae5f85d47fca56c59bed26eac5a884b68e53a01e44` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-controller-1.json` | `39b250307822f60c7a22a030862201e541fc3fec9ec3e66e45e861fbdf00cc4f` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-native-job11-1/native/proof.json` | `4892c0280317d206eac97ad2abe1491ecaa05cdd2bfc8875fe2bffcecaf387a0` |
| root_native_audit | `/private/tmp/zee-k300/K309-NATIVE-ROOT-AUDIT-1.json` | `4cfeb107b94d7cc62b877c74cffd622f13624d67795dda45bae4db95e112efe5` |
| native_peer_review | `/private/tmp/zee-k300/K309-NATIVE-PEER-REVIEW-1.json` | `f03d38cccc6bdb4e3b6446ef7818a03c484bc6ad9045a77da0d7f90a9b5b2dd1` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k309-frozen-acceptance-1.json` | `957b1bc09c54be91331d5eab10233f3fd3b410564e17a293cc3d4ab268a46714` |
| source_pins | `/private/tmp/zee-k300/k309-source-pins.json` | `e772138605af3354e93791740260897bad62a8d0ce13c7790101f2674eaf5f0c` |
| closure_template | `/private/tmp/zee-k300/K309-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `534d93dc60ae2739b4746ff1a08a3c7c800f9d32c8056438890ea1f8367dbe43` |
| closure_template_review | `/private/tmp/zee-k300/K309-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `72feb8424e1a7dac1a316585264da93cf8cb15f93fe424e2e0291b94b3fe34c2` |
| K309-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K309-ATOMIC-APPLICATION-ACTUAL-INDEPENDENT-REVIEW-1.json` | `eea7ebd5b15cee1dea16846f2ffacc5a7220e5d56a3bd817c75e18a29e670653` |
| K309-JOB-PROPOSAL3-ACTUAL-MODELS-INDEPENDENT-REVIEW-2.json | `/private/tmp/zee-k300/K309-JOB-PROPOSAL3-ACTUAL-MODELS-INDEPENDENT-REVIEW-2.json` | `22ee3e73afa325c3aa2b5caacad725ac32d5f859ef42b6cf0c70029296268559` |
| K309-NATIVE-JOB11-DRIVER-INDEPENDENT-STATIC-REVIEW-3.json | `/private/tmp/zee-k300/K309-NATIVE-JOB11-DRIVER-INDEPENDENT-STATIC-REVIEW-3.json` | `7af1483c3d4a857c4d021d791f8ab6c50bfb867edc0fb4776e307a32ae7fd985` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K309-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `9a9f30bd151ae85feafb77688d656a56ed6cc656fd992c06018a6554f1926dcf` |

Bu yalnız job diliminin dar kapanışıdır; metrics/effect iki PG tüketicisi açık kalır.39plan statüsü korunur; P09/P13/P20/P23/P36 kısmi/P37sürekli ve ürün/adoption/provider/fiziksel platform/dış proje/RCsoakIDE/C1K288P33K293 ret sınırları değişmez.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans ve compiler/LSP kalıcı durum değişmez. Ürün komutları/göç yolu değişmediğinden `docs/job-dispatcher.md`, ADR-114/126/139 ve mevcut kaynak-pinli rehberler bilinçli unchanged tutulur; ADR-117 dar harness güvenlik sınırını ve mevcut SQL-only pilot durumunu taşır. PHC/parola/cookie/request/cert/key/DSN/credential özel baytları veya hash’leri public kayda alınmaz; private runtime veya eski PID/kök aranmaz ve benimsenmez.

## K-308 — starter harness iptali ve özgün PG sahibi: dar kapsamda kapalı

K-308 / GB-117 dar kapsamda kapalı: Starter kabul harness sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e tam15/temiz commit makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/starter14 frozen9 ve özgün PG/readiness/host/dış owner kabulü doğrulandı. Diğer üç PG tüketicisi açık; P23 kısmi ve 39 plan statüsü kendi mevcut değerlerinde korunur.

**Gerçek kaynak kabulü:** Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e, tree b7f024e38f43bfb92714aa8db210ccc32bf79d49; tam15 kaynak kapısı 1146.264sn ve temiz tested-source commit/özgün owners doğrulandı. Faz998/1023 geçti,0 başarısız,25ignored kendi tam kaynak koşusudur. İlk app1 değişiklik/makbuz öncesi manifest-local-shadow TypeError/exit1/reaped olarak korundu; üç localname düzeltmesinden sonra app2 source4+OPENdocs11 gerçek uygulandı. FakeFD16, native sürücü pure18 ve integration35 ayrı hazırlık sonuçlarıdır; native/Cargo toplamına eklenmez.

**Gerçek native14 kabulü:** Yalnız bu exact kaynakta canonical tek ignored Cargo sahibi/mevcut starter14 geçti; native sürücü 67.326sn. Kaynak346/production175, özgün native14 oracleları, typed release build ve exact tools/runtime pinleri frozen9/root/peer public raw kanıtlarıyla doğrulandı; kalan24 ignored Cargo vakası bu kampanyada çalışmadı.

PostgreSQL özgün foreground Popen normal SIGINT/observed_exit=exit=0/reaped/noFallback, readiness özgün probe kapanışı ve ayrı host front_terminal exit0/reaped/stderr0 doğrulandı; private runtime kaldırıldı. Ayrı dış original owner0/reaped/errors[]/cancelled[]; iç nativechild normal0 bağımsız makbuzu iddia edilmez. Host’un iç native child cleanup’ı mevcut kaynakta H.stop_owned sınırıdır; bağımsız per-worker normal0 public makbuzu veya yeni worker hook kabulü iddia edilmez. Mevcut starter14 SQL/HTTPS/kimlik/medya/pending oracleları korunur. Tarihsel S3 native14, K307 SQL9 ve K306 dispatcher4/HTTP7 kendi exact kaynaklarında kalır. Yalnız bu yeni exact kaynakta bir ignored Cargo sahibi/starter14 native kabulü kullanılabilir; kalan24 ignored Cargo vakası bu kampanyada çalışmış sayılmaz.

**Tarihsel kontrollü kanıt:** Proposal5 dört test-harness kaynak dosyası bağımsız statik incelemede doğrulandı: sticky ledger/korumalı handler/publication, foreground StarterPgCampaign ve private-root cleanup; native14 kayıt adları/eski assertion çokluğu/özgün host Popen ve ordinary8 korundu. Bu kaynak/test/belge dilimi aynı atomik uygulama kapsamındadır; temiz kaynak veya native kabul sonucu değildir. Gerçek P5 focused18 PASS/exit0/reaped, unittest0.153sn; ayrı PG preflight23 PASS/exit0/reaped (0.224sn). PG23 helper/consumer/preflight baytları P4=P5 olduğundan özgün ilk makbuz korunur, tekrar koşulmuş sayılmaz. İlk focused18 ve combined27 koşuları birer selector TypeError/exit1/reaped olarak tarihsel kalır; bu model seçimi hatası native/runtime/OS sızıntısı değildir. Gerçek P5 fixture2 birleşik ordinary27 PASS/exit0/reaped, unittest0.499sn; yapı old8+focused18+tek nested PG23 wrapper’dır. Public fixture10 kaynak kopyası exact doğrulandı. Üç kabul özgün dış owner errors[]/cancelled[] makbuzlarına bağlıdır;18/23/27 sonuçları toplanıp Cargo/native vaka veya mevcut14’ün yeniden kabulü sayılmaz. Exact Proposal5 `/private/tmp/zee-k300/k308-starter-proposal-5/PREPARATION.json` SHA-256 `cddda1f8bbcd3973fb045fa974c7a6aa927f3e485f2416d2882272f29b585b33`; bağımsız static peer `75b005a3e0e01ad4a073970a49643fc2b66e8f2cce970669cc33448bb48522f8`; actual-results peer `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f`; selector düzeltme kaydı `6a9511ec68a6e9f8f702bb7bb865e443cdf22212e2c99a60ebe3252101c68589`. Dört source pini: `scripts/tests/starter-content-native.py`=`3dcb1da978a82ec9e2710d0d6de5e432a5c5fc2f4ce385c0ad79736fe3cbab50`, `scripts/tests/starter-content-test.py`=`93e7e23625eaf6a2732bcc99bdd86506e5e56a4d4238ba332d242e672cf1c80a`, `scripts/tests/starter-pg-owner-preflight-test.py`=`fd8c3d2b7e7c40feed710c93e410e3ce64f97535cf9a4213f43785acd70e4abe`, `compiler/tests/starter_content_testi.rs`=`d3471b0d129816c1e9c941409796172bb156a03fb905f53f979e9bb8d760d9d4`. Bu kayıtlar actual kaynak/test baytları ve bağımsız root/peer inceleme/özgün owner makbuzlarıyla bağlıdır. Ordinary baseline8, odaklı inert yöntemler ve birleşik ordinary wrapper sonucu ayrı kapsamdır; model sayıları Cargo/native sayısına eklenmez.

Shared `native-pg-owner.py` exact6e5f korunur. `ZEE_STARTER_NATIVE_OUT` yalnız internal ignored testin taze mutlak output seçimi olup ürün CLI/API kapasitesi açmaz;0700 geçici-root varsayılanı korunur.

PHC/parola/cookie/request/cert/key/DSN özel baytları veya bunların hash’leri public kanıta yazılmaz. Public proof/progress/loglar yalnız izinli yaşam döngüsü/sonuç metadata’sıdır; eski private runtime/kök/PID/credential aranmaz veya benimsenmez. Mevcut route/PHC/session/CSRF/media güvenlik oracleları dar test-harness düzeltmesiyle gevşetilmez.

**Kapanış önkoşulu:** Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler.

| Gerçek public kayıt | Konum | SHA-256 |
| --- | --- | --- |
| source_gate | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-gates-1/PROOF.json` | `4ce1698419f7d3aecdc2e13cbfdefaa159689e2d42e3acd79cdafae2e18471d8` |
| source_gate_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-gates-controller-1.json` | `a6f7950a3753c76839caaa818a9c9bcdda06101f65c3d33929ae89d9af732bb7` |
| source_commit | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-commit-1.json` | `d68bd9fd28a9bacafcdd5955b8c292ec99b3f0bc8b83c632de44c4c1e59abe29` |
| source_commit_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-source-commit-controller-1.json` | `df9be65ff3cd523f08a93617a973c679f1a4746be8e8489146951eb9b681bb97` |
| native_driver_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-1/PROOF.json` | `feb09e70e89b9946e30331c7ae3b044da0c77f9c5f48ce05c1ff0c18b376d558` |
| native_driver_owner | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-controller-1.json` | `08471af35cade3240e94f09028f37ea98b7f4389dc29afc973b77ca49fa41138` |
| native_inner_proof | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-native-starter14-1/native/proof.json` | `1b67d601d6f3fa2eb54c78a89c6041b2e7f8cf1ee5d5ac0629ebc7bbba1ca8d9` |
| root_native_audit | `/private/tmp/zee-k300/K308-NATIVE-ROOT-AUDIT-1.json` | `0fe921fc2c1994bae56773a1185a4fa6ca961a966ca502afc07e9de46d183b7b` |
| native_peer_review | `/private/tmp/zee-k300/K308-NATIVE-PEER-REVIEW-1.json` | `2acfbd8dba35ea13371407b288078f3df5c3277f33c83d11119b2dab2852c856` |
| frozen_acceptance | `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/k308-frozen-acceptance-1.json` | `9e7538a35c86a142bfe7de6892a490994c748a908e9ba3094967ba0ca4b5a1a4` |
| source_pins | `/private/tmp/zee-k300/k308-source-pins.json` | `2808178e2ebe9af8af556a7b3fc685d3005d23dd0eab763126dedeff2e762180` |
| closure_template | `/private/tmp/zee-k300/K308-CLOSURE-FINAL-SOURCE-TEMPLATE-1.json` | `e29d24881a61b806bb46eb16456d928c6cedaa9a293251fee8526e99d3595c08` |
| closure_template_review | `/private/tmp/zee-k300/K308-CLOSURE-TEMPLATE-SOURCE-BINDING-INDEPENDENT-REVIEW-1.json` | `e97aca7c28c26b83623c12b714d575b7e5f681c465762982bbed39ff1d67cca8` |
| K308-ATOMIC-APPLICATION2-ACTUAL-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-ATOMIC-APPLICATION2-ACTUAL-INDEPENDENT-REVIEW-1.json` | `f4668a08a0614528d8add1ed04570f76d7864b3ec82d3f6acd49f0f14306a2e1` |
| K308-CONTROLLED-RESULTS-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-CONTROLLED-RESULTS-INDEPENDENT-REVIEW-1.json` | `836b53ceb61ec95286b37e358c7d94de9ebf3be754e8ddd24ef73902c260aa4f` |
| K308-ATOMIC-LOCAL-SHADOW-CORRECTION-INDEPENDENT-REVIEW-1.json | `/private/tmp/zee-k300/K308-ATOMIC-LOCAL-SHADOW-CORRECTION-INDEPENDENT-REVIEW-1.json` | `81641b09dd35f20c1017cadadea211abac7a6a06402e026cd820a029c706e784` |
| native_driver_preparation_review | `/private/tmp/zee-k300/K308-NATIVE-DRIVER-ACTUAL-INERT-MODELS-INDEPENDENT-REVIEW-1.json` | `94dba2a083f1f565de1ce69c83a683ec0d5027466be7faaa74b25628e0e7a328` |

Diğer üç PG tüketicisi, P20/P23 kısmi ve diğer açık ürün/insan/platform/dış girdiler, RC/soak/IDE ve C1/K288/P33/K293 ret sınırları korunur; 39 plan statüsü değişmez. Yeni compiler özelliği yoktur.

Bilinçli etki incelemesi: production Rust175, compiler grammar/API/ABI/dependency/katman, spec normatif madde/RFC, kullanıcı tanısı/katalog, sabit performans iş yükü ve compiler/LSP kalıcı durum değişmez. Native tüketilen `docs/versioned/0.8/icerik-paneli.md` ürün komutları değişmediğinden bilinçli unchanged tutulur; kapanışta source-pinli rehber baytları değiştirilerek native kanıtı bayatlatılmaz. ADR-148 ve bu dar durum kayıtları atomik belge kapsamını taşır.

## 17 Ekim 2026 — kullanıcı kapılarını atlayarak teknik devam

K-312 birleşik UTF-16/NFC/morfoloji ordinary testlerini ekler; P19 genel
IDE/ürün/platform kabulü kapanmaz. İnsan, lisans, fiziksel cihaz, destek
teyidi ve freeze takvimi atlandıktan sonra da açık kalır. ürün sunucusunda mevcut
canlı ürünlerin kabul kaydı [teknik kapanışta](v1-teknik-kapanis-20261017.md)
izlenir. Bu çalışma V1 etiketi değildir.

K-313 aynı fiziksel PostgreSQL backend'inde tenant bağlamının COMMIT ve
ROLLBACK sonrası temizliğini iki yeni native testle korur. Koşucu on exact
TLS vakası ve 15 kontrol modeli ister; yeni grammar/production davranışı
veya gerçek RLS/ürün kabulü eklenmez.
