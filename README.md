# zee — Türkçe programlama dili

**Türkçe düşün. Türkçe yaz.**

zee, Türkçenin cümle yapısını temel alan bir programlama dilidir. Dosyalar
`.dil` uzantısını kullanır; komut satırı aracı `dil`, dil sunucusu `dillsp` adını taşır.
Adı **“ze”** diye okunur.

*Zeynep Eliz Erbay için. Kendi dilinde düşünüp kendi dilinde inşa edebilsin diye.*

```text
isim "Zeynep" olsun
"Merhaba " ile isim yaz

toplam 3 ile 7 nin toplamı olsun
toplam yaz
```

**Başla:** [Kurulum ve ilk proje](docs/baslangic.md) ·
[Örneklerle dil turu](docs/dil-turu.md) ·
[Örnek projeler](projeler/) ·
[Sürüm notları](docs/surumler.md)

## Mevcut durum

K-309 / GB-118 dar kapsamda kapalı: Job SQL dispatcher native kabul harness cleanup/sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak d3d6a9044f1646871954e0933fe1fe43ecd5ac96 tam15/temiz commit ve özgün owner makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/job11 frozen kabulü; özgün PG/readiness/cleanup ve yalnız ilk vaka iki host0 gözlemi doğrulandı. Baseline2, child-free tek beklenen failure ve kontrollü standalone44/ordinary3 ayrı kapsamda korunur. Diğer metrics/effect iki PG tüketicisi açık; genel P09 kısmi ve39plan statüsü mevcut değerlerindedir. Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler. [ADR-117](adr/117-sinirli-sql-dispatcher.md).

K-308 / GB-117 dar kapsamda kapalı: Starter kabul harness sticky iptal/publication ve özgün foreground PG sahipliği dar dilimi. Exact kaynak 05e12f748503339ac18e92dabbe498c49437131e tam15/temiz commit makbuzlarıyla doğrulandı. Aynı exact kaynakta tek ignored Cargo/starter14 frozen9 ve özgün PG/readiness/host/dış owner kabulü doğrulandı. Diğer üç PG tüketicisi açık; P23 kısmi ve 39 plan statüsü kendi mevcut değerlerinde korunur. Bu dar kapanış yalnız exact belge overlay’inde gerçek tam final11 kabulü ve onu kaydeden temiz kapanış commit’inin kendi özgün dış owner/public makbuzlarıyla geçerlidir. Her gerçek kabul kendi exact başarılı deneme kayıtları ve hash’leriyle izlenir; başarısız önceki deneme kapanış kanıtı değildir. Bu belgeler kendi kapanış commit SHA’sını üretmez; dış makbuzlar izler. [ADR-148](adr/148-icerik-yonetim-medya-baslangici.md).

K-307 / GB-116 dar kapsamda kapalı: Yeni exact kaynakta mevcut tek ignored Cargo sahibi ve SQL9 dar kabulü geçti; kaynak344 ve foreground PG özgün sahipliği bağımsız frozen kayıtla doğrulandı. Yeni exact ikinci kaynak tam15 kapısı geçti (1118.889 sn); gerçek tam faz998/1023 geçti,0 başarısız,25 ignored. İlk216d7 kaynak15 sonucu bu sonuca eklenmez. Temiz ikinci kaynak 9df1b4abb52372dce86c92255392032b6fa08574, tree e7de0cfc2dce9e9c271979cc963c9629c49263e8; sourcegate2 tested17 overlay ve sourcecommit2 özgün owner makbuzları eşleşti; kaynak344/production175 korundu. K307/GB116 dar kapanışı yalnız exact belge overlay son11 kapısı ve sonraki tested-overlay temiz kayıt commit’i ile iki özgün owner makbuzuna bağlıdır; bu metin kendi başına kapı/commit kanıtı değildir. Diğer dört PG tüketicisi ve genel P20 kısmi kalır. [Kapsam](docs/module-sql-lifecycle.md).

Ana geliştirme ve CI deposu [merbay-erp/zee](https://github.com/merbay-erp/zee).
Enterprise deposu yalnız aynadır.

zee aktif olarak geliştiriliyor. Kaynak ağacının sürümü **0.8.0-dev**;
son sürüm notu **v0.7.0**. V1 öncesi doğrulama çalışmaları sürüyor.

| Alan | Bugünkü durum |
|---|---|
| Dil ve araçlar | Lexer, parser, tür denetimi, yorumlayıcı, proje/paket araçları ve LSP çalışıyor. |
| Derleme | Bootstrap derleyici Rust ile yazılıyor; araç zinciri Rust **1.93.1** sürümüne sabit. |
| Çalıştırma | Derlenmiş `dil` ve `dillsp` için Rust kurulumu gerekmiyor; sistem kitaplığı gereksinimleri platforma bağlı. |
| Platform doğrulaması | Önceki kayıtlı checkpoint'lerde Linux, macOS ve Windows tatbikatı geçti; güncel kaynağın yerel macOS iki-klon kabulü ayrı kayıttadır. |
| Açık işler | Gerçek kullanıcı testleri, minimum işletim sistemi/fiziksel makine kanıtı ve native derleme araçlarının tam envanteri. |

**Zee kaynaklarını doğrudan native/WASM koduna derleyen backend ve
self-hosting henüz tamamlanmadı.** Mevcut playground, Rust derleyicisinin
WebAssembly sürümünü tarayıcıda çalıştırır.

Ayrıntılar: [Derleme ve bağımlılıklar](docs/derleme-ve-bagimliliklar.md) ·
[V1 sürüm kapıları](docs/v1-surum-kapilari.md) ·
[Öncelikli backlog](docs/oncelikli-backlog.md).

[K-298 yerel kabulü](docs/urunlesme-k298-kabul-20261016.md) `a57290f7a1c1fad07cda3f266b2a221f09aa6404`
kaynağında dört stdio kohortunun gözlenen p95 değerlerinin 200 ms hedefini
karşıladığını, dar IDE/UTF-16 ve macOS teslim/kurulum kabulünü kaydeder. Aynı kaynakta ikinci 1800 saniyelik RSS kabulü geçti;
RC dört ASan/Miri3 ve 998/0/25 tam kapıyla geçti; K-298 dar kapsamı kapalıdır; [P19](docs/olcumler.md) ürün/UI/HIR kapsamıyla kısmi kalır.
[S4](docs/urunlesme-s4-kabul-20261015.md) kendi exact kaynağında tarihsel kanıttır.
[39 maddelik plan](docs/urunlesme-plani.md) ürün, insan ve fiziksel platform
açıklarını korur; [14 Ekim teslimi](docs/urunlesme-sonucu-20261014.md) tarihseldir.
[Devam dalgası](docs/urunlesme-devam-20261014.md) native COMMIT hata kanıtını
ve kaynak arşivi sınırını ayrıca izler.

[K-299 kabul koşucusu bakımı](docs/kabul-kosuculari-yasam-dongusu-20261016.md)
için durum: **dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli**. Kaynak `8178920e03fed782a817d1864b8280fcb80812a3` üzerinde
temiz C2 native2 iki Cargo sahibiyle geçti: signed PDF9 ve service-fault2; alt vakalar Cargo owner sayısına eklenmez; C2 öncesi gate4: 13 adım geçti; tam faz 998/1023 geçti, 0 başarısız, 25 atlanan. GB-111: kapalı; toplu commit öncesi son belge kapısı gerekli.
Üretim compiler/host değişmedi; K-298 RC/soak/IDE/stdio kabulü kendi kaynağında kalır.

[K-300 mixed-revision koşucusu bakımının](docs/mixed-revision-wave.md) dar
kaynak/native kabulü ve GB-112 kapalıdır. Hazırlık13, C3 öncesi kaynak
kapısı13/tam faz998/0/25 ve temiz C3 native9/dört dalga/160 HTTP200 geçti.
Son11 belge kapısının geçmesi takip commit'inin önkoşuludur. Üretim compiler/host
değişmez; P14 ve genel ürün/platform/SLA kısmi, eski RC/soak/IDE kabulleri
kendi kaynaklarında kalır.

K-304 owned web kabul koşucusunun geç iptal/deadline düzeltmesi ve GB-113
dar kapsamda kapalı: hazırlık22, kaynak kapısı15 ve temiz native4 geçti.
[Kaynak, makbuz ve açık kapsam](docs/owned-web-test-kiti.md).

K-305 bridge negatif koşucusunun erken iptal/owned cleanup düzeltmesi ve
GB-114 dar kapsamda kapalı: hazırlık15, kaynak kapısı15 ve temiz exact
bridge Cargo1 ile unit kaynak kapsamı/native4/altı negatif geçti. Son11 belge
kapısı kayıt commit önkoşuludur; genel ürün/platform kapsamı kısmi kalır.
[Kaynak, makbuz ve kapsam sınırı](docs/dene-native-web-koprusu.md).

K-306 / GB-115 dar kapsamda kapalı: tarihsel temiz C12 `7d5e4b0` kaynak ve 12 tarihsel + 5 yeni komutla kaynak doğrulaması kayıtlı; kontrollü modeller ve iki inert prob geçti. Fresh yeni kaynakta iki mevcut Cargo sahibi üzerinden dispatcher4 ve HTTP7 geçti; 102 komutluk bootstrap48/build18/dispatcher18/HTTP18 zinciri ve özgün owner kapanışı doğrulandı. Yeni kaynak tam15 kapısı geçti (1136,782 sn); gerçek tam faz 998/1023 geçti, 0 başarısız, 25 ignored. C12 12 tarihsel + 5 yeniden komut birleşimi bu yeni tam15 sonuçtan ayrıdır. Temiz kaynak commit’i 42ca6817c010e97cb6dd5ff73cb7f7cd970984aa, tree 5585785367353af22dd35b539b50ce4233dbbac2; kaynakgate4 tested overlay ve sourcecommit3/özgün owner makbuzları eşleşti, kaynak341 pini ve production175 sınırı korundu. K-306/GB-115 dar kapanışı yalnız bu exact belge kümesinin son11 kapısı ve tested-overlay temiz kayıt commit’i makbuzlarıyla geçerlidir: E/k306-final-gates-1/PROOF.json, E/k306-final-gates-controller-1.json, E/k306-final-commit-1.json ve E/k306-final-commit-controller-1.json. Bu metin kendi başına kapı veya commit kanıtı değildir. [Kanıt ve açık sınırlar](docs/urunlesme-k298-kabul-20261016.md).

## Kaynaktan hızlı başlangıç

Git, rustup ve platformunun C/linker araçları kurulu olmalı.
`compiler/rust-toolchain.toml` gerekli Rust sürümünü seçer.

```bash
git clone https://github.com/merbay-erp/zee.git
cd zee/compiler
cargo build --locked --release --bin dil --bin dillsp
```

Linux / macOS:

```bash
./target/release/dil çalıştır ../merhaba.dil
```

Windows PowerShell:

```powershell
.\target\release\dil.exe çalıştır ..\merhaba.dil
```

Çıktı: `Dünyaya merhaba`.

Doğrulanmış bir sürüm artefaktından kurulum için
[kurulum ve kaldırma rehberini](docs/surum-runbook.md#kullanıcı-kurulumu) izle.
Platform araçları ve bağımlılıkları [derleme rehberinde](docs/derleme-ve-bagimliliklar.md).

## İlk projen

`dil` PATH'e kurulduktan sonra:

```bash
dil yeni ilk-projem
dil çalıştır ilk-projem
dil denetle ilk-projem
dil dene ilk-projem
```

`proje.dil` giriş dosyasını, proje bilgilerini, bağımlılıkları ve dış dünya
yetkinliklerini tanımlar. Dosyayı doğrudan çalıştırmak da mümkündür:
`dil çalıştır merhaba.dil`.

## Dil neler sunuyor?

- **Türkçe cümleler:** `olsun`, `yaz`, koşullar, döngüler ve Türkçe eklerle ad çözümleme.
- **Veri ve işlemler:** tam sayılar, keyfî hassasiyetli ondalıklar, metin,
  listeler, sözlükler, yapılar, işlemler ve desen eşleştirme.
- **Hata yönetimi:** `Seçenek`, `Sonuç`, kaynak konumlu Türkçe tanılar ve öneriler.
- **Uygulamalar:** dosya, JSON/CSV, tarih/saat, HTTP istemcisi,
  deneysel web sunucusu ve PostgreSQL adaptörü.
- **Proje araçları:** yerel/registry paketleri, kilit dosyası, test blokları ve biçimleyici.
- **Editör desteği:** tanılar, tamamlama, hover, tanıma gitme ve Türkçe ekleri koruyan yeniden adlandırma.

Çalışan örnekler [golden/](golden/), Zee ile yazılmış kitaplıklar
[kitaplik/](kitaplik/), gerçek kullanım projeleri [dogfood/](dogfood/) içinde.
Özelliklerin sözdizimi ve sınırları [dil turunda](docs/dil-turu.md) açıklanıyor.

## Tarayıcıda dene

Kurulum gerekmeden: [zeedili.com](https://zeedili.com) üzerindeki
[30 dakikada Zee](https://zeedili.com/playground/) dersleri aynı derleyicinin
WebAssembly sürümünü tarayıcıda çalıştırır; kod sunucuya gönderilmez.
Çevrimdışı tek dosya için:

```bash
bash playground/olustur.sh
```

Üretilen `playground/zee-playground.html` dosyasını tarayıcıda aç.
Dosya üretildikten sonra internet bağlantısı gerektirmez.
Öğretmenler için [sınıf rehberi](docs/ogretmen-rehberi.md) mevcut.

## Geliştirme ve belgeler

| Aradığın bilgi | Belge |
|---|---|
| Tasarım ilkeleri ve yol haritası | [Manifesto](MANIFESTO.md) · [Master plan](docs/master-plan.md) |
| Dilin normatif davranışı ve kararları | [Spesifikasyon](spec/) · [RFC'ler](rfcs/) · [ADR'ler](adr/) |
| Derleme, dağıtım ve bağımlılıklar | [Derleme rehberi](docs/derleme-ve-bagimliliklar.md) · [Sürüm runbook'u](docs/surum-runbook.md) |
| Güvenlik ve dış dünya erişimi | [Güvenlik politikası](SECURITY.md) · [Yetkinlikler](docs/yetkinlik-ve-ag-guvenligi.md) |
| Testler ve doğrulama kanıtları | [Depo bütünlüğü](docs/depo-butunlugu.md) · [Faz matrisi](docs/faz-test-matrisi.md) · [Kanıt özeti](docs/kanit-ozeti.md) |
| Rust gömme ve tarayıcı arayüzü | [Public API](docs/public-api-politikasi.md) · [WASM ABI](docs/wasm-c-abi.md) |

Temel katkı kontrolleri (`compiler/` içinden):

```bash
cargo test --locked
cargo fmt --all -- --check
```

Değişikliğe özel test, belge ve güvenlik kapıları [AGENTS.md](AGENTS.md) içinde.
İşlerin sırası ve kapanma kanıtları [backlog'da](docs/oncelikli-backlog.md) tutulur.

<details>
<summary>Canlı depo sayıları</summary>

<!-- ZEE-DEPO-SAYILARI:BEGIN -->
<!-- `cd compiler && cargo run --bin depo_sayilari -- --yaz` üretir. Elle değiştirme. -->
### Canlı depo sayıları

| Ölçüm | Tek kaynaklı değer |
|---|---:|
| Golden program | **33/33** |
| Rust + doctest vakası | **1025** |
| Tanı kimliği | **162 etkin + 3 ayrılmış** |
| RFC | **34** (2 kabul, 29 geçici kabul, 3 taslak) |
| ADR | **148** (148 kabul) |
| Normatif spec bölümü | **30** |
<!-- ZEE-DEPO-SAYILARI:END -->

</details>

PostgreSQL backend mesajları 32 MiB length sınırında erken reddedilir;
kopan transaction ve RLS bağlamı yeni bağlantıda sürdürülmez.
[Davranış ve hata verisi](spec/25-postgresql-veri-erisimi.md) sonuç bütçesi
ile transport sınırını ayırır; bu global RSS garantisi değildir.

Gerçek ürün geçişleri için mevcut parola yüklemi sınırlı legacy bcrypt
($2a/$2b/$2y) doğrular; yeni özet Argon2id kalır. Handler `istemci adresi`
ile oran sınırının kullandığı kanonik IP'yi okur.
[Kullanım ve güvenlik sınırı](spec/12-web-guvenlik-profili.md) ve
[ürün kanıtı](dogfood/zee-cekirdek/KOKEN.md) aynı değişikliktedir.

### ürün E özelliklerinin ana kaynak hattı — K-213

[ADR-086](adr/086-site-hatlarinin-birlestirilmesi.md) mevcut ürün E
parola üretimi/TOTP/SMTP, rota birimi, görünüm, görsel ve gizli değer
yüzeylerini K-208/K-209 bcrypt/IP ile aynı hatta toplar. Ürün kabulü
dosya kontrolüyle değil bütün proje `dil dene .` ile alınır. Ham ürün E
şablonu normal katalog üretimini ister. Exact source/CI kapanışı beklenir;
canlı dağıtım ve otomatik parola göçü bu işte yapılmaz.

### K-214: Anonim açık depo kanıtı

Müşteri ve altyapı kimliği anonimleştirildi; yeni içerik hash politikasıyla
CI kapısında denetlenir. Önceki geçişler de K-215 kapsamında temizlendi.
Tarihsel ölçümler eski kaynak kimliklerini korur; yeniden açılış ve
GitHub PR önbelleği temizliği ayrıca onay/teyit ister. [Kayıt](docs/incelemeler/k214/README.md).

### K-215: Önceki ürün geçişleri ve bağımsız yerel yama

Yayımlanmış yedi dal ve on bir etiketin kaynak geçmişi anonimleştirildi;
ilk commit dahil taranan 406 commit'te sıfır eşleşme var. Eski müşteri satırı
istisnası kaldırıldı. Compiler/fuzz PostgreSQL yaması yerel path bağımlılığıdır;
SBOM hem kaynak commit'ini hem derlenen paket baytlarını doğrular.

Yerelde **771/771 test** ve iki temiz macOS klonunda ikili eşitliği geçti.
Güncel uzak CI işleri hesap ödemesi/harcama limiti nedeniyle başlamadı;
Linux/Windows ve uzak kapı kanıtı açık. Enterprise yalnız aynı kaynak aynasıdır,
Actions kapalıdır ve özel ürün pini korunur. GitHub PR refs/cache teyidi ile
yeniden açılış onayı bekleniyor; depo gizli kalır. [Kayıt](docs/incelemeler/k215/README.md).

### K-216: Git2 ve Gitsrv üzerinde koşular

Kullanıcı CI koşularını özel Git2/Gitsrv sistemine taşıdı. Ana GitHub kaynağı
merbay-erp/zee, Enterprise yalnız aynadır; ikisinde Actions kapalıdır.
Forgejo akışı tam Linux faz/kalite, gizlilik/sır, tedarik/offline vendor ve
WASM kapılarını exact seçili kaynakta çalıştırır; ilgisiz özel dal/etiket
almaz, PR kodu otomatik çalıştırılmaz. Uzak sonuç doğrulanınca kaydedilir;
Windows runtime kanıtı ve GitHub destek teyidi açık kalır.
[Karar](adr/088-git2-gitsrv-kosulari.md).

Üretim web kipinde ürünün proxy'si ortak saldırı kotası sağlıyorsa
`--web-oran-deposu bellek` süreç başına oran deposunu seçebilir. Oturum
paylaşımı sürer; restart ve ayrı worker sayaçları ayrıdır.
[Karar ve sınırlar](adr/089-bellek-oran-deposu.md).
Bu kipte `--web-oran-siniri`, `--web-oran-kapasitesi` ve
`--web-oran-doluluk en-eski` seçilebilir. Tahliye uç nokta kotasını sıfırlayabilir;
giriş/CSRF canlı sayaçları korunur. [Ayarlar](adr/090-web-oran-ayarlari.md).

Native dış isteklerde başlıklı GET/POST ve gövde gönderimi başarısızlıkları
`Sonuç` olarak yönetir. Ortam sırları explicit proje listesiyle okunur;
[Testli kullanım örneği](docs/http-post-kullanim.md),
[HTTP sınırları](adr/091-http-post-baslik-ve-gizli-ortam.md).

Ürünleşme A6: bilinmeyen CLI seçenekleri kaynak dosyası adı gibi
yorumlanmaz; exit 2 ve etkin sürümle reddedilir (K-221).

CI biçim denetimi: `dil biçimle --denetle .` (uygun biçim0, fark1, yazma yok).

Proje runtime host ayarları için [zee.ayar profili](docs/zee-ayar.md) ve
`dil sürüm --ayar-semasi` yetenek sorgusu vardır; eski-reader rollback
ve canlı ürün kabulü ayrı açıktır.

Native [ekli e-posta profili](rfcs/0034-ekli-eposta.md) readonly PDF/görsel
eki, Reply-To ve HTML/düz alternatif gövdeyi destekler. Queue ve canlı
provider/ürün kabulü açık; IO izi/WASM gönderim yapmaz.

K-248 LSP yüzde UTF-8 dosya yolu düzeltmesi ve gerçek stdio regresyonu
[ADR-108](adr/108-lsp-uri-utf8-yolu.md) ile kaydedilir; exact source/faz
kabulü yeni devam kaydında tamamlanır.

Sınırlı POSIX [credential host köprüsü](docs/credential-baslat.md),
[worker yaşam döngüsü](docs/worker-supervisor.md) ve mevcut SQL ile
[kalıcı iş kuyruğu prototipi](dogfood/zee-cekirdek/is-kuyrugu/README.md)
teknik pilot dilimleridir; P09/P13/P25 tam ürün kabulü açık kalır.

K-302, temiz `e6322db` üzerinde mevcut TLS8/göç1 yerel kabulünü
tazeledi; genel P31/P36 ve ürün/uzak CI sınırları [raporda](docs/urunlesme-k298-kabul-20261016.md).

K-303, temiz `47812bf` üzerinde mevcut pool sahibini 1/0/0 ve 16 metrik
anlık görüntüsüyle doğruladı; tek timeout ve normal PG kapanışı geçti.
K-302 farklı kaynaktadır; sonuçlar birleştirilmez, native metrik tüketicisinin
sekiz vakası bu dilime dahil değildir. Son 11 belge/güvenlik kapısının
geçmesi kayıt commit’inin önkoşuludur; [kapsam ve makbuzlar](docs/urunlesme-k298-kabul-20261016.md).

K-301 worker test doğruluğu dar kapsamda tamamlandı: üçüncü gerçek koşu
tek Cargo sahibi 1/0/0 ve wrapper'ın 25/0/0 doğrulamasıyla geçti. V3 normal
`web-oturumu`/CSRF GET/çerez/form POST ve dosya yazan `eylem` kullanır;
CSRF atlanmaz. İlk GET fixture'ının T045 reddi ile ikinci koşunun yalnız
inflight hatası (25 test/1 hata) korunur; ikinci kayıtta assertion ayrıntısı
yoktur, CSRF/çerez eksikliği kaynak incelemesidir. Son 11 belge/güvenlik
kapısının geçmesi kayıt commit'inin önkoşuludur. Üretim davranışı ve genel
P34/drain/fault kapsamı değişmez; [worker rehberi](docs/worker-supervisor.md)
kanıtları ayırır.

K-257–K-260 aday dalgası [native request/DB metriklerini](docs/native-metrics.md),
[sonlu SQL dispatcher'ı](docs/job-dispatcher.md) ve
[sahipli listener nesil devrini](docs/native-listener.md) ekler. Descriptor,
credential ve süreç sahipliği sınırları rehberlerde kayıtlıdır; temiz ortak
kampanya, uzun süreli kontroller ve gerçek ürün benimsemesi henüz açıktır.

K-249 [anonim TR–EN sayfa](dogfood/zee-cekirdek/tr-en/README.md) mevcut
görünüm ve plain Metin ile native escaping/ret kanıtıdır; genel
layout/i18n ve gerçek ürün kabulü P21 kapsamında açık kalır.

[K-245–K-249 temiz pilot kabulü](docs/urunlesme-pilot-k245-k249.md): POSIX
credential/worker, dar SQL queue, TR–EN yardımcı ve LSP UTF-8 regresyonu.
Tam ürün ve sürüm kapıları raporda açık sınırlarıyla izlenir.

K-265 [exact kaynak dış RC kanıtı](docs/rc-kaniti.md) release kapısındaki tarihçe self-reference sınırını açık dış paketle ele alan kısa doğruluk dilimidir; gerçek final kaynak ASAN/Miri/soak kampanyası yerine geçmez.

K-267: dar Unix credential/native listener bileşimi ve ayrı host-only startup capability için [ADR-125](adr/125-credential-native-listener-kompozisyonu.md). Varsayılan noinherit korunur; P25, final temiz kaynak/uzun kampanyalar ve canlı/platform benimsemesi kısmi kalır.

### Yerel host/başlangıç araçları

[Sonlu SQL timer](docs/sonlu-dispatcher-timer.md), [kopyalanabilir site/panel](sablonlar/README.md) ve [yerel compiler import/proje launcher](docs/derleyici-yerel.md) dar owned pilotlardır. Gerçek hizmet/platform/live benimseme ve final temiz kaynak kabulü ilgili rehberlerde açık tutulur.

## Dar owned Unix geliştirme araçları

[K-271 native web test kiti](docs/owned-web-test-kiti.md), gerçek CLI/HTTP session,
CSRF ve public snapshot ile vaka başına fresh PG fixture'ını ayırır; source coverage
ve public HTTP injection sözü değildir. [K-272 tam TR/EN golden](docs/tr-en-list-detail.md)
sabit typed HTML6route/new9HTTP/5Zee ile önceki7HTTP/4Zee davranışını korur.
[K-273 yönetilen iskelet](docs/yonetilen-moduller.md) yeni owned projede modül,
menu/content metadata ve journal yaşam döngüsünü sınar; SQL arşivi yürütmez.
[K-274 süreli hazırlık](docs/worker-supervisor.md) explicit default-off TTL,
finite DB watcher ve config atomic replace/HUP profilidir. P22/P20 bütünü,
gerçek ürün/P08/platform ve final exact clean/900/1800/RC kapıları ayrıca açıktır.

### Sabit PDF host örneği

[K-275 rehberi](docs/pdf-katalog-host.md) mevcut Zee JSON örneğini ayrı owned backend ile PDF üretme/sayfa seçmeye bağlar. PDF intrinsic/FFI/resmi eklenti değildir; P11 kısmi ve ürün/finalclean kabulü açıktır. Normal stdlib modelleri PDF paketleri istemez; exact backend kabulü explicitignored/operatör runtime ile ayrıdır.

### K-276–278: Linux taşıyıcı hazırlığı, native dış etki ve editör şablonu

K-276/[ADR-134](adr/134-linux-ozel-paket-tasiyicisi.md) özel inert Linux taşıyıcı hazırlığıdır: normal wrapper
12 preflight +9 carrier model/negatifi yürütür. ELF header doğrulaması Linux
ABI/çalıştırma kanıtı değildir. İki temiz klon kanonik teslimi, final kaynak S
üzerinde gerçek .deb build/install/remove/user-prefix activation/container
kabulü ve yayıncı/lisans/imza/apt repository ayrı açıktır. Namespace/image
manifest önkoşulu tamamlanmış Zee container kabulüne dönüştürülmez.

K-277/[ADR-135](adr/135-sabit-native-dis-etki-tuketicisi.md), mevcut SQL-only dispatcher'dan ayrı fixed owned
loopback native SQL→MIME SMTP→ack profilidir. Effect-start latch bilinen committed
olmadan SMTP başlatılmaz; owner/token/generation/lease korunur. Yalnız kanıtlı
pre-effect SQL reddi, sıfır send invocation ile toplam3/1–2sn backoff yapabilir.
C029/DATA/ackCOMMIT belirsizliği guard+exit3/manual incelemedir; SMTP otomatik
replay/exactly-once sözü yoktur. Bilinen completion sonrası görünür guard unlink
fakat final fsync hatası ayrı exit4'tür; olmayan guard yeni admission'ı engelliyor
denmez. Normal hazırlık8 ve explicit ignored native11 ayrı kapılardır. Kaynak
harness yalnız dört pinli fixture dosyasını kopyalar; ek SQL/.dil/FIFO alınmaz.
Tarihsel dirty aday kanıtı finalclean değildir. Gerçek B1/B2 katalog, sağlayıcı
Gmail/M365, K-275 PDF üretimiyle ürün entegrasyonu ve tam P09/P08 açıktır.

K-278/[ADR-136](adr/136-jetbrains-yerel-lsp-sablonu.md) mevcut K-270 launcher'ını kullanan POSIX LSP4IJ veri
şablonudur; TextMate mevcut VS Code bundle'ından gelir. Npm bağımlılığı olmayan
elle yazılmış VS Code istemcisi için owned `dillsp` mutlak yolu açıkça seçilir;
IDE macro/tokenizer/kurulum/etkileşim/Windows kabulü henüz açık kalır. Şablon
installer/download/yeni grammar/capability/cache veya gerçek IDE kabulü eklemez.
[PDF/SMTP'den ayrı editör rehberi](editors/jetbrains/README.md) bu sınırı korur.

### K-279: gerçek önceki küçük sürüm korpusu ve açık göç

[ADR-137](adr/137-onceki-kucuk-surum-korpusu.md) gerçek v0.7.0 exact
`d08a947713ddc38d2163360c17d07fbffcb1213d` kaynağındaki33 golden girdisini
değişmeyen blob/SHA ile korur.32 directcheck eşliği ve15 saf runtime çıktısı
gerçek eski/yeni CLI'da sınandı; birim örneği K-086 publictype sözleşmesi için
yalnız birimde iki annotation satırıyla göç ister. Ham33 kaynak gizlice
yenilenmez. Normal iki Rust vakası49 modernCLI altkontrol+5 erken bootstrap
reddini yürütür;54 libtest veya gerçek anonim ürün/P08 kanıtı sayılmaz.

Eski CLI config capability JSON sunmadığı için mevcut BuildJSON/selector
reddi korunur; yeni `zee.ayar` için otomatik legacy rollback açılmaz. Native
çalışma altkümesi effect-free'dir; ağ/web/dosya/saat/rastgele/async/hardware
runtime örnekleri çalıştırılmaz. Sabit33 kaynakta provenance olarak duran bu örnekler
compiler bugfix/dogfood veya minOS/platform kabulü değildir.

Linux'ta e76 retained leader cleanup ölçümü komut başına yaklaşık3.001s'dir.
52 sınırlı komutun yaklaşık156s toplam bakım yükü için yalnız test harness
aggregate bütçesi120→240s seçildi; her native çağrı8s, owned cleanup/deadline/
capture, kaynak/output oracles ve production SLA/perf eşikleri değişmedi.
Bağımsız Mac49+5 replay ayrı; gerçek tam Linux ve temiz finalS kabulü ayrıca
gerekir. Tam P04/P05, ürün/kanal/platform/adoption açık kalır. [Rehber](docs/onceki-kucuk-surum-korpusu.md).

K-271–K-279 commit öncesi birleşik kapısı (15 Ekim 2026): tam faz matrisi
**943/959 geçti, 0 başarısız, 16 explicit ignored**; faz süreleri toplamı
429.100 saniyedir. Koşu boyunca 1.477 kaynak/belge/fixture dosyası aynı hash'te
kaldı. Fmt, tüm hedeflerde Clippy, işlev64, spec214, katalog/tazelik,
katman/SCC/public API/uyumluluk/tedarik, gerçek Zee–Rust–Go çıktı eşliği,
sürekli güvenlik, semantic/core-freeze ve kanonik sır taraması geçti.
Üç yeni Rust test sarmalayıcısındaki sabit platform assertion'ı açık Unix
koşullu gövde ve desteklenmeyen platform panic'iyle kanonik biçime taşındı;
Unix vakaları ve üretim compiler davranışı değişmedi. İlk Clippy retleri
ayrı dış kayıtta korunur. Normal fazdaki16 dedicated atlama temiz kaynak
kampanyasında ayrıca yürütülmelidir; bu sonuç final temiz kaynak/native,
900 saniye işletim, 1800 saniye soak, ASan/Miri, paket/kurulum veya gerçek
ürün/platform/IDE/insan kabulü değildir. Yeni compiler/src, grammar, spec
normu, public Rust API veya production bağımlılık değişikliği yoktur.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

Temiz `8b49113c413f62b35d4b251ed31068524d2fe26c` kaynağında943/959normalfaz,16dedicatedowner/25ayrınativegrup, macOS/Linuxkanonik teslim/kurulum, Linuxcarrier/container/timer,900snownedyük ve1800sncompiler/LSPsoak tamamlandı. DörtASAN hedefi1801sn, ikiMiri grubunda3test ve aynıS RCkapısı510.198758sn geçti; GB082dar kapsamda kapalıdır. [Son rapor](docs/urunlesme-final-kabul-20261015.md) gerçek teknik/ürün/insan/platform ve IDEcompletion/orderlyshutdown açıklarını korur. E yalnız kanıt/belge arşividir; imza/releaseetiketi/yayın yapılmadı.

[RC sonuç makbuzu](docs/incelemeler/urunlesme-20261014/final-8b49113/rc/FINAL-RESULT.json), [aynı-S kapı makbuzunun türetilmiş temsili](docs/incelemeler/urunlesme-20261014/final-8b49113/rc/POST-GATE-RECEIPT.derived.json), [bağımsız kök incelemesi](docs/incelemeler/urunlesme-20261014/final-8b49113/rc/root-independent-review.json).

Editörün UTF-16 eşitlemesi, biçim ve opt-in uyarıları [editör analizi](docs/editor-analizi.md);
`dene` içindeki pinli HTTP/DB araçları [native test köprüsü](docs/dene-native-web-koprusu.md);
mevcut adayların kabul sınırları [ikinci teknik dalga](docs/urunlesme-ikinci-dalga-20261015.md)
belgelerinde tutulur. Dar yerel S3 kabulü tamamlandı; resmî sürüm ve açık ürün/insan/platform kapıları ayrıdır.

## S3 ikinci dalga yerel teknik kabulü

Kaynak `66e4149e53451111d153f47283c219079a8d1cab` üzerinde ikinci dalganın native senaryoları, macOS/Linux için iki temiz klondan teslim ve sekizer kurulum testi, gerçek JetBrains tamamlama/biçimlendirme/tanı/kapanış akışı, 1.800 saniyelik soak, 1 ve 41 açık belge ölçümü, dört ASan hedefi ve seçili Miri testleri geçti. Kanonik kanıt dışa aktarımı ve denetimi ile aynı kaynakta güvenlik kapısı geçti: tam Cargo koşusunda 989 başarılı, 0 başarısız, 25 varsayılan dışı vaka. Bu 25 vaka ayrıca çalıştırılan kampanyalara eşlendi; alt senaryolar Cargo toplamına eklenmez. GB-110 exact regression/native26/RC zinciriyle kapalıdır.

[S3 teknik kabulü](docs/urunlesme-s3-kabul-20261015.md), sonuçları ve önceki başarısız denemeleri ayrı kaydeder. E commitleri belge ve kanıtları kaydeder. C1/K-288 ve P33/K-293 araç retleri; artımlı HIR/type, gerçek ürün/provenance/adoption, insan/takvim, fiziksel Windows/minOS ve yayın/lisans/destek kapıları açıktır. Bu kayıt tam V1 ya da public signed release kabulü değildir.
