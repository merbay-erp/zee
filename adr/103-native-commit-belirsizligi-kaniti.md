# ADR-103 — Native COMMIT belirsizliği kanıtı

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-242, GB-065
- **Karar zinciri:** ADR-084/086, spec/11, K-223; P15/P34

## Karar

Mevcut PostgreSQL adaptörünün COMMIT hatasında C027 üretmesi korunur.
ErrorResponse, SQLSTATE, ERROR severity veya ReadyForQuery idle durumu tek
başına transaction'ın geri alındığını kanıtlamaz. PostgreSQL'in durable
commit noktasından sonraki cleanup/callback hataları da istemciye hata
gönderebilir. SQLSTATE sınıfından genel bir C021 allowlist'i türetilmez.

Bu sınır [PostgreSQL 16.15 transaction kaynağındaki](https://github.com/postgres/postgres/blob/REL_16_15/src/backend/access/transam/xact.c)
RecordTransactionCommit sonrası callback/cleanup sırası ve
[driver'ın ErrorResponse işlemesiyle](https://github.com/sfackler/rust-postgres/blob/tokio-postgres-v0.7.18/tokio-postgres/src/client.rs)
incelendi. Buradaki kaynaklar açıklayıcıdır; gerçek kabul aşağıdaki yerel
koşunun byte/hash/exit ve bağımsız veritabanı gözlemleridir.

## Native kabul

Bu kararın temiz exact native kabulü [devam raporunda](../docs/urunlesme-devam-20261014.md)
ayrıca tutulur. Aday çalışma ağacındaki 11/11 sonuç yeni exact kaynak
kapanışı yerine geçmez; bu aşamada temiz commit tekrarı bekleniyor.

K-223'ün gerçek anonim helper'ı ve dış eylemi değiştirilmeden yürütülür.
Mevcut yedi TLS vaka korunur; yalnız testin yönettiği loopback PostgreSQL
profilinde dört ek kontrol/hata vakası çalışır:

- Başarılı COMMIT kontrolü sonraki SMTP yoluna gerçekten ulaşır.
- COMMIT backend'e iletilmeden bağlantı kesilir; C027, otomatik tekrar yok,
  bağımsız gözlemcide legacy özet ve sıfır oturum/audit kalır.
- COMMIT uygulanır, yanıt istemciden saklanır; aynı C027, otomatik tekrar
  yok, bağımsız gözlemcide modern özet ve tek oturum/audit vardır.
- Deferred trigger'ın ERROR/23514 reddi istemciye iletilir; bu fixture'da
  bağımsız gözlemci rollback'i görür. İstemcinin C027 koruması değiştirilmez
  ve bu gözlem bütün provider/extension hataları için genellenmez.

Hata yolları başarı stdout'u ve SMTP devamı üretmez. SQL attempt sayaçları
ve COMMIT gözlemi gerçek çağrı sayısını doğrular; sonuçlar tahmin edilmez.
Proxy yalnız test aracıdır: loopback, sınırlı startup/frame, toplam son
tarih ve owned lifecycle kullanır. `after` başarı sinyali için exact
CommandComplete COMMIT, hata yokluğu ve ReadyForQuery idle birlikte aranır.
ErrorResponse + idle, başarılı COMMIT kanıtı sayılamaz.

## Sınır ve doğrulama

`compiler/tests/parola_goc_kapisi_testi.rs` skor kartı ve fake wire
negatiflerini normal Cargo koşusunda yürütür. Gerçek on bir native vaka
dedicated koşuda ayrı raporlanır; Python vaka sayısı Cargo toplamına eklenmez.
Üretim Rust modülleri, grammar, tanılar, public facade, IO izi veya retry
politikası değişmez. Bu kanıt plaintext test fault profili içindir; mevcut
TLS yedi vaka ayrı kalır. Canlı login, gerçek form, ürün uzlaştırma/yedek,
provider ve fiziksel platform/uzak CI/RC kabulü açık kalır.
