# ADR-095 — PostgreSQL göç durum ve kuru çalışma

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-228, ürünleşme B4

## Karar

Mevcut `dil göçür` bütün SQL dosyalarını tek transaction/advisory lock ile
yürütür. `--durum` bütün adımları, `--kuru` yalnız bekleyen adımları
version/ad/SHA-256/durum olarak gösterir. SQL metni ve connection credential
çıktıya girmez. Kipler tekil/ayrı, proje yolu en çok birdir.

Plan kanonik dosya ve uygulanmış kayıt checksum/ad/kayıp dosya kontrollerini
kullanır; REPEATABLE READ READ ONLY transaction'da metadata görüntüsü alır.
Tablo yoksa uygulanmış küme boştur; metadata tablo oluşturulmaz, migration
DDL/DML'i veya apply lock'u yürütülmez. Normal apply'nin transaction/lock
ve typed decoder davranışı korunur; yeni plan safe decoder kullanır.
Kuru plan ilerideki apply başarısı veya provider rotasyonu garantisi değildir.

Owner credential ile apply, read-only runtime rolüyle status ayrı sınanır.
Runtime rolünün DDL/DML yetkisi açılmaz; yeni secret/rol config alanı yoktur.
Ürün Python göç aracı/canlı rol geçişi gerçek ürün kabulü sonrası ayrı iştir.

## Kanıt ve etki

İki saf plan testi checksum/ad/kayıp dosya/sıra; dedicated gerçek PG testi
no-DDL/no-DML, rol ret, apply/skip, eşzamanlı owner ve rollback'ı kapsar.
CLI testleri kip/yol/P015 ret sırasını sınar. Default faz native vakayı
ignored gösterir; explicit koşucu 1 passed/0 ignored ister.

Spec/25, RFC-0026/ADR-060 ve kaynak/uyumluluk/faz/kanıt kayıtları aynı
değişikliktedir. Owner/edge/SCC, supported API/ABI, dependency, cache veya
uzun ömürlü tampon yoktur. Transient plan Vec'i CLI kapsamındadır.
ADR-046 ve ADR-045/049/069 fixture/tarihçesi korunur; kalıcı benchmark/soak
tabanı üretilmez. Yeni kalıp/koşul/komut yok; iki host seçeneği profil kaydı
taşır ve K-174 fingerprint değişmez. Yeni tanı kodu yok; C026 korunur.
