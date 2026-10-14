# Zee çekirdek D3 credential host köprüsü

K-245/P25'in kaynağı kayıtlı `zee-cekirdek` ürününün D3 talebidir. Ürün
commit'i `cd5d12f0008620819e6167af5146e83503fd2b38`; exact harici talep
`/Users/mustafaerbay/IdeaProjects/zee-cekirdek/docs/dil-talepleri/2026-10-14-zee-urunlesme.md`
satır 180'dir. Talep dosyası SHA-256
`694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7`.
Bu kayıt ortam allowlist'i, credential dosyası ve hata/günlük maskelemesi
sürtünmesinin kökenidir; özel müşteri, sunucu veya sır değerleri içermez.

Mevcut üründe ortam bağlarını taşıyan exact dosyalar:

| Harici ürün dosyası | Kullanım | SHA-256 |
| --- | --- | --- |
| `cekirdek/proje.dil` | SMTP satır 12, gizli anahtar 14, DB 17 ortam adı bağları | `310ef82a60af6827e5b828852f178faa0d7962a18210452a329610222722c169` |
| `cekirdek/kaynak/eposta_ayarlari.dil` | Açık SMTP değeri yoksa mevcut ortam fallback'i | `8116cd0de756e40840841377c9ca1ea212ad79aae564f0c894ca0e5a91342edf` |
| `cekirdek/kaynak/rota_ayarlar.dil` | Mevcut native SMTP gönderim çağrısı | `fb31ae9cd296e93649c96acfe7682cc2f15d2ae87f0b629a3788216fbdf4a543` |

Bu dosyaların byte'ları talep commit'iyle eşleşir; gerçek ürün kaynağı
değiştirilmedi. İncelenen 68 `.dil` dosyasında `/run/secrets` veya
`CREDENTIALS_DIRECTORY` okuyan kaynak yoktur. Talep ve mevcut env bağları
kanıtlıdır; canlı file-credential entegrasyonu henüz kanıt değildir.

Minimal çözüm [credential-baslat host aracı](../../../docs/credential-baslat.md)
ile doğrulanan dosyayı yalnız çocuk env'sine aktarmaktır. Compiler grammar,
API, config veya proje yetkisi büyümez. Anonim reproducer
`scripts/tests/credential-baslat-test.py` içinde üretilen sahipli geçici
projedir; yeni izlenen `.dil`/golden eklenmedi. Gerçek native CLI ortam
allowlist'i kapalıyken erişimi reddeder; owned loopback SMTP fixture gerçek
native gönderimde dosya kaynaklı anonim AUTH ve tek DATA'yı gözler. Bunlar
canned IO sonucu değildir. Fixture credential'ları public sentineldir;
ürünün credential/hostname/değerleri kopyalanmaz.

Dosya/argv/env yetki olumsuzları, FIFO/TOCTOU/boyut, sir stdout/stderr'inin
metadata-only bastırılması ve restart-only rotasyon aynı host testinde
yer alır. Başlangıçta eksik `yerel-ağ` izni nedeniyle reddedilen SMTP fixture
doğru mevcut izinle onarıldı; compiler bugfix/regression olarak kaydedilmez.

P25 kısmi kalır: Windows credential DACL/handle, gerçek Linux systemd servis
kurulumu, canlı ürün/provider, full P08 korpusu ve süreç belleğinde/env'de
file-native gizlilik tamamlanmış değildir. Host köprüsü systemd dosyasını
env'ye dönüştürdüğü için child inheritance sınırını güçlendirmiş gösterilmez.
