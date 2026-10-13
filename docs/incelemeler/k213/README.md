# K-213 — iki compiler hattının bütün proje kabulü

Eski sunucu kaynağı 86631fb / ürün E head 34fbc0a ile master 9482ce8
aynı kaynak hattında birleşir. Source/provenance ve clean kabul sonucu
takip kaydında tam SHA ile eklenecektir. Canlı sunucu yükseltmesi değildir.

İlk debug bütün-proje denemesi, dört sitede dosya-yerel satır numarasıyla
rota SymbolId kapsamının çakışmasını C000 ile buldu. Görünümün HIR
tanım/yazım kaydı ve gizli IO olayının allowlist/şeması da tamamlandı.
Üç minimal regression girdisi ve gerçek Cargo testleri korunur.

Geliştirme ağacında `dil dene .`: site-a 54/54, site-b 48/48, site-c
49/49, site-d 47/47, hazırlanmış ürün E 44/44. ürün E'nin ham
`zee-cekirdek/cekirdek` şablonu `modul_katalogu`/`ozellik_katalogu`
üretilmediğinde A010 verir. Bu hata gizlenmez: ürün E kabul kopyası normal
`araclar/zee-catal.py --profil urun-e/zee/profil.py --ust cekirdek
--hedef <geçici-kök>` ile üretilir; kaynak depoları değiştirilmez.

Katman/SCC/API/fmt/uyumluluk/spec/faz/tedarik kapıları, compiler/fuzz
kilitleri ve artefakt SBOM/provenance etkisi incelenir. Image ve XChaCha
eski ürün davranışını taşır; yeni compiler/LSP cache yoktur. Yeni benchmark
iş yükü veya tarihçe satırı eklenmez (ADR-045/049). Otomatik parola
göçü, gerçek SMTP/TOTP/tarayıcı ve sunucu yükseltmesi ürün kabulüdür.

K-213 tedarik birleşimi: XChaCha kitaplığı exact `chacha20poly1305=0.11.0`
(`alloc,zeroize`, default özellikler kapalı) ile bcrypt/Argon2/PostgreSQL'in
ortak cipher/crypto-common sürüm hattına taşındı. MSRV 1.85, sabit Zee
Rust 1.93.1 ile uyumludur; [resmî manifest](https://docs.rs/crate/chacha20poly1305/0.11.0/source/Cargo.toml).
Image exact 0.25.10, iki lock'ta flate2 1.1.9; PNG ve flate2 aynı
miniz_oxide 0.8.9'u kullanır. `multiple-versions=deny` ve istisnaları
değiştirilmedi. Yeni sürümle 0.10.1 üretimli eski sentetik `zee1:`
şifreli metin bayt bayt eşit üretilir ve çözülür; yanlış anahtar/bozulma
reddi aynı testte korunur. SBOM/provenance ve iki-clone/offline kabulü
yeni exact graph'tan üretilir.

## Temiz kaynak kabulü

Feature/merge source `e09493eb560ff14652280500c7d4848a0b7ed990`;
exact beyan ve kabul kaynağı `0531432f0face3106ce271acefffb2f85440459f`.
[769/769 faz](faz-clean.md), [ürün snapshot/komut/sonuç](acceptance.json),
[offline vendor](vendor.log) ve [iki temiz klon](release/accepted.log) kalıcıdır.
Vendor 187 paket, aynı iki üretimin manifest SHA-256'sı
`124aceb6bca19e9aee16db8cc83bb243e050f0184fe46f937ba7e6852bc9e115`.
`dil` SHA-256 `a8b534f1a1d2addf8326972fd2ceecd3114e25c37156aedb3431058740b68404`,
`dillsp` `1141adda211ba35bf5ba79165dbff7036311418312c6b6d566e12edc13a09749`.
[SBOM](release/sbom.spdx.json) ve [provenance](release/provenance.intoto.json)
aynı clean kaynağın macOS arm64/Rust 1.93.1/release kabulünü taşır.
İmza/sürüm etiketi, Linux sunucu ikilisi veya minimum OS kabulü değildir.

[site-a](site-a.log) 54/54, [site-b](site-b.log) 48/48,
[site-c](site-c.log) 49/49, [site-d](site-d.log) 47/47,
[hazırlanmış ürün E](urun-e.log) 44/44 aynı clean `dil` ile geçti. Her ürünün
kaynak SHA-256'sı koşu öncesi/sonrası aynı; kaynak ürün depoları değiştirilmedi.
ürün E normal üretim adımıyla geçici köke hazırlandı; ham şablon katalog eksikliği
bu kayıtta açıkça korunur.

İlk Linux CI 38391037280/job 115274603591, geçici merge testinin Git
committer kimliğini hosttan aldığı için başarısızdı. Merge komutu kendi
sentetik test kimliğini açıkça alır; global/system config kapalı test ayrıca
geçer. Bu test düzeltmesi production compiler kaynağını değiştirmez.
