# K-250–K-279: temiz kaynakta yerel teknik kabul

Bu rapordaki tamamlanmış dar kohortların test kaynağı **S = `8b49113c413f62b35d4b251ed31068524d2fe26c`**,
ağaç `09fa6e764bb317b4aa5b32b6ca15ce02f4cf389d`’dir. Bu belgeyi ve
makbuzları ekleyen sonraki arşiv commit’i **E**, S üzerinde yapılan koşuların
kaynağı olarak gösterilmez. İkililer yayımlanmadı, release etiketi veya
üretim imzası oluşturulmadı. Önceki dirty/baseline/başarısız denemeler kendi
kimlikleriyle korunur.

| Kabul | Gerçek sonuç | Sınır |
|---|---|---|
| Ortak faz | 943/959 geçti; 0 başarısız, 16 explicit ignored | 117 gerçek test ikilisi; 16 ignored exact sahip eşliği |
| Ayrı native kabul | 25 zorunlu grup; 163 sahipli komut; 268 ham dosyanın hashleri bağımsız denetlendi | Alt senaryolar Cargo toplamına eklenmez |
| PostgreSQL ve dış etki | TLS/fault8, göç1, parola11, queue21, job11, SMTP/PG11; readonly readiness15 ve TTL5 | K-277 yalnız sabit sentetik loopback; belirsiz SMTP/COMMIT otomatik tekrarlanmaz |
| Web/PDF/TR–EN | Owned web+PG4, fixed PDF backend8 ve her Python3.9/3.13 için legacy+liste/detay HTML kohortları | PDF önce gerçek Zee job JSON üretir; keyfi PDF sandbox veya ürün entegrasyonu değildir |
| macOS teslim/kurulum | İki temiz klonda eş CLI/LSP, iki eş kaynak ZIP, restore/fsck doğrulanmış bundle; kurulum8 | Yerel unsigned geliştirme teslimi |
| Linux teslim/kurulum | Ayrı temiz Debian12 ARM64 VM’de aynı kanonik iki-klon teslimi ve kurulum8 | Fiziksel/minOS/Windows veya genel ABI garantisi değildir |
| Linux taşıyıcı | Kanonik inert .deb build, kur/etkinleştir/ret/kaldır4 | Genel apt repository/otomatik etkinleştirme yok |
| Linux konteyner | Kanonik tarifte UID65532/BuildJSON/denetle/çalıştır/kaynakhash/LSP6 | Digest-pinned official base, ağ kapalı, salt okunur, capdrop/nnp; kamuya yayın yok |
| Linux timer | Gerçek kanonik user-unit/proc/PG2; normal tick bir kez tamamlandı, kesilen SQL otomatik tekrarlanmadı | Sonlu SQL profili; bilinçli paused latch korunur |
| Önceki küçük sürüm | Mac ortak kabulü ve Linux S üzerinde modern49+bootstrap5 | Eski CLI JSON capability sunmaz; legacy rollback açılmaz; K-086 açık göçü korunur |
| 900 saniye | 900.059566 sn; 360/360 sabit yük HTTP200; 91 RSS örneği | Ham health_probe_failures=1 korunur; başlangıç sınıflaması kontrol akışı incelemesine dayanır, zaman damgalı doğrudan oracle değildir; genel production SLA değildir |
| 1800 saniye soak | 1801.090583 sn; 360 örnek; derleyici20112→20080 KiB, LSP17120→17696 KiB | %10 ve32MiB birlikte aşılmadı; paylaşımlı makinede gözlemsel, benchmark tabanı değil |
| ASAN/Miri | Dört hedef ayrı ayrı1801sn geçti; iki seçili Miri grubunda3test(ondalık2,zaman1) | crash/input-timeout/sanitizer bulgusu yok; Miri setup test sayılmaz; seed import edilmedi |
| Son RC güvenlik kapısı | Kanonik export/check ve aynı temiz S kapısı exit0;510.198758sn;118grupta943/0/16 | GB082 dar exactS protokolü için kapalı; E veya signedrelease kabulü değildir |
| JetBrains | IntelliJ2025.3.1/LSP4IJ0.21.0: şablon/macro/TextMate, gerçek sunucu açılışı, tanı/düzeltme, hover, tanıma git, rename görüldü | Tamamlama ve düzenli UI/LSP kapanışı doğrulanmadı; CUA timeout ve OS kısayol çakışması; P19 kısmi |

İlk Linux delivery denemesi gerçek Git2.39.5’in `git archive --mtime`
desteği bulunmadığı için arşiv aşamasında kaldı. Kaynak betiği değiştirilmeden,
resmî kaynaktan ayrı owned prefix’e kurulan Git2.56.0 ile kanonik teslim
baştan geçti. İlk carrier denemesi private parent şartında, ilk container
denemesi dış metadata denetimindeki NOFOLLOW readelf symlink reddinde
yan etki öncesi durdu. Özgün retler saklandı; yeni sahipli koşular ayrı
makbuzlar taşır. Carrier/timer’ın ilk gerçekleşen koşularındaki ayrı
dış-driver başlangıç/bitiş kayıt eksikliği sonradan uydurulmadı; aynı S ve
değişmeyen onaylı driver ile açık başlangıç/bitiş kayıtlı gerçek tekrar
koşuları yapıldı.

Tarihsel K-277 ad-hoc denemesindeki kayıp Popen yetkisi
`UNVERIFIED_AUTHORITY_LOST_NO_ADOPTION` olarak kalır. Sonraki kernel
yol gözleminde yokluk geçmiş temizliği kanıtlamaz. Yeni kohortların
sahipli temizliği bu eski kaydı geriye dönük kapatmaz.

## Arşivin dosya kapsamı

Tam Linux dış arşivinin411 yaprağı138.192.679 bayt bağımsız doğrulandı.
Git arşivine aday kopya127 temsilden oluşur:120 özgün metadata dosyası ve7 tersine çevrilebilir `.derived.json` temsili. `linux/metadata/TRACKED-SUBSET.json` seçimi bağlar. Tam411 dış paket Git’e kopyalanmaz; ham günlük/ikili/ZIP/.deb dosyaları dışarıda korunur. Değişmeyen özgün Linux MANIFEST/SHA256SUMS bu dış dosyalara da işaret eder ve Git altkümesini tek başına doğrulayan manifest değildir. [Yerel arşiv açıklaması](incelemeler/urunlesme-20261014/final-8b49113/README.md) özgün/derived ayrımını ve üst dizindeki normalization haritasını açıklar.

İlk RC kampanyası source-clean ve producer exit0 olmasına rağmen HTTP
dış süre702.404sn ile raw1921sn, WASM695.844sn ile raw1914sn uyuşmadığı
için doğrulayıcıdan exit2 aldı. Bu makbuz ve ham sayılar korunur;
birinci dalganın geçerli süresi dört-hedef RC kabulü gibi birleştirilmez.
Kanonik koşucu resume desteklemediğinden değişmeyen S üzerinde ayrı dört-hedef+Miri recovery kampanyası yürütüldü. Owned macOS caffeinate utility ömrüyle sınırlı kaldı; duvar saati ve monotonik saat kayıtları ayrı korundu.

Temiz S üzerinde dört ASAN hedefi ayrı ayrı1801sn geçti; lexer_parser7.117.460, morfoloji8.265.144, http_istegi11.194.063 ve wasm_abi9.011.299 yürütme kaydedildi. İki seçili Miri grubunda toplam3test(ondalık2,zaman1) geçti. Kanonik export/check ve aynı S güvenlik kapısı510.198758sn exit0 ile geçti; kapının118 Cargo/libtest grubunda943passed/0failed/16ignored vardır. GB082 yalnız bu exactS yerel protokol ve kanıt kapısı için kapalıdır; E kaynakS gibi kabul edilmez.

| ASAN hedefi | Gerçek süre(sn) | Yürütme |
|---|---:|---:|
| lexer_parser | 1801 | 7117460 |
| morfoloji | 1801 | 8265144 |
| http_istegi | 1801 | 11194063 |
| wasm_abi | 1801 | 9011299 |

[RC sonuç makbuzu](incelemeler/urunlesme-20261014/final-8b49113/rc/FINAL-RESULT.json), [aynı-S kapı makbuzunun türetilmiş temsili](incelemeler/urunlesme-20261014/final-8b49113/rc/POST-GATE-RECEIPT.derived.json), [bağımsız kök incelemesi](incelemeler/urunlesme-20261014/final-8b49113/rc/root-independent-review.json). Bundle manifest SHA256 `5e174eb9179e79e75b5127e12c1760b65d448704256ad69dbfa3abd5b343805c`; dış ham aynıS gate receipt SHA256 `13e3130aba5b1c91b5f69d50a6429ab92622522812d4cdd2abe5613b09237c0e`.

## Açık ürün ve süreç sınırları

P08 tam anonim beş site+kök korpusu ve gerçek ürün provenance’ı henüz
sağlanmadı. Bu nedenle SMTP/PDF/managed-module/HTML/web-kit profilleri
gerçek katalog, canlı sağlayıcı ve ürün benimsenmesi kabulü değildir.
K-161/K-162 gerçek katılımcı çalışmaları, fiziksel temiz Windows/minOS,
GB-046 dış destek yanıtı ve 14 Ekim–11 Kasım syntax takvimi açık kalır.
Lisans kararı ve public release ayrıca kullanıcıya aittir.

P09 genel daemon/cron/provider exactly-once; P11 Zee içinden resmî signed
PDF plugin invocation; P13 sürekli production systemd hizmeti; P19 bütün
IDE/Windows, tamamlama ve düzenli UI kapanışı; P20 genel CRUD/SQL goc
yaşam döngüsü; P22 source branch coverage bu dar kabulün dışındadır.
Kalan işlerin tümü dış girdi değildir: genel daemon/sürekli hizmet, signed PDF invocation, source branch coverage ve doğrulanmamış IDE completion/kapanış ayrı teknik dilimlerdir. Gerçek ürün provenance/adoption, insan, takvim, platform ve lisans kapıları ayrı gerekçelerle açıktır.
Yerel Linux VM kabulü fiziksel cihaz yokluğuna dönüştürülmez; yerel
macOS kabulü temiz fiziksel macOS/minOS testine dönüştürülmez.

## Dokümantasyon etkisi

Bu arşiv yeni compiler/src, grammar, normatif spec maddesi, tanı kodu,
public Rust API veya production bağımlılık kenarı eklemez. Spec/RFC/ADR
indeksleri ve hata kataloğu bu nedenle davranış bakımından değişmez;
ilgili ADR’lere actual kanıt bağlantıları eklenir. Performans tarihçesi
ve ADR045/049 için yeni sabit makine benchmark tabanı yazılmaz.
Kanıt kayıtları, güvenlik GB082/092 durumu, soak tarihçesi ve doğrulanmış S RC sonucunun append-only fuzz tarihçesi, backlog,
master-plan, sürüm kapıları ve öğretici rehberler aynı arşiv dalgasında
gözden geçirilir.

Kapı makbuzu bağlantısı tersine çevrilebilir türetilmiş JSON temsilidir; özgün hashli ham makbuz dış arşivde korunur. Eşlik parsed JSON içindir, ham byte eşliği veya imzalı provenance değildir.


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
