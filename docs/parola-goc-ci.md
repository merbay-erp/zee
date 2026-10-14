# K-223 / K-242 — gerçek PostgreSQL parola göçü kabulü

`scripts/parola-goc-ci.py`, [anonim ürün helper'ını](../dogfood/zee-cekirdek/parola-goc/giriste-yukselt.dil)
ve [hermetik kabul modelinin](../compiler/tests/zee_cekirdek_urun_kabulu_testi.rs)
`ATOMIK_GIRIS` kaynak sabitini doğrudan kullanır. Kaynaklar yeniden tasarlanmaz;
geçici projenin `ana.dil` dosyasına aynı helper/dış eylem ve yalnız sentetik
hesap girdileri yazılır. Gerçek derlenmiş `dil çalıştır` native PostgreSQL,
bcrypt ve Argon2id yollarını yürütür.

K-242 ile güncel rapor şeması `zee-parola-goc-ci-2`dir: mevcut yedi TLS
vakası ve dört owned loopback plaintext COMMIT kontrol/arızası ayrı
kaydedilir. Bu genişletmenin final temiz `316a313` native kabulü 11/11,
sıfır atlamayla geçti; eski 7/7 raporlar 11/11 kabulü olarak yeniden etiketlenmez.
[Karar ve sınıflandırma sınırı](../adr/103-native-commit-belirsizligi-kaniti.md).

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

Yedi TLS vaka aynı profilde kalır. Dört ek wire vakası yalnız koşucunun
kendi geçici kümesinde, loopback'e bağlı plaintext PostgreSQL ve sentetik
SMTP ile yürütülür; TLS trafiği çözülmez veya proxy'de downgrade edilmez.
Bu test düzeni public plaintext bağlantı desteği veya güvenli production
proxy sözü değildir.

Yerel `initdb` profilinde plaintext fixture erişimi yalnız `127.0.0.1/32`
HBA kuralıyla açılır. Docker'ın kendi geçici konteynerinde HBA `samenet`
kuralı, host proxy'sinin bridge üzerinden gelen bağlantısını karşılar;
host portu mevcut `DockerCluster` tarafından yalnız `127.0.0.1` üzerinde
yayımlanır. Bu kurallar koşucunun oluşturduğu sentetik kümeye aittir.
Yeni 11-vakalı kampanyanın bu dalgadaki gözlemi source-built PostgreSQL
16.15 yerel motorundadır; Docker kipinde 11/11 kabulü henüz yürütülmemiştir.

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

Güncel attempt gözlemcisi `BEFORE STATEMENT` trigger'ıdır; sıfır satırla
sonuçlanan UPDATE/INSERT de sayılır. Koşucu native vakalardan önce iki
sıfır-satır yazıyla bu gözlemciyi doğrular. Böylece eski snapshot'la
yinelenen CAS'ın sıfır satır üretmesi, tekrarın sayaçtan gizlenmesine yol açmaz.

Güncel strict koşucu tam on bir vaka adı/sırası, gerçek invariant değerleri/türleri,
başarılı exit ve her vakada açık `ignored=false` ister. Eksik/tekrarlanan,
atlanan/başarısız, iki kazanan veya commit öncesi görünür oturum ret testleri
`scripts/tests/parola-goc-ci-test.py` içindedir. Bunlar Cargo/libtest vakası
sayısına eklenmez; native kampanya IO/DB/crypto/eylem kapsamını ayrı kaydeder.

## K-242: dört ek COMMIT kontrol/arızası

Her vaka aynı anonim helper ve dış eylemi çalıştırır. SQL sequence
gözlemcisi tam **bir UPDATE attempt + bir oturum INSERT attempt**;
proxy tam **bir COMMIT girişimi** ister. Sonraki bağımsız SQL bağlantısı
kalıcı sonucu doğrudan okur; sadece hata metni veya wire etiketiyle sonuç
tahmin edilmez.

| Vaka | Wire gözlemi | İstemci ve bildirim | Bağımsız kalıcı sonuç |
|---|---|---|---|
| `commit_olumlu_kontrol_bildirim_bir_kez` | COMMIT iletildi; exact `CommandComplete(COMMIT)` ve `ReadyForQuery(I)`, ErrorResponse yok; cevap istemciye iletildi | Exit 0; stdout `gecti` ve `bildirildi`; bir SMTP bağlantısı/bildirimi | Doğrulanan modern özet, bir oturum ve bir audit |
| `commit_iletilmeden_kayip_c027_kayit_yok` | COMMIT görüldü fakat backend'e iletilmeden kesildi | Exit 1/C027; stdout boş, SMTP bağlantısı/bildirimi sıfır | Legacy özet korunur; oturum/audit sıfır |
| `commit_uygulandi_yanit_kayip_c027_kayit_var` | COMMIT iletildi; exact COMMIT completion, hata yokluğu ve idle doğrulandı; bütün COMMIT yanıtı yutuldu | Exit 1/C027; stdout boş, SMTP bağlantısı/bildirimi sıfır | Doğrulanan modern özet, bir oturum ve bir audit |
| `commit_deferred_ret_c027_gozlemci_rollback` | Deferred trigger'ın `ERROR/23514` cevabı ve idle istemciye iletildi; COMMIT completion yok | Exit 1/C027; stdout boş, SMTP bağlantısı/bildirimi sıfır | Bu fixture'da legacy özet ve sıfır oturum/audit gözlenir |

Hiçbir arızada otomatik write/COMMIT/SMTP tekrarı yoktur. Son client
backend sayısı sıfır ve bütün owned socket/thread/kümeler kapalı olmalıdır.
Başarılı kontrol, arızalardaki sıfır bildirimin erişilemeyen bir SMTP
fixture'ından kaynaklanmadığını gösterir.

Native adaptör bütün dış COMMIT hatalarında konservatif C027'yi korur.
Bu tek stock PostgreSQL `ERROR/23514` deneyindeki rollback, bütün
provider/extension hatalarının sonucu değildir. Hermetik `Commit::Red`
modeli backend'in sonucu açıkça bildiği genel ret/C021'yi temsil eder;
ham PostgreSQL ErrorResponse bu modele eşdeğer sayılmaz.

[Owned proxy](postgresql-commit-belirsizligi.md) 64 KiB startup ve 32 MiB
frame sınırını gövdeyi okumadan, toplam absolute deadline'ı bütün akışta
zorlar. `after` için exact COMMIT completion, ErrorResponse yokluğu ve
idle birlikte gereklidir. Uyumsuz/eksik ack, ERROR+idle, sınır/deadline
ve lifecycle retleri [sahte wire testlerinde](../scripts/tests/postgresql-commit-proxy-test.py)
sınanır; bu Python test sayısı Cargo/native vaka toplamına eklenmez.

## Kayıt ve açık sınırlar

Varsayılan rapor `compiler/target/parola-goc-ci.json`dur. Koşu başında eski
başarılı çıktı `failed/initializing` ile değiştirilir; yalnız on bir gerçek
kabul tamamlanınca `passed` olur. Rapor kaynak başlangıç/bitiş SHA'sını,
çalışma ağacı temizliğini, sabit helper/model/koşucu hash'lerini, ikili hash ve
build kimliğini, sunucu/image kimliğini ve her invariant sonucunu taşır.
DSN, sentetik parola/PHC özetleri ve CA/private key içeriği rapora yazılmaz.
Şema 2, `tls_cases=7`, `plaintext_fault_cases=4` ayrımını ve bütün native
vakalar bitince `commit_response_loss_probed=true` kaydını taşır.

14 Ekim yerel koşusu, kaynak noktası `8e1c8c86693a74f94aa21ff4ccc3d4c34e81c5d6`,
PostgreSQL 16.15 Docker üzerinde **7/7, 0 ignored** geçti. Çalışma ağacı kirli
olduğundan `source_acceptance=working-tree-only`dur. Bu kayıt yeni temiz exact
checkpoint veya uzak CI kabulü değildir; temiz kaynakta yeniden koşu aynı
rapor sözleşmesiyle ayrıca kaydedilir.

**Tarihsel yedi vakalık schema 1 kampanyada COMMIT yanıtının ağda kaybı enjekte edilmez**
(`commit_response_loss_probed=false`). Hermetik ürün modeli iki belirsiz
DB sonucunun C027 olduğunu ve otomatik tekrar yapılmadığını kapsar;
[K-206 native frame/COMMIT matrisi](incelemeler/k206/frame-guard/README.md)
ayrı adaptör kanıtıdır. Bunlar bu helper için yeni native C027 ağ-fault
kampanyası, gerçek ürün göçü, canlı sunucu yükseltmesi veya signed release
kabulü diye birleştirilmez. Ürün staging/login, ikinci faktör, canlı oran
limiti, migration/yedek/rollback ve COMMIT uzlaştırma politikası ürün sahibinde
açık kalır.

K-242'nin şema 2 kampanyası helper için bu eksik native fault kanıtını
hedefler. Aday veya kirli-ağaç koşusu final temiz exact kabul değildir;
exact kaynak/ikili kimliği, 11/11 sonuç ve kaynak başlangıç/bitiş eşliği
birlikte kaydedilmeden bu yeni kapı geçti gösterilmez. TLS fault interception,
postcommit extension deneyi, live login ve uzak CI/RC sınırları açık kalır.
