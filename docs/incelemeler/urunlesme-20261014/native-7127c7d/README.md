# Temiz exact checkpoint native kabulü

Kaynak: `7127c7d1f6da19d532946ded55fae8bddfeb0d24`.
Bağımsız temiz klonda native Rust derlemesiyle, kendi boş loopback PostgreSQL
16.15 Docker kümelerinde 14 Ekim 2026'da koşuldu. Kümeler koşu sonunda kaldırıldı.

- [TLS/NULL 4/4](postgresql-tls-ci.json): CA/SAN ret, TLS 1.2 olumlu, NULL/non-text ve transaction sağlık.
- [Göç planı 1/1](postgresql-goc-ci.json): no-DDL/no-DML, owner/runtime rolü, checksum/kayıp dosya, rollback/concurrent apply.
- [Parola yükseltme 7/7](parola-goc-ci.json): üç bcrypt prefix, ikinci giriş, yanlış parola, gerçek CAS yarışı, UPDATE/bind rollback, commit görünürlüğü.

Hiçbiri atlanmadı. Kaynak/build/fixture/sunucu kayıtları JSON'lardadır.
Bu exact nokta sonraki K-235 kaynak-limit testini içermez; K-222'nin dört
vakası tarihsel olarak korunur. Yeni uzak CI, canlı ürün, provider TLS1.1,
C027 ağ-fault, RC veya imzalı dağıtım kabulü değildir.
