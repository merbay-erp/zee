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
