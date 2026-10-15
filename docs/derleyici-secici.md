# K-233 çevrimdışı compiler seçicisi — sertleştirilmiş pilot

Bu araç exact proje pinini yalnız operatörün sağladığı trusted yayıncı
kataloğu, exact SHA-256 doğrulayıcı ve yerel imzalı bundle ile çalıştırır.
İndirme, otomatik güncelleme, `latest`, global compiler fallback, main/master
push veya release yayını yapmaz. İlk config desteği K-234 `zee-ayar-1`
yeteneğidir; config okuyucusuz gerçek 0.7 ve gerçek önceki minor ürün rollback
kabulü açıktır.

Önerilen depo yerleşimi `scripts/derleyici-secici.py`,
`scripts/tests/derleyici-secici-test.py`, `docs/derleyici-secici.md`dir.
Python bootstrap ve modül/loader yolu operatörün güven sınırıdır; örnek
çağrılar izolasyonlu Python (`-I`) kullanır. Alt süreç temizliği daha önce
kompromize edilmiş interpreter/OS ortamını geri alamaz.

```sh
python3 -I scripts/derleyici-secici.py seç \
  --proje /exact/proje --depo /yerel/bundle-deposu \
  --guven /trusted/katalog.json --guven-anahtari sha256:<trusted-key-id> \
  --dogrulayici /trusted/surum_artefakti --dogrulayici-sha256 <exact-sha256>
```

Pin `derleyici.kilit.json` exact beş alan taşır: `schema=zee-derleyici-pin-1`,
`version`, `source_sha` (40 küçük hex), `channel`, `platform`. Metadata
executable/yayıncı/verifier yolu belirleyemez. `pinle` ilk pinin no-clobber
publication'ını, yalnız explicit `--degistir` bilinçli değişikliği sağlar.
Existing pin kaynak/katalog doğrulanmadan yazılmaz. Development/dirty/unknown
local ikili explicit `--pilot` ister; stable unknown/dirty kabulü pilotla
açılamaz. Operatör trusted publisher binary'si kullanıcının bütün kod
yetkileriyle çalışır; seçici bunu sandbox'a dönüştürmez.

Doğrulama sırası: explicit verifier byte hash → katalog Ed25519 imzası →
exact pin/katalog kaydı → bütün sidecar ve binary hash/metadata eşliği →
SHA256SUMS Ed25519 imzası → özel temporary byte kopyası → authenticated build
JSON ve varsa config capability → aynı projenin çağrısı. Original bundle
sonradan değiştirilse bile çalıştırılan byte kopyası değişmez. Kimlik/provenance
uyumu gerçek release üretimini kanıtlamaz; test fixture'ları sentetik,
imzalı protokol vakalarıdır. Bir mevcut yerel debug çiftinin testi iki temiz
klonda üretilmiş release kabulü değildir.

## Beş sertleştirme

Dosya okuyucusu LSTAT normal-file, sınırlı boyut, nonblocking/nofollow/close-on-exec
open (platformda mevcut bayraklar), FSTAT normal-file ve inode/device/boyut/mtime
eşliği, bounded read, actual bytecount/post stat ve yol kimliği kontrolleri
uygular. Stat/open arasında writer'ı olmayan FIFO'ya geçiş bloklanmadan
reddedilir. Symlink ve aynı yolun farklı dosyaya geçişi reddedilir. Bu denetim
aynı kullanıcıdan namespace/metadata restoration saldırısına karşı atomik
filesystem snapshot veya OS sandbox sözü değildir. Regular/NFS OS read ve
Windows fiziksel reparse/minOS ayrı sınırlardır.

| Dosya rolü | Maksimum |
|---|---:|
| Proje pin / `zee.ayar` / imza | 64 KiB |
| Trusted katalog | 2 MiB |
| SBOM | 4 MiB |
| Provenance | 16 MiB |
| Native verifier / `dil` / `dillsp` | 64 MiB |
| SHA256SUMS | 64 KiB |
| GIT_SHA | 64 bayt |

JSON bütün derinliklerde duplicate anahtarı reddeder; duplicate tanısı alan
adını/değerini basmaz. Invalid UTF-8/JSON, decoder integer/recursion ValueError
ve yanlış container/field tipleri sabit ret ve exit 2'dir; traceback veya
kullanıcı JSON alıntısı yoktur. Yalnız üst sınırların içinde olmak zarf/type/
crypto kabulü değildir.

Verifier, build/capability probe ve program child environment'inden bütün
`LD_`, `DYLD_`, `PYTHON*` kontrol alanları çıkarılır. Özel uygulama ayarları ve
operatör PATH'i korunur. PATH/dynamic dependency closure, trusted bootstrap ve
Windows DLL/physical-platform kabulü ayrıca operatör/sürüm kanıtı ister.
Bu çalışma Linux'ta LD_AUDIT saldırı yürütmesi veya OS runtime attestation
kanıtı değildir. Trusted executable probe'larının timeout ve çıktı boyutu
kontrolü untrusted arbitrary-code memory sandbox'ı değildir.

Seçici `çalıştır` tail'inde çocuk/web host bayraklarını (`--güvenli`,
`--guvenli`, `--deneysel-web`, proxy alias'ları, worker-port ve `--web-oran-`
alanını, `=value` yazımını da) reddeder. Current compiler bunları kaynak
sonrasında da global yakalayabildiği için tail uygulama argv'si diye
aktarılmaz. Olağan `--foo`, `--json`, `--uygulama-secimi` program argümanları
korunur. Seçiciyle web/çocuk ayarı yalnız geçerli `zee.ayar` üzerinden yapılır;
doğrudan compiler CLI precedence sözleşmesi ayrı sürer. Diğer proje yolu ve
LSP ek executable argümanı kabul edilmez.

Pin temp dosyasının creation/write/flush/fsync/publication aşamaları tek
`finally` sahibindedir. Fsync/write hatasında current pin korunur ve geçici
dosya temizlenir; tempfile/özel bundle context'leri normal istisnalarda
kapanır. OS SIGKILL/güç kaybı veya cleanup yetkisi kaldırılması için mutlak
cleanup/durability garantisi verilmez.

## Test

```sh
# Önce aynı kaynakta dil, dillsp ve surum_artefakti derlenir.
python3 -I scripts/tests/derleyici-secici-test.py \
  --compiler-dir compiler/target/debug
```

Bu standalone aday dizininde aynı test `python3 -I tests.py --compiler-dir
/private/tmp/zee-k219-20261014/compiler/target/debug` ile çalışır.
17 existing exact routing/crypto/private snapshot/config/real debug-pair
vakası + 8 hardening vakası vardır. Yeni vakalar gerçek controlled stat→FIFO
swap, 20 KB/10,000-depth pin ret, integer/duplicate/type ret, role-bound
pre-open ret, environment filtering, actual launch global flag ret/ordinary
argv korunması ve fsync orphan=0/current-pin byte eşliğini doğrular.
POSIX FIFO vakasının diğer platformda atlanması Windows kabulü yerine
geçmez; burada native POSIX sonuçları ayrı exact artifact kaydındadır.
Root repo CI/platform/release/insan kapıları ve final source kayıtlarını
aynı toplu değişiklikte tamamlar; temp aday testi final temiz release kanıtı
olarak gösterilmez.

Cargo faz kapısında `derleyici_secici_testi` aynı build araçlarıyla 25
Python vakasını izolasyonlu interpreter (`-I`) ile sıfır atlamalı çalıştırır. Windows'ta bu Cargo vakası
explicit ignored raporlanır; native Windows pilot fixture kabulü açıktır.

## K-270 — Yerel import ve proje launcher

[Yerel araç rehberi](derleyici-yerel.md) mevcut signed selector kabulüne explicit owned bundle import ve `.zee-araclar/zee` launcher ekler. Compiler pin `derleyici.kilit.json`, gerçek paket graph kilidi `proje.kilit` v3 olarak ayrı kalır. Eski25 ve yeni14 vaka aynı Rust testi sayısı olarak toplanmaz; yeni14 gerçek P008 ve framed LSP içerir. Global PATH/download, production key/release trust ve Windows/minOS/live adoption kabulü açık kalır. [ADR-128](../adr/128-yerel-compiler-import-ve-proje-launcher.md).
