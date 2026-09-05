# zee — V1 öncesi öncelikli mühendislik backlog'u

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
`itwise-admin` deposundaki temiz `codex/k-163-catli-dogfood` dalında başladı.
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
kritik/yüksek bulgu sıfırını makinece zorunlu kıldı; sıradaki makine işi
K-172 dogfood corpus'u ile K-166 için drift/bulgu boşluklarının kapatılmasıdır.
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
| K-163 | **KAPALI** | Çatlı 1037 Zee LOC/5 modül/12 testte gerçek PG16.11, TLS/bounded pool, binary saga, failure reconciliation ve F032 liveness/readiness zincirini geçti. Bakım skor kartı LSP p95 41,416 ms ve dosya medyanı 5,5 ile kapalı; exact ürün `bb8e1ac` |
| K-164 | **AÇIK** | Farklı workload'da ikinci gerçek proje |
| K-165 | **AÇIK** | Aynı uygulamanın Zee–Go/Rust veri temelli dogfood karşılaştırması |
| K-166 | **AÇIK** | En sık 50 hata için span/öneri/noise düzeltme başarısı |
| K-167 | **KAPALI** | RFC-0028/ADR-064/spec-27: sekiz dondurulmuş yüzey envanteri, süreli `DEP-NNN` deprecation kaydı, `-dev` sürüm kimliği ve B-072 kaldırması `uyumluluk_testi` ile CI'da |
| K-168 | **AÇIK** | İki temiz ortamda eş hash, SBOM, imza ve provenance |
| K-169 | **AÇIK** | Linux/macOS/Windows kurulum-kaldırma ve release runbook tatbikatı |
| K-170 | **KAPALI** | ADR-066 `guvenlik-kapisi.sh` + `GB-NNN` bulgu kaydı + SECURITY.md: 15 kapalı/4 kabul/3 açık düşük-orta, açık kritik/yüksek sıfır; sürüm adayı kipi exact HEAD RC fuzz + clippy + tam test ister |
| K-171 | **KAPALI** | ADR-065 `spec_drift`: 169 normatif spec maddesi parmak-izi kimliği + exact test işlevi; 151 kanıtlı/16 kısmi/2 açık deterministik raporla CI'da; RFC/ADR belge düzeyinde kalır |
| K-172 | **AÇIK** | Yalnız dogfood boşluklarından büyüyen başarı+başarısızlık corpus'u |
| K-173 | **AÇIK** | Uzun compiler/LSP workspace soak ve kaynak sızıntısı kanıtı |
| K-174 | **AÇIK** | 2–4 hafta yeni syntax kapalı V1 freeze |
| K-175 | **AÇIK** | İnsan+iki proje+üç platform+signed reproducible V1 RC |

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
