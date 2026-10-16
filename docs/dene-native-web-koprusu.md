# dene native web test köprüsü

Unix'te açık operator profili kullanılır. Önce olağan Zee testleri başarılı
olmalıdır. Altı rolün tümünü verin; output henüz var olmamalı ve kanonik
sahipli parent altında olmalıdır:

```sh
dil dene --web-suite /abs/fixtures/suite.json \
  --web-suite-sha256 SUITE_SHA256 --web-fixtures /abs/fixtures \
  --test-python /abs/trusted/python3 \
  --test-postgres-bin /abs/trusted/pg16.15/bin \
  --test-output /abs/owned/new-output /abs/project
```

Suite sözleşmesi [owned web kit](owned-web-test-kiti.md) ile aynıdır.
Python ve PostgreSQL16.15 binary'leri operator tarafından güvenilmiş roller;
suite bunları değiştiremez. Kit kaynakları compiler içine gömülüdür; proje
klasöründe Python aramaz. Credential/provider hesabı veya canlı hizmet
kullanmayın. Fixture SQL güvenilmiş test verisidir, SQL sandbox değildir.

`unit.json`, `runtime/scripts/` ve `results/proof.json` çıktı kökünde korunur.
Başarısız olağan test `native_started:false` bırakır, native suite çalışmaz.
Native rapor dört vaka/19HTTP isteği gibi gerçek sayıları kaydeder; güncel
suite'in sayıları dışında genelleme yapılmaz. Worker başına kaynak raporu
`results/coverage-<case>.json` altındadır. Unit ve native kaynak kapsamı ayrı
sonuçlardır; bağımsız worker sayaçları tek çalıştırmaymış gibi birleştirilmez.

İki PG vakasının cluster kimliği farklı olmalıdır; readonly observer commit1,
request rollback0 ve sonraki taze vaka0 durumunu doğrular. Bu, tüm HTTP
isteklerine ortak transaction rollback desteği değildir.

[ADR-144](../adr/144-dene-native-web-test-koprusu.md),
[kaynak kapsamı](kaynak-kapsami.md) ve
[ADR-142](../adr/142-zee-kaynak-dal-kapsami.md) güven sınırlarını açıklar.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## K-304 etki sınırı

K-304 bu köprüye gömülen kitin son iptal/deadline sonucunu korur; Python
bayt değişikliği yeni derlemede yer alır. [Direct kit kabulü](owned-web-test-kiti.md)
GB-113'ü dar kapsamda kapatır. `abdc5cf36b4d55021df5edad08cee3a7ad953ebd` native4 sonucu ayrı
`dene`/source-coverage sahibinin çalıştığı anlamına gelmez. S3 dört vaka/19HTTP
ve admission retleri kendi exact kaynağında kalır; yeni köprü kabulü veya
compiler grammar/API/coverage kapasitesi ileri sürülmez.

## K-305 — Negatif kabul koşucusunda erken iptal hazırlığı

**AÇIK (GB-114, düşük):** `scripts/tests/web-test-bridge-test.py`, Supervisor
sınıfını import eder; Supervisor'ın bağımsız main handler kurulumu burada
çalışmaz. Varsayılan SIGTERM, harness finally akışını atlayabilir; ayrı
oturumda başlatılmış özgün child'ın kapanışı bu nedenle garanti edilmez.
K-305, ilk child öncesinde yalnız bayrak işaretleyen handler, bekleme sırasında
iptal kontrolü ve original Popen/WNOWAIT üzerinden final cleanup/proof
sonucunun korunması için dar kaynak hazırlığıdır.

Hazırlık kaydı: Kontrollü hazırlıkta 15 Python metodu ve 27 senaryo geçti; eski sürümde 17 yapısal eksik, iki cleanup hatasında eksik makbuz ve bir terminal süre aşımında yanlış passed ayrıldı. Özgün dış owner eski exit1 ve yeni exit0 ile reaped; gerçek çocuk, OS sinyal teslimi, PG veya ağ kabulü değildir. `K305-PREPARATION.json`
SHA-256: `e445af8d0a82448488bd72d47e1360bba990751db0354f91061a93603c4918b1`.
Kontrollü hazırlık, gerçek bridge/PG/HTTP veya OS sinyal teslimi sonucu diye
sunulmaz; makbuz bu ayrımı ve gerçekten yürütülen kapsamı belirtmelidir.
Selector oluşturmanın iç run try'si önünde olması tek başına kaçak kanıtı
değildir: main finally owner cleanup'ını zaten çağırır.

Mevcut normal CLI/admission Cargo sahibi Unix hazırlığını da taşır; eski üç
CLI guard ve ayrı ignored bridge sahibi korunur. Python alt vakaları yeni
Cargo sahibi değildir. Tam kaynak/faz/Clippy/güvenlik kapıları, yeni exact
temiz kaynakta mevcut bridge sahibinin unit/source-coverage, dört native vaka
ve altı admission negatifi ile kabulü ve son belge kapıları tamamlanmadan
K-305/GB-114 kapanmaz. Şu an yeni kaynak SHA'sı veya bu kabuller atanmaz.

Bu değişiklik negatif harness/test hazırlığıyla sınırlıdır; production Rust,
gömülü owned web kiti ve Supervisor davranışı değişmez. S3 köprü ve K-304
C8 direct kit kabulü kendi kaynaklarında kalır. P22/P36 ve ürün/platform
kapsamı açıktır. [ADR-144](../adr/144-dene-native-web-test-koprusu.md),
[ortak rapor](urunlesme-k298-kabul-20261016.md) kanıt sınırını taşır.

Etki incelemesi: grammar/spec/RFC, kullanıcı tanısı, public API, production
katman/bağımlılık, sürüm artefaktı, performans iş yükü veya compiler/LSP kalıcı
durum değişmez. Semantic `.dil` regression ve compiler/core beyanı yeni bir
üretim değişikliği olmadan eklenmez; RC/soak/IDE tarihçesi bu hazırlıkla yenilenmez.
