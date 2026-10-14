# K-223 — gerçek PostgreSQL parola göçü kabulü

`scripts/parola-goc-ci.py`, [anonim ürün helper'ını](../dogfood/zee-cekirdek/parola-goc/giriste-yukselt.dil)
ve [hermetik kabul modelinin](../compiler/tests/zee_cekirdek_urun_kabulu_testi.rs)
`ATOMIK_GIRIS` kaynak sabitini doğrudan kullanır. Kaynaklar yeniden tasarlanmaz;
geçici projenin `ana.dil` dosyasına aynı helper/dış eylem ve yalnız sentetik
hesap girdileri yazılır. Gerçek derlenmiş `dil çalıştır` native PostgreSQL,
bcrypt ve Argon2id yollarını yürütür.

## Tek komut

```bash
python3 scripts/tests/parola-goc-ci-test.py
python3 scripts/parola-goc-ci.py
```

Varsayılan komut önce locked `dil` ikilisini derler. `--dil /yol/dil` açık
ikili seçebilir; koşucu ikiliyi geçici alana kopyalar, SHA-256'sını kaydeder
ve `sürüm --json` kaydının seçili HEAD'i adlandırmasını ister. Eski ikili yeni
HEAD kabulü olamaz. Helper/model byte'ları hash'lenip aynı sabit kopyadan
üretilir; koşu ortasındaki kaynak değişikliği yeni bir exact kabul sayılmaz.

Docker Engine ve OpenSSL gerekir. TLS koşucusunun aynı [izole küme
lifecycle'ı](postgresql-tls-ci.md) kullanılır: resmî PostgreSQL 16.15 image
exact OCI digest ile seçilir, kendi UUID kümesi ve sentetik CA/key'i üretilir,
yalnız loopback portu yayımlanır ve kapanışta kendi kümesi kaldırılır.
Bağlantı hedefi/CA fixture projesine aittir; mevcut veritabanı URL'si, ürün
verisi veya gerçek parola/sır girdi seçeneği yoktur.

Docker socket'i olmayan Linux CI ortamında kendi boş `initdb` kümesi:

```bash
python3 scripts/parola-goc-ci.py --engine local \
  --postgres-bin-dir /usr/lib/postgresql/15/bin
```

Bu yol mevcut hizmeti kullanmaz; sunucu sürümü raporda ayrıca kaydedilir.
15 numaralı örnek yol, PostgreSQL 16 Docker kanıtı veya desteklenen minimum
PostgreSQL sürümü sözü değildir.

## Yedi native kabul

| Vaka | Gerçek SQL/crypto kanıtı |
|---|---|
| İlk legacy giriş | Üç `$2a$/$2b$/$2y$` sentetik vector native helper'da Argon2id'e geçer; yeni özet native doğrulayıcıyla kabul edilir ve birer oturum satırı commit edilir. |
| İkinci modern giriş | Güncel özetle giriş başarılıdır; özet byte'ları aynı, ek UPDATE/UPDATE attempt sıfır, ikinci oturum satırı vardır. |
| Yanlış parola | Ret; özet değişmez, UPDATE/bind attempt ve commit edilmiş yazı/oturum sıfırdır. |
| Aynı eski snapshot CAS yarışı | Bağımsız iki Zee süreci aynı eski özeti taşır. Dış row lock altında ikisi de `pg_stat_activity` içinde gerçek UPDATE/Lock bekler; kilit bırakılınca bir başarı/bir ret, yalnız bir güncelleme ve bir oturum vardır. |
| UPDATE arızası | Sentetik SQLSTATE `23514` trigger reddi; attempted UPDATE vardır, bind yok, legacy özet korunur ve commit edilmiş yazı/oturum sıfırdır. |
| Oturum bind arızası | Önce gerçek UPDATE, ardından bind trigger reddi; SQL attempt sayaçları iki yola da ulaşıldığını gösterir. Dış eylem önceki güncellemeyi geri alır, legacy özet ve sıfır oturum kalır. |
| Commit görünürlüğü | AFTER INSERT trigger'ı yalnız fixture advisory lock'unda bekler. Diğer bağlantı legacy özeti ve sıfır oturum/audit görür; sequence attempt'leri UPDATE+INSERT'in gerçekleştiğini gösterir. Kilit bırakılınca yeni özet ve tek oturum/audit birlikte görünür. |

Sequence'ler yalnız rollback'ten etkilenmeyen test gözlemcisidir; ürün
şemasının veya runtime API'sinin parçası değildir. Probe'daki oturum satırı
anonim modelin SQL kimlik bağlamasıdır; production web oturum çerezi veya
gerçek ürün login kabulü değildir.

Strict koşucu tam yedi vaka adı/sırası, gerçek invariant değerleri/türleri,
başarılı exit ve her vakada açık `ignored=false` ister. Eksik/tekrarlanan,
atlanan/başarısız, iki kazanan veya commit öncesi görünür oturum ret testleri
`scripts/tests/parola-goc-ci-test.py` içindedir. Bunlar Cargo/libtest vakası
sayısına eklenmez; native kampanya IO/DB/crypto/eylem kapsamını ayrı kaydeder.

## Kayıt ve açık sınırlar

Varsayılan rapor `compiler/target/parola-goc-ci.json`dur. Koşu başında eski
başarılı çıktı `failed/initializing` ile değiştirilir; yalnız yedi gerçek
kabul tamamlanınca `passed` olur. Rapor kaynak başlangıç/bitiş SHA'sını,
çalışma ağacı temizliğini, sabit helper/model/koşucu hash'lerini, ikili hash ve
build kimliğini, sunucu/image kimliğini ve her invariant sonucunu taşır.
DSN, sentetik parola/PHC özetleri ve CA/private key içeriği rapora yazılmaz.

14 Ekim yerel koşusu, kaynak noktası `8e1c8c86693a74f94aa21ff4ccc3d4c34e81c5d6`,
PostgreSQL 16.15 Docker üzerinde **7/7, 0 ignored** geçti. Çalışma ağacı kirli
olduğundan `source_acceptance=working-tree-only`dur. Bu kayıt yeni temiz exact
checkpoint veya uzak CI kabulü değildir; temiz kaynakta yeniden koşu aynı
rapor sözleşmesiyle ayrıca kaydedilir.

**COMMIT yanıtının ağda kaybı bu kampanyada enjekte edilmez**
(`commit_response_loss_probed=false`). Hermetik ürün modeli iki belirsiz
DB sonucunun C027 olduğunu ve otomatik tekrar yapılmadığını kapsar;
[K-206 native frame/COMMIT matrisi](incelemeler/k206/frame-guard/README.md)
ayrı adaptör kanıtıdır. Bunlar bu helper için yeni native C027 ağ-fault
kampanyası, gerçek ürün göçü, canlı sunucu yükseltmesi veya signed release
kabulü diye birleştirilmez. Ürün staging/login, ikinci faktör, canlı oran
limiti, migration/yedek/rollback ve COMMIT uzlaştırma politikası ürün sahibinde
açık kalır.
