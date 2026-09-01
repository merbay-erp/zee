# RFC-0020 — Paket Yayını ve Registry Güven Zinciri

- **Durum:** geçici kabul — yayın çekirdeği gerçeklendi; uzak kayıt POUF'u ve
  istemci saldırı kapıları tamamlanmadan registry kısmı yürürlükte değildir
- **Tarih:** 1 Eylül 2026
- **İlgili günlük kaydı:** K-094
- **Mimari karar:** ADR-006
- **Normatif çalışan yüzey:** [spec/18](../spec/18-paket-yayini.md)

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
- **B — uzak kayıt (kararı kabul, gerçekleme sürüyor):** eşik kök, dört rol,
  exact sürüm çözümü, doğrulanmış cache, yanked ve güvenlik duyurusu.

## 1. Komut yüzeyi

```text
dil anahtar üret yayinci.zee-anahtar
dil paketle . --anahtar yayinci.zee-anahtar
dil paketle . --anahtar yayinci.zee-anahtar --çıktı hedef/paket
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

Girdiler yolun Unicode byte sırasına göre kesin artandır; yinelenemez. Yalnız
gerçek `.dil` dosyaları girer. Zaman, sahip, grup, dosya izni, sıkıştırıcı
sürümü ve platform ayıracı pakete yazılmaz. Böylece aynı kaynak ağacı aynı
byte dizisidir.

### 3.1 Güvenlik limitleri

- toplam paket: 64 MiB;
- tek kaynak: 16 MiB;
- girdi sayısı: 10.000;
- yol: 1.024 UTF-8 byte;
- mutlak yol, `.`/`..`, boş bileşen, ters bölü, NUL, denetim karakteri,
  sembolik bağ: YASAK;
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

## 6. Registry POUF v1 — bağlayıcı tasarım, henüz çalışan yüzey değil

### 6.1 Roller

`root`, `targets`, `snapshot`, `timestamp` TUF anlamlarıyla ayrıdır. Her rol
bir anahtar kümesi ve `1..N` eşik taşır. Root başlangıç özeti ağ dışında
sabitlenir; ardışık root sürümü hem eski hem yeni root eşiğince imzalanır.
Metadata sürümü pozitif ve monoton; süre sonu RFC 3339 UTC'dir.

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

Görülen en yüksek metadata sürümleri cache durumunda kalıcıdır. Mirror yalnız
base URL'dir; kök kimliği ve hedef kararına katılmaz. Derleme/çalıştırma ağ
kullanmaz; kilitli doğrulanmış cache olmadan P-serisi tedarik tanısı verir.

### 6.3 Exact sürüm

İlk uzak yüzey `ad@X.Y.Z` dışında sürüm ifadesi kabul etmez. SemVer aralığı,
ön-sürüm seçimi, özellik çözümü ve çoklu registry önceliği dependency confusion
riski nedeniyle bu RFC'nin v1 POUF'una sessizce eklenemez.

## 7. Yanked ve duyuru politikası

Yank, içeriği silmez ve kilidi başka sürüme taşımaz. Yeni ekleme varsayılan
reddedilir; mevcut exact kilit ancak görünür politika kaydıyla yeniden
üretilebilir. Güvenlik duyurusu imzalı targets zincirindedir; sabit kimlik,
paket, etkilenen exact sürümler, önem ve düzeltilen sürüm taşır. Kritik etkin
duyuru yeni kilidi varsayılan engeller; baypas gerekçesi kilitte görünürdür.

## 8. Hata ve atomiklik

Yayın/anahtar üretim yüzeyinin kararlı kodu P012'dir. Uzak istemci kodları
gerçeklemeyle birlikte P013+ alanında ayrılaştırılacaktır; çalışmayan koda
şimdiden sahte tanı atanmaz. `dil paketle`, kendi çıktısını tüketici
doğrulayıcıyla doğrulamadan hiçbir çıktı yazmaz. Her çıktı atomik tek-dosya
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
- üretici çıktısının aynı doğrulayıcıdan geçmesi.

Registry aşaması tamam sayılmadan ayrıca root eşik/rotasyon, eski root,
rollback, freeze/expiry, mix-and-match, fast-forward, endless-data, yanlış
yayıncı, yanked, kritik duyuru, bozuk cache, çevrimdışı hit/miss ve kötü ayna
testleri zorunludur. V1-P1-07 bu ikinci liste tamamlanana kadar açık kalır.
