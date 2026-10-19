# Sürüm runbook'u

K-169/ADR-071'in bağlayıcı sürüm günü sırası. Her adım makine kapısıdır;
atlanmaz.

1. **Uyumluluk ve sürüm kaydını önce hazırla.** `docs/deprecation-kayitlari-v1.tsv`
   ve `dil-yuzeyi-v1.tsv` bu sürümde giren/kaldırılan yüzeyi taşır; sürüm notu
   `docs/surumler.md` içinde “Yolda” bölümünden sürüm başlığına taşınır. Bu
   değişiklikler kaynak S dondurulmadan önce commitlenir.
2. **Temiz exact kaynak ve kapılar.** `bash scripts/guvenlik-kapisi.sh --surum-adayi`
   (exact HEAD için RC fuzz kanıtı, clippy, tam test, koruklar; explicit dış paket
   seçimi aşağıdaki K-265 akışındadır). Ayrıca `tani_kalitesi`, `spec_drift`,
   `islev_egilimi` ve faz matrisi yeşil. Başarılı gate ile etiket arasında S
   değişmez; yeni commit yeni kaynak/gate kabulü ister.
3. **Etiket.** `git tag vX.Y.Z` → `surum-adayi`, `kurulum-tatbikati` ve
   `soak` workflow'ları tetiklenir. Etiket sonrası Cargo sürümü
   `X.(Y+1).0-dev` serisine çekilir (`uyumluluk_testi` bunu ister).
4. **Artefakt.** `surum-adayi` artefaktı (`dil`, `dillsp`, `SHA256SUMS`,
   `sbom.spdx.json`, `provenance.intoto.json`, varsa `SHA256SUMS.zee-imza`)
   indirilir; yerel iki-klon özetiyle karşılaştırılır.
5. **Tatbikat.** Üç platformda `kurulum-tatbikati` yeşil; exact commit+OS
   `zee-calisma-ortami-1` raporları kurulu ikili hash'ini, sistem kitaplıklarını
   ve Rust araçları PATH dışında program/LSP provasını taşır. Yerelde
   `bash scripts/kur.sh --artefakt <klasör>` → `dil sürüm` →
   `bash scripts/kaldir.sh`.
6. **Duyuru.** `yayinla` işi her platform imzasını sabit
   [yayıncı anahtarına](yayinci-anahtari-v1.txt) karşı doğrulayıp paketleri GitHub
   Releases'e koyar (ADR-156); imzasız artefakt yayımlanmaz. Sürüm notu, yayıncı
   anahtar kimliği ve `SHA256SUMS` içeriği yayımlanır; README canlı sayıları `depo_sayilari --yaz` ile yenilenir.

## Kullanıcı kurulumu

```bash
bash scripts/imza-dogrula.sh ~/İndirilenler/zee-surum      # önce imza (OpenSSL 3+; macOS: brew install openssl@3)
bash scripts/kur.sh --artefakt ~/İndirilenler/zee-surum      # ~/.local/bin
dil sürüm
bash scripts/kaldir.sh
```

Windows: `powershell -File scripts/kur.ps1 -Artefakt <klasör>` ve
`scripts/kaldir.ps1`. Betikler yalnız `SHA256SUMS` ile doğrulanan `dil`/`dillsp`
ikililerini kurar, var olan dosyanın üzerine yazmaz ve yalnız kendi
manifestindeki dosyaları kaldırır. Editör bağı için `editors/README.md`.

## K-194 kurulum hata sınırı

İkililerin tamamı geçici alanda kopyalanıp SHA-256 ile doğrulanmadan hedefe
ikili yayımlanmaz. Hedefte yabancı dosya/kırık sembolik bağ ve yinelenen
checksum kaydı reddedilir. Yayınlama hatasında bu koşunun oluşturduğu
ikili dosyalar geri alınır; manifest en son yayımlanır ve yollar mutlak olur.
Manifest UTF-8/BOM'suz/LF'tir; PowerShell kaldırma açık UTF-8 okur, Türkçe
karakterli hedef/veri yolları gerçek ikili testindedir. Windows PowerShell
5.x kaynak kodlamasının gerekçesi [Microsoft kodlama belgesindedir](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_character_encoding).
Unix dosya sistemi hard-link desteklemelidir; Windows File.Move var olan
hedefi ezmez. Güç kaybı/SIGKILL sonrası otomatik uzlaştırma bu sözleşmede yoktur.
`kurulum_testi` artık Windows'ta da gerçek ikililerle ret/kur/kaldır akışını
çalıştırır; platform sonuçları [backlog](oncelikli-backlog.md) durumuyla izlenir.

GB-032/K-194: `GIT_SHA` varsa exact 40 küçük hex olmak zorundadır; eksik
legacy metadata yalnız `-` kaydı üretir. Hedef/veri yollarında TAB/CR/LF
reddedilir; kanonik fiziksel yol da denetlenir. Bu alanlar manifest satırı
ve sahiplik kaydı oluşturamaz. Bilinçli yerel manifest değiştirme ve aynı
kullanıcı kimliğiyle düşman süreçlere karşı OS sandbox'ı vaat edilmez.

### K-244 — PowerShell literal kaldırma yolları

`kaldir.ps1`, manifestin varlığını ve kayıtlı dosyaların varlık/hash değerini
`-LiteralPath` ile denetler; dosyaları ve manifesti aynı literal yolla siler.
`bin[1]` gibi bir kurulum yolu wildcard eşleşmesi olarak yorumlanmaz. Aynı
hash'e sahip `bin1` kardeşindeki yabancı dosyalar ve kardeş manifest korunur.
Değişmiş hash zorlamasız kaldırmayı bütün dosyalar silinmeden reddeder;
`-Zorla` yalnız hash reddini kaldırır, manifestteki literal yol kapsamını
genişletmez. Eksik/bilinmeyen şemalı manifest silme etkisinden önce reddedilir.

`scripts/tests/kurulum-literal-yol-test.ps1` sahipli geçici alanda altı vaka
çalıştırır: normal, Unicode, köşeli parantez ve aynı hash'li yabancı kardeş,
değişmiş hash ile bütün dosyaların korunması, literal zorla kaldırma ve
eksik/bozuk şema reddi. Sentetik ikili baytları çalıştırılmaz; her koşu geçici
alanını kaldırır. Windows'ta mevcut `kurulum_testi` gerçek ikili vakası ve
tatbikatın Windows adımı bu native PowerShell kampanyasını çağırır; yeni
libtest vakası yoktur. Kaynak UTF-8 BOM ve PowerShell 5.1 sözdizimi korunur.

Yerel macOS arm64 PowerShell 7.5.11 provasındaki önceki betik, kurulu iki
dosyayı bırakıp aynı hash'li yabancı kardeşteki iki dosyayı silerek hatayı
yeniden üretti; literal düzeltmeyle altı vaka geçti. Runtime resmî Microsoft
release tar'ı ve yayımlanmış checksum dosyasından exact SHA-256 ile doğrulandı,
yalnız geçici portable dizinde çalıştı. Bu semantik kanıt fiziksel Windows,
PowerShell 5.1 çalıştırma, minimum OS veya uzak CI kabulü değildir; bu sonuçlar
ayrı kanıt ister. [ADR-071](../adr/071-kurulum-kaldirma-sozlesmesi.md).

K-195: Windows MSVC üretimi de `scripts/surum-artefakti.sh` ile iki temiz klon ister. Statik CRT tarifi provenance alanlarında görülmeli ve çalışma ortamı kapısı dinamik CRT importunu reddetmelidir. Eski tek-derleme tatbikat artefaktı tekrar üretilebilir sürüm yerine kullanılamaz.

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

Tarihsel K-196 Linux tatbikatı kuruluş koşucusunda başlamadan kuyrukta
kalmıştı. O sıradaki linux_runner geçişi K-217 ile kaldırıldı; GitHub
workflow'ları sabit hosted hedefler taşır ve mevcut repo Actions'ı kapalıdır.
Kurum işi özel Git2/Gitsrv akışında yürür (ADR-080/088). Önceki run/SHA
kanıtları tarihsel kalır; güncel çapraz derleme gerçek Windows runtime
kabulü yerine konmaz.

K-197 ayrı temiz Windows provasını mevcut successful release ile başlat:

```sh
gh workflow run kurulum-tatbikati.yml --ref codex/zee-derleme-bagimlilik -f mode=clean-windows -f source_run=38244549913 -f source_sha=c7e3d04adf27905ade8b622d24bd76762123d2dc
```

Source_run/source_sha aynı depodaki incelenmiş exact release'e ait olmalı;
90 günlük artefakt süresi dolduğunda yeni successful release kimliği
kullanılır. Bu kip iki-klon üretimini tekrarlamaz, ready byte'ları temiz
Nano Server'da sınar. Varsayılan/tag kipinde üç platform release tatbikatı
sürer. Prova kaynak SHA'sı release SHA'sından ayrı kayıttadır.

K-196 kapanma: b153064/run 38252942210; iki klon eş hash ve gerçek
kur/program/LSP/kaldır üç platformda geçti. GCC 13.3.0, Apple Clang 21.0.0
+ SDK 26.5, MSVC FileVersion 14.51.36260.0 + SDK 10.0.26100.0 dört
link kaydında doğrulandı. Ham runtime/provenance ve artifact hash'leri
[platform defterinde](derleme-platform-kaniti-v1.tsv) korunur. K-192 minimum OS, bütün native C
araçları ve fiziksel temiz makine kanıtı ayrıca açıktır.

K-198 aynı stock Server Core container'ında kur/program/LSP/kaldır:

```sh
gh workflow run kurulum-tatbikati.yml --ref codex/zee-derleme-bagimlilik -f mode=clean-windows-install -f source_run=38252942210 -f source_sha=b1530641a3f902047b57ad6e7c0c004ec5f1c420
```

Kaynak kendi incelenmiş successful release koşusudur; release ve prova
commit'i farklı olabilir ve ayrı kaydedilir. Server Core run 38255788989/job 114833388588
kur/program/LSP/kaldır kapısını geçti ve K-198 bu kapsamda kapandı. Masaüstü/minimum OS/fiziksel macOS ayrı.

K-199 compiler/wrapper ENV/config gerçek küçük Cargo provası:

```sh
gh workflow run kurulum-tatbikati.yml --ref codex/zee-derleme-bagimlilik -f mode=toolchain-probe
```

Üç GitHub platformunda mevcut exact 1.93.1 kurulumu kullanılır; depsiz
probe network/offline Cargo ile derlenir. Resmî release/offline helper
RUSTC'yi exact kurulu executable'a bağlar ve wrapper'ları kapatır.
Custom compiler/wrapper geliştirme resmi release'in dışında kendi
Cargo çağrısıyla yapılır. Tam Zee release, tüm Cargo profil/linker ve
native C envanteri bu küçük probe'un kapsamı değildir.

K-200 macOS release caller deployment ENV'ını kullanmaz; exact Rust
default'unu C/Rust için ortak uygular ve a/b native .o başlıklarını
provenance'a kaydeder. Üst minos/yanlış platform/eksik toplam native
kanıt yayımlamayı durdurur. Bu target başlığı gerçek minimum OS
çalıştırma/desktop desteği değildir; o kapı ayrıca korunur.

K-202: ana GitHub/CI deposu `merbay-erp/zee`; `itwise-bilisim/zee` yalnız
ayna ve repository Actions kapalıdır. SBOM/provenance yeni ana kaynak
URI kullanır; eski CI/fixture kanıtları tarihsel bağlantılarıyla korunur.

### K-216 — Git2 / Gitsrv ana CI koşusu

Kullanıcı isteğiyle GitHub Actions kapalıdır; ana kaynak GitHub'da,
Enterprise yalnız aynadır. Özel Git2'de `ci/zee-gitsrv` dalına exact
kaynak gönderimi veya `zee-ci.yml` manuel başlatması Linux kapılarını
çalıştırır. Git2 erişim adresi/kimliği özel ortamda kalır. Örnek:

```sh
git push git2 HEAD:refs/heads/ci/zee-gitsrv
```

`.forgejo/workflows/zee-ci.yml` yalnız seçili dalın tam geçmişini alır;
PR kodu ve ilgisiz özel dallar çekilmez. İki action SHA'sı ayrı
`docs/forgejo-actions-pinleri-v1.tsv` envanterinde resmî kaynağa bağlıdır.
Faz/kalite, gizlilik/sır, tedarik/offline vendor, WASM ve eşiksiz shared
performans gözlemi aynı kaynakta çalışır; raporlar 90 günlük özel artefakt
olur. Uzak başarısızlık veya başlamamış iş geçiş sayılmaz. Windows runtime
ve release candidate fuzz/Miri/kurulum kanıtları bu Linux CI'ın yerine
geçmez; ilgili sürüm runbook kapıları ayrıca korunur. [ADR-088](../adr/088-git2-gitsrv-kosulari.md).

K-241 üretici kabulünde iki klonun derleme öncesi/sonrası temizliği ve
gerçek ikililerin exact SHA/`dirty: false` kimliği ayrıca zorunludur.
Build logu/native raporu kaynak klonuna yazılmaz. Eş hash'e rağmen kirli
kimlik veya değiştirilmiş klon sürüm kümesi yaratmadan reddedilir;
`--kirli-kabul` yalnız caller'ın HEAD'ini üretir. [ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md).

## K-265 explicit dış RC kanıtı

[Exact kaynak dış kanıt rehberi](rc-kaniti.md) ve [ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md) temiz S’ye bağlı dış paketi tanımlar. Explicit geçersiz paket fallback yapmaz; etiket/build S’de, append-only arşiv commit’i E’de S kimliğini korur. Default tracked yolun self-reference sınırı açıkça korunur. Gerçek uzun kampanya henüz bu kısa entegrasyon testlerinden tamamlanmış sayılamaz.

## 15 Ekim 2026 — S kanıtı ve E arşivi

Kaynak S=`8b49113c413f62b35d4b251ed31068524d2fe26c`, sonraki belge/kanıt commit’i E’dir. Temiz S üzerinde dört ASAN hedefi ayrı ayrı1801sn geçti; lexer_parser7.117.460, morfoloji8.265.144, http_istegi11.194.063 ve wasm_abi9.011.299 yürütme kaydedildi. İki seçili Miri grubunda toplam3test(ondalık2,zaman1) geçti. Kanonik export/check ve aynı S güvenlik kapısı510.198758sn exit0 ile geçti; kapının118 Cargo/libtest grubunda943passed/0failed/16ignored vardır. GB082 yalnız bu exactS yerel protokol ve kanıt kapısı için kapalıdır; E kaynakS gibi kabul edilmez.

İlk producer PASS paketinin HTTP dış702.404sn/raw1921sn ve WASM dış695.844sn/raw1914sn süre uyuşmazlığı nedeniyle export reddi korunur. Recovery ayrı kampanyadır; eski dalganın sonuçlarıyla birleştirilmez. Duvar saati ve monotonik saat kayıtları ayrıdır.

[RC sonuç makbuzu](incelemeler/urunlesme-20261014/final-8b49113/rc/FINAL-RESULT.json), [aynı-S kapı makbuzunun türetilmiş temsili](incelemeler/urunlesme-20261014/final-8b49113/rc/POST-GATE-RECEIPT.derived.json), [bağımsız kök incelemesi](incelemeler/urunlesme-20261014/final-8b49113/rc/root-independent-review.json). Gate `--surum-adayi --rc-kanit PATH --rc-kanit-sha256 HASH` ile aynı temiz S’de gerçekten exit0 verdi; bundle SHA256 `5e174eb9179e79e75b5127e12c1760b65d448704256ad69dbfa3abd5b343805c`. S source/runtime before/end pinleri değişmedi.

Tracked fuzz ve soak geçmişine root yalnız doğrulanmış actual S satırlarını append eder; eski koşular değiştirilmez. E S gibi test edilmiş/etiketlenebilir kabul edilmez; S için üretilmiş releaseartefaktı ve tag kimliği S kalır. İmza/yayın/tag/push yapılmadı. Bütün ürün/insan/platform/IDE kapıları bu dar kabulden çıkarılmaz.

Linux411external/127metadata(120raw+7derived) ayrımı korunur; final RC metadata assembly sonrasında root current arşiv manifestini tamamlar. [Güncel dar S raporu](urunlesme-final-kabul-20261015.md).
