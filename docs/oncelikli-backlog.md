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
19. Sıradaki makine omurgası B-008 `zee-tr-1` immutable profil kapısıdır.
20. Sonraki işler aşağıdaki öncelik ve bağımlılık sırasını korur.

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
  tamamlandı; proje/paket izin politikası B-023 kapsamındadır.
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
- **B-008 · KISMEN — `zee-tr-1` profilini immutable koru.** RFC-0018/spec-13
  kırıcı değişikliği `zee-tr-2`ye yönlendirir; bağımsız uyumluluk denetimiyle
  bu kural CI'da görünür olmalıdır.
- **B-009 · AÇIK — morfoloji conformance korpusunu compiler'dan bağımsızlaştır.**
  Yüzey→kök→ek zinciri→belirsizlik/hata→profil makine-okunur fixture olmalıdır.
- **B-010 · KAPALI (K-101) — semantic ID modelini kur.** `YapiId`, `IslemId`
  ve `SymbolId` newtype'ları eklendi. `Tur::Yapi` artık depolama indeksi değil
  kimlik taşır; yapı erişimi ayrı kimlik→konum dizinindedir. İşlem imzaları ve
  özyineleme yığını `IslemId`, sembol tablosu ad→(`SymbolId`, tür) kullanır.
  Checker `Degisken`/`YeniYapi`/`IslemCagrisi` bağlarını AST'ye yazar. ADR-014,
  [rehber](semantic-kimlik-modeli.md), üç davranış ve bir mimari testle
  V1-P0-12 kapandı. Faz tipleri B-018/K-102, typed HIR ve bağlı runtime tüketimi
  B-019/K-103–K-104 ile tamamlandı.
- **B-011 · KISMEN — gözlenebilir concurrency determinizmini V1 garantisi yap.**
  spec/14 tek-thread semantiği tanımlar; gelecekte multicore yürütmenin gözlenen
  sıra/sonucu değiştiremeyeceği açık compatibility sözüne bağlanmalıdır.
- **B-012 · KISMEN — Ondalık↔binary float dönüşümünü yalnız açık ve kayıplı yap.**
  Bugün implicit dönüşüm yoktur; RFC-0012'deki eski `GerçekSayı↔double`
  kalıntıları FFI gerçeklenmeden temizlenmelidir.

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
  sahiplik testiyle V1-P0-18 kapandı; toplam 427 test yeşildir.
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
- **B-050 · AÇIK — kesin source span'i bütün AST ifadelerine yay.** K-108
  konumsuz HIR düğümünü kapattı; bugün değişken dışındaki eski AST varyantları
  satır zarfı taşır. Parser token aralıklarını bütün bileşik/leaf düğümlerde
  koruyup tanı, LSP ve gelecek lowering'e kesin sütun+uzunluk sağlamalıdır.

## P1 — Runtime ve güvenlik

- **B-023 · SIRADA — web/sensör/HTTP'yi capability modeline bağla.** Ağ,
  sandbox dosya sistemi ve sensör yetkisi merkezi compile/runtime politikası
  olsun; outbound hedef/SSRF politikası B-049 ile aynı sınırda kapanır.
- **B-024 · KAPALI İLKE — HTTPS/TLS'yi elle yazma.** Gerektiğinde kilitli,
  battle-tested backend kullan; Zee kriptografi/TLS gerçeklemeye dönüşmez.
- **B-025 · KISMEN (K-105) — ortak `KaynakSinirlari` modeli.** K-105 native
  HTTP istemcisini varsayılan 30 saniye + 8 MiB wire yanıtla, yerel sunucu
  okumasını 10 saniyelik mutlak bütçeyle sınırladı. Recursion, ortak input/body,
  allocation, koleksiyon, görev, eşzamanlı bağlantı ve output bütçelerini tek
  modelde merkezileştirme hâlâ açıktır.
- **B-026 · AÇIK — cancellation-safety audit'i.** Dosya temp'i, web yanıtı,
  oturum mutation'ı ve diğer yan etkilerin iptal/yarım kalma davranışını testle.
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
  belleği ve çıktı bütçesi B-025'te kalır.
- **B-048 · AÇIK — atomik replace metadata sözleşmesini tamamla.** İzin biti
  dışındaki owner/group, ACL, xattr ve platform güvenlik etiketlerinin korunma
  veya açıkça desteklenmeme davranışı platform testleriyle belgelenmelidir.
- **B-049 · AÇIK — outbound ağ güven profilini kapat.** Battle-tested HTTPS
  backend, redirect/yanıt sınırları ve host/IP/port capability politikası
  birlikte tasarlanmalı; Zee TLS'yi elle yazmamalı ve güvenilmeyen kod varsayılan
  olarak iç ağ/metadata hedeflerine erişememelidir.

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

- **B-033 · KISMEN — archive hijyeni.** `.DS_Store` Git dışında; kaynak/yayın
  arşivleri `__MACOSX`, target ve geçici dosyaları yapısal olarak dışlamalıdır.
- **B-034 · AÇIK — tekrar üretilebilir compiler source snapshot komutu.** Yalnız
  gerekli kaynak/test/Cargo/belgeleri alan, özetli geliştirme arşivi üret.
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
- **B-043 · KAPALI (K-118) — spec↔code kanıt haritası.** 23 RFC, 26 ADR ve
  22 spec bölümü `docs/kanit-haritasi-v1.tsv` içinde `kanitli/kismi/taslak`
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
ilan edilmiş eşikleri bekler. Makine hattında B-007/K-121 kapandı; sıradaki iş
B-008 `zee-tr-1` profilinin immutable uyumluluk kapısıdır.
