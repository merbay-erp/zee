# ADR-111 — PostgreSQL statement fault kanıtı

- **Durum:** kabul, dar yerel fault kanıtı
- **Tarih:** 15 Ekim 2026
- **İlgili iş:** K-250 / P33–P34
- **Karar zinciri:** ADR-062, ADR-103, spec/25

## Karar

Mevcut PostgreSQL adaptörünün statement hatası, SQLSTATE korunması ve
otomatik yazı tekrarı yapılmaması gerçek owned PostgreSQL üzerinde sınanır.
Yeni compiler davranışı, retry, C025/C027 tanısı veya public API eklenmez.
Önceki beş TLS/NULL/kaynak sınırı vakası aynı dedicated kapıda korunur.

İki yeni vaka stock PostgreSQL statement-time hatası üretir:

- Deadlock: iki ayrı gerçek TLS session, önce ters satırlara başarılı UPDATE
  ve acknowledged barrier; sonra karşı satıra UPDATE. Bir gerçek 40P01 ve
  bir ilerleyen actor vardır. Her actor açık outer ROLLBACK yapar.
- Serialization: gerçek SERIALIZABLE snapshot okuması, diğer TLS session'ın
  aynı satır için başarılı commit'i, sonra eski snapshot'ta UPDATE. Gerçek
  40001, açık outer ROLLBACK ve server idle kontrolü vardır.

Rollback'ten etkilenmeyen BEFORE STATEMENT sequence'leri gerçek UPDATE
çağrılarını sayar. Deadlock toplam dört, serialization victim toplam iki
çağrıdır; otomatik tekrar gizlenemez. Bağımsız TLS observer kalıcı sonucu
okur: deadlock toplamı sıfır; serialization'da diğer commit bir, victim'ın
önceki marker'ı sıfır. Aynı adaptör yeni ayrı backend'de sağlık okur.

## C027 sınırı

SQLSTATE veya ErrorResponse'tan COMMIT'in sonucu türetilmez. Testin açık
ROLLBACK onayı ve stock fixture gözlemi vardır; arbitrary extension/provider
commit callback hatalarına genellenmez. ADR-103 aynen korunur; gönderilmiş
COMMIT yanıt kaybı ayrı native parola kampanyasının işidir.

## Kanıt ve kapsam

`compiler/tests/postgresql_testi.rs` IO birincil sahibinde iki ignored native
vaka ekler. Dedicated `scripts/postgresql-tls-ci.py` exact yedi test,
7 passed/0 failed/0 ignored ve iki exact fault gözlemi ister. Dokuz Python kabul/ret testi isim/skip/exit/SQLSTATE/sayaç/kalıcı sonuç/health sahte kabulünü reddeder.
Normal Cargo/faz bu yedi dedicated vakayı skipped gösterir; yeni iki test
parser/runtime yürütmez. Aynı test grubundaki eski DSL vakalarının ek kapsamı
bu iki raw-adapter testine faz icat etmek için genişletilmez.

İlk aday `965c3a979bf39ad575898785deaa4496c3384713` tabanlı kirli test ağacı,
Rust1.93.1 ve official-source PG16.15'te 7/7 ve 9/9 geçti. Immutable
snapshot/input/bytehash raw kanıtı ayrı tutulur. Temiz exact integration
sonrası native tekrar, tazelik/phase/count kapıları root dalgasında gereklidir.
Uzak CI, provider rotasyonu, TLS1.1, bütün fault matrisi, minimum OS/PG,
canlı ürün, signed release, ASan/Miri kabulü bu kanıtla kapatılmaz.

Production Rust, bağımlılık graph'ı, toolchain, kaynak limiti, IO izi ve
kalıcı cache değişmez. ADR-045/049 benchmark geçmişi değişmez; yeni hard eşik
veya genel performans sonucu yoktur. Yeni compiler bug olmadığı için
regression/fixed_by/compiler semantic beyanı gerekmez.
