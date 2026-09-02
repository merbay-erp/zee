# ADR-006 — Paket registry güven modeli

- **Durum:** kabul ve gerçeklenmiş (yayın K-094; metadata K-095; kanonik yol
  K-117; taşıma/cache/kalıcı durum/offline K-135; exact proje/kilit/CLI K-136)
- **Tarih:** 1 Eylül 2026
- **Normatif ayrıntı:** RFC-0020, spec/18

## Bağlam

Bir paketi HTTPS üzerinden indirmek, o paketin doğru yayıncıdan geldiğini,
registry'nin eski ve açık içeriği yeniden oynatmadığını ya da bir aynanın
metadata sürümlerini karıştırmadığını kanıtlamaz. zee'nin okul aynaları ve
çevrimdışı kullanım hedefi ayrıca güveni tek bir canlı sunucuya bağlamamayı
gerektirir.

Tehdit modeli ağın, CDN/aynanın ve registry depolamasının kötü niyetli
olabileceğini; saldırganın eski ama bir zamanlar geçerli dosyaları
oynatabileceğini ve tek bir çevrimiçi anahtarın ele geçirilebileceğini varsayar.
Yerel makine, açıkça güvenilen eşik sayıdaki kök anahtar ve derleyici ikilisi
ele geçirilmişse güvence verilemez. Ağ saldırganının güncellemeyi tümüyle
engellemesi de çözülemez; fakat başarısızlık sessiz kabul yerine görünür olur.

## Karar

### 1. İki katmanlı kimlik

- Registry kimliği, ilk kurulumda ağ dışından sabitlenen kök metadata özetiyle
  başlar. TLS ek savunmadır; güven kökü değildir.
- Kök rolü çevrimdışı birden çok Ed25519 anahtar ve eşik sayısı taşır. Yeni kök
  hem eski hem yeni kök eşiğince imzalanmadan güven devri olmaz.
- Kök; `targets`, `snapshot` ve `timestamp` rollerine ayrı anahtar/eşik devreder.
  Çevrimiçi timestamp anahtarı paket yayımlama yetkisi taşımaz.
- Paket yayıncısı kendi yayın bildirimini imzalar; fakat bu öz-imza tek başına
  güven değildir. Güncel `targets` rolü ad+sürüm için izin verilen yayıncı
  anahtar kimliğini ve dört hedef dosyanın özet/boyutunu bağlar.

Bu düzen TUF'un rol ayrımı, eşik imzası ve çevrimdışı kök modelini izler. zee
wire formatı ve istemci sırası RFC-0020'de ayrı POUF olarak sürümlenir; “TUF
uyumlu” sözü yalnız o POUF'un bütün zorunlulukları gerçeklenip conformance
testleri geçtiğinde kullanılır.

### 2. Güncellik ve tutarlılık

- `timestamp`, `snapshot`, `targets` ve `root` metadata sürümleri monoton
  artar; istemci gördüğü en yüksek sürümü kalıcı tutar ve rollback'i reddeder.
- Bütün roller kesin UTC süre sonu taşır. Süresi dolmuş metadata çevrimiçi
  yenilemede ve yeni paket eklemede reddedilir; doğrulanmış kilitli mevcut
  derleme ağ/duvar saati gerektirmez.
- Timestamp snapshot'ın, snapshot targets metadata'nın byte boyutu+SHA-256
  özetini bağlar. Böylece mix-and-match ve sonsuz veri saldırıları doğrulama
  öncesinde sınırlanır.
- Hedef dosya adı içerik özetiyle adreslenir. Aynalar transport katmanıdır;
  imza, boyut ve özet denetimini değiştiremez.

### 3. Paket yayını

Bir zee yayın birimi değişmez dört dosyadır:

1. sıralı ve metadata'sız deterministik `.zep` kaynak paketi;
2. SPDX 3.0.1 JSON-LD SBOM;
3. in-toto Statement v1 içindeki SLSA provenance v1;
4. ilk üçünün tam adı, byte boyutu ve SHA-256 özetini bağlayan Ed25519 imzalı
   `zee-yayin-v1` bildirimi.

Paket yalnız UTF-8 `.dil` kaynaklarını içerir. Dosya sistemi bileşenleri
`.zep`e girmeden NFC'ye çevrilir; bu dönüşümden doğan yol çakışması reddedilir.
Tüketici yalnız zaten NFC olan arşiv yolunu kabul eder. Mutlak/üst dizin
yolları, UTS #39 ayraç/nokta/iki nokta benzerleri, görünmez bidi biçim
karakterleri, sembolik bağ, post-install betiği ve yerel yol bağımlılığı
YASAKTIR. Dosya sayısı, yol, tek dosya ve toplam paket boyutu doğrulamadan önce
sınırlıdır. Ayrıntılı yol profili ADR-028 ve spec/18'dedir.
İmzaya giren JSON, zee'nin kapalı şemasından deterministik alan sırasında
yeniden serileştirilir ve `zee-yayin-v1\0` alan ayrımıyla imzalanır.

`dil anahtar üret` özel anahtarı var olan dosyayı ezmeden üretir; Unix'te
ilk açılıştan itibaren 0600'dür. Bu dosya biçimi başlangıç/kişisel yayın
akışıdır. Organizasyon registry'si kök anahtarlarını çevrimdışı veya donanım
destekli saklamalıdır; zee bir düz dosyayı HSM eşdeğeri saymaz.

### 4. Çözüm ve önbellek

- v1 istemcisi yalnız tam `ad@X.Y.Z` ister. Sürüm aralığı/çözücü ayrı RFC ve
  dependency-confusion analizi olmadan eklenmez.
- Ağ yalnız açık `ekle/kilitle/yenile` komutlarında kullanılır. Derleme,
  denetleme ve çalıştırma doğrulanmış `proje.kilit` ile içerik-adresli yerel
  önbellekten çalışır; sessiz ağ erişimi YOKTUR.
- İndirilen byte'lar doğrulanmadan kullanılabilir önbellek yoluna taşınmaz.
  Doğrulanan içerik salt-okunur, özet adreslidir; bozuk cache yeniden hash'lenip
  reddedilir. Çevrimdışı kip yalnız önceden doğrulanmış tam zinciri kullanır.
- Kilit, registry kök kimliği, metadata sürümleri, paket/yayıncı kimliği ve
  bütün hedef özetlerini taşır. Ayna değiştirmek paket kimliğini değiştirmez.

### 5. Yank ve güvenlik duyurusu

- `yanked`, yeni çözümü/eklemeyi engeller; daha önce kilitlenmiş sürüm ancak
  açık `--yanked-kabul` politikasıyla yeniden üretilebilir. Yank sessiz sürüm
  değiştirmez.
- İmzalı güvenlik duyurusu paket, etkilenen tam sürümler, önem, sabit kimlik ve
  düzeltilen sürümü taşır. Kritik/etkin duyuru varsayılan olarak yeni kilidi
  engeller; mevcut kilitte denetim görünür hata üretir. Politika baypası kilide
  kaydedilir. Kritik kabul güncel sıralı etkin duyuru kimliği kümesine bağlıdır;
  küme değişirse eski gerekçe yetmez.
- Namespace ilk sahiplik ve benzer Unicode/typosquatting denetimi registry
  sunucusunun ek politikasıdır; istemcide S028 ile aynı normalleştirme ilkesi
  korunur.

## Aşamalı gerçekleme durumu

K-094'ün ilk dilimi §3'ü gerçekler: anahtar üretimi, `.zep`, SPDX, SLSA,
imzalı yayın, çapraz doğrulama, limitler ve oynama testleri çalışır. K-117;
NFC kanonik yolunu, Unicode 17.0 güvenlik kümesini, üç Tier-1 işletim sistemi
CI fixture'ını ve 80 vakalık kalıcı saldırı korpusunu ekler. K-095;
ağ dışı root sabitlemesini, eşik/çift eşikli ardışık rotasyonu,
timestamp→snapshot→targets bağlarını, tek güncelleme saatini, sürüm+özet
rollback/equivocation durumunu ve exact yayıncı/yanked/duyuru politikasını
çalışan byte doğrulayıcısına dönüştürür. K-135 HTTPS-only statik taşıma,
ardışık root güncellemesi, tam zincir sonrası salt-okunur içerik-adresli cache,
CAS korumalı atomik monoton durum ve çevrimdışı yeniden doğrulamayı ekler.
K-136 exact `proje.dil` bildirimi, `proje.kilit` v3, proje-local atomik ve
salt-okunur kaynak kurulumu, gerekçeli yanked/kritik politika kayıtları ve açık
ağ kullanan CLI bağını ekler. Normal derleme ve LSP yalnız doğrulanmış cache'i
kullanır. Böylece V1-P1-07 **KAPALIDIR**.

## Sonuçlar

- Tek bir CDN/HTTPS veya yayıncı öz-imzası güven kökü sayılmaz.
- Dört yayın dosyasından herhangi bir byte değişikliği imza/özet zincirini
  bozar; doğrulayıcı kaynakla imzalı kimliği de çapraz denetler.
- Eşik kök ve rol ayrımı işletim yükü getirir; tek çevrimiçi anahtar
  kolaylığından bilinçli olarak vazgeçilir.
- `ed25519-dalek`, `serde`, `serde_json` ve UAX #15 için
  `unicode-normalization`, `Cargo.lock` ile sabitlenir. Ed25519, JSON kodlama ve
  Unicode normalizasyonu elde yazılmaz; ADR-001'in küçük ama uzman kitaplık
  kullanma kuralı korunur.

## Dayanaklar

- [The Update Framework Specification 1.0.36](https://theupdateframework.github.io/specification/latest/)
- [RFC 8032 — EdDSA / Ed25519](https://www.rfc-editor.org/info/rfc8032/)
- [SLSA v1.2 Build Provenance](https://slsa.dev/spec/v1.2/build-provenance)
- [SPDX Specification 3.0.1](https://spdx.github.io/spdx-spec/v3.0.1/scope/)
- [Unicode UAX #15 — Normalization Forms](https://www.unicode.org/reports/tr15/)
- [Unicode UTS #39 — Security Mechanisms](https://www.unicode.org/reports/tr39/)
