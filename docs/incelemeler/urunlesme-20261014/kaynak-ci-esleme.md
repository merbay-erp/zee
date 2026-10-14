# K-224 — kaynak/CI eşlemesini yeniden denetleme

`scripts/kaynak-ci-esleme.py` yalnız Python standart kitaplığı ve yerel Git
gerektirir. Checkout'u değiştirmez, ağa bağlanmaz ve CI artefaktı indirmez.
Exact kaynak commit'inin Git arşivi ile `zee/` ZIP kaynaklarını, tek izinli
ek dosya `KAYNAK-SHA256.txt` içindeki her özeti, ZIP yorumunu ve teslim ZIP
SHA-256 değerini eşler. Fazladan kaynak/metadata, yinelenen yol, geçiş yolu ve
değişmiş hash reddedilir.

İstanbul, 14 Ekim 2026 snapshot'ının denetimi:

```bash
python3 scripts/kaynak-ci-esleme.py \
  --sha be5699c3913fe66594fb9d9509253fd9a0b7f5b9 \
  --zip /teslim/zee-snapshot-2026-10-14-be5699c.zip \
  --zip-sha256 1555749c7783e7b34a72b61b85985e83d1770494d0769a45ef4b768750664ede \
  --ci-kaydi docs/incelemeler/urunlesme-20261014/source-ci.json \
  --rapor compiler/target/kaynak-ci-esleme.json
```

`/teslim/` örnek yoldur; `--zip` kendi doğrulanmış ZIP yoluna
bağlanır. Beklenen ZIP SHA-256 doğrulanan teslim/sidecar kaydından alınır.
Rapor yalnız Git dışı hedefe veya depo dışına yazılır; kaynak/CI girdisinin
üzerine yazılması reddedilir. Bir ret, önceki başarılı raporu `rejected`
sonucuyla değiştirir ve exit 2 döndürür; başarı exit 0'dır.

CI kaydı açık `--ci-kaydi` argümanıdır. Araç, bu dizindeki
`zee-source-ci-evidence-1` gözlem kaydını veya `zee-ci-kaydi-1` şemalı
`records` dizisini okur. Her koşu `provider`, `run_index`, `source_sha`,
`status` ve `evidence_level` alanlarını taşır. Kabul için istenen kaynak
SHA'sında `success` durum kaydı gerekir. #29 compiler tree eşliği #30 exact
HEAD durum kaydı yerine geçmez.

Bu denetim kayıt dosyasının hash'ini ve içindeki exact commit durumunu
eşler; uzak durumun güncel olduğunu veya kaydı imzalayan bir hizmetin
kimliğini yeniden doğrulamaz. #30 görünür commit status başarılıdır; adım
günlükleri 404 ile erişilememiştir. Okunmamış günlük/artefakt, platform
runtime, imzalı release veya canlı ürün kabulü ileri sürülmez.

Olumlu ve değiştirilmiş SHA/hash/comment/korpus-yolu, eski CI, bekleyen CI ve
girdi üstüne yazma olumsuz kontrolleri:

```bash
python3 scripts/tests/kaynak-ci-esleme-test.py
```
