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
| Rust + doctest vakası | **728** |
| Tanı kimliği | **155 etkin + 3 ayrılmış** |
| RFC | **29** (2 kabul, 24 geçici kabul, 3 taslak) |
| ADR | **77** (77 kabul) |
| Normatif spec bölümü | **27** |
<!-- ZEE-DEPO-SAYILARI:END -->

</details>
