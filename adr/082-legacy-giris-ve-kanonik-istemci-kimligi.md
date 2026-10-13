# ADR-082 — Legacy giriş ve kanonik istemci kimliği

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıt:** K-208/K-209, RFC-0030, spec/12, GB-043/044

## Karar ve ürün kanıtı

Etkin ürün `zee-cekirdek`, exact ebca9a6ddcc9c7535ee3204cfe2af3bbb7fc95d7
kökeninde legacy panel legacy bcrypt girişini ve Site F audit IP'sini talep eder.
[Repo içi talep/köken](../dogfood/zee-cekirdek/KOKEN.md) ile gereksinim aynası
korunur. Yeni algoritma üretme/serbest header okuma/istemcinin IP beyanına
inanma ihtiyacı yoktur; en küçük değişiklik mevcut parola yüklemini ve
trusted request kimliği okumasını genişletir. Core freeze istisnası yalnız
bu kanıtlı iki dogfood ihtiyacıdır; genel core genişletme izni değildir.

Bcrypt native bağımlılığı exact 0.19.3, yalnız verify çağrısıyla kullanılır;
büyük maliyet ve 72 bayt üstü parola crypto işinden önce reddedilir.
Node uyumu için verify önünde kendi byte sınırı uygulanır; upstream'in
non_truncating_verify yolu ek NUL'u da sayabildiğinden 72-byte Node parolası
haksız reddedilmez. Bozuk hash panik veya ayrıntılı kimlik tanısı üretmez.
Argon2 üretimi/doğrulaması ve WASM fail-closed sınırı korunur.

İstemci IP'si WebGuvenligi'ndeki mevcut oran sınırı kimliğidir. Native IO
yalnız aktif request'te bu alanı sunar; cleanup sonrası boş döner.
Socket/Forwarded canonical seçimini yapan mevcut ADR-034/036 host yolu
aynen kullanılır. Yeni header parse, global değişken veya kalıcı cache yoktur.
Sırf audit için tenant/user/session kimliği IP'den türetilmez.

Yeni intrinsic mevcut parser/type/HIR/runtime kaydından ilerler. Mimari
sahip/temel→adaptör yönü ve SCC fixture'ı incelenir; crypto paketleri dış
bağımlılıktır, internal yeni owner değildir. Bounded request kimliği eskiden
vardır, yeni compiler/LSP tamponu yoktur; soak gerektiren cache eklenmez.
Offline vendor, SBOM/provenance, iki-clone artefakt ve tedarik kapıları
bağımlılık değişikliği için yeniden doğrulanır; native build bayrakları aynı.

K-209 iki contextual kalıbı kayıtlı yüzeye ekler; K-174 syntax gözlem
penceresi 13 Ekim–10 Kasım olarak yeniden başlar. Önceki 30 Eylül–28 Ekim
penceresi tarihtedir. B-001/B-002 gerçek insan verisi kapanmaz; usability
olmadan RFC tam kabul veya V1 hazır sonucu çıkarılmaz. Syntax/core kapıları
gevşetilmez; yeni yüzeyin exact hash'i, sürümü ve ürün gerekçesi birlikte kaydedilir.

Yükseltme örneği: başarılı legacy girişte ürün Argon2 özeti oluşturup kendi
transaction'ında kaydeder; IP handler'dan audit eylemine parametre olur.
Sunucu ikilisi/CI pin yükseltmesini ürün sahibi yapar.

K-209 sıfır argümanlı CSRF/IP eşlemesi ayristirici/ifade/web.rs
modülüne taşınır; parser faz dosyası 1220 satır bütçesini korur.
CSRF sözdizimi ve span yerleştirmesi değişmez; katman sahibi ayristirici,
intrinsic/agac kenarları mevcut yöndedir. Yeni üst owner veya SCC yoktur.

Bcrypt doğrulama/test alanı guvenlik/legacy_bcrypt.rs, canonical IP iz
şeması io_izi/istemci.rs, host politika delegasyonu yetkinlik/web.rs
sorumluluğundadır. Mevcut parent bütçeleri korunur; rota policy hata
metni/yönü aynı, yeni owner/SCC yoktur. Taşınan crypto testleri Web/security
birincil fazına exact selector ile bağlıdır.
