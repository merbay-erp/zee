# Sürüm runbook'u

K-169/ADR-071'in bağlayıcı sürüm günü sırası. Her adım makine kapısıdır;
atlanmaz.

1. **Temiz ağaç ve kapılar.** `bash scripts/guvenlik-kapisi.sh --surum-adayi`
   (exact HEAD için RC fuzz kanıtı, clippy, tam test, koruklar). Ayrıca
   `tani_kalitesi`, `spec_drift`, `islev_egilimi` ve faz matrisi yeşil.
2. **Uyumluluk kaydı.** `docs/deprecation-kayitlari-v1.tsv` ve
   `dil-yuzeyi-v1.tsv` bu sürümde giren/kaldırılan yüzeyi taşır; sürüm notu
   `docs/surumler.md` içinde “Yolda” bölümünden sürüm başlığına taşınır.
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
6. **Duyuru.** Sürüm notu, yayıncı anahtar kimliği ve `SHA256SUMS` içeriği
   yayımlanır; README canlı sayıları `depo_sayilari --yaz` ile yenilenir.

## Kullanıcı kurulumu

```bash
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

K-196 Linux tatbikat işi kuruluşun `itwise-gitsrv` koşucusunda başlamadan
kuyrukta kaldı. `kurulum-tatbikati` manuel girişindeki `linux_runner`
seçimi `configured` (mevcut kuruluş ayarı, varsayılan) veya `ubuntu-latest`
olabilir. Bu koşuya özel seçim kuruluş/repo değişkenlerini değiştirmez;
tag koşuları mevcut kuruluş ayarını izler. Aynı exact kaynağın üç platform
kanıtı için `gh workflow run kurulum-tatbikati.yml --ref codex/zee-derleme-bagimlilik -f linux_runner=ubuntu-latest`
ile GitHub Linux koşucusunda yeni tatbikat başlatılır.

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
