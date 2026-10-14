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

Çalışma ağacında tam faz **888/894, 0 başarısız, 6 dedicated atlama**;
aday native PostgreSQL on bir vaka, PowerShell altı vaka, fmt, sürekli
güvenlik ve istisnasız yeni sır taraması geçti. Temiz exact kaynakta aynı
faz, dedicated 5 TLS + 1 göç + 11 parola/COMMIT, PowerShell ve kaynak ZIP
kabulü takip kanıtıyla kaydedilecektir. Bunlar henüz yeni exact kabul veya
yeni imzalı ikili iddiası değildir.

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
