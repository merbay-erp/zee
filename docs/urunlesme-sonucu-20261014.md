# 14 Ekim 2026 ürünleşme teknik teslimi

Uygulanmış teknik dilimlerin kod, test, belge ve exact kaynak kayıtları
aşağıdadır. K-241 sonrası kanonik iki-klon ve gerçek temiz ikili kimliği
yerel macOS kabulünü geçti.
**39 maddelik ürünleşme planının bütünü tamamlanmadı.**
Çalışan kapasite ve kalan kabul sınırları
[plan envanterinde](urunlesme-plan-v1.tsv) ayrı tutulur.

Son temiz yerel teknik checkpoint:
`2e970e47d59e2408eff836cdacef9bd0fe0b3028`.
[K-241 / ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md) logları kaynak
klonunun dışına taşır; derleme öncesi/sonrası klon temizliği ve gerçekten
derlenen ikilinin exact SHA/`dirty: false` kimliği kanonik kapıda doğrulandı.
Bu raporu tamamlayan takip commit'i yalnız belge ve kanıt içindir.
Aşağıdaki koşular o belge HEAD'ine veya uzak CI'a aitmiş gibi yeniden
etiketlenmez.

## Çalışan değişiklikler

| Alan | Teslim edilen kapsam | Ayrıntı |
|---|---|---|
| CLI ve build kimliği | Bilinmeyen seçenek exit 2; CLI/LSP/worker/SBOM kaynak kimliği eşliği; SHA, UTC kaynak tarihi, kanal ve kirli ağaç bilgisi; kanonik a/b/çıktı exact SHA ve `dirty: false` kabulü | [ADR-093](../adr/093-deterministik-build-kimligi.md), [ADR-102](../adr/102-surum-klonu-ve-build-kimligi.md) |
| Proje ayarı ve seçici | Sürümlü `zee.ayar`, güvenlik anahtarlarında ret, CLI önceliği; imzalı yerel katalogdan exact proje pini ve özel ikili snapshot'ı | [Ayar](zee-ayar.md), [pilot seçici](derleyici-secici.md) |
| PostgreSQL | Gerçek CA/hostname/NULL ve eşik/üstü bütçe testleri; yazmadan göç durum/kuru planı; bcrypt→Argon2 CAS yarış/rollback/commit görünürlüğü | [TLS kapısı](postgresql-tls-ci.md), [parola kapısı](parola-goc-ci.md) |
| Ürün yardımcıları | Mevcut modülün metin ankrajı olmadan kur/kaldır akışı; anonim form yardımcıları; formatter `--denetle` ile yazmadan exit 0/1 | [Modül kanıtı](../dogfood/zee-cekirdek/modul-kompozisyonu/README.md), [ADR-094](../adr/094-bicimleyici-yazmadan-denetim.md) |
| E-posta | Typed MIME, Reply-To, HTML alternatif ve sınırlı PDF/görsel ekleri; dosya/FIFO koruması; SMTP kimlik bilgisinin C029'da maskelenmesi; LF/EOF öncesi 16 KiB yanıt bütçesi | [RFC-0034](../rfcs/0034-ekli-eposta.md), [ADR-099](../adr/099-smtp-hata-maskelemesi.md), [ADR-100](../adr/100-smtp-yanit-okuma-siniri.md) |
| Güvenlik ve bakım | Compiler/fuzz lock'larında rustls 0.23.46; özel inceleme bağlantılarının izole geçmişten temizlenmesi; aynı sorumluluk sahibinde handler/test/profil ayrımı | [ADR-101](../adr/101-rustls-advisory-yamasi.md), [SHA eşlemesi](incelemeler/urunlesme-20261014/sanitized-sha-map.tsv) |

Yeni Rust facade export'u veya kalıcı önbellek eklenmedi. MIME kalıbının
ürün gerekçesi ve uyumluluk kaydı kendi kaynak dilimindedir. K-174 gözlem
penceresi **14 Ekim–11 Kasım 2026** olarak sürer.

## Son doğrulama

| Kapı | Gerçek sonuç | Kapsam ve sınır |
|---|---|---|
| Tam faz matrisi | **887 geçti, 0 kaldı, 6 atlandı; toplam 893**, 138.698 sn | Altı dedicated PostgreSQL vakası aşağıdaki ayrı koşuda yürütüldü. [Faz raporu](incelemeler/urunlesme-20261014/final-2e970e4/faz-test-matrisi.md) |
| Dedicated PostgreSQL TLS/bütçe | **5/5**, sıfır atlama | İki CA/hostname negatif yolu, NULL/tür, tam eşik ve bir fazlası |
| Dedicated göç | **1/1**, sıfır atlama | Read-only durum/kuru, roller, checksum, rollback ve yarış |
| Yerel parola CAS | **7/7**, sıfır atlama | Gerçek PostgreSQL transaction/row-lock; canlı ürün login'i değildir |
| Sürekli güvenlik | **Geçti** | Compiler/fuzz deny, stable fuzz derlemesi, tedarik ve bulgu kapıları; ASan/Miri RC koşusu değildir |
| Kanonik iki temiz klon | **`dil`/`dillsp` eş SHA-256; a/b/çıktı exact SHA ve `dirty: false`** | Rust 1.93.1, macOS arm64, release profili; her klonda 24/24 native nesne başlığı 11.0. İmzasız yerel kabul; yeni Linux/Windows veya fiziksel minimum OS kabulü değildir. |
| Diğer incelemeler | Fmt, clippy, katalog/tazelik, katman/SCC/API, uyumluluk, semantic/core-freeze, spec/tanı, gizlilik/sır ve offline vendor geçti | Spec 203 maddede 196 kanıtlı, 5 kısmi, 2 açık; bir doctest ayrıca geçti |

[Koşu özeti ve log hash'leri](incelemeler/urunlesme-20261014/final-2e970e4/gate-summary.json),
[native kanıtı](incelemeler/urunlesme-20261014/final-2e970e4/native-proof.json)
ve [iki-klon/gerçek ikili kimliği kanıtı](incelemeler/urunlesme-20261014/final-2e970e4/build-proof.json)
exact 2e970e4 checkpoint'ine bağlıdır. README'nin statik test sayısı koşuda geçen
test sayısı yerine kullanılmadı; ayrı koşuların örtüşen vakaları toplanıp
yeni bir tekil test toplamı üretilmedi.

Native koşu, [resmi PostgreSQL **16.15** kaynak kaydından](incelemeler/urunlesme-20261014/final-2e970e4/postgres-runtime-build-proof.json) geçici dizinde derlenen
ve yalnız bu çalışmanın yönettiği loopback kümelerinde yürütülen kabulüdür.
Kaynak ve ikili başlangıç/sonunda aynı kaldı; sahip olunan kümeler kapatıldı.
Native testlerin Rust sürümü 1.98.1, kanonik release üretiminin sürümü
1.93.1'dir. Bu kabul Docker imajı, müşteri sunucusu, temiz fiziksel OS veya
uzak CI kabulü değildir.

2e970e4 exact kaynağının kabul edilen yerel iki-klon çıktı hash'leri:

```text
dil     348d0691b7a7714b3b8f5495e2dae6d8bf566d8c2b5d73219af87e6d958606c5
dillsp  d954a76f2aff12210cad4ec26e056b9ee258e5a12989af8c7f5b40578b841841
```

## Açık kalan kapasite ve kanıt

| İş | Kalan gerçek iş |
|---|---|
| P06/P30 dağıtım ve RC | K-241 yerel macOS temiz kimlik/iki-klon kabulü geçti; yeni exact Linux/Windows üretimi, gerçek yayıncı imzası, kurulum ve birleşik ASan/Miri RC kabulü ayrıca gerekir. |
| P08 ürün korpusu | Beş tam anonim site ve çekirdek korpusu hazır değil; kullanıcı açık tutulmasını istedi. Küçük fixture'lar tam ürün N/N, süre/RSS veya staging kabulü sayılmaz. |
| P09/P13/P14/P25 | Kalıcı iş kuyruğu/cron, readiness/log/metrics/draining/reload, socket devriyle restart ve dosyadan credential izin/symlink/rotasyon hattı tamamlanmadı. |
| P07/P11/P18–P23 | Tam compiler yöneticisi/kurulum, PDF backend, genel modül/layout/çeviri/test kompozisyonu, lint/JetBrains ve artımlı LSP/200 ms ölçümü açık. |
| Canlı entegrasyon | Altı gerçek form/staging, canlı parola/veri göçü, provider/queue, C027 COMMIT yanıt kaybı, CA rotasyonu/TLS 1.1 negatif yolu ve global RSS/full audit açık. |
| İnsan ve platform | K-161/K-162 katılımcı oturumları; temiz fiziksel Windows/macOS/minimum OS kabulü yok. Geliştirici Mac'i temiz cihaz kanıtı değildir. |
| Yayın ve kaynak | Lisans seçimi bekletildi; sınırlı pilot destek seçildi. Trusted yayıncı anahtarı, yeni exact uzak CI/primary kaynak yetkisi, GB-046 GitHub destek cevabı ve birleşik RC/V1 kapıları açık. |
| Bilinçli erteleme | P17 S3, P26 sanitizer API, P35 yeni ergonomi grammar'ı ve P38 native/WASM backend. |

[GitHub destek metni](incelemeler/k215/github-destek-talebi.md) hazırdır;
kullanıcı gönderecek. Public açma, deploy, push, PR veya imzalı release
yapılmadı. Mevcut kullanıcı checkout'u ve müşteri servisleri korunur.

## Kaynak paketi ve tarihsel kanıt

Yerel `zee-source-2e970e4.zip`, **1134** Git archive dosyasını exact ZIP
comment ve `KAYNAK-SHA256.txt` ile doğrular. Tek fazlalık bu manifesttir.
ZIP SHA-256:
`838d64c8d87ba853d8f0529ffd93b9b9e649d4d5cd05d9042efc225187199032`.
[Arşiv kaydı](incelemeler/urunlesme-20261014/final-2e970e4/source-archive.json)
CI kaydı alınmadığını açıkça gösterir. Bu kaynak paketi imzalı ikili veya
release değildir. Kalıcı teslim checkout'unda paketler ignored `dist/`
altındadır; takip edilen kanıtlar bu raporun yanındaki inceleme dizinindedir.

a2b8557 hashleri eş olsa da klon içi `build.log` nedeniyle gerçek ikili
`dirty: true` bildirdi; [kimlik tekrar kontrolü](incelemeler/urunlesme-20261014/final-a2b8557/build-identity-recheck.json)
temiz kimlik kabulünü reddeder. Eski hashler ve faz/native sonuçları kendi
SHA'larında tarihsel kalır. e9145844 ara checkpoint'indeki gerçek dosya
checksum'unu sır sanan JSON anahtar eşleşmesi yalnız path/sha256 dizi
biçimine çevrildi; yeni sır istisnası eklenmedi. e914 raporları yeni
2e970e4 kabulü olarak yeniden etiketlenmedi.

K-239 yalnız bu iş serisindeki özel URL'leri temizler. Eski native/iki-klon
raporlarının özgün SHA'ları korunur; sanitizasyon eşlemesi eski koşuyu yeni
SHA kabulüne dönüştürmez. GitHub destek cevabı veya yönetilen refs/cache
temizliği bu yerel işlemle kapanmaz.

Bu rapor 2e970e4 yerel koşularını ve önceki tarihsel sınırları anlatır. K-241 sürüm
üreticisinin araç davranışı düzeltmesidir; compiler dil yüzeyini veya API'sini
genişletmez. İlgili davranış belgeleri kendi kaynak dilimlerinde incelenir;
performans, soak ve RC tarihçelerine kayıtsız kabul eklenmedi.
