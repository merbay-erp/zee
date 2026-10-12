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
