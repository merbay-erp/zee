# ADR-086 — Site derleyici hatlarının birleştirilmesi

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıt:** K-210–K-213, urun-e, zee-cekirdek, RFC-0030

## Bağlam ve karar

`ci/urun-e-dil` 34fbc0aab7498c53af6fbee0c534966e024a14e4 (sunucu kaynağı
86631fb2facda2945ec6174e31f7b3fc6b4d2291), master 9482ce8cca293f89ffbc53aaf047d01c60a97324
ile bcc17b4406ee9a088777577c0ae366378e6fe691 tabanından ayrılmıştı.
K-208/K-209 ana dalda, sitelerin parola üretimi/TOTP/SMTP/rota/görünüm/gizli
şifreleme yüzeyleri yalnız ürün E dalındaydı. Dosya dosya `denetle` kabul değildir;
proje bildirimi, rota birimleri ve testleri aynı proje yüklemesiyle doğrulanır.

Mevcut ürün E özellikleri kaynak commit geçmişi korunarak ana hatta birleştirilir.
Yeni bir site sürüm hattı açılmaz. GitHub işlemleri `merbay-erp/zee`de yapılır;
Enterprise salt ayna kalır, Actions açılmaz. Ürün CI'ı bu ana deponun
`dil-kur` action'ını immutable tam SHA ile kullanabilir; ürün pinleri bu depoda
kendiliğinden değiştirilmez.

## Kayıt kimliği eşlemesi

| ürün E dalındaki tarihsel kimlik | Ana hattaki kimlik | Anlam |
|---|---|---|
| K-181 / ADR-075 | K-210 / ADR-083 | Parola üretimi, TOTP, SMTP, görsel, gizli değer |
| K-182 / ADR-076 | K-211 / ADR-084 | Rota birimleri |
| K-183 / ADR-077 | K-212 / ADR-085 | Görünüm bloğu |

Master'daki aynı eski numaralı kararlar korunur. ürün E source SHA'ları yeniden
üretilmez; ana hatta ilk kez alınan beyanların yolları/işleri bu eşlemeye
bağlanır. K-213 bu birleşimin ve kabul sırasında bulunan düzeltmelerin işidir.
Merge SHA'sı da exact compiler/core-freeze beyanı taşır; koruk, çok ebeveynli
commit'in ilk ebeveyne kaynak farkını inceler. Eksik merge beyanı reddedilir.

## Doğruluk ve mimari sınırlar

Rota kapsamı dosya-yerel satırdan değil, bağlanmış yol ifadesinin tekil HIR
kimliğinden gelir. Farklı birimlerde aynı satır numarası SymbolId çakıştıramaz.
Görünüm hedefi `olsun` gibi HIR tanım/yazım kaydı taşır. Gizli şifrele/çöz
olayları trace allowlist ve şemasında birlikte bulunur; replay dış etki yapmaz.
Bu üç arıza minimal regression girdileri ve exact `fixed_by` ile korunur.

SMTP altyapısı yalnız temel zaman sahibine, görsel adaptörü yalnız image
crate'ine dayanır; CLI bu iki adaptörün sahibidir. Güvenlik ilkelleri temel
katmanda kalır. Yeni sahipler `eposta` ve `gorsel`, yeni exact kenarlar
katman fixture'ında incelenmiştir; yeni SCC yoktur. Görünüm ve IO yardımcıları
ayrı alt modüllerdedir; mevcut dosya bütçeleri büyütülmez. V1 Rust facade
export'u değişmez. Derleyici/LSP'ye cache veya kalıcı tampon eklenmez;
istek scope'u zaten tutulan HIR kaydını kullanır. ADR-045/049 performans
kapsamı incelendi; sabit benchmark iş yükü ve kalıcı tarihçe değiştirilmez.

## Kabul ve açık sınır

site-a, site-b, site-c, site-d ve normal `zee-catal` üretimiyle hazırlanan
ürün E bütün köklerinde `dil dene .` geçmelidir. Ham `zee-cekirdek/cekirdek`
şablonunda modül/özellik katalogları yoktur; bunlar normal üretim adımıyla
hazırlanır. Eksik katalog compiler başarısı diye gösterilemez. Exact ürün
kaynakları, üretim komutu ve sonuçlar [kabul kaydındadır](../docs/incelemeler/k213/README.md).

Argon2 üretim DSL'i bu birleşimle ana dalda bulunur; bcrypt yalnız doğrulanır.
Ürünün otomatik parola göçü henüz uygulanmış/onaylanmış değildir: özet ve
kimlik sürümüyle kurulan oturum bağı birlikte tasarlanıp sınanmalıdır.
Sunucu yükseltmesi, canlı giriş/IP/tarayıcı kabulü ve minimum OS kanıtı bu
kaynak birleşiminin sonucu değildir. C027/COMMIT belirsizliği korunur.

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
