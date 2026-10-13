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
