# RFC-0020 — Paket Yayını ve Registry Güven Zinciri

- **Durum:** geçici kabul — yayın, metadata doğrulayıcısı, K-135
  taşıma/cache/kalıcı durum/offline ve K-136 exact proje/kilit/CLI bağı gerçeklendi
- **Tarih:** 1 Eylül 2026
- **İlgili günlük kaydı:** K-094, K-095, K-117, K-135, K-136
- **Mimari karar:** ADR-006, ADR-028
- **Normatif çalışan yüzey:** [spec/18](../spec/18-paket-yayini.md),
  [spec/19](../spec/19-registry-metadata-guveni.md)

## Özet

zee paketi “bir klasörü sıkıştırıp sunucuya koymak” değildir. Yayınlanabilir
bir sürüm; deterministik kaynak paketi, onun SPDX SBOM'u, SLSA provenance'ı ve
üçünü Ed25519 imzasıyla bağlayan kapalı şemalı yayın bildirimidir. Uzak registry
bu yayınları TUF'tan uyarlanan kök/targets/snapshot/timestamp rol zinciriyle
yetkilendirir. Ayna ve TLS taşıma güvenliğini güçlendirir ama paket kimliğini
belirlemez.

Bu RFC iki aşamayı bilinçli ayırır:

- **A — yayın çekirdeği (çalışıyor):** `dil anahtar üret`, `dil paketle`,
  `.zep`, SPDX 3.0.1, SLSA v1, Ed25519 yayın bildirimi ve doğrulama.
- **B1 — metadata güveni (çalışıyor):** ağ dışı sabit root, eşik ve çift eşikli
  rotasyon, dört rol, rollback/expiry/mix-and-match, exact targets yetkisi,
  yanked ve güvenlik duyurusu politikası.
- **B2 — uzak kullanım (çalışıyor):** limitli taşıma, kalıcı metadata durumu,
  doğrulanmış cache, offline hit/miss, exact manifest/kilit v3, atomik kaynak
  kurulumu ve açık ağ kullanan CLI.

## 1. Komut yüzeyi

```text
dil anahtar üret yayinci.zee-anahtar
dil paketle . --anahtar yayinci.zee-anahtar
dil paketle . --anahtar yayinci.zee-anahtar --çıktı hedef/paket
dil ekle örnek@1.2.3 . --registry https://registry.example \
  --kök 1@sha256:<64-küçük-hex>
dil kilitle . [--çevrimdışı]
dil paketler . [--yenile]
```

Varsayılan çıktı `<proje>/hedef/paket`tir. Dört dosya:

```text
<ad>-<sürüm>.zep
<ad>-<sürüm>.spdx.json
<ad>-<sürüm>.intoto.json
<ad>-<sürüm>.zee-yayin.json
```

`SOURCE_DATE_EPOCH` verilirse SPDX CreationInfo zamanı ondan doğar; aynı kaynak,
anahtar, derleyici sürümü ve zaman dört dosyayı da byte-byte aynı üretir.
Verilmezse SBOM gerçek UTC üretim zamanını kullanır; `.zep` ve zamansız SLSA
provenance yine deterministiktir.

## 2. Anahtar biçimi

Başlangıç yayıncı anahtarı UTF-8 dört satırdır:

```text
zee-ed25519-private-v1
gizli <32-byte-küçük-hex>
açık <32-byte-küçük-hex>
kimlik sha256:<açık-anahtarın-sha256-özeti>
```

Üretici var olan hedefi ezmez. Unix'te `create_new` ve 0600 izin aynı açma
işlemindedir; özel byte'lar daha geniş izinli ara dosyaya yazılmaz. Okuyucu
gizli anahtardan açık anahtarı yeniden türetir ve açık alanla kimliği çapraz
doğrular. `.zee-anahtar` önerilen uzantıdır; `dil yeni` tarafından oluşturulan
`.gitignore` bu uzantıyı dışlar. Düz dosya özel anahtar, çevrimdışı eşik
kök/HSM yerine geçmez ve güvenli ayrı bir yedek olmadan tek kopya tutulmamalıdır.

## 3. `.zep` v1 kanonik kaynak paketi

Sayısal alanlar unsigned big-endian'dır:

```text
8 byte  "ZEEZEP\0\x01"
u32     girdi sayısı
tekrar:
  u32   UTF-8 yol byte uzunluğu
  byte  `/` ayraçlı göreli yol
  u64   içerik byte uzunluğu
  byte  UTF-8 kaynak içeriği
```

Üretici dosya sistemi yolunun her bileşenini Unicode 17.0 UAX #15 NFC'ye
çevirir, `/` ile birleştirir ve yolları UTF-8 byte sırasına göre kesin artan
biçimde yazar. NFC sonrası yinelenen yol varsa seçim yapmadan reddeder.
Tüketici arşiv yolunu dönüştürmez; yol zaten NFC değilse paketi reddeder.
Yalnız gerçek `.dil` dosyaları girer. Zaman, sahip, grup, dosya izni,
sıkıştırıcı sürümü ve platform ayıracı pakete yazılmaz. Böylece aynı kaynak
ağacı aynı byte dizisidir.

### 3.1 Güvenlik limitleri

- toplam paket: 64 MiB;
- tek kaynak: 16 MiB;
- girdi sayısı: 10.000;
- yol: 1.024 UTF-8 byte;
- mutlak yol, `.`/`..`, boş bileşen, ters bölü, NUL, denetim karakteri,
  sembolik bağ: YASAK;
- Unicode 17.0 UTS #39'da `/`, `\\`, `.`, `:` iskeletine giden işaretler,
  tam genişlikli `/`/`.` ve görünmez bidi/biçim denetleyicileri: YASAK;
- son girdiden sonra byte: YASAK;
- yerel yol bağımlılığı: YASAK;
- post-install/build betiği: biçimde yoktur ve YASAKTIR.

Doğrulayıcı boyut alanını ayırmadan önce sınar; taşma kontrollüdür. Paket
kimliği bütün `.zep` byte'larının SHA-256 özetidir.

## 4. SBOM ve provenance

SBOM, SPDX 3.0.1 JSON-LD `core` + `software` profillerinde `SpdxDocument`,
`software_Sbom`, `software_Package`, üretici agent ve `dil` tool öğelerini
taşır. Paket öğesi ad/sürüm ile `.zep` SHA-256 özetini içerir. Lisans alanı
henüz `proje.dil`de bulunmadığı için `NOASSERTION` açıkça yazılır; lisans
keşfedilmiş gibi davranılmaz.

Provenance bir `https://in-toto.io/Statement/v1`dir; predicate type
`https://slsa.dev/provenance/v1`dir. Subject `.zep` adı ve SHA-256 özetidir.
Build definition ad, sürüm ve morfoloji profilini; builder kullanılan `dil`
sürümünü taşır. Bu yerel üretici provenance'ı **SLSA Build L1 bilgisidir**;
hosted/hardened build veya L2/L3 iddiası değildir. Registry targets rolünün
yayıncıyı yetkilendirmesi, provenance builder güvencesiyle karıştırılmaz.

## 5. `zee-yayin-v1` imzası

Yayın JSON zarfı `imzali` nesnesi ile tek `imzalar` girdisi taşır. İmzalı
nesne şunları bağlar:

- şema, paket adı, exact sürüm, morfoloji profili;
- yayıncı açık Ed25519 anahtarı ve `sha256:` kimliği;
- `.zep`, SBOM ve provenance için güvenli dosya adı, u64 byte boyutu, SHA-256.

İmza girdisi:

```text
"zee-yayin-v1\0" || serde-kapalı-şema-kanonik-JSON(imzali)
```

Alan ayrımı başka protokoldeki imzanın burada yorumlanmasını engeller. Şema
bilinmeyen/tekrarlı alanı reddeder. Doğrulayıcı Ed25519 imzasından sonra:

1. üç dosyanın adı/boyutu/özetini;
2. `.zep` biçimi ve limitlerini;
3. iç `proje.dil` ad+sürüm+morfoloji kimliğini;
4. SBOM paket kimliği/özetini;
5. provenance subject adı/özetini

çapraz doğrular. Başarıdan önce kaynaklar yürütülmez veya cache'e kabul edilmez.

## 6. Registry POUF v1 — metadata doğrulayıcı çalışıyor

Her rol kapalı şemalı `{ "imzali": ..., "imzalar": [...] }` zarfıdır. Dosya
`serde_json::to_vec_pretty` girintisi ve tek son satır sonuyla kanoniktir;
haritalar anahtar sırasında, imzalar `anahtar_kimligi` sırasında kesin artar.
Bilinmeyen/tekrarlı alan veya farklı byte gösterimi reddedilir. İmza girdisi:

```text
"zee-registry-v1\0" || UTF8(rol) || "\0" || COMPACT_JSON(imzali)
```

Root şeması `zee-registry-root-v1`; diğerleri sırasıyla
`zee-registry-targets-v1`, `zee-registry-snapshot-v1` ve
`zee-registry-timestamp-v1`dir. Wire alanları ve bütün limitler spec/19'da
normatiftir.

### 6.1 Roller

`root`, `targets`, `snapshot`, `timestamp` TUF anlamlarıyla ayrıdır. Her rol
bir anahtar kümesi ve `1..N` eşik taşır. Root başlangıç özeti ağ dışında
sabitlenir; ardışık root sürümü hem eski hem yeni root eşiğince imzalanır.
Metadata sürümü pozitif ve monoton; süre sonu yalnız kesin
`YYYY-MM-DDTHH:MM:SSZ` UTC'dir. `tutarli_anlik` true olmak zorundadır.

`targets` exact `ad@X.Y.Z` için yayıncı kimliği, dört hedefin özet+boyutu,
yanked durumu ve duyuru kimliklerini bağlar. `snapshot`, bütün targets rol
dosyalarının sürüm/özet/boyutunu; `timestamp` güncel snapshot'ı bağlar.

### 6.2 İstemci sırası

1. tek bir güncelleme başlangıç zamanı kaydedilir;
2. güvenilen root yüklenir, ardışık root rotasyonu yapılır;
3. timestamp imza/süre/sürüm ve rollback'e karşı doğrulanır;
4. snapshot timestamp'in boyut/özet/sürümüyle doğrulanır;
5. targets snapshot'ın boyut/özet/sürümüyle doğrulanır;
6. exact hedef, yayıncı yetkisi/yanked/duyuru politikasıyla seçilir;
7. dört hedef limitli indirilir, RFC §5 zinciri doğrulanır;
8. ancak bundan sonra aynı dosya sistemindeki geçiciden içerik-adresli cache'e
   atomik taşınır ve kilit üretilir.

Görülen en yüksek metadata sürümleri ve aynı sürümde eşdeğerliği koruyan tam
byte SHA-256 özetleri cache durumunda kalıcıdır. Sürümü aynı ama özeti farklı
metadata da rollback/equivocation olarak reddedilir. Durum yalnız bütün zincir
başarıyla doğrulandıktan sonra tek işlem olarak uygulanır; geçersiz ileri
sürümlü bir timestamp kalıcı fast-forward zehirlenmesi yaratmaz. Mirror yalnız
base URL'dir; kök kimliği ve hedef kararına katılmaz. Derleme/çalıştırma ağ
kullanmaz; kilitli doğrulanmış cache olmadan P-serisi tedarik tanısı verir.

Manifest HTTPS origin, pozitif ilk root sürümü, ağ dışı `sha256:` root özeti ve
exact `uzak_bağımlılıklar` listesini birlikte taşır. `proje.kilit` v3 ilk ve
etkin root/rol kimliğini, yayıncıyı, dört hedef özetini, yanked/kritik durumunu
ve varsa insan gerekçeli kabul kaydını sabitler. Metadata nesneleri
`.zee/registry/<root-özeti>`, exact açılmış kaynaklar
`.zee/paketler/sha256/<zep-özeti>` altında proje-local ve salt-okunurdur.
Normal derleme/LSP sessiz ağ açmaz; `paketler` de yalnız `--yenile` ile ağ açar.

### 6.3 Exact sürüm

İlk uzak yüzey `ad@X.Y.Z` dışında sürüm ifadesi kabul etmez. SemVer aralığı,
ön-sürüm seçimi, özellik çözümü ve çoklu registry önceliği dependency confusion
riski nedeniyle bu RFC'nin v1 POUF'una sessizce eklenemez.

### 6.4 Limitler ve targets şeması

Root 1 MiB, timestamp 64 KiB, snapshot 1 MiB, targets 8 MiB; anahtar ve zarf
imzası 256, hedef ve duyuru ayrı ayrı 100.000 ile sınırlıdır. Targets
`<ad>@X.Y.Z` anahtarında paket/sürüm/morfoloji, izinli yayıncı kimliği,
yanked, duyuru kimlikleri ve dört yayın dosyasının kesin ad/boyut/SHA-256
bağını taşır. `.zep` 64 MiB, SBOM/provenance 8 MiB, yayın bildirimi 1 MiB
sınırındadır. Üst rolün boyut bağı ayrıştırmadan önce uygulanır.

## 7. Yanked ve duyuru politikası

Yank, içeriği silmez ve kilidi başka sürüme taşımaz. Yeni ekleme varsayılan
reddedilir; mevcut exact kilit ancak görünür politika kaydıyla yeniden
üretilebilir. Güvenlik duyurusu imzalı targets zincirindedir; sabit kimlik,
paket, etkilenen exact sürümler, önem ve düzeltilen sürüm taşır. Kritik etkin
duyuru yeni kilidi varsayılan engeller; baypas gerekçesi kilitte görünürdür.
Kritik baypas anahtarı root+exact paket yanında güncel sıralı etkin duyuru
kümesini de taşır; yeni duyuru eski gerekçeyle sessiz kabul edilemez.

## 8. Hata ve atomiklik

Yayın/anahtar üretim yüzeyinin kararlı kodu P012'dir. Registry metadata/kök
güven zinciri P013, exact hedef/yayıncı/yanked/duyuru politikası P014'tür.
HTTPS taşıma, cache miss/bozulma ve kalıcı durum hatası P016'dır. Durum dosyası
atomik karşılaştır-ve-değiştir olduğundan yarışan bayat süreç yeni durumu
ezemez. `dil paketle`,
kendi çıktısını tüketici doğrulayıcıyla doğrulamadan hiçbir çıktı yazmaz. Her çıktı atomik tek-dosya
yazımı kullanır; dört dosyanın süreç çökmesine dayanıklı tek transaction olduğu
sözü verilmez. İçerik-adresli adlar ve her kullanımda doğrulama yarım kümeyi
yayın saymaz.

## 9. Conformance kapıları

Çalışan A aşaması şunları kanıtlar:

- aynı girdiyle dört dosyanın byte-byte tekrar üretimi;
- özel anahtarın ezilmemesi ve Unix 0600;
- imzalı nesne, `.zep`, SBOM ve provenance tek-byte oynamasının reddi;
- sembolik bağ ve yerel yol bağımlılığının reddi;
- path traversal, fazladan byte, sıra/tekillik ve limit olumsuzları;
- Linux/macOS/Windows'ta aynı Türkçe Unicode kaynak ağacının sabit `.zep`
  fixture byte'ı;
- NFD→NFC üretimi, NFC çakışma reddi ve 80 vakalık kalıcı Unicode/yol saldırı
  korpusunun eksiksiz fail-closed reddi;
- üretici çıktısının aynı doğrulayıcıdan geçmesi.

K-095 metadata aşaması root eşik/çift eşikli rotasyon, rollback,
freeze/expiry, aynı sürümlü farklı içerik, mix-and-match, geçersiz
fast-forward durum zehirleme, endless-metadata sınırı, yanlış yayıncı, yanked
ve kritik duyuru testlerini kanıtlar. K-135 yalnız HTTPS origin, redirect/proxy
reddi, DNS sonrası public-IP kapısı, 64 ardışık root sınırı, sürümlü statik
metadata yolları, CAS korumalı atomik monoton durum ve yalnız tam zincirden
sonra yazılan salt-okunur SHA-256 cache'i gerçekler. Bozuk cache, çevrimdışı
hit/miss, taşıma boyutu ve başarısız zincirin durum/cache yayımlamaması testlidir.
K-136 exact `proje.dil`, `proje.kilit` v3, atomik/salt-okunur kaynak kurulumu,
çevrimiçi/çevrimdışı CLI ve normal ağsız derleme entegrasyonunu gerçek uçtan
uca testle kanıtlar. V1-P1-07 **KAPALIDIR**.
