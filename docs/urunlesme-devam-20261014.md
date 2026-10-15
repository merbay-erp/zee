# 14 Ekim 2026 ürünleşme devamı

Önceki [2e970e4 teslimi](urunlesme-sonucu-20261014.md) kendi exact
kaynağında geçerlidir. Bu dalga yalnız K-242/243/244 araç ve kanıt değişikliğidir;
üretim compiler kaynaklarını değiştirmez.

K-242 yedi TLS vakaya dört owned loopback kontrol/fault vakası ekler. C027,
otomatik tekrar etmeme ve SMTP devamını durdurma korunur; SQLSTATE
rollback kanıtı sayılmaz. Normal Cargo kapısı 13 skor kartı ve 10 bounded
wire testi çalıştırır; on bir gerçek PostgreSQL vaka ayrı dedicated koşudur.

K-243 eşzamanlı HEAD, canonical çıktı kaçışı ve quoted UTF8 hatalarını
gerçek Git testleriyle kapatır. Kaynak archive exact SHA, manifest ve ZIP
byte eşliği yayın öncesi doğrulanır; ZIP/sidecar iki dosya atomikliği ve aynı
kullanıcıya karşı genel filesystem sandbox iddiası yoktur.

## Kabul kaydı

Temiz exact kaynak **`316a313ed6798d084450097d79281dce7f3fc82d`**:

| Kapı | Gerçek sonuç | Kanıt |
|---|---|---|
| Tam faz | 888/894; 0 başarısız, 6 dedicated atlama; 94,303 sn faz toplamı | [Faz](incelemeler/urunlesme-20261014/devam-316a313/faz-test-matrisi.md) |
| Dedicated PostgreSQL | Rust 1.93.1 / PostgreSQL 16.15; 5 TLS + 1 göç + 11 parola/COMMIT, 0 atlama | [Native](incelemeler/urunlesme-20261014/devam-316a313/native-proof.json) · [11 vaka](incelemeler/urunlesme-20261014/devam-316a313/parola-goc-ci.json) |
| Literal kaldırma | PowerShell 7.5.11 / macOS arm64; 6/6, başlangıç/son temiz ve kaynak hash'leri aynı | [PowerShell](incelemeler/urunlesme-20261014/devam-316a313/powershell-proof.json) |
| Sürekli güvenlik | İki graph deny, fuzz derleme, kayıt/tedarik; sır geçmişi 435 commit + kaynak ağacı temiz; yeni istisna yok | [Kapı özeti](incelemeler/urunlesme-20261014/devam-316a313/gate-summary.json) |
| Kaynak ZIP | 1146 kaynak; exact comment, dosya kümesi/manifest ve byte hash'leri eş | [Arşiv](incelemeler/urunlesme-20261014/devam-316a313/source-archive.json) |

Normal fazdaki altı dedicated vaka ayrıca gerçek koşuda yürütüldü. Python
ve PowerShell vaka sayıları Cargo toplamına eklenmez. Native debug ikili
actual SHA ve `dirty=false` kimliğini verdi; başlangıç/son ikili hash'i,
kaynak SHA/temizliği, statement sayaç gözlemcisi ve küme kapanışı denetlendi.
[Byte doğrulaması](incelemeler/urunlesme-20261014/devam-316a313/native-byte-verification.json)
on bir ham native artefaktın SHA-256 eşliğini ayrıca kaydeder.

K-243 [aday önce/sonra kanıtı](incelemeler/urunlesme-20261014/devam-316a313/k243-candidate-proof.json)
F0 çalışma ağacına ve exact dosya hash'lerine aittir; yeni SHA kabulü olarak
yeniden etiketlenmedi. [K-244 bağımsız incelemesi](incelemeler/urunlesme-20261014/devam-316a313/k244-independent-review.json)
salt kaynak incelemesidir, Windows koşusu değildir. İlk başarısız native
proxy/alias/PowerShell sonuçları kalıcı özel kanıt paketinde korunur.

Yeni ZIP SHA-256 `3a64814a1858ebe1b7530bf55cd321ec8b9914a8aec007890dab09faaf0deaf8`
ve dosya boyutu 4.532.595 bayttır. Paket ignored `dist/urunlesme-20261014/source-316a313/`
altındadır. Önceki kanonik iki-klon ikilileri 2e970e4 kaynağında tarihsel
kalır; bu dalgada yeni signed release veya yayın hazırlanmış gösterilmez.

## Etki incelemesi

ADR-103/104, indeks ve kanıt haritası güncellenir. Spec/RFC normları, hata
kataloğu/tanı kalitesi, public API/uyumluluk, production katman/bağımlılık
graph'ı, iki Cargo lock, workflow action pinleri ve benchmark iş yükü aynı
kalır. Compiler semantiği veya yeni kalıcı state/cache yoktur; compiler
bugfix/fixed_by, compiler/freeze beyanı, performance veya soak tarihçesine
yeni kayıt gerekmez. Yeni Cargo wrapper'ının faz sahibi IO'dur; mevcut
arşiv vakası mühendislik, kurulum vakası tedarik fazında korunur. Native
Python/PowerShell vaka sayısı Cargo
toplamına eklenmez. Yeni dogfood kaynak veya çıktı sözleşmesi eklenmez;
güvenlik kayıtlarının özeti aynı sözleşmeyle yenilenir.

Lisans kararı bekler; sınırlı pilot seçimi korunur. İnsan oturumları, temiz
Windows/minimum OS, tam anonim ürün korpusu ve GitHub destek yanıtı yoktur.
Canlı ürün kabulü, uzak CI, signed release ve RC kapıları açık kalır.

K-244/GB-067 wildcard yol kullanımı resmî geçici PowerShell 7.5.11
macOS runtime ile doğrulandı: eski betik kurulu iki dosyayı bırakıp aynı
hashli yabancı iki dosyayı sildi. LiteralPath düzeltmesi altı senaryoda
normal/Unicode/bracket kaldırma, yabancı sahipliği, değişmiş hash/Zorla ve
manifest retlerini doğrular. Mevcut Windows Cargo dalı ve tatbikat aynı
fixture'a bağlıdır; bu yerel kanıt fiziksel Windows/minOS kabulü değildir.

Microsoft'un [Remove-Item](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/remove-item?view=powershell-7.5)
ve [Get-FileHash](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.utility/get-filehash?view=powershell-7.5)
belgelerindeki Path/LiteralPath farkı kaynak incelemesini destekler. Gerçek
sonuçlar ayrı runtime/source/hash kanıtlarıyla kaydedilir.

## K-267 — Dar credential/listener bileşimi

Kayıtlı D3/P25 ile B5/B6 ihtiyacı için Unix owned launcher/native listener köprüsü, host-only startup FIFO ve typed presecret capability denetimi ADR-125/GB-083 ile kaynakta bütünleştirildi. Model5 ve dirty d53 native5 kısa kanıtı final temiz kabul değildir. Normal Cargo/faz, finalclean ortak kampanya ve uzun işletim/RC ayrı açıktır; Windows/systemd/file-native/live kapsamı kapanmaz.

## K-271–K-274 yeni dar dilim

Temiz K-268–270 kaynak `62282aec29235b30d1c313a540b817311702f40f` üzerine native web
fixture kiti, typed TR/EN tam golden, owned modül scaffold'u ve opt-in TTL/finite
DBwatch/configHUP kaynakları bağlanır. Dış kısa kanıt20+4,9HTTP/5Zee+old7/4,
23 native/fault,16model+native5/dedicatedPG15 sınırındadır; bunlar unique toplam
veya yeni currentclean kabulü sayılmaz. P22 coverage/P20 DB lifecycle/P13 sürekli
service, P08/private ürün/platform ve final900/1800/RC açık kalır. Generated
sayı/kanıt/faz raporları ve ortak serial fullphase commit öncesi root kapılarıdır.

## K-275 — P11 host PDF adayı

K-275/ADR-133/GB-091 sabit owned PDF host profili: mevcut Zee JSON örneği, exact ReportLab4.4.9/pypdf6.10.0/Pillow12.3.0, stdlib8 normal ve hostbackend8 explicitignored ayrımı. Sabit kaynak/font/notice, bounded job/output ve nofollow roller korunur. Finalclean S native emitter+backend8 ortak kohortu ayrı zorunludur; tarihsel dirty kanıt temiz S sayılmaz. P11/B3 Zee içinden invocation/resmi signedplugin, ürün/adoption/P08/platform ve proje lisansı açıktır; full900/1800/RC/soak/kurulum kabulü ileri sürülmez. Üretilmiş QA PDF/PNG/evidence run dosyaları source tree’ye taşınmaz; golden assets sabit girdi fixture’larıdır.

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
