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

Ana geliştirme ve CI deposu [merbay-erp/zee](https://github.com/merbay-erp/zee).
Enterprise deposu yalnız aynadır.

zee aktif olarak geliştiriliyor. Kaynak ağacının sürümü **0.8.0-dev**;
son sürüm notu **v0.7.0**. V1 öncesi doğrulama çalışmaları sürüyor.

| Alan | Bugünkü durum |
|---|---|
| Dil ve araçlar | Lexer, parser, tür denetimi, yorumlayıcı, proje/paket araçları ve LSP çalışıyor. |
| Derleme | Bootstrap derleyici Rust ile yazılıyor; araç zinciri Rust **1.93.1** sürümüne sabit. |
| Çalıştırma | Derlenmiş `dil` ve `dillsp` için Rust kurulumu gerekmiyor; sistem kitaplığı gereksinimleri platforma bağlı. |
| Platform doğrulaması | Linux, macOS ve Windows'ta iki temiz klonla eş hash, kurulum, program/LSP ve kaldırma tatbikatı geçti. |
| Açık işler | Gerçek kullanıcı testleri, minimum işletim sistemi/fiziksel makine kanıtı ve native derleme araçlarının tam envanteri. |

**Zee kaynaklarını doğrudan native/WASM koduna derleyen backend ve
self-hosting henüz tamamlanmadı.** Mevcut playground, Rust derleyicisinin
WebAssembly sürümünü tarayıcıda çalıştırır.

Ayrıntılar: [Derleme ve bağımlılıklar](docs/derleme-ve-bagimliliklar.md) ·
[V1 sürüm kapıları](docs/v1-surum-kapilari.md) ·
[Öncelikli backlog](docs/oncelikli-backlog.md).

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
| Rust + doctest vakası | **829** |
| Tanı kimliği | **159 etkin + 3 ayrılmış** |
| RFC | **33** (2 kabul, 28 geçici kabul, 3 taslak) |
| ADR | **91** (91 kabul) |
| Normatif spec bölümü | **28** |
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
