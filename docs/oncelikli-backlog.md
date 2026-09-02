# zee — V1 öncesi öncelikli mühendislik backlog'u

Bu belge 1 Eylül 2026 tarihli ayrıntılı dış incelemenin depo içindeki kalıcı,
sıralı iş karşılığıdır. Yeni dil özelliği P0 omurga işleri kapanmadan öne
alınmaz. Bir madde yalnız kodla değil; gerekiyorsa RFC/ADR, normatif spec,
olumlu/olumsuz test, sürüm notu ve v1 kapısı birlikte güncellendiğinde kapanır
(ADR-010 ve kök [AGENTS.md](../AGENTS.md)).

Durumlar: **SIRADA** · **AÇIK** · **KISMEN** · **KAPALI**.

## Uygulama sırası

1. Tamamlanan önkoşul: K-095 registry metadata güveni (378 test).
2. İnsan kanıtı bekleyen kapılar: B-001/K-096 + B-002/K-093.
3. Tamamlanan compiler omurgası: B-003/K-097, B-004/K-098, B-005/K-099,
   B-006/K-100, B-010/K-101, B-018/K-102 ve B-019/K-103–K-104 (413 test).
4. Güvenlik incelemesi: K-105 ağ deadline/bellek, K-106 process içi oturum
   kotası ve K-107 LSP tek-girdi sınırını kapattı (426 test).
5. K-108/ADR-020 her semantic HIR düğümünde kaynak aralığını zorunlu yaptı;
   B-020 kapandı (427 test).
6. K-109/ADR-021 production panic audit'inde 46 noktayı temizleyip dört crate
   köküne kalıcı lint kapısı koydu; B-014 kapandı (429 test).
7. K-110/ADR-022 lexer/parser'a kalıcı saldırı korpusu, deterministik UTF-8
   üretimi ve gecelik libFuzzer hattı kurdu; B-015 kapandı (432 test).
8. K-111 morfoloji üret→çöz uzayını, bütün-aday A002 kararını, NFC/NFD
   sınırını ve gecelik mutation hattını büyüttü; B-016 kapandı (435 test).
9. K-112/ADR-023 parser AST ve bağlı typed-HIR arasındaki yürütülebilir
   invariant kapısını kurdu; B-017 kapandı (440 test).
10. K-113/ADR-024 parser kurtarmasını cümle+dengeli girinti sınırlarına,
    çoklu tanıyı kaynak sırası+20 kayıt bütçesine bağladı; B-021 kapandı
    (447 test).
11. K-114/ADR-025 tanı kodu↔anlam bağını 148 kayıtlı sürüm fixture'ı ve üç
    yeniden kullanılamaz mezar taşıyla sabitledi; B-022 kapandı (448 test).
12. K-115/RFC-0022/ADR-026 bütün runtime IO çağrılarını sürümlü kanonik
    trace/replay protokolüne ve `dil iz kaydet/oynat` CLI'ına bağladı; B-027
    kapandı (455 test).
13. K-116/RFC-0023/ADR-027 tohum→dizi algoritmasını, tam i64 aralığını,
    sanal saati ve hermetik adaptör gözlemlerini `zee-io-1` profiline bağladı;
    B-028 kapandı (460 test).
14. K-117/ADR-028 `.zep` yollarını NFC'ye kanonikledi; Tier-1 işletim sistemi
    fixture'ı ve 80 vakalık kalıcı saldırı korpusuyla B-030/B-031 kapandı
    (463 test).
15. K-118/ADR-010 revizyonu 71 RFC/ADR/spec belgesini test dosyalarına bağladı;
    README canlı sayılarını depo ağacından üreten araçla B-043/B-044 kapandı
    (465 test).
16. K-119 tam parser token izini 33 golden programın dağınık-boşluk
    varyantında eşitleyip iki tarafı ayrıştırarak B-042'yi kapattı (467 test).
17. K-120 definition/rename'i `SymbolId`/`IslemId`/`YapiId` typed-HIR
    dizinlerine geçirerek B-041'i kapattı (475 test).
18. K-121 iki fazlı yerel çağrı kısıtı birleşimiyle B-007'yi kapattı
    (481 test).
19. K-122 `zee-tr-1` davranışını SHA-256 semantic kayıt + Git-geçmişli CI
    koruğuyla immutable yaptı; B-008 kapandı (482 test).
20. K-123 kök `conformance/` alanındaki JSON Schema, 27 çözüm/karar ve 21
    üretim vakasıyla B-009'u kapattı (483 test).
21. K-124 `zee-esz-1` JSON Schema ve 10 kaynak+gözlem vakasıyla B-011'i
    kapattı (484 test).
22. K-125/ADR-029 örtük Ondalık↔binary float FFI eşlemesini yasaklayıp
    B-012'yi kapattı (485 test).
23. K-126/ADR-030 bütün yaprak ve bileşik AST ifadelerine kesin kaynak zarfı
    yayıp HIR, tanı ve LSP tüketimini bağladı; B-050 kapandı (490 test).
24. K-127/RFC-0024/ADR-031 proje+paket yetkinliklerini compile/runtime
    kapısına, native istemciyi rustls HTTPS + origin/DNS/IP/redirect
    korkuluklarına bağladı; B-023/B-049 kapandı (501 test).
25. K-128/ADR-032 atomik replace'i Linux/macOS mode+uid+gid+ACL/xattr ve
    Windows DACL/security/named-stream korumasına bağladı; taşınamayan metadata
    fail-closed kaldı ve B-048 kapandı (506 test).
26. K-129/RFC-0025/ADR-033 ortak `KaynakSinirlari` profilini kurdu; kaynak,
    token, proje toplamı, çalışma adımı, çıktı, koleksiyon, görev, dosya okuma
    ve LSP toplam bellek/yanıt dilimini merkezîleştirdi. B-025 heap-byte ve
    bağlantı muhasebesi için kısmen açıktır (513 test).
27. K-130 aynı profile 16 MiB bütçeli metin üretimini, yaklaşık 64 MiB
    muhafazakâr saklanan değer zarfını, görev ortamı klonlarını ve 64
    süreç-geneli inbound+outbound bağlantı iznini bağladı. B-025 eski domain
    sabitlerinin tek tipe göçü için kısmen açıktır (517 test).
28. K-131 HTTP/ağ, web oturumu, IO izi, LSP, paket/registry, tanı, atomik
    metadata ve kilit sürelerinin eski yerel sayılarını davranış değiştirmeden
    ortak profilin domain görünümlerine taşıdı. Mimari sahiplik testiyle B-025
    yalnız LSP outbound yanıtının sonuç büyümeden bütçelenmesi için kısmen
    açıktır (518 test).
29. K-132 bütün LSP outbound gövdelerini tahsis sırasında 8 MiB'ta kesen
    sınırlı JSON yazıcısına taşıdı. Rename düzenlemeleri ve diagnostics tek tek
    akar; taşma kimlikli istekte `-32001`, bildirimde sınırlı `logMessage`
    üretir. Geçerli JSON ve mimari bütçe regresyonlarıyla B-025/V1-P0-31
    kapandı (521 test).
30. K-133 B-026'nın ilk audit dilimini kapattı: çıktı, girdi, dosya, web,
    oturum, eyleyici ve rastgelelik etkilerinin hemen önünde son tarihi yeniden
    denetler. Görev HTTP öncesi sıra verdikten sonra kalan süreyi yeniden
    hesaplar; dolmuş istek adaptöre hiç girmez (523 test).
31. Sıradaki makine işi K-134 ile web istek yaşam döngüsünü transaction'a
    bağlamaktır; sonraki işler aşağıdaki öncelik ve bağımlılık sırasını korur.

## P0 — V1 öncesi dil ve derleyici omurgası

- **B-001 · KISMEN (K-096 deney hazır) — K-016 işlem çağrısı sözdizimini
  kesinleştir.** Serbest üretim, kör A/B/C kartları, dengeli sıra, anonim form
  ve alt grup+teknik karar eşikleri
  [karar paketinde](k016-cagri-karar-paketi.md) önden bağlandı. Gerçek 10 çocuk
  + 5 profesyonel verisi gelmeden tek yüzey seçilmiş veya iş kapanmış sayılmaz.
- **B-002 · AÇIK — K-093 gezme zihinsel modelini kullanıcıyla doğrula.**
  Değer-sonuç imleci makine tarafında tamamdır; V1-P1-05 için 10 öğrenci +
  5 profesyonel eşiği [usability kitinde](usability-kiti.md) bekler.
- **B-003 · KAPALI (K-097) — expression grammar büyüme mimarisini
  kararlaştır.** Primary → erişim/postfix → çağrı → aritmetik → birleştirme →
  karşılaştırma → boolean sırası, tam bölge tüketimi ve yeni yüzey uzatma
  protokolü RFC-0021/spec-20/ADR-002'de bağlandı. Sekiz bağımsız conformance
  testi güçlü birleşimleri, işlem-adı kuyruğunun postfix'i gölgelememesini,
  tam sıfır-argüman çağrısını, en uzun çağrıyı ve fail-closed sınırları korur;
  fiziksel fonksiyon/modül parçalama B-005/K-099'da davranış korunarak bitti.
- **B-004 · KAPALI (K-098) — domain özelliklerini core AST'den ayır.**
  HTTP, sensör, CSRF ve parola yüzeyleri kaynak yazımı değişmeden tek generic
  `Intrinsic { kimlik, argumanlar }` düğümüne indirildi. Tür imzası,
  yetkinlik ve etki ADR-011'deki merkezi kayıtta birleşti; kapalı sensör koşulu
  genel olumsuzlamayı kullanır. Yedi lowering/imza testi ve mevcut davranış
  korpusuyla V1-P0-09 kapandı. Fiziksel handler ayrımı B-005/K-099'da
  tamamlandı; proje/paket izin politikası B-023/K-127 ile kapandı.
- **B-005 · KAPALI (K-099) — mega fonksiyon büyümesini durdur.** Parser
  cümle/ifade; checker cümle/ifade/çağrı; runtime cümle/ifade handler'larına
  ayrıldı. Kök dosyalar sırasıyla 2709→1160, 3067→963 ve 3181→1965 satıra
  indi. ADR-012 ve [faz rehberi](derleyici-faz-sinirlari.md) sahipliği bağlar;
  üç mimari test handler'ların köke dönmesini ve ilanlı satır bütçelerinin
  sessizce aşılmasını engeller. Kullanıcı yüzeyi ve davranış değişmedi.
- **B-006 · KAPALI (K-100) — type checker'ı katmanlaştır.** Checker kökü
  yalnız 143 satırlık geçiş orkestrasyonu ve public yeniden dışa aktarım
  taşır. Türler, bağlam, sembol, akış, çağrı, public sözleşme,
  etki/yetkinlik ve dönüş/control-flow tek sahipli modüllere ayrıldı. Eski
  `eylem.rs` davranışı checker'ın `etki` katmanına alındı; public
  `Tur`/`VeriTuru`/`SozlukDegerTuru`/`ad_cozumle` API'si korundu. ADR-013,
  [katman rehberi](checker-katmanlari.md) ve beş mimari test yeniden birleşmeyi
  durdurur; kaynak semantiği değişmedi ve V1-P0-11 kapandı.
- **B-007 · KAPALI (K-121) — yerel çağrı kaynaklı inference'ı sıra bağımsız yap.**
  Checker kopya AST'de erişilebilir ana/test çağrılarını keşfeder; her
  `IslemId` ve parametre konumu için eşit/sayısal-kapsayıcı kısıtları
  birleştirir. Asıl AST, tanı ve typed HIR yalnız nihai imzayla bir kez
  üretilir. Skaler, liste, çok parametre ve iç içe çağrı grafiğinin iki
  sırası, iç blok kurtarması ve birleşmeyen T017 çifti altı regresyonda
  sabittir. Ayrıntı
  [çağrı çıkarımı rehberindedir](cagri-cikarimi.md).
- **B-008 · KAPALI (K-122) — `zee-tr-1` profilini immutable koru.** Profil
  dökümü, 11 ham sınır yüzeyi ve 4.096 kökün 53.248 üretim+çözüm vektörü
  kanonik SHA-256 kaydına iner. Davranış değişirse test, yayımlanmış fixture
  güncellenir/silinirse Git tabanlı CI koruğu kırılır. Yeni davranış yalnız
  yeni `zee-tr-N` kimliği ve yeni fixture ile eklenebilir. Bakım protokolü
  [profil uyumluluk rehberindedir](morfoloji-profil-uyumlulugu.md).
- **B-009 · KAPALI (K-123) — morfoloji conformance korpusunu compiler'dan
  bağımsızlaştır.** Depo kökündeki sürümlü JSON Schema ve `zee-tr-1.json`;
  profil/ek tablosunu, 27 yüzey→sıralı kök+ek→kapsam kararı ile 21
  kök+ek→kanonik/geçersiz üretimi taşır. Rust testi veriyi yalnız tüketir;
  ikinci compiler iç adlara bağlı değildir. Korpus+şema K-122 tarih koruğunda
  immutable'dır. Ayrıntı [conformance rehberindedir](morfoloji-conformance.md).
- **B-010 · KAPALI (K-101) — semantic ID modelini kur.** `YapiId`, `IslemId`
  ve `SymbolId` newtype'ları eklendi. `Tur::Yapi` artık depolama indeksi değil
  kimlik taşır; yapı erişimi ayrı kimlik→konum dizinindedir. İşlem imzaları ve
  özyineleme yığını `IslemId`, sembol tablosu ad→(`SymbolId`, tür) kullanır.
  Checker `Degisken`/`YeniYapi`/`IslemCagrisi` bağlarını AST'ye yazar. ADR-014,
  [rehber](semantic-kimlik-modeli.md), üç davranış ve bir mimari testle
  V1-P0-12 kapandı. Faz tipleri B-018/K-102, typed HIR ve bağlı runtime tüketimi
  B-019/K-103–K-104 ile tamamlandı.
- **B-011 · KAPALI (K-124) — gözlenebilir concurrency determinizmini V1
  garantisi yap.** `zee-esz-1` profili kaynak sıralı poll/uyanış, ortam
  snapshot'ı, örtüşen sanal süre, sonuç bağlama, iç görev ağacı, ortak IO,
  hata/son tarih/çıkış iptali ve atomik eylemi 10 derleyiciden bağımsız
  kaynak+gözlem vakasına bağlar. Gelecekte çok çekirdekli runtime yalnız çıktı,
  süre, dosya etkisi ve sonlanma kodunu birebir koruyan iç optimizasyon olabilir.
  JSON Schema+korpus genel Git-tarih conformance koruğunda immutable'dır;
  ayrıntı [eşzamanlılık conformance rehberindedir](eszamanlilik-conformance.md).
- **B-012 · KAPALI (K-125/ADR-029) — Ondalık↔binary float dönüşümünü yalnız
  açık ve kayıplı yap.** Eski `GerçekSayı↔double` taslak eşlemesi kaldırıldı.
  `Tur`/`Deger` binary float taşımaz; Stage 0 FFI yüzeyi sunmaz. Gelecekteki
  `float`/`double` köprüsü deklarasyon+çağrıda görünür `kayıplı` işareti,
  `Sonuç` ve açık IEEE 754 yuvarlama/özel-değer sözleşmesi olmadan eklenemez.
  Mimari regresyon exact Ondalık değerini ve RFC/ADR tazeliğini korur;
  V1-P0-28 kapandı (485 test).

## P1 — Compiler sağlamlığı

- **B-013 · KAPALI (K-096 hazırlığı) — stale v0 parser yorumlarını temizle.**
  `Ayristirici::islem_adlari` artık dosya/birim başlıklarının ön-tarandığını,
  tanım sırasından bağımsız çağrı ve karşılıklı özyinelemeyi doğru açıklar;
  tarihsel karar günlüğü eski davranışı açıkça tarihsel diye korur.
- **B-014 · KAPALI (K-109) — production `unwrap/expect` audit'i.** 46
  production `unwrap`/`expect`/`panic!`/`unreachable!` noktası kaldırıldı.
  Kullanıcı/bozuk AST/IO durumları T016, C000, `Result` veya açık CLI hatasına
  iner; scheduler process'i düşürmez. `lib`, `dil`, `dillsp` ve `olcum`
  crate'leri test dışı derlemede bu dört kalıbın yanında `todo!` ve
  `unimplemented!`ı da Clippy `deny` ile reddeder. `SymbolId`nin iki 32-bit
  kapasite assertion'ı ayrı `usize` bileşenlerle kaldırıldı. ADR-021,
  [uygulama rehberi](production-panic-politikasi.md) ve iki regresyonla
  V1-P0-19 kapandı; toplam 429 test yeşildir.
- **B-015 · KAPALI (K-110) — lexer/parser fuzzing.** `&str` libFuzzer hedefi
  başarılı lexer çıktısını hem normal hem hata-kurtarmalı parser'dan geçirir.
  Sekiz başlangıç girdisi ve mutation sözlüğü Unicode/homoglyph, emoji,
  combining im, CRLF, girinti, metin kaçışı, dev sayı, virgül ve blokları
  kapsar. Her ana testte korpusun yanında 4.096 deterministik UTF-8 bileşimi
  ve 64 KiB uç girdiler yürür. Sabit nightly+cargo-fuzz gece işi korpusu cache
  ile büyütür, crash girdisini artifact yapar. İlk yerel smoke 1.048.287
  girdiyi crash/panic olmadan tamamladı. ADR-022, [fuzz rehberi](fuzzing.md)
  ve üç regresyonla V1-P0-20 kapandı; toplam 432 test yeşildir.
- **B-016 · KAPALI (K-111) — morfoloji property/fuzz testini büyüt.** Profilin
  bütün geçerli tek/iki katman zincirleri 4.096 deterministik kökte üret→çöz
  değişmezini korur. Üretilmiş 2.048 yüzey bütün adayları kapsama alınarak
  sınandı; çoklu aday daima A002'dir. `ğ/ö/ş/â/İ` NFC biçimleri kabul, NFD
  ayrıştırmaları S029'dur. Byte girdiden geçerli kök üreten ayrı libFuzzer
  hedefi gecelik korpus büyütür; ilk smoke 527.966 girdiyi ihlalsiz tamamladı.
  [Doğrulama rehberi](morfoloji-dogrulama.md) ve üç yeni regresyonla
  V1-P0-21 kapandı; toplam 435 test yeşildir.
- **B-017 · KAPALI (K-112) — AST/HIR invariant doğrulayıcı ekle.** Parser
  AST'sinde çözülmüş ad/semantic ID/checker işareti yasaktır. Bağlanmış
  programda her AST ifadesi tek ve benzersiz `HirDugumId`, açık tür, kaynak
  aralığı ve varyantla uyumlu semantic bağ taşır; HIR tablosunda yetim kayıt
  kalamaz. Debug/test faz geçişleri ihlali yapısal yol taşıyan C000'e çevirir.
  Özellik→alan dönüşümündeki klon kaynaklı gerçek bir yetim HIR kaydı bulunup
  alt kutuyu taşıyan güvenli dönüşümle düzeltildi. ADR-023,
  [invariant rehberi](ast-hir-invariantleri.md), üç integration ve iki unit
  regresyonuyla V1-P0-22 kapandı; toplam 440 test yeşildir.
- **B-018 · KAPALI (K-102) — compiler faz sınırlarını kodda görünür yap.**
  `KaynakMetni → TokenAkisi → AyristirilmisAst → BaglanmamisProgram →
  BaglanmisProgram → interpreter` hattı gerçek API türleri oldu. Standart
  denetle/çalıştır/dene yolu yalnız başarılı checker'ın ürettiği bağlı
  programdan geçer; raw `Program` yüzeyi v0 embedding adaptörüdür. ADR-015,
  [faz rehberi](derleyici-faz-modeli.md), iki davranış, bir mimari ve bir
  compile-fail testi V1-P0-13'ü kapattı. Resolution+tür bugün birleşik checker
  geçişidir; Typed HIR B-019/K-103–K-104 ile sonradan tamamlandı.
- **B-019 · KAPALI (K-103/K-104) — typed HIR tasarla.** Başarılı checker
  artık her denetlenmiş ifade için `HirDugumId`, açık `Tur` ve varsa
  `SymbolId`/`IslemId`/`YapiId` bağı üretir; `BaglanmisProgram` zorunlu
  `HirProgram` sahibidir. ADR-016, [HIR rehberi](typed-hir-modeli.md), iki
  üretim davranışı, iki kaynak-adı bozma regresyonu ve iki mimari testle faz
  sahipliği kanıtlandı. Standart çalıştırma ve `dene` hattı
  `CalistirmaProgrami::Hir` kullanır; değişken, işlem ve yapı kararları kaynak
  AST adından değil yalnız `HirBagi`ndan gelir. Raw `Program` yolu v0 embedding
  uyumluluğudur. V1-P0-14 kapandı.
- **B-020 · KAPALI (K-108) — her semantic node'da source span garanti et.**
  Her `HirIfadeBilgisi` kurucuda zorunlu `HirKaynakAraligi` alır; alan
  `Option` değildir ve `NonZeroUsize` bileşenleri sıfır/konumsuz kaydı
  engeller. Lexer konumu korunmuş değişken `Kesin { satir, sutun, uzunluk }`,
  diğer mevcut AST ifadeleri sahte sütun uydurmayan `Satir { satir }` zarfı
  taşır. ADR-020, kesin değişken + bileşik ifade davranış kanıtı ve mimari
  sahiplik testiyle V1-P0-18'in kaynak-kökeni dilimini kapattı; bu tarihsel
  satır-zarfı geçişi K-126/B-050 ile bütün ifadelerde kesinleştirildi.
- **B-021 · KAPALI (K-113) — LSP odaklı error recovery planı.** Hatalı cümle
  satır sonunda, yalnız ona ait alt ağaç dengeli girinti çıkışında
  senkronlanır. Sağlam kardeş aynı blokta kalır; parser derinliği sonraki üst
  tanıma sızmaz. Yapı alanı, `göre` kolu, eşzamanlı görev ve devam kolları
  sonraki geçerli satırı korur. CLI/LSP tanıları kaynak konumunda kararlı ve
  belge başına en çok 20 kayıttır. ADR-024,
  [kurtarma rehberi](parser-hata-kurtarma.md), altı parser ve bir LSP
  regresyonuyla V1-P0-23 kapandı; toplam 447 test yeşildir.
- **B-022 · KAPALI (K-114) — diagnostic code stability kapısını güçlendir.**
  Kaynak↔katalog birebirliğine ek olarak katalog↔sürüm fixture'ı 145 etkin ve
  3 ayrılmış kodun durumunu, aile uyumlu tekil semantik anahtarını ve kanonik
  anlamını korur. Anlam değişikliği yeni kod ister; emekli kod mezar taşı
  olarak kalır. ADR-025, [tanı kimliği rehberi](tani-kimligi.md) ve bağımsız
  fixture regresyonuyla V1-P0-24 kapandı; toplam 448 test yeşildir.
- **B-050 · KAPALI (K-126/ADR-030) — kesin source span'i bütün AST
  ifadelerine yay.** Parser'ın her yaprak ve bileşik ifadesi sıfır olamayan
  `AstKaynakAraligi` taşıyan tek bir `Ifade::Kaynakli` zarfındadır; çocuklar
  kendi token bölgelerini ayrıca korur. Checker aralığı HIR'a birebir aktarır,
  invariant eksik/iç içe zarfı ve AST↔HIR uyuşmazlığını reddeder. Eski `1:1`
  ifade tanıları gerçek düğüme yükselir; LSP işlem/yapı adını yalnız kesin
  semantic ifade içinde seçer. Örtük çoğul kaynak bile döngü adı tokenına
  bağlanır. Bileşik+yaprak, tanı, spansiz AST, örtük kaynak ve aynı yazımlı
  çağrı argümanı regresyonlarıyla toplam 490 test yeşildir.

## P1 — Runtime ve güvenlik

- **B-023 · KAPALI (K-127) — web/sensör/HTTP'yi capability modeline bağla.**
  Sekiz kararlı yetkinlik proje bildiriminden compile taramasına, genel
  `PolitikaliIo` runtime kapısına ve gerçek native adaptöre tek modelle akar.
  Testler ve çağrılmayan işlemler gizlenemez; paket ana projenin izin/hedef
  kümesini aşarsa P015, kaynak kullanımı izni aşarsa kesin aralıkta T054'tür.
  Proje dosyası kök/canonical symlink sınırı taşır. RFC-0024/spec-23 ve
  [rehber](yetkinlik-ve-ag-guvenligi.md) ile kapandı.
- **B-024 · KAPALI İLKE + GERÇEKLEME (K-127) — HTTPS/TLS'yi elle yazma.**
  Native outbound, exact sabitlenmiş `ureq 3.4.0` + rustls backend'indedir;
  Zee kriptografi/TLS gerçeklemeye dönüşmez.
- **B-025 · KAPALI (K-105/K-129/K-130/K-131/K-132) — ortak `KaynakSinirlari`
  modeli.** K-129
  değişmez tek profilde 8 MiB kaynak, 1 milyon token, 4.096/128 MiB proje
  kaynağı, mevcut C019/500 çağrı derinliği, 10 milyon çalışma adımı, 1 milyon
  koleksiyon öğesi, 1.024 görev, 16 MiB/100 bin çıktı olayı, 16 MiB dosya
  okuma ve LSP 256 belge/128 MiB/8 MiB outbound sınırını bağladı. S045/C023
  aşımı host panic veya sessiz truncate yerine kontrollü tanıdır. K-105'in ağ
  deadline/body zarfı korunur. K-130 tek metni 16 MiB'ta bütçeli üretir;
  ortam/koleksiyon yazımı ile görev klonunu iade edilmeyen yaklaşık 64 MiB
  saklama zarfına, inbound+outbound ağı 64 süreç-geneli RAII iznine bağlar.
  C024 değer/metin aşımının ayrı append-only kimliğidir. K-131 HTTP/ağ, web
  oturumu, IO izi, LSP, paket/registry, tanı, atomik metadata ve kilit
  sürelerindeki bütün eski sayısal sahipleri davranış değiştirmeden domain
  görünümlerine taşıdı. K-132 LSP'nin bütün outbound yollarını ortak 8 MiB
  bütçeli yazıcıdan üretir; JSON kaçışı, zarf, diagnostics ve rename
  düzenlemeleri her append öncesi ölçülür. Dev ara `Vec<String>`/`join` yoktur;
  aşım kimlikli istekte `-32001`, bildirimde bounded `logMessage` olur.
- **B-026 · KISMEN (K-133) — cancellation-safety audit'i.** Temp dosya,
  kilit, son tarih ve çalışma bütçesi nöbetçileri `Drop` ile sahipli temizlenir;
  eylem transaction'ı hata halinde rollback eder. K-133 çıktı/girdi, dosya,
  sunucu, yanıt/yönlendirme/çerez/oturum, eyleyici, CSRF, parola, rastgelelik
  ve eylem başlangıcına tam yan etki öncesi deadline kapısı koydu. Görev HTTP
  öncesi scheduler'a sıra verdikten sonra eski kalan süreyi kullanmaz; dolmuş
  dosya/HTTP etkisinin hiç başlamadığı sanal saat regresyonları vardır. Açık
  kalan dilim, bir web isteğinin oturum mutation'ı ile henüz gönderilmemiş
  yanıtını birlikte commit/rollback eden istek yaşam döngüsüdür.
- **B-027 · KAPALI (K-115) — deterministik IO trace/replay biçimi tasarla.**
  Bütün `GirdiCikti` çağrıları işlem, argüman, sonuç ve kesintisiz sırayla
  şema-1 kanonik izine girer. 64 MiB/100.000 olay/4.096 alan sınırı ve kapalı
  işlem şeması yürütme öncesi doğrulanır; replay dış etki uygulamaz, argüman
  farkı veya artan olayda fail-closed durur. `dil iz kaydet/oynat`, RFC-0022,
  ADR-026, [IO izi rehberi](io-izi.md) ve yedi regresyonla V1-P0-25 kapandı;
  toplam 455 test yeşildir.
- **B-028 · KAPALI (K-116) — saat/rastgele semantiğini sürümle.**
  `zee-io-1`; tohum karışımı+xorshift64* conformance vektörünü, yansız ve tam
  i64 uçları-dahil eşlemeyi, takvim/tekdüze saat ayrımını, sanal bekleme ve
  geriye-gitmeme kuralını, FIFO rastgele/girdi fallback'lerini ve görünür
  sahte dosya/ağ/sensör davranışını bağlar. RFC-0023, ADR-027,
  [spec/22](../spec/22-deterministik-io-profili.md),
  [profil rehberi](deterministik-io-profili.md) ve beş conformance testiyle
  V1-P0-26 kapandı; toplam 460 test yeşildir.
- **B-046 · KISMEN (K-106) — web oturum deposunu sınırlı ve ölçeklenebilir
  yap.** Process içi depo 4096 toplam/1024 anonim kotası, anonim LRU tahliyesi
  ve kaymayan mutlak 10/30 dakika ömür taşır. Yalnız kimlikli kayıtlarla dolu
  depo yeni girişi fail-closed reddeder. Per-IP/rate-limit ve atomik çok süreçli
  ortak depo hâlâ açık deployment dilimidir; mevcut profil tek process'tir.
- **B-047 · KAPALI (K-107) — LSP çerçeve ve JSON girdisini sertleştir.** Tek
  çerçeve 8 KiB başlık/8 MiB gövde; JSON 128 iç içelik/100 bin düğüm sınırı
  taşır. `Content-Length` tahsis öncesi ve tekil doğrulanır. Yanlış/eksik
  surrogate, tek düşük surrogate ve kaçışsız U+0000..U+001F reddedilir.
  ADR-019 ve yedi olumsuz/sınır testi V1-P0-17'yi kapattı. Toplam açık belge
  belleği ve outbound çıktı bütçesi K-129/B-025 ile ayrıca sınırlıdır.
- **B-048 · KAPALI (K-128) — atomik replace metadata sözleşmesini tamamla.**
  Normal dosyanın Linux/macOS mode+uid+gid'si korunur. Linux görünür xattr'ı
  (ACL/security label dâhil) kaynak bütçeli descriptor kopyasıyla, macOS
  ACL+xattr'ı `fcopyfile` ile taşır. Windows hata-yoksaymasız `ReplaceFileW`
  ve kurtarma yedeğiyle DACL/security resource/named stream'i korur. Symlink,
  taşınamayan metadata ve Tier-1 dışı Unix replace'i fail-closed'dur.
  RFC-0016/ADR-032/spec-08 ve platform-koşullu beş regresyon davranışı bağlar.
- **B-049 · KAPALI (K-127) — outbound ağ güven profilini kapat.** Exact
  şema+host+port allowlist'i, DNS sonrası bütün-IP kontrolü, varsayılan public
  HTTPS, ayrıca onaylı private/loopback+düz HTTP, her profilde kapalı metadata/
  link-local ve IANA public olmayan özel-kullanım/geçiş önekleri, sıfır
  redirect/proxy, 30 saniye ve 64 KiB+8 MiB zarfı tek istemcide uygulanır.
  RFC-0024/ADR-031/spec-23 ve loopback/redirect/SSRF/body-limit regresyonları
  davranışı bağlar.
- **B-051 · AÇIK — LSP JSON-RPC ayrıştırmasını protokol-kesin yap.** RFC 8259
  sayı durum makinesi, duplicate object-key reddi, parse error `-32700`,
  invalid request `-32600` ve finite olmayan sayının serializer'a çıkmaması
  ayrı differential/regresyon kanıtı ister.
- **B-052 · AÇIK — web proxy origin'ini tek kanonik tipe geçir.** CLI
  `GuvenliOrigin` ayrıştırması `AgHedefi` kadar sıkı DNS/IPv6/port semantiği
  taşımalı; iki parser drift edemez. Güvenilir proxy profili yalnız loopback
  bind invariant'ıyla açılabilmelidir.
- **B-053 · AÇIK — HTTP istek ayrıştırıcısını byte tabanlı ve fuzz kanıtlı
  yap.** Request-line/header CRLF, bare-LF, obs-fold, NUL, absolute-form,
  geçersiz UTF-8, TE/CL ve duplicate CL yüzeyi byte parser'da fail-closed
  olmalı; ayrı libFuzzer hedefi crash girdisini regression'a yükseltmelidir.

## P1 — Paketleme ve supply chain

- **B-029 · KISMEN — registry protokolünü önce normatifleştir.** RFC-0020,
  spec/18 ve spec/19 wire/imza/expiry'yi koddan önce bağladı; K-095 metadata
  doğrulayıcısı çalışır. Taşıma/cache/CLI hâlâ açıktır.
- **B-030 · KAPALI (K-117) — platformlar arası kanonik paket testi.** Üretici
  dosya sistemi bileşenlerini NFC'ye çevirir, çakışmayı reddeder; tüketici
  yalnız kanonik NFC yolu kabul eder. Türkçe Unicode dosya adlı sabit `.zep`
  fixture'ı aynı testle Linux/macOS/Windows CI'da byte-byte doğrulanır.
- **B-031 · KAPALI (K-117) — adversarial archive korpusunu büyüt.** Unicode
  17.0 UTS #39 ayraç/nokta/iki nokta benzerleri ve görünmez bidi denetleyicileri
  dahil 80 kalıcı yol vakası ayrı TSV korpusundadır. Traversal, symlink,
  duplicate/sıra, fazladan byte ve limit yapısal testleriyle birlikte ADR-028,
  [spec/18](../spec/18-paket-yayini.md) ve
  [conformance rehberi](zep-conformance.md) V1-P1-08'i kapatır.
- **B-032 · KAPALI — lockfile formatını sürümle.** `proje.kilit` baştan
  `kilit_sürümü 2` taşır; sonraki formatlar migration testi istemelidir.

## P2 — Tooling ve bakım

- **B-033 · KISMEN — archive hijyeni.** Gerçek repoda `.gitignore`, `.DS_Store`
  ve target dışlaması vardır; fakat dış incelemeye giden ZIP'te `__MACOSX`,
  `._*`, `.DS_Store` ve `compiler/fuzz/target` bulundu. Kaynak/yayın arşivi Git
  durumuna güvenmeden bunları yapısal olarak dışlamalıdır.
- **B-034 · AÇIK — tekrar üretilebilir compiler source snapshot komutu.** Yalnız
  gerekli kaynak/test/Cargo/belge/golden/conformance ve `.github/workflows`
  alan, SHA-256 manifestli geliştirme arşivi üret. CI tanımı arşivden eksik
  kalmamalı; build cache hiçbir koşulda pakete girmemelidir.
- **B-035 · AÇIK — function size/complexity trend bütçesi.** Kör hard limit
  yerine kritik modüllerde büyüme raporu ve gözden geçirme eşiği koy.
- **B-036 · KAPALI — Clippy `-D warnings` kapısı.** CI ve yerel toplu doğrulama
  bunu uygular; release işlerinde korunur.
- **B-037 · AÇIK — `cargo fmt --check` kapısı.** Mevcut geniş format borcu
  kontrollü tek seferlik committe temizlenmeden hard gate açılamaz.
- **B-038 · AÇIK — faza özgü test matrisi.** Lexer/parser, tür, morfoloji,
  runtime, concurrency, güvenlik, package ve LSP ayrı raporlanmalıdır.
- **B-039 · AÇIK — semantic regression corpus.** Düzeltilen her compiler bug'ı
  minimal kalıcı `.dil` success/fail fixture'ına dönüşmelidir.
- **B-040 · KISMEN — performans baseline arşivi.** `src/bin/olcum.rs` vardır;
  parser/checker/runtime p50/p95 CI artefact ve trend olmalıdır.
- **B-054 · AÇIK — dependency advisory/lisans/tekrar üretim kapısı.** CI'da
  sabit sürümlü `cargo audit` veya `cargo deny`, RustSec advisory, lisans/ban
  politikası ve V1 için lock+checksum/offline/vendor prosedürü tanımlanmalıdır.
- **B-041 · KAPALI (K-120) — LSP'yi SymbolId/HIR'a bağla.** Definition ve
  rename yalnız başarılı checker'ın `SymbolId`/`IslemId`/`YapiId` typed-HIR
  bağından hedef seçer. HIR ilk tanım, yeniden atama ve okuma aralıklarını
  aynı sembol kimliğinde toplar; iki bloktaki aynı yazım ayrıdır. Morfoloji
  kimlik seçiminden sonra yalnız seçilen sembolün yüzeylerini giydirir; çok
  kelimeli işlem adı tek varlıktır. A002/derleme hatasında metin tahmini yoktur.
  Dış birim tanımı yerel başlık gibi doğrulanmadıkça eksik tek-dosya rename
  üretilmez. [Semantic gezinme rehberi](lsp-semantic-gezinme.md), bir HIR ve
  yedi yeni LSP regresyonuyla kapsam, yeniden atama, işlem/yapı ve fail-closed
  sınırı korunur.
- **B-042 · KAPALI (K-119) — formatter parse-equivalence property.** C011
  güvencesi artık `SatirSonu`, `Girinti`, `Cikinti` ve `DosyaSonu` dahil tam
  parser token izini byte-konumlarından bağımsız kıyaslar. Sayısal 33 golden
  programın metin/yorumları koruyan deterministik dağınık-boşluk varyantı
  biçimlenir; önce/sonra izi eşit ve iki parser geçişi de başarılı olmak
  zorundadır. İdempotence ve proje/kitaplık resmî biçim kapıları korunur.
- **B-043 · KAPALI (K-118) — spec↔code kanıt haritası.** Bugünkü 25 RFC, 31
  ADR ve 24 spec bölümü `docs/kanit-haritasi-v1.tsv` içinde `kanitli/kismi/taslak`
  durumu, yürütülebilir test yolları ve açık kapsam notuyla birebir izlenir.
  Tazelik testi eksik/yinelenen belgeyi, olmayan ya da test taşımayan kanıt
  dosyasını ve testsiz tamamlanmış satırı reddeder.
- **B-044 · KAPALI (K-118) — hareketli README sayılarını tek kaynaktan üret.**
  `depo_sayilari`; golden, Rust+doctest, tanı kimliği, RFC durumları, ADR ve
  spec sayılarını gerçek dosyalardan çıkarır. `--yaz` README bloğunu atomik
  üretir, `--denetle` byte farkını CI hatası yapar; bakım sözleşmesi
  [depo bütünlüğü rehberindedir](depo-butunlugu.md).
- **B-045 · KAPALI İLKE — self-hosting'e erken atlama.** P0 omurga ve semantik
  V1 yaklaşmadan Rust bootstrap'tan ikinci compiler'a borç kopyalanmaz.

## Bir sonraki somut kapı

İnsan kanıtı hattında B-001, doldurulmuş gerçek usability formları ve önceden
ilan edilmiş eşikleri bekler. Makine hattında K-133 yan etkilerin deadline
kapısını sıkılaştırdı ve B-026'yı kısmen kapattı. Sıradaki iş K-134 web istek
yaşam döngüsü transaction'ıdır.

## 2 Eylül 2026 ikinci dış inceleme ayrımı

- **Doğrulandı ve K-129/K-130/K-131/K-132 ile kapandı:** B-025 kaynak/token,
  proje toplamı, bounded stdin/dosya okuması, heap/metin/çıktı, koleksiyon,
  görev, LSP toplamı/outbound ve süreç-geneli bağlantı sayısı. Eski sabit göçü
  bitti; LSP'nin 8 MiB reddi JSON kurulurken uygulanır.
- **Doğrulandı ve çalışılıyor:** B-026 cancellation K-133 ile etki öncesi
  deadline denetimine ilerledi; web istek transaction'ı K-134'e kaldı.
  Ardından B-029 registry taşıma/cache/kalıcı rollback, B-046 rate-limit
  ve çok süreçli oturum, B-051 kesin JSON-RPC, B-052 origin tekilleştirme,
  B-053 byte HTTP+fuzz, B-034 temiz snapshot ve B-054 advisory/reproducibility.
- **Mevcut repoda zaten kapalı:** çağrı derinliği C019/500 ve ayrı regresyonu;
  atomik metadata `unsafe` bloklarının her birindeki `SAFETY` gerekçesi; kök
  `.gitignore`; üç platformlu `.github/workflows/ci.yml`; tekil P2 başlığı.
- **Arşiv kaynaklı bulgu:** CI dosyasının yokluğu, `target`, `__MACOSX` ve
  `.DS_Store` sızıntısı gerçek Git ağacından değil, incelemeye gönderilen ZIP
  üretiminden kaynaklanır. Bu nedenle bulgu silinmedi; B-033/B-034 altında
  üretim hattı problemi olarak tutuldu.
