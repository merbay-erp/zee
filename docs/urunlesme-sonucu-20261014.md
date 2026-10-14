# 14 Ekim 2026 ürünleşme teknik teslimi

Mevcut teknik dilimler kod, test, belge ve exact kaynak kayıtlarıyla
tamamlandı. **39 maddelik ürünleşme planının bütünü tamamlanmadı.**
Çalışan kapasite ve kalan kabul sınırları
[plan envanterinde](urunlesme-plan-v1.tsv) ayrı tutulur.

Son temiz teknik checkpoint:
`a2b8557ba04897c60802d04b911a7eae6e91ae43`.
Kaynak güvenlik/bakım commit'i
`b70955d3d82e22d028a0295500e2fb8e980bb556`; ardından exact compiler ve
core-freeze beyanı kaydedildi. Bu raporu ekleyen commit yalnız belge ve
kanıt içerir. Aşağıdaki koşular bu rapor commit'ine veya uzak CI'a aitmiş
gibi yeniden etiketlenmez.

## Çalışan değişiklikler

| Alan | Teslim edilen kapsam | Ayrıntı |
|---|---|---|
| CLI ve build kimliği | Bilinmeyen seçenek exit 2; CLI/LSP/worker/SBOM kaynak kimliği eşliği; SHA, UTC kaynak tarihi, kanal ve kirli ağaç bilgisi | [ADR-093](../adr/093-deterministik-build-kimligi.md) |
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
| Tam faz matrisi | **887 geçti, 0 kaldı, 6 atlandı; toplam 893**, 125.041 sn | Altı dedicated PostgreSQL vakası aşağıdaki ayrı koşuda yürütüldü. [Faz raporu](incelemeler/urunlesme-20261014/final-a2b8557/faz-test-matrisi.md) |
| Dedicated PostgreSQL TLS/bütçe | **5/5**, sıfır atlama | İki CA/hostname negatif yolu, NULL/tür, tam eşik ve bir fazlası |
| Dedicated göç | **1/1**, sıfır atlama | Read-only durum/kuru, roller, checksum, rollback ve yarış |
| Yerel parola CAS | **7/7**, sıfır atlama | Gerçek PostgreSQL transaction/row-lock; canlı ürün login'i değildir |
| Sürekli güvenlik | **Geçti** | Compiler/fuzz deny, stable fuzz derlemesi, tedarik ve bulgu kapıları; ASan/Miri RC koşusu değildir |
| İki temiz klon | **`dil` ve `dillsp` eş SHA-256** | Kanonik üretici, Rust 1.93.1, macOS arm64, release profili; imzasız yerel çıktı |
| Diğer incelemeler | Fmt, clippy, katalog/tazelik, katman/SCC/API, uyumluluk, semantic/core-freeze, spec/tanı, gizlilik/sır ve offline vendor geçti | Spec 203 maddede 196 kanıtlı, 5 kısmi, 2 açık; bir doctest ayrıca geçti |

[Koşu özeti ve log hash'leri](incelemeler/urunlesme-20261014/final-a2b8557/gate-summary.json),
[native kanıtı](incelemeler/urunlesme-20261014/final-a2b8557/native-proof.json)
ve [iki-klon kanıtı](incelemeler/urunlesme-20261014/final-a2b8557/build-proof.json)
exact checkpoint'e bağlıdır. README'nin statik test sayısı koşuda geçen
test sayısı yerine kullanılmadı; ayrı koşuların örtüşen vakaları toplanıp
yeni bir tekil test toplamı üretilmedi.

Native koşu, [resmi PostgreSQL **16.15** kaynak kaydından](incelemeler/urunlesme-20261014/final-a2b8557/postgres-runtime-build-proof.json) geçici dizinde derlenen
ve yalnız bu çalışmanın yönettiği loopback kümelerinde yürütülen kabulüdür.
Kaynak ve ikili başlangıç/sonunda aynı kaldı; sahip olunan kümeler kapatıldı.
Native testlerin Rust sürümü 1.98.1, kanonik release üretiminin sürümü
1.93.1'dir. Bu kabul Docker imajı, müşteri sunucusu, temiz fiziksel OS veya
uzak CI kabulü değildir.

İki-klon çıktı hash'leri:

```text
dil     722f7294b0b7da839bdb9f7e5adc724a1d9489bfdb7b9e94e1320780e103cd84
dillsp  5c5dd31aa4fa54b1f6c9ce9861f1ac646cc31fc86ada2c0b3278aff2703a514a
```

## Açık kalan kapasite ve kanıt

| İş | Kalan gerçek iş |
|---|---|
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

Yerel `zee-source-a2b8557.zip`, **1124** Git archive dosyasını exact ZIP
comment ve `KAYNAK-SHA256.txt` ile doğrular. Tek fazlalık bu manifesttir.
ZIP SHA-256:
`fc253f9e2695a112ce09d0fe0236484fdb6968d5193632487059fff350b317b2`.
[Arşiv kaydı](incelemeler/urunlesme-20261014/final-a2b8557/source-archive.json)
CI kaydı alınmadığını açıkça gösterir. Bu kaynak paketi imzalı ikili veya
release değildir. Kalıcı teslim checkout'unda paketler ignored `dist/`
altındadır; takip edilen kanıtlar bu raporun yanındaki inceleme dizinindedir.

K-239 yalnız bu iş serisindeki özel URL'leri temizler. Eski native/iki-klon
raporlarının özgün SHA'ları korunur; sanitizasyon eşlemesi eski koşuyu yeni
SHA kabulüne dönüştürmez. GitHub destek cevabı veya yönetilen refs/cache
temizliği bu yerel işlemle kapanmaz.

Bu son kayıt belge/kanıt değişikliğidir: spec/RFC/ADR, tanı, dil yüzeyi,
production modülleri, iş yükleri ve API'de yeni davranış tanımlamaz.
İlgili davranış belgeleri kaynak dilimlerinde incelenip güncellendi;
performans, soak ve RC tarihçelerine kayıtsız kabul eklenmedi.
