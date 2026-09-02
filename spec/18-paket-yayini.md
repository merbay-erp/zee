# 18 — Tekrar Üretilebilir Paket Yayını

Normatif kaynak: RFC-0020 §1–5/8. Mimari sınır: ADR-006, ADR-028. Tanı: P012.

Bu bölüm çalışan yerel yayın zincirini tanımlar. Registry rol metadata
doğrulaması, K-135 taşıma/cache/offline katmanı ve K-136 exact proje/CLI bağı
[spec/19](19-registry-metadata-guveni.md)'da çalışır.

## Komutlar (TANIMLI)

```text
dil anahtar üret <dosya>
dil paketle [proje] --anahtar <dosya> [--çıktı <klasör>]
```

`anahtar üret` yalnız yeni dosya oluşturur; var olanı YASAK olarak korur.
Unix'te hedef ilk açılıştan itibaren 0600 izne sahiptir. Biçim
`zee-ed25519-private-v1`dir; 32 byte gizli/açık Ed25519 değerleri ve açık
anahtarın `sha256:` kimliği küçük harfli hex taşır. Okumada üçü yeniden
çaprazlanır. Özel anahtar için önerilen `.zee-anahtar` uzantısı yeni proje
`.gitignore` dosyasına otomatik eklenir; bu yalnız yanlışlıkla commit riskini
azaltır, güvenli ayrı yedekleme sorumluluğunu kaldırmaz.

`paketle` geçerli `proje.dil` ister. Yerel yol bağımlılığı, sembolik bağ veya
UTF-8 olmayan `.dil` kaynak varsa P012 ile hiçbir yayın çıktısı üretmeden
durur. Varsayılan çıktı `<proje>/hedef/paket`tir.

## Çıktı kümesi (ZORUNLU)

Her başarılı yayın aynı `<ad>-<X.Y.Z>` tabanında şunları üretir:

- `.zep` deterministik kaynak paketi;
- `.spdx.json` SPDX 3.0.1 JSON-LD SBOM;
- `.intoto.json` in-toto Statement v1 / SLSA provenance v1;
- `.zee-yayin.json` Ed25519 imzalı `zee-yayin-v1` bildirimi.

`SOURCE_DATE_EPOCH` negatif olmayan Unix saniyesiyse SPDX CreationInfo zamanı
ondan alınır. Geçersiz değer P012'dir. Değişken yoksa SBOM gerçek UTC üretim
zamanını kullanır. Aynı kaynak, özel anahtar, `dil` sürümü ve üretim saniyesi
dört çıktıyı byte-byte aynı üretmek ZORUNDADIR. `.zep` ve zamansız provenance
zaman girdisinden bağımsızdır.

## `.zep` v1 biçimi (TANIMLI)

Arşiv `ZEEZEP\0\x01` sekiz byte sihir, big-endian u32 girdi sayısı ve her
girdi için big-endian u32 yol uzunluğu + yol + u64 içerik uzunluğu + içerik
taşır. Yollar `/` ayraçlı NFC UTF-8 göreli yoldur ve byte sırasıyla kesin
artar. Üretici dosya sistemi bileşenlerini Unicode 17.0 UAX #15 NFC'ye
çevirir; normalizasyondan sonra çakışan iki yolu reddeder. Tüketici arşiv
yolunu normalize etmez, zaten NFC değilse reddeder. Yalnız gerçek UTF-8 `.dil`
dosyaları pakete girer. Zaman/sahip/izin/platform metadata'sı girmez.

ZORUNLU limitler:

- toplam 64 MiB;
- tek dosya 16 MiB;
- en çok 10.000 dosya;
- yol en çok 1.024 byte.

Mutlak yol, ters bölü, NUL, boş/`.`/`..` bileşen, yinelenen/sırasız yol,
sembolik bağ ve son girdiden sonra byte YASAKTIR. Unicode 17.0 UTS #39'da
`/`, `\\`, `.`, `:` iskeletine giden karakterler; tam genişlikli `/` ve `.`;
U+200B–U+200F, U+202A–U+202E, U+2060–U+206F ve U+FEFF görünmez biçim
denetleyicileri de yol içinde YASAKTIR. Bu küme `.zep` v1'e dondurulmuştur;
Unicode veri yükseltmesi RFC/spec/conformance incelemesi ister. Uzunluk toplama
taşması hata olur; ayırma sınır denetiminden sonra yapılır. Pakette `proje.dil`
ZORUNLUDUR.

## SBOM (ZORUNLU)

SBOM `https://spdx.org/rdf/3.0.1/spdx-context.jsonld` bağlamını ve en az
CreationInfo, üretici Agent, `dil` Tool, SpdxDocument, software_Sbom ve
software_Package öğelerini taşır. software_Package:

- `proje.dil` ad ve sürümüyle eşleşir;
- `.zep` SHA-256 özetini `verifiedUsing` içinde taşır;
- lisans bildirilmediği sürece `NOASSERTION` yazar; lisans uydurmaz.

## Provenance (ZORUNLU)

Provenance `_type=https://in-toto.io/Statement/v1` ve
`predicateType=https://slsa.dev/provenance/v1` taşır. Tek subject `.zep` adı
ve SHA-256 özetidir. Build definition paket adı, sürüm ve morfoloji profilini;
builder kullanılan `dil` sürümünü tanımlar. Yerel üretim hosted/hardened build
veya SLSA L2/L3 iddiası DEĞİLDİR.

## Yayın imzası (ZORUNLU)

`zee-yayin-v1` kapalı JSON şeması; paket ad/sürüm/morfoloji, yayıncı açık
Ed25519 anahtarı/kimliği ve `.zep`/SBOM/provenance için güvenli dosya adı,
u64 boyut, SHA-256 taşır. Tam bir imza vardır.

İmza girdisi `zee-yayin-v1\0` byte alan ayrımı ile `imzali` nesnesinin serde
kapalı şema alan sıralı kompakt JSON byte'larının birleşimidir. Bilinmeyen,
tekrarlı veya eksik alan YASAKTIR. Açık anahtar kimliği açık anahtarın
SHA-256'sıyla eşleşir; Ed25519 imzası geçerli olmak ZORUNDADIR.

Doğrulama başarısı için ayrıca:

1. üç eşlikçi dosyanın ad/boyut/özeti;
2. `.zep` biçimi ve limitleri;
3. arşiv içindeki `proje.dil` ad/sürüm/morfolojisi;
4. SBOM paket ad/sürüm/özeti;
5. provenance subject ad/özeti

imzalı bildirimle eşleşmek ZORUNDADIR. Bu zincirin herhangi bir byte'ı
değişirse yayın reddedilir. Öz-imza yalnız yayıncı bütünlüğünü kanıtlar;
registry targets rolü bu yayıncıyı ad+sürüm için yetkilendirene dek üçüncü
taraf güveni oluşturmaz.

## Yan etki ve atomiklik

Üretici dört dosyayı yazmadan önce bütün kümeyi tüketici doğrulayıcıyla
denetler. Dosyalar ayrı ayrı atomik replace ile yazılır. Süreç çökmesinde dört
dosyanın tek transaction olarak görünmesi TANIMLI DEĞİLDİR; tüketici her
kullanımda tam kümeyi yeniden doğrular ve yarım kümeyi yayın saymaz.

## Conformance

Uyumlu gerçekleme en az şu olumluları kanıtlar:

- aynı girdi/zaman için dört dosyanın byte-byte eşitliği;
- Türkçe Unicode adlı kaynak ağacının Linux/macOS/Windows'ta tek sabit `.zep`
  fixture byte'ına eşitliği;
- üretilmiş tam zincirin doğrulanması;
- CLI'ın Türkçe komut/çıktısı.

Ve şu olumsuzları kanıtlar:

- var olan anahtarı ezme;
- yayın imzası, arşiv, SBOM ve provenance oynama;
- yerel bağımlılık, sembolik bağ, path traversal, fazladan byte;
- NFD arşiv yolu, NFC çakışması, Unicode ayraç/nokta/iki nokta benzerleri ve
  görünmez bidi karakterleri;
- açık anahtar/kimlik uyumsuzluğu ve limit aşımı.
