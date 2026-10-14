# ADR-097 — Sürümlü proje runtime ayarı

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-234, ürünleşme P04/A6(2), GB-057

## Karar

A6(2)'nin ordinary gelecek ayarını uyarıyla geçme isteği, bütün bilinmeyen
anahtarların güvenli varsayılması anlamına gelmez. `zee.ayar` JSON zarfı
`sema=zee-ayar-1`, `surum=1`, `ayarlar` ve `guvenlik` nesnelerini zorunlu
kılar. Üst alanlar exact'tir; şema/sürüm bilinmiyorsa kaynak çalıştırılmaz.
`ayarlar` şema 1'de davranış etkisi olmayan uzantı dalıdır: alanlar değer
basmadan sabit uyarıyla atlanır. Güvenlik ayarları yalnız `guvenlik` dalında
tek explicit allowlist ile kabul edilir. Unknown güvenlik anahtarının adı
üzerinden heuristic sınıflandırma yoktur; bilinmeyen alan ve yanlış bilinen
alan türü/aralığı/bağımlılığı fail-closed exit 2'dir. Duplicate JSON anahtarı
her derinlikte reddedilir; last-key-wins yoktur.

İzinli ayarlar mevcut çocuk/web/proxy/bellek oran bayraklarına bağlanır.
Config kendi başına yetkinlik bildirimini genişletmez. Güvenli mevcut
default → config → explicit CLI field precedence uygulanır; config'in ilk
validasyonu override'dan önce yapılır. Kompozisyon sonrası mevcut CLI'nin
tekrar/mod/çocuk ve oran bağımlılık validator'ı son kabul sahibidir.

Config yalnız `dil çalıştır <proje-dizini>` kökündeki sabit dosyadır;
CWD/ancestor/home keşfi ve standalone kaynak config'i yoktur. Normal
`proje.dil`, 64 KiB config, 4096 bayt/64 bileşen root, parent traversal ve
symlink reddi, bounded read/nofollow ve standart JSON recursion sınırı
kullanılır. Hatalar kullanıcı config anahtar/değerlerini veya yolları
kopyalamaz. Aynı kullanıcıdan çalışan proje ağacı mutasyonu atomik FS
transaction kabulü değildir; Windows fiziksel reparse/minOS ayrı açıktır.

## Geri dönüş

İlk okuyucu tabanı 0.8.0-dev + `zee-ayar-1` capability'dir. 0.7'de okuyucu
yoktur; eski binary `zee.ayar` güvenlik ayarını sessizce atlayabilir. Gerçek
önceki küçük sürüm rollback **açıktır**. Schema 1 tüketicisinde gelecekteki
olağan alanı atlama simulated kabulü yalnız config-aware tüketiciyi
kanıtlar, okuyucusuz eski binary'yi değil.

Authenticated compiler selector'ın config projeyi başlatmadan önce `dil
sürüm --ayar-semasi` sorgusunda exit 0, `zee-config-capability-1`, destekli
`zee-ayar-1`, `unknown_security=reject` ve build version eşliğini doğrulaması
gerekir. Probe eksik/başarısız/bozuk/uyumsuzsa başlatılmaz. Ayrı capability
sorgusu mevcut `zee-build-v1` CLI/LSP/SBOM kimlik eşliğini bozmaz.

## Etki ve kanıt

[Runtime ayarı rehberi](../docs/zee-ayar.md), gerçek CLI kabul/ret vakaları
`cli_ayar_testi`, `dil_ayar::tests` effective web/worker/oran doğrulaması,
`cli_secenek_testi` ve `build_kimligi_testi` bu adayın zinciridir. Var olan
RFC-0028/RFC-0032, spec/27/spec/12, ADR-064/090 güvenlik ve sürüm politikası
korunur. Yeni RFC veya gramer ihtiyacı yoktur. Internal CLI modülü mevcut
`dil_cli` sahibindedir; API/ABI/export, dependency, owner/kenar/SCC, yeni
parser/checker tanı kodu, cache/soak ve sabit benchmark workload değişmez.
Source freeze beyanı correctness/security, semantic davranış beyanı bu
ADR'ye bağlanır; ortak spec/yüzey/faz/sayı/indeks ve GB-057 kaydı source ile
aynı toplu değişiklikte tamamlanır. Native local aday testi exact temiz
checkpoint/remote CI, gerçek önceki minor rollback veya ürün deployment
kabulü yerine geçmez.
