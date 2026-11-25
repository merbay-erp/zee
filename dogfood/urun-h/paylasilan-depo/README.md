# K-394 — Ayrı projeler arasında paylaşılan ek deposu (ürün H, RFC-0045)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün deposu özeldir; sunucu adı, alan adı, adres ve yerel yol taşınmaz
(ADR-087).

| Exact ürün dosyası (ürün commit'i `35df031accc4d010ed1ca3db1f2c479df40f2916`, 24 Kasım 2026) | Git blob | SHA-256 |
|---|---|---|
| Sürtünme kaydı (`ZM-F038` satırı) | `f6075725d63356b6a40ff96e7ec656a4a0fd33f6` | `2946a1105af457f3e7b6650f66dfff12ea07d6b84f82578892618cc7f6fdd437` |
| Web ek indirme rotası (`… adıyla ek olarak göndermeyi dene`) | `0dc025eab3bdfe7fd8ea289eb5e724d4c1246a6e` | `53855291b64d9b300c14ffafae985c316cc23f928a659bfb45149809f6eed942` |
| SMTP alıcısı (bugünkü ek uyarısı ve 256 KB kesimi) | `5567d6abadc61b34f35161e9bf56bd47ba8c492e` | `74d0ae1648bb868e349cb1f7e963ce9104da78d9a8899275dac088fc49c78c1b` |

## Sürtünme (ZM-F038)

Ürünün SMTP alıcısı, gönderim kapısı ve IMAP hizmeti web panelinden ayrı Zee
projeleridir (ayrı veritabanı rolleri, en az ayrıcalık). K-389 ile ekleri
bayt-kesin çözebiliyorlar; ama ayrı web projesi aynı dosyayı sunamıyor:
proje dosya sınırı her projeyi kendi köküne kapatıyor, `..`/mutlak yol
yasak, başka projenin klasörüne giden sembolik bağ reddediliyor. Ürün
bugün eki görünür uyarıyla kesiyor, ekli gönderimi `554` ile reddediyor.
Geçici çözümler (hizmetleri web köküne taşımak, ekleri PostgreSQL `bytea`
saklamak) en az ayrıcalığı ya da 64 KiB parametre sınırıyla 16 MiB eki
bozduğu için reddedildi. K-164 kataloğunda sınıf "yerleşik yüzey eksik";
K-382'nin `veri_kökü` takip önerisinin ikinci gerçek tüketicisidir.

## Kesit

- [alici/](alici/) — `veri-okuma` + `veri-yazma`: DATA bloğu proje kökünde,
  ek `veri:gelen/` altına çözülür, içerik adresli `veri:ekler/<kiracı>-<sha256>`
  adına no-clobber taşınır.
- [web/](web/) — yalnız `veri-okuma`: aynı kökten RFC-0041 dosya yanıtı.

İki proje aynı `veri_kökü "POSTA_DEPO"` adını bildirir; mutlak yol işletim
ortamında (systemd `SetCredential=`/`Environment=`) verilir. Değişiklik
öncesi her iki `proje.dil` P001 (`veri_kökü` alan değil) verirdi; şimdi
`denetle` temizdir. Gerçek süreçte paylaşım, salt okur web, sembolik bağ
ve hard-link reddi `compiler/tests/veri_koku_testi.rs` ile sınanır.
