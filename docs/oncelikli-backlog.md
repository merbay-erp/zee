# zee — V1 öncesi öncelikli mühendislik backlog'u

> Güncel kaynak [S4 yerel kabulü](urunlesme-s4-kabul-20261015.md) ile RC, soak, macOS teslim/kurulum ve dar IDE/stdio ölçüm kanıtı taşır. [S3](urunlesme-s3-kabul-20261015.md) native kabulü tarihsel kaynağında kalır; ürün, insan, fiziksel platform ve yayın kapıları açıktır.

Bu belge 1 Eylül 2026 ayrıntılı incelemesi ile 2 Eylül'deki ikinci ve üçüncü
dış incelemelerin depo içindeki kalıcı, sıralı iş karşılığıdır. Yeni dil
özelliği P0 omurga işleri kapanmadan öne
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
31. K-134 web istek yaşam döngüsünü transaction'a bağladı. Oturum ve çerez
    mutation'larıyla ilk HTTP yanıtı başarıda birlikte commit edilir; timeout,
    runtime hatası, yanıtsız rota veya socket yazma hatasında birlikte geri
    alınır. Gerçek TCP ve hermetik regresyonlarla B-026 kapandı (527 test).
32. K-135 B-029'un taşıma/cache/kalıcılık dilimini kapattı. HTTPS-only statik
    ayna, redirect/proxy ve private-IP korkulukları; ardışık root taşıması;
    CAS korumalı atomik monoton durum; tam metadata+yayın doğrulamasından sonra
    salt-okunur SHA-256 nesne deposu ve çevrimdışı hit/miss çalışır (533 test).
33. K-136 exact `ad@X.Y.Z` bağımlılığı HTTPS origin + ağ dışı root pinli
    `proje.dil` alanlarına, `proje.kilit` v3 kimliğine ve açık ağ kullanan
    `ekle/kilitle/paketler --yenile` CLI zincirine bağladı. Normal derleme/LSP
    proje-local doğrulanmış cache'den ağsız çalışır; `.zep` kaynakları atomik,
    exact doğrulanan ve salt-okunur kurulur. P017 ve uçtan uca gerçek imzalı
    registry ve duyuru-kümesi politika regresyonuyla V1-P1-07/B-029 kapandı
    (539 test).
34. K-137/ADR-034 `Depo` sınırını, production'da kalıcı ortak oturum ve oran
    deposunu, tek-hop kanonik `Forwarded` kimliğini ve process başına tek
    worker/N süreç modelini kurdu. İki gerçek CLI sürecinde login, restart,
    çapraz logout ve ortak altıncı-deneme 429 kanıtıyla B-046 kapandı
    (547 test).
35. K-138/ADR-035 JSON-RPC sınırını protokol-kesin yaptı. RFC 8259 sayı
    durum makinesi sayıyı float'a çevirmeden kayıpsız lexeme taşır; duplicate
    alan reddi, `-32700/-32600/-32601/-32602`, UTF-8 hata devamı ve bildirim
    sessizliği ayrı regresyonlarla B-051'i kapattı (556 test).
36. K-139/ADR-036 CLI'a özel `GuvenliOrigin` tipini kaldırdı. `--web-proxy`,
    `Host`, `Forwarded host` ve unsafe `Origin`, outbound allowlist ile aynı
    `AgHedefi` DNS/IPv6/port parser'ını kullanır. Production listener sabit
    loopback bind'i ve kabul edilen loopback peer'i birlikte doğrular; B-052
    kapandı (558 test).
37. K-140/ADR-037 HTTP request-line, header ve gövdeyi tek byte parser'ında
    yalnız CRLF, origin-form, HTTP/1.0/1.1, tek kurallı Content-Length ve exact
    UTF-8 gövde profiline bağladı. TE, obs-fold, NUL, kayıplı dönüşüm ve fazla/
    eksik framing kalıcı korpus ile ayrı libFuzzer hedefinde reddedilir; B-053
    kapandı (566 test).
38. K-141/ADR-038 sabit `cargo-deny` ile compiler+fuzz RustSec/lisans/ban/
    kaynak denetimini, kilitsiz CI fallback yasağını ve iki üretimli SHA-256
    manifestli gerçek offline vendor derlemesini kurarak B-054'ü kapattı
    (570 test).
39. K-142/ADR-039 playground köprüsünü sürümlü ABI v2'ye taşıdı. Kayıtlı
    başlangıç pointer'ı+exact boy, strict UTF-8, sorgulanabilir sonuç kaydı,
    yanlış/çift bırakmada durum koruması, native+gerçek wasm32 host testi ve
    ayrı libFuzzer hedefiyle B-055 kapandı (575 test).
40. K-143/ADR-040 playground kaynak ve soru girdisini merkezî fakat ayrı
    8 MiB kaynak, 1 MiB/4.096 satır soru bütçesine bağladı. ABI v3 limitleri
    hosta bildirir; Rust kopya/satır tablosundan, tarayıcı UTF-8 byte dizisi ve
    WASM tahsisinden önce reddeder. B-056 kapandı (579 test).
41. K-144/ADR-041, 48 kritik üretim işlevini sabit Clippy ölçüsü ve incelenmiş
    tabana bağladı. Yeni/kayıp işlev, biriken satır/karmaşıklık büyümesi ve
    bayat Markdown raporu CI'ı durdurur; B-035 kapandı (585 test).
42. K-145/ADR-042, 53 dosyadaki 349 fark bloklu biçim borcunu sabit
    `rustfmt` ile tek mekanik dilimde temizledi; üç platformlu
    `cargo fmt --all -- --check` kapısını açtı ve B-037'yi kapattı.
43. K-146/ADR-043 bütün gerçek Cargo/libtest vakalarını 21 birincil faza
    sahipledi. Sahipsiz/yinelenen test ve bayat matris fail-closed; her Tier-1
    işi test/pass/fail/ignored/süre ile regression, fuzz ve conformance bağını
    ayrı summary+artefakt olarak raporlar. B-038 kapandı.
44. K-147/ADR-044 parser, checker, typed HIR, runtime, morphology,
    concurrency ve security altındaki 17 minimal `.dil` vakayı K-kimliği,
    faz, kip, kesin tanı spanı, exit ve stdout beklentisiyle manifestledi.
    Ağaç birebirliği/minimality ve ayrı faz raporuyla B-039 kapandı.
45. K-148/ADR-045 dokuz parse/checker/HIR/runtime/yürütme/LSP/bellek yüzeyini
    25 turluk ham örnek, p50/p95, JSON, Markdown ve sürümlü TSV tarihçesine
    bağladı. Shared CI yalnız summary+90 günlük artefakt üretir; hard eşik
    ancak sabit adanmış runner'da açıkça etkinleşir. B-040 kapandı (603 test).
46. K-149/ADR-046 bütün production Rust ağacını bugün 38 üst sahibe, exact doğrudan
    kenar tabanına ve izinli katman yönüne bağladı. Yeni/kayıp modül,
    eklenen/kaldırılan kenar ve ters katman geçişi fail-closed'dur. Morfoloji→
    paket SHA-256 ve tedarik→runtime takvim terslikleri temel sahiplere
    taşındı; B-057 kapandı (607 test).
47. K-150/ADR-047 exact production graph'ına SCC kapısı koydu. Tanı↔kaynak
    bütçesi ve checker↔HIR çevrimleri bağımsız temel/model sahipleriyle
    kırıldı. Tek kalan paket/registry/tedarik SCC'si K-160 kaldırma işi ve
    1 Ekim 2026 son tarihli gerekçeli geçici izindir; açıklamasız çevrim
    sıfırdır. B-058 kapandı.
48. K-151/ADR-048 bütün GitHub Actions kullanımlarını immutable commit SHA'ya,
    sürümlü exact pin kaydına ve haftalık fakat otomatik birleşmeyen Dependabot
    güncelleme akışına bağladı. Workflow token'ları salt-okunur, checkout
    kimliği kalıcı değildir; B-059 kapandı.
49. K-152/ADR-049 her benchmark satırını gerçek tam Git SHA, ayrı milestone,
    temiz çalışma ağacı, OS/CPU/RAM/Rust/release ve ölçüm başına gerçek
    örnek/ısınma semantiğine bağladı. RSS'in tek süreç-tepe görüntüsü olduğu
    artık makine-okunurdur; B-060 kapandı.
50. K-153/ADR-050'nin uygulama dilimi eski engine ölçümünü
    `lsp_engine_initialize` diye doğru adlandırdı; gerçek dillsp process
    spawn→stdio framing→tam capabilities yanıtını ayrı
    `lsp_process_cold_start` olarak ölçüyor. Tier-1 gerçek ikili testi ve CI
    kablosu hazırdır. Exact temiz `a2693d6…` uygulama commit'indeki 25 örnek
    process p50 1,557 ms/p95 1,997 ms tabanını verdi; B-061/K-153 kapandı.
51. K-154/ADR-051'in uygulama dilimi 2k/5k/10k/20k satır tam-metin
    `didChange` eğrisini, p95 250/500/1000 ms ilk-aşım raporunu ve bugünkü tam
    belge→lexer/parser→resolver/checker→typed-HIR invalidation sınırını
    görünür yaptı. Exact temiz `59580cd…` uygulama commit'indeki 25 örnek
    2k/5k/10k/20k p95'i sırasıyla 167,281 ms / 1.081,746 ms / 4.669,372 ms /
    20.159,636 ms ölçtü; üç eşik de ilk kez 5k'da aşılır. Optimizasyon
    yapılmadı; B-062 kapandı.
52. K-155/ADR-052, 17 semantic regresyonu exact `fixed_by`, kanıtlıysa
    `introduced_by` ve `guaranteed_since=0.8.0-dev` alanlı v2 manifeste
    taşıdı. Tarihsel introduced commit'ler reproducer olmadan tahmin edilmedi.
    Git koruğu yayımlanmış provenance'ı korur. K-155A/ADR-053 başlık regex'ini
    kaldırdı; sabit başlangıçtan sonraki her compiler kaynak commit'i mesajdan
    bağımsız beyan taşır ve bugfix exact fixture ister. Geçici gerçek Git deposu
    red/kabul/yeniden-yazım yollarını uçtan uca kanıtlar; B-063/B-064 kapandı.
53. K-176/ADR-054 sorgu ve form alanlarında eksik/kuralsız `%xx` ile çözüm
    sonrası geçersiz UTF-8'i rota çalışmadan 400'e çeviren strict uygulamayı
    ekledi. `web` kipli kalıcı fixture exact `32247c7…` uygulama SHA'sına ve
    compiler semantic bugfix beyanına bağlıdır; B-065 kapandı.
64. K-165/ADR-073 aynı aracı Rust ve Go ile bayt bayt eşdeğer yazıp kaynak
    satırı, test, derleme/çalışma süresi, tepe RSS ve sürtünme sayısını exact
    SHA tarihçesine bağladı; Zee'nin kaybettiği yerler belgede açık.
63. K-164/ADR-072 ikinci gerçek ürünü farklı iş yükünde kurdu: `kanit-ozeti`
    dokuz kayıt defterinden `docs/kanit-ozeti.md` üretir (10 birim/19 test,
    bayt bayt tazelik kapısı). On altı sürtünmeden ikisi compiler düzeltmesi
    (elmas birim içe alımı, birim kökenli tanı), on biri korpus çifti oldu.
62. K-169/ADR-071 kurulum-kaldırma sözleşmesini ve sürüm runbook'unu kurdu:
    `SHA256SUMS` doğrulamalı kur, manifestli ve no-clobber kaldır betikleri
    (bash + PowerShell), gerçek ikililerle Unix entegrasyon testi, üç platform
    tatbikat workflow'u ve sürüm günü sırası.
61. K-166/ADR-070 tanı kalitesi kapısını kurdu: golden'a 11 acemi hatası
    mutasyonu, sınıf başına öneri/işaret/gürültü ölçütü; S007 öneri, gövde
    dışı parametre satırı S004 ve A001/A007 tekrar ayıklaması ilk altı ihlali
    sıfırladı; insan verisi gelene kadar “en sık” korpus frekansıdır.
60. K-173/ADR-069 uzun soak kapısını kurdu: golden derleme + yürütme döngüsü ile
    gerçek `dillsp` didChange döngüsü süre bütçesince koşar, iki sürecin RSS'i
    örneklenir, ısınma sonrası büyüme %10 ve 32 MiB'ı birlikte aşarsa kalır;
    tarihçe temiz ağaçtan exact SHA ile yazılır, haftalık workflow 30 dk koşar.
59. K-168/ADR-068 sürüm ikilisini tekrar üretilebilir yaptı: iki bağımsız temiz
    klonda sabit toolchain + `--remap-path-prefix` + `SOURCE_DATE_EPOCH` ile eş
    SHA-256 zorunlu; SPDX SBOM, SLSA provenance ve Ed25519 imza deterministik
    üretilir/doğrulanır; `surum-adayi` workflow'u `v*` etiketinde koşar.
58. K-172/ADR-067 dogfood korpusunu kurdu: `dogfood/` altındaki her `.dil`
    ürün slug'ı, K-işi/dilimi, kip, beklenti ve exact çıktı/tanıyla manifestte;
    ilk taban 4 gereksinim aynası ve F007/F012/F030/F032 ret+çözüm çiftleri
    ürün politikasıyla derlenip hermetik koşar. Eksik F030–F032 günlük
    dilimleri yazıldı; GB-020/GB-021 gerçek adaptör birim testleriyle kapandı.
57. K-170/ADR-066 güvenlik kanıtını tek kapıda birleştirdi: ardışık `GB-NNN`
    bulgu kaydı (15 kapalı, 4 kabul edilen sınır, 3 açık düşük/orta), açık
    kritik/yüksek sıfır kuralı, `tedarik` workflow'unda sürekli kip ve etiket
    öncesi exact HEAD RC fuzz + clippy + tam test isteyen sürüm adayı kipi;
    kök `SECURITY.md` bildirim kanalını ve önem sözlüğünü tanımlar.
56. K-171/ADR-065 spec altındaki 169 normatif maddeyi parmak-izi kimliğiyle
    exact `<dosya>::<işlev>` test kanıtına bağladı; 151 kanıtlı, 16 gerekçeli
    kısmi ve 2 açık madde deterministik raporda görünür, kayıtsız/bayat madde
    ve kayıp test işlevi CI'ı durdurur. Kısmi liste K-166/K-172 girdisidir.
55. K-167/RFC-0028/ADR-064 kalıp kelimesi, koşul yüklemi, CLI komutu, biçim,
    profil, ABI, API ve tanı yüzeyini giriş sürümlü tek envantere, kaldırmayı
    ardışık `DEP-NNN` kaydı + bir alt sürüm serisi süre + göç yoluna bağladı.
    Cargo sürümü `0.8.0-dev` serisine çekildi; `dil::tedarik` DEP-004 ile
    kaldırılıp B-072 kapandı. K-177 önce CI görmemiş 126 commit için bütün
    kapıları yerelde koşup kritik işlev eğilim kırığını bölmeyle kapattı.
54. K-156/ADR-055 dört gecelik hedefin koşu sonu coverage korpusunu cache'ten
    bağımsız, SHA-256 seed manifestli ve kaynak commit+run provenance'lı 90
    günlük artefakta taşır. Doğrulayıcı Linux/macOS hash yollarını ve
    manifest↔ağaç birebirliğini gerçek geçerli/bozuk artefaktla sınar; repo
    seed'i `cmin`+stable replay+insan review'u ister. B-066 kapandı.

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
  ve üç regresyonla V1-P0-20 kapandı; toplam 432 test yeşildir. K-156/B-066
  daha sonra bütün coverage korpusunu cache dışı provenance'lı artefakta aldı.
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
- **B-026 · KAPALI (K-133/K-134) — cancellation-safety audit'i.** Temp dosya,
  kilit, son tarih ve çalışma bütçesi nöbetçileri `Drop` ile sahipli temizlenir;
  eylem transaction'ı hata halinde rollback eder. K-133 çıktı/girdi, dosya,
  sunucu, yanıt/yönlendirme/çerez/oturum, eyleyici, CSRF, parola, rastgelelik
  ve eylem başlangıcına tam yan etki öncesi deadline kapısı koydu. Görev HTTP
  öncesi scheduler'a sıra verdikten sonra eski kalan süreyi kullanmaz; dolmuş
  dosya/HTTP etkisinin hiç başlamadığı sanal saat regresyonları vardır. K-134
  rota seçimi ve gövde yürütmesini ayrı `web_istek` sahibine aldı; oturum,
  çerez ve ilk yanıt request transaction'ında tamponlanır. Başarıda yanıt
  socket'e yazılınca birlikte commit olur. Deadline/runtime hatası, yanıtsız
  rota veya socket yazma hatası session/cookie/yanıtı birlikte geri alır;
  kısmi TCP yazımı bağlantı kapanışı ve doğru `Content-Length` ile başarı
  sayılamaz.
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
- **B-046 · KAPALI (K-106/K-137, ADR-018/034) — web oturum deposunu sınırlı
  ve ölçeklenebilir yap.** 4096 toplam/1024 anonim kota ile kaymayan mutlak
  10/30 dakika ömür iki adaptörde korunur. `--web-proxy`, proje kökündeki
  CAS-korumalı kalıcı ortak depoda login/revoke/expiry ile endpoint,
  CSRF ve Argon2id oran pencerelerini atomik tutar. Yalnız canlı oran
  kayıtlarıyla doluluk fail-closed'dur. Güvenilir proxy tek-hop `Forwarded`
  içindeki kanonik IP'yi kurar; XFF kimlik değildir. İki gerçek CLI süreci,
  restart, çapraz logout ve dağıtılmış altıncı parola denemesinin Argon2id
  öncesi 429 olmasıyla sözleşme kanıtlıdır. V1 concurrency modeli aynı depoyu
  paylaşan N ayrı tek-worker süreçtir; çok-hostlu harici backend ileriki
  deployment işidir.
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
- **B-051 · KAPALI (K-138/ADR-035) — LSP JSON-RPC ayrıştırmasını
  protokol-kesin yap.** RFC 8259 sayı durum makinesi geçerli lexeme'i binary
  float'a çevirmeden saklar; baştaki sıfır/eksik kesir-üs ve `NaN/Infinity`
  reddedilir. Çözülmüş Unicode adı duplicate olan alan parse aşamasında
  kapanır. UTF-8/sözdizimi `-32700`, geçerli JSON içindeki bozuk tek-nesne
  zarf `-32600`, bilinmeyen yöntem `-32601`, bozuk yöntem parametresi
  `-32602`dir. Kimliksiz bildirime response yoktur; sayısal kimlik kayıpsız
  döner. RFC sayı differential korpusu, escaped duplicate, standart hata ve
  ardışık framing ve mimari sahiplik testleri toplam envanteri 556'ya çıkardı.
- **B-052 · KAPALI (K-139/ADR-036) — web proxy origin'ini tek kanonik tipe
  geçir.** CLI'a özel `GuvenliOrigin` kaldırıldı; `--web-proxy`, `Host`,
  `Forwarded host` ve unsafe `Origin` outbound allowlist ile aynı `AgHedefi`
  parser'ından geçer. Büyük/küçük DNS adı, varsayılan `:443`, köşeli ayraçlı
  IPv6 ve 1–65535 port tek fail-closed kurala bağlıdır. Listener sabit
  `127.0.0.1`e bind eder ve socket peer'inin loopback olduğunu ayrıca
  doğrular. Olumlu/olumsuz origin ve mimari sahiplik regresyonları envanteri
  558 teste çıkardı.
- **B-053 · KAPALI (K-140/ADR-037) — HTTP istek ayrıştırıcısını byte tabanlı
  ve fuzz kanıtlı yap.** `http_istegi.rs` request-line/header CRLF, bare-LF/CR,
  obs-fold, NUL, absolute/authority/asterisk-form, fragment, geçersiz UTF-8,
  TE, duplicate/kuralsız CL ve exact olmayan gövdeyi fail-closed ayırır. Gövde
  limiti doğrulanmış CL sonrasında tahsis edilir; kayıplı UTF-8 kaldırılmıştır.
  Yedi protokol, bir mimari sahiplik testi, beş kalıcı seed ve ayrı ham-byte
  libFuzzer hedefi envanteri 566 teste çıkardı.

## P1 — Paketleme ve supply chain

- **B-029 · KAPALI (K-136) — registry protokolünü önce normatifleştir.** RFC-0020,
  spec/18 ve spec/19 wire/imza/expiry'yi koddan önce bağladı; K-095 metadata
  doğrulayıcısı çalışır. K-135 limitli HTTPS/statik taşıma, ardışık root,
  atomik kalıcı sürüm+özet durumu, doğrulama-sonrası salt-okunur SHA-256 cache,
  bozuk cache reddi ve çevrimdışı hit/miss'i gerçekledi. K-136 exact bağımlılığı
  `proje.dil`, `proje.kilit` v3, gerekçeli yanked/kritik politika kayıtları,
  proje-local atomik kaynak kurulumu ve çevrimiçi/çevrimdışı CLI'a bağladı.
  Kritik kabul güncel duyuru kümesine bağlıdır; yeni duyuru eski gerekçeyi
  devralamaz.
  Normal derleme ve LSP sessiz ağ açmaz; gerçek imzalı uçtan uca regresyon
  cache bozulmasını da P016 ile fail-closed doğrular.
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
- **B-032 · KAPALI — lockfile formatını sürümle.** `proje.kilit` K-136 ile
  `kilit_sürümü 3` taşır; sürüm 2 yalnız eski yerel grafiğin tarihsel biçimidir.
  Sonraki formatlar açık migration/yeniden üretim testi istemelidir.

## P2 — Tooling ve bakım

- **B-033 · KAPALI (temiz arşiv bakımı) — archive hijyeni.** Kök ignore
  politikası `target`, fuzz artifact/coverage, `__MACOSX`, profiler verisi ve
  yerel ZIP'leri açıkça dışlar. `scripts/temiz-kaynak-arsivi.sh` çalışma
  klasörünü değil yalnız izlenen `HEAD` ağacını paketler ve yasaklı girdiyi
  çıktı üzerinde yeniden tarar.
- **B-034 · KAPALI (temiz arşiv bakımı) — tekrar üretilebilir compiler source
  snapshot komutu.** Sabit commit zamanı kullanan `git archive`, kaynak/test/
  Cargo/belge/golden/conformance ve CI dahil bütün izlenen ağacı alır; kökte
  sıralı dosya SHA-256 manifesti ve yanında ZIP SHA-256 özeti üretir. Aynı
  `HEAD` byte-byte aynı arşivdir; build cache hiçbir koşulda pakete giremez.
- **B-035 · KAPALI (K-144/ADR-041) — function size/complexity trend
  bütçesi.** Sabit Rust/Clippy zinciri üretim `lib`+`dil` işlevlerini ölçer;
  80 satır/12 bilişsel karmaşıklık izlemeye giriş, tasarım hükmü değildir.
  İlk 48 kayıt incelenmiş TSV tabanındadır. Tabanın %10 satır (+8..+24) veya
  %20 karmaşıklık (+2..+5) payını aşmak, yeni/kayıp kritik işlev ve bayat
  [eğilim raporu](islev-egilimi.md) fail-closed inceleme ister. Düşüş tabanı
  otomatik sıfırlamaz; `--force-warn` kaynak içi lint susturmasını atlatır.
- **B-036 · KAPALI — Clippy `-D warnings` kapısı.** CI ve yerel toplu doğrulama
  bunu uygular; release işlerinde korunur.
- **B-037 · KAPALI (K-145/ADR-042) — `cargo fmt --check` kapısı.** 53 Rust
  dosyasındaki 349 eski fark bloğu tek davranışsız toplu biçim diliminde
  temizlendi. Sabit Rust 1.93.1 `rustfmt` bileşeni ve üç platformlu
  `cargo fmt --all -- --check` CI kapısı yeni borcu reddeder. Faz satır
  bütçeleri ve K-144 eğilim tabanı yalnız kanonik biçim ölçüsüne bir kez
  yeniden kalibre edildi; semantik karar değişmedi.
- **B-038 · KAPALI (K-146/ADR-043) — faza özgü test matrisi.** Gerçek Cargo
  JSON artefaktları ve libtest envanteri lexer, parser, AST, resolver, tür,
  typed HIR, morfoloji, runtime, IO, concurrency, web/security, HTTP, package,
  registry, supply-chain, LSP, WASM, CLI, project, end-to-end ve mühendislik
  kapılarına tam birincil sahiplik verir. Sahipsiz/yinelenen test, boş seçici,
  kayıp ilişki yolu ve bayat [kanonik matris](faz-test-matrisi.md)
  CI'ı kapatır. Her Tier-1 platformu kendi `cfg` envanterini ayrı çalıştırır;
  pass/fail/ignored, duvar süresi, regression, fuzz ve conformance bağını job
  summary ile indirilebilir artefakta yazar. Süre performans eşiği değildir.
- **B-039 · KAPALI (K-147/ADR-044) — semantic regression corpus.** Parser,
  checker, typed HIR, runtime, morphology, concurrency ve security için 17
  tarihsel bug minimal `.dil` fixture'ıdır. Sürümlü TSV her vakayı K-kimliği,
  birincil faz, kip, kesin tanı spanı, exit ve sıralı stdout ile sahipler;
  ağaçta sahipsiz/yinelenen dosya, bozuk metadata veya 4 KiB/32 dolu satır
  minimality aşımı fail-closed'dur. Typed-HIR runtime çıkış kodunu kaybetmeyen
  API ile success/fail gözlemleri aynı koşucuda doğrulanır. Yeni compiler bug
  düzeltmesi fixture+satır olmadan tamamlanamaz; ayrıntı
  [korpus rehberindedir](semantic-regresyon-korpusu.md).
- **B-040 · KAPALI (K-148/ADR-045; K-152/ADR-049 revizyonu) — performans baseline arşivi.** Release
  koşucusu iki ısınma+25 turla parse, resolver/checker+HIR kanıtı, tam
  kaynak→typed-HIR, runtime başlangıcı, 100 bin tur yürütme, LSP
  cold/open/change ve Unix tepe RSS için ham örnek, min/max ve nearest-rank
  p50/p95 üretir. V2 JSON/TSV; exact Git SHA, ayrı milestone, temiz ağaç,
  OS/CPU/RAM/Rust/release ve ölçüm başına gerçek sample/warmup semantiğini
  korur; bozuk/yinelenen tarihçe fail-closed'dur. RSS tek süreç-tepe
  görüntüsüdür. Linux CI summary+90 günlük artefakt üretir fakat
  gürültülü shared runner'da hard gate yoktur. `--esik-yuzde` yalnız
  sabitlenmiş adanmış benchmark koşucusunda bilinçli seçenektir. Ayrıntı
  [ölçüm rehberindedir](olcumler.md).
- **B-054 · KAPALI (K-141/ADR-038) — dependency advisory/lisans/tekrar üretim
  kapısı.** Sabit `cargo-deny 0.20.2`, güncel RustSec'i compiler ve fuzz
  grafiğinde push/PR+günlük tarar; izinli SPDX kümesi, bilinmeyen registry/Git,
  wildcard ve duplicate politikası `-D warnings` ile fail-closed'dur. İki exact
  duplicate istisnası teknik gerekçe taşır, advisory ignore boştur. Bütün CI
  Cargo komutları `--locked` kullanır; iki lock yalnız crates.io+SHA-256 taşır.
  İki bağımsız `--versioned-dirs` vendor ağacı sıralı yol+dosya özetiyle aynı
  çıkmak ve boş Cargo home'da compiler+fuzz `--offline --locked` derlenmek
  zorundadır. Compiler/fuzz `publish = false` kalır; bu kapı bekleyen Zee ürün
  lisansını kendiliğinden seçmez.
- **B-055 · KAPALI (K-142/ADR-039) — WASM C ABI'sini hasım çağırana karşı
  kanıtla.** ABI v2 yalnız kayıtlı başlangıç pointer'ı ve exact boyu sahipli
  kopyaya alır; null/iç/kayıt dışı pointer, taşkın boy, sonuç→girdi ve invalid
  UTF-8 çekirdekten önce görünür reddir. Sonuç toplamı kayıttan sorgulanır;
  yanlış/çift bırakma sahipliği düşürmez. Sekiz tampon/64 MiB genel zarf,
  native saldırı matrisi, gerçek wasm32 Node hostu, dört K-142 byte seed ve
  1.745.134 çağrılık ayrı libFuzzer kampanyası yeşildir. Ayrıntı
  [güncel ABI rehberindedir](wasm-c-abi.md).
- **B-056 · KAPALI (K-143/ADR-040) — playground girdisine bağımsız ön-tahsis
  bütçesi koy.** Merkezî profil kaynak için 8 MiB, soru girdisi için
  1 MiB/4.096 satır belirler. Native köprü byte/satır sınırını `Vec<String>`
  öncesinde; ABI v3 byte sınırını kayıtlı tamponu kopyalamadan uygular.
  Tarayıcı limitleri WASM'den okur, UTF-8 boyunu tahsissiz hesaplar ve
  `encodeInto` ile exact tampona yazar. Sınır/bir-fazlası native+gerçek wasm32
  Node regresyonu ve kaynak+soru kipli beş seed'li fuzzer ile korunur; ilk
  kampanya 61 saniyede 1.709.869 çağrıyı ihlalsiz tamamladı.
- **B-057 · KAPALI (K-149/ADR-046) — production katman yönünü makinece
  koru.** 35 üst sahip; temel, model, altyapı, sözdizimi, semantik, proje,
  web, runtime, adaptör ve mühendislik katmanlarına atanır. Sürümlü TSV exact
  doğrudan bağımlılıkları ve tek cümlelik sorumluluğu taşır. Kaynak tarayıcı
  yorum/metin/test kodunu ayırır; hedefe özgü production yollarını birleşik
  korur. Yeni/kayıp sahip, eklenen veya artık kullanılmayan kenar ve izin dışı
  katman geçişi fail-closed'dur. Morfoloji SHA-256 için pakete, tedarik tarih
  dönüşümü için runtime'a artık bağımlanmaz; ortak ilkeller `guvenlik` ve
  `zaman` temel sahiplerindedir. Ayrıntı [katman rehberindedir](katman-mimarisi.md).
- **B-058 · KAPALI (K-150/ADR-047) — production dependency cycle kapısı.**
  Exact graph sahipler-arası SCC için denetlenir. `tani_politikasi`, tanı ile
  merkezî kaynak profili arasındaki bütçeyi bağımlılıksız taşır;
  `semantic_model`, checker ile HIR'ın ortak tür sahibidir. Böylece ilk üç
  SCC'den ikisi kaldırıldı. Paket/registry/tedarik SCC'si yalnız exact üyeli,
  ayrıntılı gerekçeli, K-160 sahipli ve 1 Ekim 2026 son tarihli C001 iznidir.
  Yeni SCC, bayat allowlist ve süre aşımı fail-closed'dur.
- **B-059 · KAPALI (K-151/ADR-048) — GitHub Actions supply-chain pinleme.**
  Üç workflow'taki bütün üçüncü taraf `uses:` değerleri resmî ref'lerden
  doğrulanmış 40 haneli commit SHA'dır. `github-actions-pinleri-v1.tsv`
  action/sürüm/SHA/kaynak kaydını kullanımla birebir tutar; yeni workflow da
  sabit dosya listesi yerine dizin taramasıyla aynı kapıya girer. Haftalık
  Dependabot güncelleme PR'ı açar ama otomatik merge edilmez. Checkout
  credentials kalıcı değildir ve workflow token'ları `contents: read` ile
  sınırlıdır.
- **B-060 · KAPALI (K-152/ADR-049) — Benchmark provenance.** Tarihçe v2 her
  satırda exact 40 haneli Git SHA, ayrı milestone, temiz Git ağacı,
  platform+OS, CPU, fiziksel RAM, Rust, release profili ve gerçek
  sample/warmup/örnekleme alanlarını zorunlu tutar. `--gecmis-cikti` HEAD'den
  farklı SHA veya kirli ağaçta fail-closed'dur. Sekiz süre metriği bağımsız
  turları; RSS yalnız bir örnek, sıfır ısınma ve süreç-tepe anlık görüntüsünü
  bildirir. İlk K-148 sayıları değiştirilmeden exact üretici commit'ine göçtü.
- **B-061 · KAPALI (K-153/ADR-050) — Gerçek LSP cold-start.** In-process
  engine initialize ile process spawn+stdio+capabilities yanıtı ayrı kimliktir.
  Gerçek `olcum`→`dillsp` entegrasyon testi ve release CI ölçümü hazırdır;
  mevcut LSP workspace yüklemediği için hayalî workspace metriği yoktur.
  Exact temiz `a2693d6…` uygulama commit'indeki 25 örnek process p50
  1,557 ms/p95 1,997 ms; engine p50 542 ns/p95 625 ns tabanını verdi.
- **B-062 · KAPALI (K-154/ADR-051) — LSP incremental analysis hazırlığı.**
  Koşucu 2k/5k/10k/20k tam-metin `didChange` p50/p95 eğrisini açık
  `--lsp-olcek` ile üretir; CI bunu JSON/Markdown/TSV artefaktına katar ve
  rapor önceden sabit 250/500/1000 ms çizgilerinin ilk p95 aşımını bulur.
  Mevcut invalidation sınırı tam belge saklama+klonlama ve tam lexer/parser/
  resolver/checker/typed-HIR yeniden kurulumudur; incremental cache yoktur.
  Optimizasyon kapsam dışıdır. Exact temiz `59580cd…` uygulama commit'inde
  25 örnekli p95 eğrisi 167,281 ms / 1.081,746 ms / 4.669,372 ms /
  20.159,636 ms'dir; 250/500/1000 ms eşiklerinin üçü de ilk kez 5k'da aşılır.
- **B-063 · KAPALI (K-155/ADR-052) — Semantic regression provenance.** Her
  `regression/v2.tsv` vakası gerçek tam `fixed_by`, varsa tam
  `introduced_by`, `0.8.0-dev` garanti serisi, K-kimliği, faz ve gözlenebilir
  sonucu birlikte taşır. İlk 17 tarihsel vakanın introduced commit'i düzeltme
  anında minimal reproducer bulunmadığından tahmin edilmez ve `-` kalır;
  sonradan yalnız kanıtlı ata SHA'ya tek yönlü zenginleştirilebilir. Koruk v1
  tabanını okuyarak v2 göçünü korur; yayımlanmış kimlik/fixed/garanti yeniden
  yazılamaz. İlk commit-başlığı sezgisi K-155A ile yürürlükten kaldırılmıştır.
- **B-064 · KAPALI (K-155A/ADR-053) — Mesajdan bağımsız compiler semantic
  değişiklik beyanı.** `1c73298…` sonrasındaki her `compiler/src` commit'i tam
  SHA ile tekil `semantic-bugfix`, `semantic-change` veya `maintenance` beyanı
  taşır. Bugfix exact fixture, semantic değişiklik normatif belge, maintenance
  en az 40 karakterlik açık gerekçe ister. Geçici gerçek Git deposu, düzeltme
  kelimesi taşımayan commit'in dahi beyansız geçemediğini kanıtlar.
- **B-065 · KAPALI (K-176/ADR-054) — Strict form URL kodlaması.** Eksik veya
  hexadecimal olmayan `%xx` ve çözüm sonrası geçersiz UTF-8 artık rota öncesi
  400'dür; kayıplı dönüşüm kaldırılmıştır. Hedefli web testi ile `web` kipli
  kalıcı fixture çalışır; `fixed_by=32247c7…` aynı exact SHA'yı compiler
  değişiklik beyanında taşır.
- **B-066 · KAPALI (K-156/ADR-055) — Fuzz corpus kalıcılığı.** Lexer/parser,
  morfoloji, HTTP ve WASM ABI hedeflerinin koşu sonu coverage korpusu,
  başarı/başarısızlıktan bağımsız 90 günlük artefakta yüklenir. Manifest kaynak
  commit, run/attempt, araç sürümleri ve her seed'in göreli yolu+SHA-256'sını
  taşır. Cache yalnız hızlandırmadır; repoya terfi doğrulama, `cmin`, stable
  replay ve insan review'u ister.
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
- **B-043 · KAPALI (K-118) — spec↔code kanıt haritası.** Bugünkü 25 RFC, 50
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
- **B-067 · KAPALI (K-157) — uzun fuzz RC kapısı.** Dört hedef aynı kaynak
  commit'inde 30'ar dakika explicit AddressSanitizer altında toplam
  136.789.564 girdi yürüttü; crash, timeout ve sanitizer bulgusu sıfırdır.
  Seçili ondalık/zaman Miri testleri 3/3 geçti. Haftalık/elle CI 60 dakika,
  90 günlük log+sonuç+korpus provenance artefaktı ve append-only tarihçe ister.
  ADR-056 ve [RC fuzz rehberi](fuzz-rc.md) bağlayıcıdır.
- **B-068 · KAPALI (K-158) — gerçek blast radius.** Matris v2, tam bir
  birincil faz sahipliğini korurken exact test seçicisine isteğe bağlı çoklu
  `ek_kapsam` bağlar. Geçersiz bağlar fail-closed reddedilir; rapor birincil
  test, çapraz gerçek kanıt ve mimari aşağı akışı ayrı gösterir. ADR-057
  bağlayıcıdır.
- **B-069 · KAPALI (K-159) — public/internal API sınırı.** Desteklenen Rust
  gömme sözleşmesi exact `dil::api::v1` facade'ıdır. Kök legacy modüller
  `doc(hidden)` internal sınıfındadır; smoke, exact export allowlist'i ve yeni
  kök modül sızıntısı test kapısıdır. ADR-058 ve public API politikası bağlayıcıdır.
- **B-070 · KAPALI (K-160) — paket sahipliği ve sıfır SCC.** Davranışsız
  `paket_modeli`, resolver, registry taşıması, `artefakt_dogrulama` güveni ve
  `yayin` orkestrasyonu fiziksel sahiplerdir. Registry artefaktı doğrulamaya
  devreder; resolver taşıma/imza ayrıntısını bilmez. Production SCC ve izin
  sayısı sıfırdır; ADR-059 bağlayıcıdır ve CORE FREEZE etkindir.
- **B-071 · KAPALI (K-160A) — executable CORE FREEZE.** Her yeni compiler
  kaynak commit'i semantic beyanına ek olarak freeze sınıfı taşır. Feature
  yalnız dogfood/security/correctness kanıtıyla; `dogfood-change` ise ürün,
  K-işi, reproducer, etkilenen proje, minimalite ve ADR/spec/RFC'nin tamamıyla
  geçer. Geçici Git testi maintenance kaçışını ve beyan yeniden yazımını
  reddeder; CI koruğu çalıştırır.
- **B-072 · KAPALI (K-167) — eski `tedarik` cephesinin ömrü.** Depo içi ve
  bilinen dış tüketicisi sıfır, ADR-058 ile zaten internal sınıfındaydı;
  DEP-004 kaydıyla kaldırıldı, kurulum katmanı `artefakt_dogrulama` sahibine
  taşındı ve production sahip sayısı 37 oldu. ADR-064 bağlayıcıdır.
- **B-073 · KAPALI — freeze kanıt referans bütünlüğü.** Append-only dogfood
  ürün kaydı tekil slug, repo içi kanıt kökü, exact harici ürün commit'i ve
  active/retired durumu taşır. Freeze kapısı K-işinin backlog/günlükte varlığını,
  etkilenen `.dil` yolunun etkin ürün kökü altında oluşunu ve karar belgesinin
  K-işi ya da ürün kimliğine açık referansını doğrular. Geçici Git testi
  kayıtsız ürün/işi, kök dışı dosyayı ve alakasız kararı reddeder. Reproducer'ın
  ihtiyaca semantik uygunluğu insan review'unda kalır; CI bunu kanıtladığını
  iddia etmez.

## Bir sonraki somut kapı

İnsan kanıtı hattında B-001 ve B-002, doldurulmuş gerçek usability formları
ile önceden ilan edilmiş eşikleri bekler. Makine hattında K-153 gerçek
process→stdio LSP cold-start yolunu exact 25 örnekli tabanla kapattı.
K-154 tam-metin değişim ölçek eğrisini ve invalidation sınırını exact tabanla,
K-155 semantic regresyon provenance zincirini v2 manifeste taşıdı; K-155A
mesajdan bağımsız fail-closed beyan kapısını kurdu; K-176 strict form decode'u,
K-156 cache dışı fuzz korpus kalıcılığını, K-157 dört hedefte toplam
136.789.564 yürütmeli uzun AddressSanitizer kampanyasını ve 3/3 Miri kanıtını
kapattı. K-158 seçici düzeyi çoklu kapsam ve gerçek blast-radius raporunu
kapattı. K-159 sürümlü facade ve SemVer sınırını, K-160 paket sahipliği
ayrımını ve son SCC'yi kapattı. CORE FREEZE sonrasında sıradaki kanıt hattı
K-160A executable freeze kapısını da kapattı. K-163 ilk gerçek Zee ürünü,
`legacy-panel` deposundaki temiz `codex/k-163-legacy-panel` dalında başladı.
`43171b9 → cef5ae3` hattındaki 495 satır Zee; duyuru, yönetici oturumu/rol,
CSRF CRUD, JSON site ayarları ve slug doğrulamalı taslak/yayında sayfa yaşam
döngüsünü taşır. 7/7 hermetik test ile gerçek TCP içerik+ayar+sayfa provası
geçmiştir. Dördüncü dilim 492 Zee satırında yerel PostgreSQL/migration,
parameterized bind, 23505 ve rollback kanıtını ekledi. F030, 970 Zee LOC/4
modül/10 testte binary upload/hash, publish,
tombstone/silme ve restart orphan uzlaştırmasını gerçek PG16.11 ile kapattı.
F031 production TLS ve kontrollü havuzu pinned CA/hostname, 4 bağlantılık
exhaustion, stale recovery, idle/lifetime ve shutdown saha kanıtıyla kapattı.
F032 ürün hattını 1037 gerçek Zee LOC/5 modül/12 teste taşıdı; DB-down
readiness 503 olurken liveness 200 kaldı ve backend dönüşünde aynı worker
readiness 200'e döndü. 1000+ skor kartında LSP full-change p95 41,416 ms,
dogfood commit'i başına dokunulan dosya medyanı 5,5'tir. Exact final ürün
kanıtı `bb8e1ac` commit'indedir; K-163 kapalıdır. Managed-provider ve
çok-worker toplam bütçe K-169/K-173; multipart/içerik güvenliği K-170
hattında açık kalır. K-177 uzak CI görmemiş 126 commit için bütün kapıları
yerelde koşup tek kırık olan kritik işlev eğilimini bölmeyle kapattı. K-167
uyumluluk sözünü RFC-0028/ADR-064/spec-27 ile yürütülebilir yaptı ve B-072'yi
kapattı. K-171 spec maddelerini exact test işlevine bağlayıp 16 gerçek kanıt
boşluğunu adlandırdı. K-170 güvenlik kanıtını tek kapıda birleştirip açık
kritik/yüksek bulgu sıfırını makinece zorunlu kıldı. K-172 dogfood korpusunu
kurup iki güvenlik test boşluğunu kapattı. K-168 sürüm ikilisini iki temiz
klonda eş özetle, SBOM/provenance/imzayla üretilebilir yaptı. K-173 uzun soak
kapısını ve ilk 30 dakikalık sızıntısız tabanı kaydetti. K-166 tanı kalitesini
ölçülebilir yapıp ilk altı ihlali kapattı. K-169 kurulum/kaldırma sözleşmesini
ve runbook'u kurdu. K-164 ikinci gerçek ürünü farklı iş yükünde (kanıt özeti
aracı) yazıp elmas birim içe alımı ve birim kökenli tanı kusurlarını kapattı;
K-165 aynı aracı Rust/Go ile bayt bayt eşdeğer yazıp veri temelli karşılaştırdı.
Üçüncü incelemenin makine ve ikinci ürün hattı burada bitti. Ardından K-178
yürütme yığınını ölçüp derinlik sözünü resmî yığında test etti; K-179 üçüncü
ürünle (kelime avı) sürtünme tekrarını ölçtü ve K-180 sıra erişimini RFC-0029
taslağı olarak açtı; repo public olunca K-182 ilk gerçek Tier-1 koşularının
(CRLF, cargo uplift, ReplaceFileW okuyucu, Windows lint'leri) bulgularını
kapattı ve `dil`/tedarik/tatbikat/fuzz/soak beş workflow'u üç platformda
yeşile çekti; K-183 kalıcı kilit açlığını adil beklemeyle (ADR-075) kapattı.
K-174 freeze takvimi ve K-175 RC ile K-161/K-162 insan kanıtı bekler; usability
kiti RFC-0029 için de kart taşır.
K-161/K-162 gerçek insan testleri de insan verisini beklemeyi sürdürür.
İnsan verisi gelmeden yeni syntax seçilmez
veya B-001/B-002 tamamlanmış gösterilmez.

## 2 Eylül 2026 üçüncü dış inceleme — savunulabilir V1 yol haritası

Bu sıra “kusursuzluk” iddiası değil; bilinen mimari borç bırakmayan,
ölçülebilir kapılardan geçen V1 hedefidir. Önceki işlerle örtüşen maddeler
tamamlanmış sayılmaz; burada istenen ek kanıt ayrıca üretilir.

| Kayıt | Durum | Bağlayıcı çıktı |
|---|---|---|
| K-150 | **KAPALI** | SCC kapısı ve ilk iki kırılmış çevrim; son geçici izin K-160'ta kaldırıldı |
| K-151 | **KAPALI** | Bütün workflow action'ları immutable SHA + kontrollü yenileme |
| K-152 | **KAPALI** | Gerçek Git SHA/milestone ve tam benchmark provenance şeması |
| K-153 | **KAPALI** | Engine/process ayrımı, gerçek ikili testi ve exact 25 örnek taban |
| K-154 | **KAPALI** | 2k/5k/10k/20k full-change eğrisi, invalidation sınırı ve üç ilk-aşım eşiği |
| K-155 | **KAPALI** | V2 manifestte exact `fixed_by`, kanıtlı `introduced_by` ve garanti sürümü |
| K-155A | **KAPALI** | Her compiler kaynak commit'inde mesajdan bağımsız semantic beyan; bugfixte exact fixture |
| K-176 | **KAPALI** | Strict `%xx`/UTF-8 form reddi, rota-öncesi 400 ve exact SHA'lı web regression fixture'ı |
| K-156 | **KAPALI** | Her hedefte SHA-256/run/commit provenance'lı 90 günlük korpus artefaktı ve review'lü seed terfisi |
| K-157 | **KAPALI** | Dört hedefte 30 dk, toplam 136.789.564 yürütme; sıfır crash/timeout/ASan ve 3/3 Miri |
| K-158 | **KAPALI** | Exact seçicide opsiyonel çoklu `ek_kapsam`; birincil/çapraz/aşağı-akış blast radius |
| K-159 | **KAPALI** | Exact `dil::api::v1` allowlist, internal kök sınıfı ve SemVer politikası |
| K-160 | **KAPALI — CORE FREEZE** | Saf paket modeli, resolver, registry, verification ve yayın sahipliği; production SCC/izin sıfır |
| K-160A | **KAPALI — EXECUTABLE FREEZE** | Exact commit sınıfı; dogfood feature için ürün+iş+reproducer+proje+minimalite+karar |
| K-161 | **İNSAN KANITI** | B-001/K-016 için 10 öğrenci/çocuk + 5 profesyonel kör oturum |
| K-162 | **İNSAN KANITI** | B-002/K-093 morphology/scope/call/error zihinsel model oturumu |
| K-163 | **KAPALI** | legacy panel 1037 Zee LOC/5 modül/12 testte gerçek PG16.11, TLS/bounded pool, binary saga, failure reconciliation ve F032 liveness/readiness zincirini geçti. Bakım skor kartı LSP p95 41,416 ms ve dosya medyanı 5,5 ile kapalı; exact ürün `bb8e1ac` |
| K-164 | **KAPALI** | ADR-072 `dogfood/kanit-ozeti`: toplu veri işleme (TSV → Markdown) ikinci ürünü, 10 birim/19 test, `docs/kanit-ozeti.md` bayt bayt tazelik kapısı; 16 sürtünme (F001–F016), elmas birim içe alımı ve birim kökenli tanı düzeltmesi, korpus `proje` kipi ve 22 yeni vaka |
| K-165 | **KAPALI** | ADR-073: aynı araç Rust ve Go ile bayt bayt eşdeğer; satır/test/derleme/çalışma/RSS/sürtünme exact SHA tarihçesinde (`docs/dogfood-karsilastirma-v1.tsv`), `dogfood_karsilastirma_testi` CI'da |
| K-166 | **KAPALI** | ADR-070 `tani_kalitesi`: 244 vaka/14 sınıf mutasyon korpusu; S007 öneri, gövde dışı parametre S004 ve A001/A007 tek tanı düzeltmeleriyle 6 ihlal → 0; öneri %100, işaret ≥ %90, gürültü ≤ 2/4 CI'da |
| K-167 | **KAPALI** | RFC-0028/ADR-064/spec-27: sekiz dondurulmuş yüzey envanteri, süreli `DEP-NNN` deprecation kaydı, `-dev` sürüm kimliği ve B-072 kaldırması `uyumluluk_testi` ile CI'da |
| K-168 | **KAPALI** | ADR-068 `surum-artefakti.sh` + `surum_artefakti`: iki temiz klonda eş SHA-256, SPDX SBOM, SLSA provenance, `zee-surum-imza-v1`; `surum-adayi` workflow'u etikette koşar |
| K-169 | **KAPALI** | ADR-071 `kur.sh`/`kur.ps1` + `kaldir.sh`/`kaldir.ps1` (SHA256SUMS doğrulama, manifest, no-clobber), `kurulum_testi`, üç platform `kurulum-tatbikati` workflow'u ve `docs/surum-runbook.md` |
| K-170 | **KAPALI** | ADR-066 `guvenlik-kapisi.sh` + `GB-NNN` bulgu kaydı + SECURITY.md: 17 kapalı/4 kabul/1 açık orta, açık kritik/yüksek sıfır; sürüm adayı kipi exact HEAD RC fuzz + clippy + tam test ister |
| K-171 | **KAPALI** | ADR-065 `spec_drift`: 169 normatif spec maddesi parmak-izi kimliği + exact test işlevi; 151 kanıtlı/16 kısmi/2 açık deterministik raporla CI'da; RFC/ADR belge düzeyinde kalır |
| K-172 | **KAPALI** | ADR-067 `dogfood/korpus-v1.tsv`: 10 vaka (4 gereksinim aynası + F007/F012/F030/F032 ret/çözüm çiftleri) ürün politikasıyla CI'da; manifest dışı dogfood kaynağı reddedilir; GB-020/021 kapandı |
| K-173 | **KAPALI** | ADR-069 `soak`: derleyici döngüsü + gerçek `dillsp`, 5 sn RSS örneği, ısınma sonrası %10+32 MiB çift eşik, exact SHA tarihçesi ve haftalık workflow |
| K-174 | **BAŞLADI (14 Ekim → 11 Kasım 2026, K-220)** | `docs/v1-syntax-freeze-v1.tsv` kalıp/koşul/komut yüzeyinin özetini dondurur; `uyumluluk_testi` pencere boyunca özeti birebir ister. Pencere sıfır syntax değişikliğiyle dolunca kapanır; K-161/K-162 insan verisi bir değişiklik isterse pencere bilinçli olarak yeniden başlar |
| K-175 | **AÇIK** | İnsan+iki proje+üç platform+signed reproducible V1 RC |
| K-178 | **KAPALI** | Ölçüm: yorumlayıcı çerçevesi debug ~194 KiB/düzey, release ~11 KiB/düzey (macOS arm64 = Linux aarch64). Eski 32 MiB yığın debug CLI'yı 167. düzeyde doğal taşmayla düşürüyordu; resmî yığın 256 MiB rezervasyon oldu, regresyon koşucusu ve derinlik testleri resmî yığında; `derinlik-499-resmi-yiginda` vakası ve bütçe testi kapıda |
| K-179 | **KAPALI** | ADR-074 `dogfood/kelime-avi`: üçüncü ürün (etkileşimli CLI oyunu, 3 birim/7 test, `kelime_avi_testi` hermetik); sürtünme tekrarı ölçüldü — 2 yeni (F001 iyelikli döngü adı A003, F003 `metnin satırları` T028), 4 tekrar (F010 ×3 yer, F009, F013, F011), 5 kılavuzla önlenen; compiler değişikliği yok
| K-180 | **AÇIK** | Liste sıra ile öğe erişimi iki bağımsız üründe (K-164/F010, K-179) beş yerde sayaç döngüsüne zorladı; syntax freeze (11 Kasım) sonrası RFC adayı, freeze içinde açılmaz. F001/F013 iyelik zinciri `zee-tr-2` morfoloji adayına eklendi; RFC-0029 taslağı sorunu, adayları (`listenin sıra. öğesi`, 1 tabanlı, aralık dışı çalışma hatası + `almayı dene`) ve seçim ölçütlerini önden bağladı |
| K-182 | **KAPALI** | İlk gerçek uzak Tier-1 koşusu (`b172957`, repo public): Windows autocrlf `.md`/`.tsv` dosyalarını CRLF indirip bayt karşılaştırmalı faz matrisini bozdu (`* text=auto eol=lf`); ubuntu `olcum` eksik provenance alanını söylemiyordu (alan adı + uname yedeği); kalıcı rate-limit testi 100 fsync'li yazmayı 5 sn kilit sınırına sıkıştırıyordu (4×5). Tatbikat üç platformda ve tedarik yeşil |
| K-184 | **KAPALI** | Public depo güvenliği: özel IO izi yazımı, platform izin/ACL testleri, SECURITY belgesi, Gitleaks geçmiş/ağaç CI taraması. ADR-026 ve güvenlik kayıtları GB-024/025. |
| K-186 | **KAPALI** | ADR-077: Zee ERP korumalı rotalarında doğrulanmış kullanıcı → security-definer resolver → transaction-local `app.tenant_id`; aynı leased bağlantı, iç eylemlerde savepoint, hata/timeout rollback ve public rotada sıfır tenant bağlamı. |
| K-187 | **KAPALI** | ADR-078: Migration transaction yasağı yalnız üst seviye SQL tokenlarında uygulanır; PL/pgSQL/dollar-quoted gövde, string, quoted identifier ve yorumlar yanlış C026 üretmez. |
| K-188 | **KAPALI** | ADR-079/GB-028: Zee ERP için güvenli özet kimlikli oturum metadata listesi ve transaction kapsamlı tekil iptal; ortak kalıcı depoda worker/restart revoke, mevcut oturumda çerez silme, ham bearer/CSRF sızıntısı yok. K-174 syntax freeze 30 Eylül–28 Ekim olarak yeniden başladı. |
| K-189 | **KAPALI** | Native HTML inline scriptleri içerik SHA-256 CSP kaynağına bağlandı; `unsafe-inline` açılmadan dinamik ERP kabuğu çalışır, exact başlık testi ve freeze beyanı vardır. |
| K-183 | **KAPALI** | ADR-075: bekleyen yazar 5 ms yoklamak yerine yardımcı iş parçacığında bloklayan `flock`/`LockFileEx` ile çekirdek kuyruğuna girer; 5 sn sözü ve C013 aynı. `iki_yazar_satir_kaybetmez` 2×40'a döndü, `kilidi_ardisik_yeniden_alan_yazar_bekleyeni_ac_birakmaz` 120 ardışık ekleme altında tek eklemeyi 2 sn içinde ister; platform katmanı `kalici_dosya/platform.rs` |

| K-210 | **KAPALI** | ADR-083 `dogfood/urun-e`: dördüncü ürün (zee ile ilk müşteri sitesi, adminpanelzee çatalı). legacy panel+ürün E'de tekrar eden dört eksik dile alındı: `parolanın argon2 özeti`, `yeni totp anahtarı` / `kod gizli anahtarıyla doğrulanıyorsa`, `alıcı konu gövde ile ileti göndermeyi dene` (SMTP, geri alınamaz etki, `e_posta_bağlantı_değişkeni`, smtp/smtps ağ hedefi), `kaynak görselini hedef dosyasına N piksele küçültmeyi dene` (`image` crate). K-174 yüzey kaydı ürün sahibi kararıyla güncellendi; C028/C029 tanıları; `urun_e_yetkinlik_testi` 8 test |

## Önceki kanıt: 2 Eylül 2026 ikinci dış inceleme ayrımı

- **Doğrulandı ve K-129/K-130/K-131/K-132 ile kapandı:** B-025 kaynak/token,
  proje toplamı, bounded stdin/dosya okuması, heap/metin/çıktı, koleksiyon,
  görev, LSP toplamı/outbound ve süreç-geneli bağlantı sayısı. Eski sabit göçü
  bitti; LSP'nin 8 MiB reddi JSON kurulurken uygulanır.
- **Doğrulandı ve K-133/K-134 ile kapandı:** B-026 cancellation; etki öncesi
  deadline, görev HTTP tazeliği ve web session/cookie/yanıt transaction'ı.
- **Kapatıldı:** B-029'un taşıma/cache/kalıcı rollback/offline dilimi K-135,
  exact manifest/kilit v3/CLI ve kaynak kurulumu K-136 ile tamamlandı.
  B-046 ortak kalıcı oturum/rate-limit, kanonik proxy kimliği ve N tek-worker
  süreç modeli K-137/ADR-034 ile kapandı. B-051 kesin JSON-RPC ayrıştırması
  K-138/ADR-035 ile kapandı. B-052 origin tekilleştirme ve loopback peer sınırı
  K-139/ADR-036 ile kapandı. B-053 byte HTTP+fuzz sınırı K-140/ADR-037 ile,
  B-054 advisory/lisans/lock/offline-vendor sınırı K-141/ADR-038 ile,
  B-055 sürümlü kayıtlı WASM C ABI sınırı K-142/ADR-039 ile, B-056 playground
  ön-tahsis bütçesi K-143/ADR-040 ile, B-035 kritik işlev boyutu/karmaşıklık
  eğilim kapısı K-144/ADR-041 ile, kanonik Rust biçim kapısı K-145/ADR-042 ile,
  tam sahipli faz test matrisi K-146/ADR-043 ile, 17 vakalı kalıcı semantic
  regression korpusu K-147/ADR-044 ile, p50/p95 performans tarihçesi ve trend
  artefaktları K-148/ADR-045 ile, production modül/kenar/katman yönü
  K-149/ADR-046 ile kapandı. İkinci incelemenin makine backlog'u burada bitti;
  üçüncü inceleme K-150–K-175 sırasını ekledi. B-001 ve B-002 gerçek usability
  verisini bekler.
  B-033/B-034 temiz snapshot hattı ayrıca kapalıdır.
- **Mevcut repoda zaten kapalı:** çağrı derinliği C019/500 ve ayrı regresyonu;
  atomik metadata `unsafe` bloklarının her birindeki `SAFETY` gerekçesi; kök
  `.gitignore`; üç platformlu `.github/workflows/ci.yml`; tekil P2 başlığı.
- **Arşiv kaynaklı bulgu:** CI dosyasının yokluğu, `target`, `__MACOSX` ve
  `.DS_Store` sızıntısı gerçek Git ağacından değil, incelemeye gönderilen ZIP
  üretiminden kaynaklanır. Bu nedenle bulgu silinmedi; B-033/B-034 altında
  üretim hattı problemi olarak tutuldu.

## 12 Ekim 2026 — Derleme ve bağımlılık hattı

İnsan kanıtı B-001/B-002 ve K-174 penceresi değişmez. Yeni compiler özelliği
CORE FREEZE ve ürün kanıtı olmadan açılmaz. Bu hat araç/dağıtım doğruluğuyla başlar.

| Kayıt | Durum | Çıktı ve bağımlılık |
|---|---|---|
| K-191 | **KAPALI** | Derleme/bağımlılık envanteri; release HEAD pini, host/rustc/sysroot/build aynı toolchain, offline vendor çalışma ağacı pini; olumlu/olumsuz shell giriş regresyonları ve CI kablosu |
| K-192 | **KISMEN** | Exact `30dcfae` kaynağında üç platform kur/program/LSP/kaldır tatbikatı geçti; raporlar ve hash'ler kalıcı defterde. Linux ikilileri Rust araçları bulunmayan, ağsız/read-only Ubuntu 24.04, Debian 12 ve AlmaLinux 9.8 (glibc 2.34) userland container'larında program+LSP testini geçti. Kanonik uzak exact kaynak erişimi doğrulandı; temiz fiziksel Windows/macOS, minimum OS ve tüm native C araçları açık; linker driver/SDK kaydı K-196 ile kapalı. [Platform kanıtı](derleme-platform-kaniti-v1.tsv) |
| K-193 | **AÇIK** | K-192 ve mevcut V1/ürün kapıları sonrası: ürün kanıtlı native/WASM backend değerlendirmesi ve semantik eşdeğerlik ölçütleri; kod üretimi henüz yok |
| K-203 | **KAPALI** | ADR-081 `dogfood/zeedili-com`: 597 Zee LOC/4 birim/9 test; on SEO sayfası, gerçek WASM C ABI v3 derleyicili `/playground/` çalışma alanı, altı çalışan örnek, düzenleyici+girdi+tohum+çıktı/tanı+indirme+yerel paylaşım, F003 ile kod-öncelikli dil platformu tasarımı (ana sayfada aynı WASM ile çalışan deneme kutusu, yüklem-odaklı vurgu, açık/koyu tema, 22 çalıştırılabilir örneğin tamamı gerçek derleyicide yeşil, `blob/main` 404 bağlantıları `master`a düzeltildi), robots/sitemap/JSON-LD, resmî standalone Zee sembolü + kurumsal palet + favicon/PWA ailesi, güvenli Nginx/systemd üretim topolojisi ve sunucu A worker dağıtımı tamam. Kaynak kodu cihazdan çıkmaz; WASM SHA/ABI kapılıdır. Stok/AI görseli ile `Zee ERP` yazı kilidi/sloganı dil ürününe taşınmadı. 7 Ekim 2026: https://zeedili.com public yayında — Cloudflare proxy'li A kayıtları (sunucu A 80/443 yalnız Cloudflare'e açık), DNS-01 Let's Encrypt sertifikası, `scripts/canli-dogrula.sh` ile 14 uç 200, 308/301 yönlendirme, WASM MIME ve CSP kanıtı; compiler değişmedi. F005 (12 Ekim): kimlik resmî antrasit+altın zee logosuna, kağıt/turkuaz/mercan paletine ve aynı-origin OFL yazı tiplerine taşındı; `/playground/` 15 derslik “30 dakikada Zee” akışı oldu, 15/15 ders gerçek WASM derleyicide beklenen çıktıyı verdi. |

Kapsam ve yeniden üretim komutları [derleme envanterindedir](derleme-ve-bagimliliklar.md).

### K-194 — Kurulum hata geri alma (K-192 sırasında bulunan correctness dilimi)

**KAPALI — üç platform kanıtlı.** İkinci checksum reddinde ilk
ikili hedefte manifestsiz kalıyordu. Unix/Windows betikleri önce bütün kopyaları
geçici alanda doğrular, sonra no-clobber yayımlar ve hata halinde kendi
kurduğu dosyaları geri alır. Unix gerçek ikili testi bozuk/eksik/mevcut ikinci
hedef, yinelenen kayıt, ikinci yayınlama hatası ve kırık bağ reddini geçti.
GB-032 kaynak metadata manifest enjeksiyonu da exact SHA/yol doğrulaması ve
yabancı dosyalı regresyonla kapandı. Aynı ana test Windows'ta gerçek PowerShell betiklerine açıldı ve uzak ana
CI'da geçti; `30dcfae` kaynağındaki `38238657801` üç platform tatbikatı
kur/çalışma ortamı/kaldır akışını geçti. GB-031 kapandı; ADR-071. Bu atomik correctness
bulgusu K-192 dağıtım kanıtı ve K-193 backend işinden önce kapatılır.

### K-195 — Windows statik CRT ve çift-klon dağıtımı

**KAPALI — üç platform exact kaynak kanıtlı.** K-192 gerçek DLL envanteri
VCRUNTIME140 bağımlılığını gösterdi. MSVC release statik CRT/explicit target
ve `/Brepro` linker tarifiyle üretilir; Windows da iki temiz klonda eş hash
ister. `c7e3d04adf27905ade8b622d24bd76762123d2dc` kaynağındaki
[38244549913 tatbikatı](https://github.com/itwise-bilisim/zee/actions/runs/38244549913)
üç platformda geçti. Windows CRT importu yok; kur/program/LSP/kaldır ve iki
ikili eş hash başarılı. Ham runtime/provenance ve kaynak/job/artifact
özetleri [platform defterindedir](derleme-platform-kaniti-v1.tsv); ADR-068.
İlk Windows eş-hash reddi tarihsel kanıtta korunur; başarısız ikili yayınlanmaz.
Göreli çıktı yolu `5a4e81f` yerel tam provasını geçti. Shell fixture retleri
macOS Bash 3.2.57 ve Linux Bash 5.2.37'de doğrulandı. Temiz Windows/macOS,
minimum OS ve linker/SDK provenance K-192 içinde ayrı açık kalır.

### K-196 — Gerçek linker/SDK derleme kaydı (K-192 dilimi)

**KAPALI — exact üç platform üretim kanıtı depoda.**
İki klonun Rust 1.93.1 link izinden dil/dillsp için çağrılan program,
çözülen araç yolu, araç byte SHA-256 ve sürümü kaydedilir. Unix linker
verbose, Windows `/VERBOSE:LIB` SDK referanslarını görünür kılar. Ham gerçek
komut ve build SDK gözlemleri provenance `buildEnvironment` alanına girer;
eksik/bozuk kayıt release yayımlamadan reddedilir. Kapanma: exact kaynakta
iki-klon eş-hash ve dört link kaydı üç platformda doğrulanmış olmalı;
macOS SDKSettings/Windows Kits sürüm referansları ayrıca gözden geçirilir.
Tüm native C build-script araçları ve minimum OS desteği bu kayıt değildir;
K-192 açık sınırları korunur. ADR-068.

### K-197 — Temiz Windows container runtime kanıtı (K-192 dilimi)

**KAPALI — exact Nano Server runtime kanıtı depoda.**
K-195 statik CRT release byte'ları exact kaynak ve SHA256SUMS ile yürütmeden
önce doğrulanır; Nano Server LTSC 2025 immutable amd64 digest'inde yalnız
iki ikili ve merhaba kaynağı read-only mount edilir. ContainerUser/network
none, sınırlandırılmış system PATH, program ve gerçek LSP initialize/
shutdown/exit kanıtı istenir. Release ve prova kaynağı ayrı SHA'larla
kaydedilir. Windows process isolation host kernel'ini paylaşır; bu tüm
IO özellikleri, desktop/minimum Windows veya temiz fiziksel macOS kanıtı
değildir. Kapanma: exact kaynak/hash/image/host/prova kimliği olan gerçek
başarılı ham rapor depoya alınmış olmalı. ADR-068/GB-034.

K-197 kapanma: release `c7e3d04adf27905ade8b622d24bd76762123d2dc`,
prova `27c0fa9f566855c433e58607c3ecc9e66804059f`; run 38253777857/job
114826470199 geçti. Program/LSP, immutable Nano Server 10.0.26100.33438,
prova aracı byte hash ve release ikili hash'leri ayrıca eşleştirildi.
[Defter](temiz-windows-platform-kaniti-v1.tsv) ve [ham rapor](temiz-windows-runtime-kaniti-2026-10-12.json)
kalıcıdır. Kurulum/kaldırma bu container provasının kapsamında değildir;
K-192 fiziksel Windows/macOS, minimum OS ve bütün IO sınırları açıktır.

K-196 kapanma: exact `b1530641a3f902047b57ad6e7c0c004ec5f1c420`
run 38252942210 üç platformda iki temiz klon eş hash ve kur/program/LSP/
kaldır kapısını geçti. Linux GCC 13.3.0; macOS Apple Clang 21.0.0 ve SDK
26.5; Windows linker FileVersion 14.51.36260.0 ve SDK 10.0.26100.0 dört
kayıtta doğrulandı. İndirilen ikili byte'ları runtime hash'leriyle eşleşir;
ham provenance/runtime JSON ve artifact/job/hash defteri kalıcıdır. İlk
Windows çıktı biçimi/help sorgusu retleri geçmiş olarak korunur.

### K-198 — Temiz Windows kurulum/kaldırma kanıtı (K-192 dilimi)

**KAPALI — exact stock Server Core kurulum/kaldırma kanıtı depoda.**
K-197 yalnız hazır byte program/LSP kanıtıdır. K-198 stock Server Core
LTSC 2025 exact digest üzerinde aynı ContainerUser container'ında Türkçe
boşluklu kurulum, manifest exact path/hash/source doğrulaması, program/LSP
ve kaldırma sonrası iki ikili+manifest yokluğu ister. Installer/helper
byte hash'leri ve release/prova source kimlikleri raporlanır; yalnız bu
provanın UUID container'ı sonunda kaldırılır. Network none/read-only bind,
host kernel ve geçici root sınırı korunur. Kapanma: başarılı exact image/
source/kurulum/kaldırma ham raporu ve kimlik/hash defteri. Desktop/minimum
Windows ve fiziksel macOS K-192'de ayrıca açık. ADR-068/GB-035.

K-198 kapanma: release b153064, prova 33176cf; run 38255788989/job
114833388588 stock Server Core 10.0.26100.33438'de geçti. ContainerUser
Türkçe boşluklu dizinde exact manifest ile kurdu, program/LSP geçti,
kaldırma sonunda iki ikili+manifest yok; yalnız prova container'ı silindi.
[Ham rapor](temiz-windows-kurulum-kaniti-2026-10-12.json),
[defter](temiz-windows-platform-kaniti-v1.tsv); source/prova/helper/installer
hash'leri bağımsız eşleştirildi. Fiziksel/desktop/minimum OS ve tüm IO ayrı.

### K-199 — Cargo compiler/wrapper pin sınırı

**KAPALI — exact üç platform gerçek Cargo pin kanıtı depoda.**
K-191 helper yalnız RUSTUP_TOOLCHAIN seçiyordu. Gerçek yerel küçük Cargo
derlemesinde helper rustc 1.93.1 bildirirken RUSTC=1.97.1 yolu gerçek
derleyiciyi değiştirdi. Artık rustup which exact executable yolu/sürümü
denetlenir, RUSTC bu yola bağlanır, RUSTC_WRAPPER ve WORKSPACE_WRAPPER
boş değerle Cargo config/ENV wrapper seçimini kapatır. Unix/Windows path
semantiği, yanlış resolved path/sürüm retleri ve poisoned Cargo config/
env altında gerçek Cargo/build-script/ikili testi birlikte kanıtlanır.
Bu diğer Cargo profil/linker/CC/AR ayarlarının tam envanteri değildir.
Kapanma: exact kaynakta üç platform gerçek küçük Cargo pin raporu/hash
defteri ve giriş regresyonları. ADR-068/GB-036.

K-199 kapanma: exact 22ea2b8/run 38259341995 üç platformda geçti.
Poisoned compiler/wrapper ENV + Cargo home config altında gerçek Cargo/
build-script/ikili Rust 1.93.1 kullandı; source tree temiz, wrapper
alanları boş, helper/prova/pin byte hash'leri kaynakla eşleşir.
[Defter](rustc-pin-platform-kaniti-v1.tsv) ve raw fixture kayıtları kalıcı.
Tam Zee release veya tüm Cargo profil/linker/native C ayarları bu probe
kapsamı değildir; sonraki release normal iki-klon kapısından geçer.

### K-200 — macOS native C/Rust deployment uyumu

**KAPALI — exact üç platform iki-klon release tatbikatı geçti.**
Locked cc 1.4.4/Ring build izinde C `-mmacosx-version-min=27.0` ve
object minos 27.0 iken Rust final Mach-O minos 11.0 çıktı. Resmî release
Rust default deployment hedefini caller ENV dışında sorgular, C/Rust
MACOSX_DEPLOYMENT_TARGET'ını ortak seçer. İki klon native build-script .o
başlığı/hash'i incelenir; üst minos/yanlış platform/eksik toplam kanıt
release'i reddeder. Başlıksız assembly ayrı sınırdır. Yerel sonrası 24/24
nesne minos 11.0 ve Rust'sız PATH program/LSP geçti. Kapanma: exact üç
platform iki-klon tatbikatı ve macOS a/b native başlık kayıtları kalıcı.
Kaynak `44fba404c79762aff1698a07978c79ea069a59df`, run 38263264133;
macOS a/b ayrı ayrı 24/24 nesne minos 11.0. Ham provenance/runtime
ve artefakt hash/job kayıtları `derleme-platform-kaniti-v1.tsv` içindedir.
Gerçek macOS 11/desktop/minimum OS ve tüm C alt süreç envanteri değildir.
ADR-068/GB-037.

### K-201 — Native build-script araç çağrıları (K-192 dilimi)

**KAPALI — exact üç platform iki-klon release kanıtı depoda.**
Release iki temiz klonda `cargo build -vv` ve `CC_ENABLE_DEBUG_OUTPUT=1`
ile locked cc 1.4.4 driver komutlarını toplar. C compile ve archive çağrısı
ikisinde de bulunmadan artefakt yayımlanmaz. Her çağrı paket sahibi,
program/argüman ve çözülen dosya path/SHA-256 kimliğini taşır; sürüm
sorgusu başarısızsa `unavailable` açık kaydı vardır, sürüm uydurulmaz.
Komut metni shell olarak yürütülmez. Windows cl/lib sürümü FileVersionInfo
ile, destekleyen Unix araçları --version ile sorgulanır. Native çağrılar
`buildEnvironment.clones.{a,b}.nativeTools` alanındadır.
Bu cc driver kapsamıdır; driver iç süreçleri, pkg-config/vcpkg, tüm
Cargo build-script alt süreçleri ve minimum OS ayrı açık kalır. ADR-068/GB-038.

### K-202 — Ana GitHub deposu ve CI yönü

**KAPALI — kurucu talimatı, remote ve Actions ayarı doğrulandı.**
Ana depo merbay-erp/zee; enterprise itwise-bilisim/zee yalnız aynadır.
Enterprise repository Actions enabled=false, aktif koşu yok. SBOM ve
provenance aynı exact kaynak SHA ile merbay URI kullanır; eski run/job
kanıtları tarihsel olarak korunur. Metadata regresyonu yeni kaynağı
birebir denetler; compiler dil semantiği değişmez. ADR-068/GB-039.

K-201 kapanma: [merbay run 38349601833](https://github.com/merbay-erp/zee/actions/runs/38349601833),
exact 25b6882; Linux a/b 41'er, macOS a/b 36'şar cc driver çağrısı.
MSVC cl/lib FileVersionInfo ve Linux gcc/ar sürüm/hash kayıtları kalıcı;
Apple ar sürüm sorgusu unavailable olarak korunur. Bu dar cc driver
kapsamında kapandı; tam native subprocess ve minimum OS K-192'de açıktır.

### K-204 — Sır taraması tarihsel yanlış pozitif doğruluğu

**KAPALI — exact geçmiş satırları ve yeni anahtar reddi doğrulandı.**
K-202 kamuya açık belge ifadesi dört generic-api-key eşleşmesi üretti.
Immutable commit/dosya/kural/satır + kaynak SHA-256 kayıt doğrulaması,
bozuk/wildcard/tekrar/bayat hash retleri ve sentetik anahtar exit 1 testi
eklendi. Geçmiş/çalışma ağacı taraması temizdir; geniş allowlist yoktur.
K-202 beyan sonrası bayat kanıt özeti yeniden üretildi; kaynak beyanı
47, core freeze beyanı 43 oldu. ADR-066/GB-040.
K-203 ve ADR-081 site PR #6'ya ayrılmıştır; GB-049 çakışması birleşme
sırasında ikinci dalda ardışık kayıt+türetilen belge güncellemesi ister.

### K-205 — 27 maddelik dış inceleme ve adversarial kanıt eşlemesi

**KISMEN — ilk kaynak/karar/test eşlemesi hazır; hedefli denetim sürüyor.**
[13 Ekim eşleme tablosu](dis-inceleme-2026-10-13.md) ZEE-001..027 için
gerçek özellik eksikliği, bilinçli sınır, tarihsel ölçüm ve henüz
doğrulanmamış riski ayırır. Yazarın önerdiği öncelikler resmi sıra
yerine geçirilmedi. Kontroller varlığı bütün saldırı yollarının kapandığı
anlamına gelmez. Yeni güvenlik bulgusu reproducer/etki/GB kaydı ister.

### K-206 — PostgreSQL sonuç belleği ve hata matrisi incelemesi

**KISMEN — akış toplama, exact native kabul ve RSS kaydı kapalı;
tek wire satırı sınırı açık.**
16 MiB uygulama sonucu driver/wire bellek sınırı diye sunulamaz. İzole
sunucu ve özel geçici veriyle çok satır, tek büyük satır, timeout, retry,
transaction/savepoint ve sonraki bağlantı sağlığı birlikte ölçülür.
Mevcut veritabanlarına müdahale edilmez; ZEE-016/ZEE-018.

### K-207 — Ad çözümleme maliyeti ve büyük dosya LSP

**KISMEN — ad/kapsam optimizasyonu ve exact ölçüm/soak kapalı;
incremental/cache kapasitesi açık.**
Ad çözümleme her başvuruda bütün adları clone/sort ediyor. Exact-ad
önceliği ve morfoloji kök eşlemeleri hash üyeliğiyle sorgulanabilir;
A001/A002 tam mesaj/span/öneri ve sıralama korunmalıdır. Dil yüzeyi,
cache/persistent state ya da incremental parser eklenmez. K-154/ADR-013
ve ADR-045/049 etkisi birlikte incelenir; 20k <1 sn yalnız hedeftir.

K-207 ilk dilim ölçümü kapalı: source 7827d8c/beyan+ölçüm 94de420, temiz
taban 29be85b; 20k p95 22.169,906 → 2.961,638 ms, yaklaşık 7,5 kat.
727/727 faz ve 26.624 oracle kararı aynı gözlemi kanıtladı. Raw sample
ve exact tarihçe [ölçümlerde](olcumler.md). **K-207 KISMEN:** kapsam
ilk ölçümde checkpoint maliyeti ve 20k <1 sn hedefi açıktı. İkinci
dilim kapanışı aşağıdadır; incremental/cache kapasitesi açık kalır.

### K-207 ikinci dilim — kapsam checkpoint

Kapsam girişi bütün adların HashSet kopyası yerine monotonic tanım sıra
sınırını alır. Sembol tablosunun geçici yeni-ad journal'ı scope çıkışında
yalnız o sınırdan sonra doğan adları kaldırır. Dış ada atama ve SymbolId
kimliği kalıcıdır; silinen kimlik sayacı geri sarılmaz. Nested scope,
atama, yerel silme ve kimlik yeniden kullanmama testi mevcut kapsam/
semantic/golden testlerini tamamlar. Sorumluluk cozumleyici sahibindedir;
yeni modül/kenar ve ters katman geçişi yoktur.

Journal derleme başına düşer, LSP belge/cache alanına taşınmaz.
K-173/ADR-069 yeni geçici tampon kapısı aşağıdaki exact koşuda geçti.
Dil/tanı/API/profil ve blokta doğan adın görünürlüğü değişmez.

K-207 kapsam dilimi kapalı: temiz exact 9e80b69 release/25 örnek/2 ısınma
20k LSP p95 165,137 ms; aynı exact kaynakta 1800 sn/264.343 döngü soak
geçti. Derleyici RSS medyanı 8.144→8.160 KiB, LSP 19.088→17.568 KiB.
Ham örnek/çevre ve tarihçe [ölçümlerde](olcumler.md). 728/728 faz,
26.624 ad oracle ve 4.096 kapsam işlemi oracle karşılaştırması geçti.
K-207 KISMEN; full didChange ve incremental/cache kapasitesi açıktır.

## K-206 akışla sonuç toplama

Native okuma `query_typed_raw` ile satır satır ilerler. Boyut kontrolü
borrowed metin üzerinde String kopyasından önce yapılır; hata sonrası
birikmiş çıktı bırakılır ve kalan satırlar kopyalanmadan tüketilir.
Sunucu/transport hatası bütün uygulama hatalarından, toplam satır sınırı
ilk hücre hatasından önce gelir. Satır/sütun sırası, tekrar eden sütun
adları, TEXT bind, NULL/tür hatası ve structured hata verisi değişmez.
Yalnız transport hatası, transaction dışında kapalı bağlantıda mevcut
tek read retry yoluna gider; uygulama sonucu hatası retry tetiklemez.
Yazma/savepoint/commit kodu ve API yüzeyi değişmez.

Sonuç sahipliği `postgresql/sonuc.rs`, transport/pool eski sahibindedir;
production owner ve temel→adaptör yönü değişmedi. Katman fixture ve
ADR-046 incelendi: yeni sahip/kenar/SCC yoktur. Yeni bağımlılık, cache,
compiler/LSP uzun ömürlü tampon eklenmedi. Per-query birikim önce de
vardı; ömrü uzatılmadı, sürücüdeki tüm satır Vec'i kaldırıldı.

**KISMEN:** tek wire satırının driver tahsisi uygulama sınırından önce
gerçekleşir. GB-041 orta/açık kalır; bütün süreç için 16 MiB veya kesin
RSS bütçesi sözü yoktur. Exact native kabul/RSS kaydı aşağıdaki kapanışta saklıdır. Saf collector testleri ve eski 4a4fd0d Vec oracle'ı mevcut
hata/sonuç gözlemini denetler; gerçek DB testi izole loopback opt-in'dir.

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

## K-213 — ürün E ve master kaynak hattının birleştirilmesi

**KAPALI:** K-210/ADR-083, K-211/ADR-084, K-212/ADR-085 ürün E dalından
kaynak geçmişi korunarak aynı derleyicide birleştirildi. Source e09493e,
exact beyan/clean kabul 0531432. Beş bütün proje 242/242, 769/769 faz,
187 paket offline vendor, macOS arm64 iki-clone ve legacy zee1 ciphertext
uyumluluğu kanıtlıdır. ADR-086 kayıt eşlemesini, HIR/trace regresyonlarını
ve ham ürün E şablonunun katalog üretim sınırını taşır. Otomatik parola göçü,
canlı sunucu yükseltmesi ve K-192 minimum OS kanıtı ayrı açık işlerdir.

K-213 clean source kabulü: source e09493e, beyan/ikili/test 0531432;
769/769 faz, beş bütün proje 242/242, 187 paket offline vendor ve
macOS arm64 iki-clone eşliği geçti. Exact [kayıt](incelemeler/k213/README.md).
İlk Linux CI'ın test Git kimliği bağımlılığı düzeltildi; son uzak CI
kapanışı beklenir. Canlı dağıtım ve otomatik parola göçü açık kalır.

Son kod/test kanıt HEAD'i `2ef362818bf0dc6b8cf4a9951d8ef2a25303a152`: Linux,
macOS, Windows, tedarik ve sır taramasının 10 kontrolü geçti.
`dil` koşuları 38392031371/38392039728, `tedarik` 38392031424/38392039739.
Önceki Linux hatası tarihsel kalır; source e09493e değişmedi.

## K-214 — Açık depoda ürün gizliliği

Anonim içerik, bugünkü kaynak geçmişi ve exact-SHA beyanlarının yeniden
yazılması, müşteri/altyapı hash kapısı ve destek taslağı aynı dilimdedir.
Önceki geçişler kullanıcı kararı, GitHub PR refs temizliği destek teyidi
bekler. Depo kullanıcı onayı olmadan açılmaz. ADR-087 ve
[K-214 kaydı](incelemeler/k214/README.md) bu sınırı taşır.

K-214 makine dilimi **KAPALI**: 51 kaynak SHA eşlemesi, anonim
PR/dogfood kanıtı ve güvenilir master'dan çalışan CI kapısı; exact
290fa52 için altı uzak kontrol yeşil. Önceki geçiş kararı K-215 kapsamında
verildi; GitHub PR refs silinme teyidi ve yeniden açılış onayı **AÇIK** (GB-046).

## K-215 — Önceki geçişlerin anonimleşmesi ve path yama

Durum: **DEVAM — yerel makine dilimi tamam, uzak CI ve dış teyit açık**.
Kullanıcı K-214 önceki geçiş dökümünün temizliğini onayladı.
Kapsam: tüm yayımlanmış dal/etiket geçmişi, istisnasız CI, exact kaynak SHA
beyanları, path PostgreSQL yaması ve doğru SBOM. Kanıt:
[K-215 kaydı](incelemeler/k215/README.md). Özel ürün `ci/site-dil` pini
korunur. 18 yayımlanmış ref'in 406 commit geçmişi sıfır eşleşmeyle tarandı;
exact-SHA geçişi ana depoya gitti, 771/771 yerel test ve iki temiz klon
ikili eşitliği geçti. Uzak altı iş hesap ödemesi/harcama limiti nedeniyle
başlayamadı (0 test adımı); Linux/Windows/uzak kapı kanıtı açık kalır.
GB-046 GitHub PR refs/cache teyidi ve yeniden açılış onayı açıktır.

## K-216 — Git2 / Gitsrv CI

Durum: **DEVAM**. Kullanıcı koşuların Git2/Gitsrv üzerinden yapılmasını
istedi. Git2 gizli kalır; GitHub ana kaynak ve Enterprise ayna Actions'ı
kapalıdır. Linux kapıları ve seçili ref/ephemeral token sınırı ADR-088
ile kayıtlı; ilk exact uzak koşu #20'de 770/771 test geçti. Tek başarısızlık,
özgün ölçüm SHA'sının anonim geçmişteki kaynak soyunu doğrulama kontrolüdür.
K-214/K-215 exact eşlemesi ve HEAD atası şartıyla düzeltildi; eski ölçüm
verisi değişmez, eski kaynak nesnesi geri alınmaz. Yeni hermetik olumlu/
olumsuz vaka ve gerçek Go aracıyla uzak tekrar sonucu beklenir. Windows runtime
kanıtı yerine çapraz veya Linux test sonucu konmaz.

## K-217 — Tek merkez ve dal incelemesi

Ürün talebi §2: site temiz içerik, issue formları, özel Forgejo MSVC
çapraz akışı ve incelenmiş action pinleri ana hatta uygulanır. Snapshot
compiler/CLI genişlemesi core freeze kanıt zinciri olmadığı için kapatılır.
Dal yedekleri özel; uzak silmeler ve Gitsrv #24 CI teyidi tamamlandı.
[Dal kararları](incelemeler/k217/README.md). Dil/spec/tanı/API değişmez.

## K-218 — Oran sınırı sıcak yolunu kalıcı dosyadan ayır

Durum: **TAMAM — yerel ürün yükü, exact 1800 sn soak ve Gitsrv #24 sürekli CI kabulü geçti**. Sayaçların bellek içi/
PostgreSQL durumu; açık süreç sınırı, oturum/revoke korunumu, atomiklik,
boyut/expiry ve gerçek performans kanıtı aynı dilimde ele alınır.

K-218 kabul dilimi: gerçek zee-cekirdek 100 kartlık yük reproducer'ı, explicit
CLI bellek backend'i, atomik/süre/kapasite/iki-süreç testleri ve ADR-089/RFC-0031.
Exact 1800 sn soak ve gerçek ürün yük kabulü [K-218 kanıtında](incelemeler/k218/README.md); Gitsrv sürekli CI tamamlandı; Windows runtime ve sürüm adayı kapıları ayrı açıktır. §3/2 doluluk/kota,
§3/3 HTTP POST, §3/4 HTML temizleyici ve §3/5 LSP artımlılık açık kalır.

## K-219 — Oran sınırı ayarı ve korunmuş doluluk

Durum: **TAMAM — 786/786 yerel faz, exact 1800 sn soak, gerçek ürün yükü ve Gitsrv #26 geçti**. Ürün §3/2: bellek backend’i için
uç nokta eşiği, sınırlı kapasite ve explicit en-eski pencere tahliyesi.
Giriş/CSRF canlı kotası korunur; tümü korunmuş dolulukta 503 sınırı sürer.
[ADR-090](../adr/090-web-oran-ayarlari.md), RFC-0032, olumlu/olumsuz CLI,
clone/churn ve gerçek 100 kart worker testi aynı dilimdedir.
Exact 1800 sn soak, tam faz ve Gitsrv kapanışı [kanıt sayfasında](incelemeler/k219/README.md).
§3/3 HTTP POST K-220 ile kapandı; §3/4 HTML temizleyici ürün sahibinin yeni kanıtıyla ertelendi. §3/5 artımlılık
K-207’de açık kalır. Eski çok saniyeli süreler güncel master’da tekrarlanmadı;
[gerçek düzenleme ve ürün gözlemi](incelemeler/k207/http-master-20261014/README.md)
yeni optimizasyon veya artımlılık kapanışı değildir.

## K-220 — Ürün HTTP POST ve başlıklar

Durum: **KAPALI (dil makine kabulü)**. Ürün §3/3 ve 2d57060 exact talebi;
POST /iletisim-gonder Turnstile, ikincil Bearer portal bildirimi.
ADR-091 / RFC-0033; sınırlı native HTTPS GET/POST, Metin sözlüğü Sonucu,
explicit proje ortam listesi, iz kapatma ve kanonik HTTP JSON helper.
Olumlu/olumsuz dil, gerçek ürün helper ve localhost TLS testleri aynı dilimde.
801/801 yerel faz, güvenlik ve Gitsrv #29 sonucu [kanıtta](incelemeler/k220/README.md) kapandı. Sunucu/ürün rotası yükseltmesi ürün tarafındadır.
§3/4 HTML temizleyici ürün sahibinin yeni kanıtıyla ertelendi; §3/5 artımlılık
K-207’de açık kalır. Eski çok saniyeli süreler güncel master’da tekrarlanmadı;
[gerçek düzenleme ve ürün gözlemi](incelemeler/k207/http-master-20261014/README.md)
yeni optimizasyon veya artımlılık kapanışı değildir.

## İnceleme §3/4 — HTML temizleyici ertelendi

14 Ekim 2026 ürün sahibinin güncel talebi: `cekirdek/kaynak/alanlar.dil` richtext alanı düz metindir; `guvenliParagraflar` satırları `html güvenlisi` ile kaçışlayıp sabit etiketler üretir. Ürün tarafının beş veritabanı taramasında HTML etiketli kayıt 0 olarak bildirilmiştir; AI içerik kaynağı yoktur. Mevcut satır içi biçim eksikliği ürün işlemiyle giderilebilir, bugün yeni dil yüzeyi açılmaz. Gerçek HTML içe aktarma kaynağı gelirse dosya kanıtıyla yeniden değerlendirilir. Sıradaki inceleme maddesi §3/5 LSP yeniden analiz maliyetidir.

## K-221 — Ürünleşme A6 bilinmeyen CLI seçeneği

Durum: uygulama ve yerel kabul; exact kaynak/provenance kayıtları tamamlandı.
A6(1) minimal correctness: bilinmeyen `--` seçenek yol değildir, exit 2
ve etkin sürümle reddedilir. Program argümanları korunur.
[ADR-092](../adr/092-cli-bilinmeyen-secenek-reddi.md).
A6(2) runtime config ayrı açık kapsamdır; K-174 grameri değişmez.

## K-222 — Hermetik PostgreSQL TLS ve NULL kabulü

Yerel Docker PG16.15 ve native PG14.18 dört dedicated vaka sıfır atlamayla
geçti; GB-022 kapandı. Aktif Forgejo CI kablosu eklendi, yeni exact uzak
CI kabulü henüz yok. [Koşucu ve sınırlar](postgresql-tls-ci.md).
Default tam faz dört ignored testi ayrı gösterir; dedicated kapı 4/4 ister.

## K-223 — Mevcut dilde girişte parola yükseltme ve form kabulü

Anonim ürün helper/CAS ve Turnstile form kabulü mevcut primitive'lerle
sınanır. Ek native PostgreSQL 16.15 kampanyası 7/7, sıfır atlamayla
gerçek CAS/row-lock yarışı, rollback ve commit görünürlüğünü kanıtladı.
[Native koşucu](parola-goc-ci.md) CI'a bağlandı; temiz exact checkpoint ve
yeni uzak CI kabulü ayrıca kaydedilir. C027 ağ-fault/live ürün/staging açıktır.
[Rehash](../dogfood/zee-cekirdek/parola-goc/README.md).

## K-224 — Pilot destek ve kaynak kanıtı

Kullanıcı lisansı bekletti, sınırlı pilot desteği seçti. Kaynak/ZIP exact
eşlik raporu, private depo/GB-046 sınırı ve site kaynak metni tutarlıdır.
Destek taslağı gönderilmedi; site değişikliği dağıtılmadı.
[Onaylı iş matrisi](urunlesme-plani.md).

## K-225 — Beş doğrudan spec kanıt boşluğu

UTF-8 ret, S005 girinti, T015 çağrı sayısı, C005 girdi bitişi ve
parametresiz public T039 ret/olumlu yolları mevcut test hedeflerinde
doğrudan kanıtlandı. Production davranışı veya grammar değişmedi.

## K-226 — Deterministik compiler build kimliği

A1 SemVer/SHA/source UTC tarih/kanal/dirty JSON; CLI/LSP/worker aynı kayıt.
[ADR-093](../adr/093-deterministik-build-kimligi.md). Yeni grammar yok;
asıl dil iki-clone release ve yeni exact uzak CI kabulü ayrı kapıdır.

## K-227 — Minimal ürün modül kompozisyonu

Yerel paket + host rota adapteri mevcut `rotalarını kullan` ile metin
ankrajı olmadan kur/kaldır kabulünü taşır; 2/2 CLI/proje kabulü.
[Exact anonim köken](../dogfood/zee-cekirdek/modul-kompozisyonu/README.md).
Genel modül yöneticisi, namespace/version-range/menu/content/göç açık kalır.

## K-230 — Güncel dış inceleme izlenebilirliği

[27/27 güncel tablo](dis-inceleme-2026-10-13.md) tarihi kaynak ile committed
K-221 ve kayıtlı yerel K-222/K-226 kanıtını ayırır. PGGB041/LSP kapanan
alt kapsamlar yeniden açılmaz; tam adversarial audit K-205'te açık kalır.

## K-229 — Biçimleyici CI denetimi

C2 --denetle dosya/proje modunda yazmadan exit0/1 verir; eksik kullanım
exit2. [ADR-094](../adr/094-bicimleyici-yazmadan-denetim.md). Lint, cache,
artımlı LSP ve JetBrains dağıtım hattı ayrı açık kalır.

## K-228 — PostgreSQL göç planı

`göçür --durum|--kuru` source/version/ad/hash/status karşılaştırır;
metadata tablo yokken oluşturmaz. [ADR-095](../adr/095-postgresql-goc-plani.md).
Native owner/runtime rol, checksum/kayıp dosya, rollback ve concurrent
apply kabulü dedicated kapıda sınanır; eski Python ürün betiği ayrı açık.

## K-235 — Spec kanıtı: DB kaynak sınırları ve sürüm eşliği

Production davranışı değiştirilmeden gerçek TLS PostgreSQL tam eşik/bir fazla
kampanyası ve var olan K-226 CLI/LSP/SBOM direct eşlik kanıtı iki kısmi
maddeyi kapatır. Dedicated kapı 5/5 ister; platform, edition, provider,
TLS1.1 ve süreç sözleşmesi boşlukları açık kalır.

## K-234 — Sürümlü runtime ayarları

`zee.ayar` mevcut çocuk/web/proxy/oran host ayarlarını taşır; unknown security
ret, ordinary no-op uyarı ve CLI precedence 17 yeni testle sınanır.
[Profil ve eski-sürüm sınırı](zee-ayar.md). Önceki küçük sürüm/full ürün
rollback ve fiziksel Windows kabulü açık; yeni grammar yok.

## K-232 — PDF/görsel ekiyle SMTP MIME

Gerçek cd5d12f B2 talebi tek typed intrinsic ve native readonly dosya/SMTP
adaptörüyle karşılanır. [ADR-096](../adr/096-ekli-eposta-ve-mime.md),
[RFC-0034](../rfcs/0034-ekli-eposta.md), [anonim ürün](../dogfood/zee-cekirdek/mime/README.md).
12 integration/16 native e-posta kabulü; queue/3retry/live M365/Gmail/ürün
katalog kabulü ve physical Windows açık. GB-058 aday FIFO riski kapandı.

## K-233 — Yerel exact compiler seçimi

[İmzalı katalog ve proje pini](derleyici-secici.md) pilot araçtır; 25 offline
kabul/ret gerçek Ed25519 doğrulayıcıyla sınanır. İki sentetik release ve
gerçek debug kopyası production iki-klon/release kanıtı değildir. Full
rustup/PATH shim/proje.kilit birleşimi, gerçek publisher ve Windows açık.

## K-236 — SMTP AUTH hata sızıntısı

GB-060 yüksek bulgusu native AUTH535 echo ve gerçek CLI kabulüyle kapandı.
[ADR-099](../adr/099-smtp-hata-maskelemesi.md): eski plain kalıpların C029
hata ayrıntısı sabit maskeye eşlenir; wire/deadline/failure akışı ve retry
olmaması korunur. Üç hedefli test ve [gerçek SMTP reproducer](../regression/security/smtp-auth-echo-sir.dil)
aynı kaynağı sınar. Exact fixed_by/compiler/freeze beyanı kaynak commit’ini
izleyen provenance kaydına bağlanır; inherited yanıt DoS K-237 ayrı iştir.

## K-237 — SMTP yanıt okumasında tahsisten önce sınır

**KAPALI — yerel güvenlik/correctness kabulü.** [ADR-100](../adr/100-smtp-yanit-okuma-siniri.md)/GB-061: mevcut 16 KiB bütçe kalan+1 Take ve 16 KiB + 1 bayt kapasiteli tek yanıt tamponuyla LF/EOF öncesinde zorlanır. Beş actualhelper ve gerçek loopback corpus kabulü; eski gövdeyle 1 MiB byte/deadline negatif ve restore pozitif kanıtı vardır. K-236 maskesi aynı; grammar/API/dependency/kalıcı state yoktur. Exact source/fixed_by beyanı takip kaydı, uzak CI/live/RSS/plain toplam deadline sınırı ayrıdır.

## K-238 — rustls advisory güvenlik yaması

Compiler ve fuzz lock'ları `ureq 3.4.0` üzerinden `rustls 0.23.46`ya
birlikte yükseltilir; RUSTSEC-2026-0346/0347/0348/0349 için ignore yok.
[ADR-101](../adr/101-rustls-advisory-yamasi.md)/GB-062 gerçek feature-tree
ve erişilebilirlik sınırını ayırır. Manifest/production source/grammar/API
değişmez. İki deny, offline vendor ve HTTP/tedarik kabulü yeni exact lock'a
aittir; eski iki-klon/CI sonucu yeni release kabulü değildir.

## K-239 — Özel CI bağlantıları ve kaynak kökeni temizliği

GB-063 kapanışı yalnız izole iş serisini ve inceleme raporunu etkiler;
başlangıç ağacı/ana kullanıcı checkout’u korunur. Exact
[SHA eşlemesi](incelemeler/urunlesme-20261014/sanitized-sha-map.tsv)
compiler/freeze/regression/TLS kaynak kayıtlarıyla birlikte tutulur.
Özgün eski checkpoint kanıtları tarihsel kalır, yeni temiz kaynak kabulü
ayrıca alınır. GitHub destek teyidi ve public yayın kapıları açıktır.

## K-240 — Mevcut handler sorumluluklarının ayrılması

GET yürütmesi HTTP handler’a, IO replay trait uygulaması aynı sahibin
alt dosyasına taşınır. 900/1250 satır sınırları yükseltilmez; mevcut
semantik/tanı/deadline/trace/state korunur. Katman/SCC/API ve olumlu/olumsuz
IO testleri değişikliğin kabulüdür; yeni cache veya dil özelliği yoktur.

K-240 SMTP testleri ayrı native test dosyasına, DB/posta profil erişicileri
mevcut domain profil dosyalarına taşınır. Test isimleri/faz sahipliği, profil
değerleri, public yüzey ve üretim gövdeleri korunur; spec kanıtı yeni exact
test dosyasına taşınmıştır.

K-238–K-240 son temiz teknik checkpoint'i a2b8557'dir;
[teslim raporu](urunlesme-sonucu-20261014.md) faz/dedicated native/
iki-klon ve sürekli güvenlik kabulünü exact kaynakla kaydeder. Bu son
belge commit'i yeni compiler işi açmaz; P08 ve kalan ürün kapasitesi,
insan/platform/uzak CI/lisans/RC sınırları açık tutulur.

## K-241 — Sürüm üretiminde kaynak klonu ve ikili kimliği

Son doğrudan ikili denetimi a2b8557 sürüm çıktısında `dirty: true` buldu;
üreticinin logu checkout'u derlemeden önce kirletiyordu. Log/ortam raporu
klon dışına taşınır; önce/sonra temizlik ve her gerçek ikilide exact
SHA/temiz kimlik kapısı eklenir. ADR-102/GB-064 ile olumlu/olumsuz shell
ve yeniden kanonik iki-klon kabulü birlikte kapanır. Dil anlamı veya
build.rs koruması değiştirilmez; yeni yayın/RC iddiası yoktur.

K-241 yerel teknik kapanışı exact
`2e970e47d59e2408eff836cdacef9bd0fe0b3028` ile kanıtlıdır:
faz 887/893 (6 dedicated atlama), ayrı native 5/1/7, sürekli güvenlik ve
istisnasız yeni sır taraması; kanonik a/b/çıktı exact SHA, `dirty: false`
ve dil/LSP eş hash. [Son teslim](urunlesme-sonucu-20261014.md) bu kapıları
bağlar. Yeni yayın, tüm platform veya tam ürün kabulü çıkarılmaz.

## K-242 — Native COMMIT yanıtı ve devam etmeme kanıtı

**KAPALI — yerel native kanıt.** Temiz `316a313` kaynağında 11/11, sıfır
atlama geçti. Mevcut C027 politikası korunur; SQLSTATE veya ErrorResponse üzerinden genel
rollback allowlist açılmaz. [ADR-103](../adr/103-native-commit-belirsizligi-kaniti.md)/GB-065:
yedi TLS vaka yanında owned loopback dört kontrol/fault vakası, bir COMMIT
ve SQL attempt sayacı, SMTP devamı ve bağımsız durable sonuç birlikte
denetlenir. Normal Cargo strict skor kartı ve gerçek fake-wire negatiflerini
atlamasız çalıştırır. Native exact kaynak kapanışı [devam raporunda](urunlesme-devam-20261014.md)
ayrıca kaydedilir; canlı ürün/uzak CI/RC sınırı açıktır. Compiler değişikliği yoktur.

## K-243 — Kaynak ZIP snapshot ve çıktı sınırı

**KAPALI — yerel araç kabulü.** [ADR-104](../adr/104-exact-kaynak-arsivi-siniri.md)/GB-066:
başlangıç SHA tek kez sabitlenir; canonical depo çıktı sınırı, UTF8 NUL
liste, staging ve yayın öncesi hash/comment/manifest eşliği normal Cargo
içindeki gerçek izole Git olumluları/olumsuzlarıyla sınanır. Python3 stdlib
yerel araç gereğidir. İki sidecar atomikliği veya uzak CI kabulü çıkarılmaz.

## K-244 — Windows kaldırmada literal yol

**KAPALI — yerel PowerShell araç kabulü.** GB-067: eski kaldırıcı, gerçek
Mac PowerShell senaryosunda `bin[1]` yerine aynı hashli yabancı `bin1`
dosyalarını sildi. Bütün yol işlemleri LiteralPath yapıldı; altı izole
pozitif/negatif kurulum-kaldırma senaryosu geçti. Mevcut Cargo vakasının
Windows dalı ve tatbikat workflow'u native fixture'ı zorunlu yürütür.
Resmî geçici 7.5.11 macOS runtime kanıtı Windows/minOS veya yeni uzak CI
kabulü değildir; source exact son kabul [devam raporundadır](urunlesme-devam-20261014.md).
[ADR-071](../adr/071-kurulum-kaldirma-sozlesmesi.md) receipt/checksum/Zorla
sözleşmesi korunur; yeni dil özelliği veya sistem kurulumu yoktur.

## K-248 — LSP dosya URI UTF-8 correctness

**KAPALI.** Kaynak düzeltmesi, olumlu/olumsuz testler, exact source/provenance
ve temiz c67a69e faz/stdio regresyon kabulü tamamlandı. [ADR-108](../adr/108-lsp-uri-utf8-yolu.md) gerçek
stdio Türkçe klasör arızasına minimal byte çözümünü ve iki satırlık native
regresyonu bağlar. P08, insan/platform/RC kabulü bu işten kapanmaz.

## K-245 — P25 credential host pilotu

**KISMİ.** [ADR-105](../adr/105-credential-host-koprusu.md) mevcut env
yüzeyine POSIX dosya köprüsü ekler;14 native dahil actual vaka hazırdır,
temiz c67a69e host14 kabulü tamamlandı. Windows/systemd/canlı ürün ve file-native
sır inheritance sınırı P25 için açıktır.

## K-246 — P13 worker host pilotu

**KISMİ.** K-246/P13 (ADR-106/GB-069) worker host identity+ayrı readiness ve sahipli
cleanup pilotudur; public launcher cleanup bug'ı temiz c67a69e üzerinde 25-vaka kabulüyle kapandı.
Tam request drain/reload/metrics ve P14 socket devir açıktır.

## K-247 — P09 kalıcı SQL queue prototipi

**KISMİ.** K-247/P09 (ADR-107/GB-070) dar idempotent SQL queue helper'ıdır;11 native
PG crash/COMMIT observer ve16 validator temiz c67a69e kabulündedir. Canlı ürün adoptionı, SMTPretry ve full P08 açık kalır.

## K-249 — P21/C4 TR–EN plain slot pilotu

**KISMİ.** [ADR-109](../adr/109-tr-en-plain-slot-yardimcisi.md): mevcut
görünüm ve sözlükle dört Zee testi/yedi nativeCLI escaping/ret senaryosu.
Python3.9/3.13 temiz c67a69e kabulü tamamlandı. Genel
layout/i18n/list/detail/router ve gerçek ürün/P08 açık kalır.

K-245–K-249 [temiz pilot kabulü](urunlesme-pilot-k245-k249.md)
`c67a69e6ed9c24c4a540a55eeaaa986a348a2de3` üzerinde 895/901 faz,
0 başarısız/6 dedicated atlama; ayrıca 28 PostgreSQL, credential14,
supervisor25 ve iki Python sürümünde TR–EN7+4 ile kanıtlıdır. İki temiz
klondan Mac ARM64 ikilileri eş ve dirty=false; yeni sır istisnası yoktur.
Lisans/insan/fiziksel/tam ürün/uzak CI/RC kapıları ve genel P09/P13/P14/
P21/P25 kapsamı açık kalır. Takip belge commit'i test kaynağı değildir.

## K-252 — SMTP mutlak süre correctness

**Correctness dilimi kapalı.** Kaynak `c0f79aded235266f75cda1bda9244d766d8a32cc`, exact regression fixed_by ve semantic/core-freeze takip kaydı 021bff4'tür. [Temiz ortak kabul](incelemeler/urunlesme-20261014/common-021bff4/README.md) normal fazdaki gerçek native slow-trickle testlerini ve sürekli kapıları doğrular. [ADR-110](../adr/110-smtp-mutlak-sure-butcesi.md) mevcut MIME deadline sarmalayıcısını plain yolla paylaşır; DNS iptali vaat edilmez.

## K-250 — PostgreSQL statement fault kanıtı

[ADR-111](../adr/111-postgresql-statement-fault-kaniti.md): gerçek 40P01 ve
40001, no-retry sayaçları, outer rollback onayı ve bağımsız durable observer
iki native vakayla sınanır. Dedicated TLS kapısı yedi vakadır; COMMIT
belirsizliği ADR-103/C027 olarak kalır. Çalışma ağacı native7/strict9 geçti;
021bff4 [temiz exact ortak kabul](incelemeler/urunlesme-20261014/common-021bff4/README.md) tamamlandı; provider/full fault matrisi açık.

## K-251 — DB ve göç hazırlık probu

[ADR-112](../adr/112-db-goc-hazirlik-probu.md), mevcut CLI ile uygulama
sahipli tek seferlik DB+migration gate üretir. Native13/strict10 yerel aday
kabulü vardır; compiler özelliği yoktur. DB/worker nesli değişince yeniden
prob gerekir; continuous/TTL, request metrics/drain/config reload ve P14
socket devri açık kalır. 021bff4 [exact ortak temiz kabul](incelemeler/urunlesme-20261014/common-021bff4/README.md) tamamlandı; gerçek ürün/platform adoptionı ayrıca gereklidir.

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

## K-262 — Mevcut web oran CLI argüman kayması

Temiz17b preexisting CLI bug kanıtlıdır: mevcut dört değerli oran seçeneği kaynak öncesi tüketilmediğinde APP_ARG yerine bellek görünür. K-262 dar kaynak öncesi skip onarımı, kaynak sonrası oran argümanları ve eski globalproxy sınırını korur. Yeni flag/grammar/export yoktur. Dar bugfix KAPALIDIR: kaynak commit `7c56518cbfc6f5c99ac56bfe7767b5bbdc106179`, olumlu/olumsuz gerçek CLI testleri, ADR-121 ve `regression/v2.tsv` içindeki `web-oran-program-argumani` exact fixed_by kaydı semantic-bugfix/core-freeze correctness zincirini tamamlar. Final exact RC ve birleşik özellik kabul kapıları AÇIKTIR; bu kayıt sürüm veya ürün adoption kapanışı değildir.

## K-257–K-260 — Native ürünleşme aday dalgası

**KISMİ — dar kod kapasitesi ve yerel aday kabulü.** [ADR-116](../adr/116-native-owned-metrics-pipe.md),
[ADR-117](../adr/117-sinirli-sql-dispatcher.md), [ADR-118](../adr/118-native-listener-nesil-hazirligi.md)
ve [ADR-119](../adr/119-credential-native-metrics-capability.md) kabulü sınırlı
profiller içindir; P09/P13/P14/P25 ürün benimsemesi kapanmış değildir.

- K-257/P13: native request ve actual PostgreSQL pool sahibi sayaçları, bounded
  içeriksiz FIFO snapshot ve iki worker/restart toplamı; dedicated8 kabulü.
  Probe gecikmesi request/DB metriği diye sunulmaz. Telemetry best effort'tür.
- K-258/P09: [sabit SQL niyet dispatcher'ı](job-dispatcher.md), separate lease
  heartbeat üzerine K246 grup/WNOWAIT ve K245 credential reader kompozisyonu;
  native11. 1..100 tick pilotu, calendar/daemon/cron ve SMTP/PDF etkisi değildir.
- K-259/P14: [owned native listening socket](native-listener.md), fresh nonce/PID/
  BuildJSON ve generation receipt ile ACTIVATE öncesi accept kapalı; successful
  accept sonrası TERM'de stream tamamlanır. Native17+FD16; ortak JSON kota/revoke
  restart/overlap'ta korunur. Default direct bind K253 davranışını korur.
- K-260/P13/P25: credential launcher yalnız designated nonblocking write-only
  metric FD'yi aktarır;7+2 vaka. Listener/ready/activate veya arbitrary descriptor
  transferi yoktur; default close_fds sınırı korunur.

Bu dalga yerel çalışma ağacı/adayı kabulüdür. Listener17/FD16, metrik8 dedicated,
credential7 model/subprocess+2 native ve dispatcher11 sayıları ayrı kampanyalara
aittir; Cargo/libtest toplamına eklenmez. Listener birleşim kanıtı pinlenmiş
`2caa26e55e50bf35b4220e4f6d16ac7273f337159243b6e97c8f2443581adbf8` ikilisinin
actual BuildJSON'unda `17b29da1b8321fe6dcc4d34bd2027c4c969bf406` tabanı ve
`dirty=true` taşır. Bu kimlik temiz kaynak kabulü değildir. Final exact temiz
ortak faz/native, 900 sn host tatbikatı, 1800 sn soak, dört hedef explicit ASAN
RC/Miri, imza/kurulum ve fiziksel platform kanıtı henüz bu yeni kaynağa alınmadı.
Önceki d42f147 temiz kabulü kendi tarihsel kaynağında kalır.

B-001'den başlayan bağlayıcı sıra değişmez. Gerçek katalog scheduling/SMTP retry,
continuous DB readiness/TTL, güvenlik ayarını gerçekten yeniden okuyan config
reload, 900 sn/systemd/Windows, farklı app revision'larının atomik dağıtımı,
canlı ürün/anonim tam P08 korpusu açık kalır. Lisans, gerçek usability, fiziksel
minOS, GitHub yanıtı, uzak CI ve RC/release kapıları ayrıca açıktır.39 plan
satırı bu dalga yüzünden tamamlanmış sayılmaz. K262 rate/argv correctness işi
ayrı kaydıyla birleştirilir; bu bölüm onun regression kapanışını üstlenmez.

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

## K-265 — Exact kaynak dış RC kanıtı

[ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md) sourceS/historyE self-reference doğruluk boşluğunu explicit dış immutable paketle ele alır; kaynak eşdeğerliği/bypass yoktur. İzole25 sentetik protokol+9 kısa ownedprocess test dilimi, tam4ASAN2Miri/rawhash/initialsourcecorpus/erkenHEADpin/streamingdizinbütçesi negatiflerini kapsar. K-265 kaynak commit’i `1367ce1a7d0281d8a7f3d9307021afbc452da15f` ile kayıtlıdır: kaynak öncesi tam faz928/940 geçti, başarısız0/atlanan12; son EOF boşluğu düzeltmesi sonrası25 protokol testi yeniden geçti. Final gerçek cleanS uzunASAN/Miri kampanyası,900/1800/release/ürün adoption kapıları açıktır. Kullanıcı AGENTS değiştirilmez, exactsourceS ve append-only arşivE ayrımı açık tutulur.

## K-267 — Typed credential/native listener bileşimi

[ADR-125](../adr/125-credential-native-listener-kompozisyonu.md) mevcut D3/P25 ve B5/B6 ürün ihtiyacını dar Unix host profilinde birleştirir. Credential okumadan all-or-none FD/rol/socket/port/argv doğrulaması, native child tarafından devralınmayan ayrı startup FIFO ile PID/generation/binary kimliği ve varsayılan noinherit korunur. İki normal Cargo vakası model/subprocess5 ile gerçek native crypto/overlap/metrics/admission/cleanup5 kampanyalarını ayırır; alt-senaryolar eşsiz toplam diye sayılmaz. İzole dirty d53 kısa aday kanıtı final temiz kaynak kabulü değildir. P25 bütünü, Windows/systemd/file-native sır aktarımı, canlı benimseme ve 900/1800/RC kapıları açıktır. GB-083 bilinçli capability sınırını kaydeder; yeni grammar/compiler/public Rust API eklenmez.

## K-268–K-270 — Sonlu timer, başlangıç projeleri ve yerel araçlar

K-268/ADR-126/GB-084 sonlu timer37 hazırlığı; K-269/ADR-127/GB-085 kopyalanabilir site/panel2 mevcut CLI/HTTP pilotu; K-270/ADR-128/GB-086 yerel import/proje launcher14 ve korunmuş selector25. Yeni üç normal Cargo vakası gerçek alt-senaryolarıyla ayrı sayılır. Native Linux/systemd/SQL timer, gerçek kullanıcı/live ürün, production publisher/platform ve final temiz kaynak kampanyaları açık; P09/P23/P07 kısmi kalır. Yeni compiler/src/grammar/facade/dependency veya spec normatif madde yoktur; spec214 ve önceki source54/c86 maintenance/provenance, K267 kayıtları korunur.

[Timer](../docs/sonlu-dispatcher-timer.md), [başlangıç](../sablonlar/README.md), [yerel araç](../docs/derleyici-yerel.md).

## K-271–K-274 — dar ürünleşme dilimleri

- **K-271/P22, ADR-129/GB-087:** mevcut native CLI/HTTP ve fresh PG fixture kiti;
  normal hazırlık20, ayrı explicit ignored actual4. Public injection/coverage/P08 açık.
- **K-272/P21, ADR-130/GB-088:** sabit typed tam TR/EN liste–detay golden9HTTP/5Zee;
  eski7HTTP/4Zee korunur. Genel layout/i18n/gerçek ürün kabulü açık.
- **K-273/P20, ADR-131/GB-089:** yalnız yeni owned scaffold modül metadata/route,
  journal, EX/SH ve offline marker sınırı; actual23. SQL archive bytes yürütülmez;
  DB lifecycle/CRUD/extensionpoint/ürün benimsenmesi açık.
- **K-274/P13, ADR-132/GB-090:** opt-in TTL, finite DB watch ve mevcut config
  atomic replace/HUP; normal models16/native5, dedicated PG15 eski13dahil.
  Continuousservice/systemd900s/platform ve final1800/RC açık.

Bu dört dar uygulama ve kısa dış kanıt ortak kaynakta bağlandı; root Cargo,
üretilecek raporlar ve serial frozen fullphase toplu commit öncesi kapılardır.
Bağlayıcı backlog sırası, önceki exact source/provenance ve geniş P-* açık işleri
bu kayıttan dolayı tamamlanmış sayılmaz.

## K-275 — P11/B3 sabit host PDF backend

K-275/ADR-133/GB-091 sabit owned PDF host profili: mevcut Zee JSON örneği, exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0, stdlib8 normal ve hostbackend8 explicitignored ayrımı. Sabit kaynak/font/notice, bounded job/output ve nofollow roller korunur. Finalclean S native emitter+backend8 ortak kohortu ayrı zorunludur; tarihsel dirty kanıt temiz S sayılmaz. P11/B3 Zee içinden invocation/resmi signedplugin, ürün/adoption/P08/platform ve proje lisansı açıktır; full900/1800/RC/soak/kurulum kabulü ileri sürülmez. [Rehber](pdf-katalog-host.md).

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

K-250–K-279 dar teknik profilleri ve K-265 exactS RC kapısı [15 Ekim S raporu](urunlesme-final-kabul-20261015.md) ile kaydedilir. Temiz S üzerinde dört ASAN hedefi ayrı ayrı1801sn geçti; lexer_parser7.117.460, morfoloji8.265.144, http_istegi11.194.063 ve wasm_abi9.011.299 yürütme kaydedildi. İki seçili Miri grubunda toplam3test(ondalık2,zaman1) geçti. Kanonik export/check ve aynı S güvenlik kapısı510.198758sn exit0 ile geçti; kapının118 Cargo/libtest grubunda943passed/0failed/16ignored vardır. GB082 yalnız bu exactS yerel protokol ve kanıt kapısı için kapalıdır; E kaynakS gibi kabul edilmez. K-276actualLinuxdar kabulü ve finiteSQLtimer, bütün teknik/ürün işlerinin tamamlanması değildir. P08/adoption/human/platform ve IDEcompletion/orderlyshutdown açık; yeni compilerözelliği için mevcut freeze kuralları aynıdır.

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

## K-281–K-294 — 15 Ekim ikinci teknik dalga

Durum: dar yerel S3 teknik kabulü kapandı; C1/P33 araç retleri ve
ürün/insan/platform/kanal sınırları açıktır.
[İş bazında sonuç ve açık kapılar](urunlesme-ikinci-dalga-20261015.md) bağlayıcıdır.
K-284/C5 ve K-287/C2 exact cd5d12f0008620819e6167af5146e83503fd2b38
zee-cekirdek talebinin minimal araç yüzeyleridir; grammar genişletilmez.
K-294 mevcut control-FD correctness düzeltmesi, kaynak SHA + regression
follow-up olmadan kapanamaz. K-288 ve K-293 araç retleri nedeniyle açıktır.

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

## K-295 — daemon unit'in explicit yürütme düzeltmesi

S2 `e35b617095e6a9d00915794d6adee90842238a5f` gerçek Linux koşusunda
daemon ExecStart yalnız --config taşıdığı için check-only kipinde exit0 dönmüş,
hiç SQL tick başlamamıştır (0/4; cleanup_errors boş, owned_units_released true).
Üretilen daemon artık explicit --run, OnCalendar hizmeti yalnız --once taşır.
Doğrudan CLI varsayılan preflight ve render'ın servis başlatmama sınırı değişmez.
Eski unit dosyaları yeniden --render ile üretilmelidir. Exact S2 kaynağına karşı
yeni regresyon kırmızı, düzeltmeyle hazırlık22 yeşildir; S3 gerçek Linux dört vaka kabulü geçti. Host correctness düzeltmesidir; compiler bug/grammar/tanı/API değişmez.

## K-296 — deduplikasyonlu state olayı yerine gerçek liveness oracle

S2 release kabulünde worker-loss vakası geçti; stale-gate vakası aynı ready=false
durumunda yeni state olayı beklediği için kırıldı. Supervisor state'i yalnız
readiness değişince yayımlar; üretim davranışı korunur. Native koşucu iki worker'ın
Build kimliği doğrulanmış HTTP200 liveness'ını doğrudan owned Supervisor probe
ile gözler. Startup reddi, hiç ready=true olayı olmaması, eski gate bayt/mtime
değişmezliği ve bütün owned handle kapanışları birlikte zorunlu kalır. Yalnız
test oracle düzeltilmiştir; native DNS veya production lifecycle genişlemez.
S3 gerçek native iki vaka kabulü geçti.

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

## K-298 — P19 bildirim ömürlü açık metin dizini bakımı

**AÇIK — kaynak adayı ve temiz ölçüm/kabul bekliyor.** S4'te lint açık/41 belge
stdio p95 223,698 ms ile 200 ms hedefi karşılanmadı. Mevcut açık URI envanterinin
gerçek yol/tutarlı ödünç metin dizini bir bildirim boyunca ortak kullanılarak
tekrarlı kanonikleştirme azaltılır; kaynak metinleri ödünç alınır. Çelişen alias, disk/paket/kilit
kontrolü, tanı ve yayın sırası korunur; yeni kalıcı cache veya HIR/type kapasitesi
eklenmez. Üç alias'tan her birinin çatışması/iki lint kipi ve sonraki bildirimde Unix
symlink hedefinin yenilenmesi iki dar testle izlenir.

[ADR-145](../adr/145-artimli-editor-analizi-ve-lint.md) kararı ve etki incelemesi,
[ölçüm sözleşmesi](olcumler.md) ile aynı dilimdedir. Exact semantic/core-freeze
`maintenance` kayıtları kaynak commit'ini izler. Başarılı test, yeni temiz stdio
ölçümü ve gerekli kaynak kabulü kaydedilmeden K-298 kapandı denmez. Gerçek ürün,
UI, platform ve tam P19; C1/K-288 ve P33/K-293 retleri ayrıca açık kalır.

## K-299 — Kabul koşucularında erken iptal ve kısmi setup temizliği

**AÇIK — stdlib PDF9/service10 geçti; ilk C native kampanyası başarısız.**
PDF9 ayrı geçti; service-fault kanonik olmayan depo kökünü iç kampanya öncesi
reddetti. Cargo test wrapper `root()` kanonikleştirmesi takip adayındadır;
yeni source/tree ve gerekli kapılar/native9/2 yeniden kabul beklenir. PDF kabul betiği ilk owned komuttan önce handler kurar;
setup/keygen finally kapsamına alınır, fresh held dirfd'deki iki exact sentetik
key temizlenir. Başarısız nested host komutunun grace'i10 saniye olur. Service
prepare handler-before-H.run, kısmi Fixture temizliği ve cleanup sırasında iptal
bayrağının korunması aynı dar işi tamamlar. Üretim host zaten cooperative'dir;
compiler/Supervisor/lifecycle veya native oracle değişmez.

[ADR-141](../adr/141-imzali-pdf-stdio-host.md), [ADR-150](../adr/150-native-worker-fault-ve-kontrollu-resolver.md),
[GB-111 ve actual kısa test kaydı](kabul-kosuculari-yasam-dongusu-20261016.md)
aynı kaynak dilimindedir. Beş yeni Python iç vaka yeni Cargo owner değildir.
Sonraki temiz SHA'da iki native seçicinin proof/BuildJSON/hash/owned-close zinciri
ve ortak kapılar doğrulanmadan kapalı olmaz. K-298 RC/soak/IDE kanıtı kendi
kaynağında kalır; ürün/FFI/DNS/fault, C1/P33 ve dış kabul açıklarını kapatmaz.
