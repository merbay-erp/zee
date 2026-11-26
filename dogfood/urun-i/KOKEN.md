# ürün I ürün kökeni — K-407

Harici ürün: ZeeOS (özel depo), zee uygulamalarını A/B kök dilimleriyle
güncelleyen ve sağlık denetimi düşen dilimden önceki dilime geri dönen
işletim imajı. Uygulamalar aynı PostgreSQL veritabanını dilimler arasında
paylaşır.

Exact ürün commit'i: `6e8af89a6c5caa39ec4bd3ab1317036f56997141`
(ürün CI dışa aktarım commit'i, 26 Kasım 2026). Karar belgesi ürünün
`docs/adr/086-app-set-change-and-rollback-data.md` dosyasıdır (git dışı,
SHA-256 `01e40563bb899c83306d17c00f91201a885cb7b4392756c2499452e620ba1692`);
kullanıcı kararı 26 Kasım 2026: "İleri şema toleransı".

| Ürün dosyası (depo-göreli) | Git blob |
|---|---|
| `image/phase2-app-r12/apps/defter/proje.dil` (küme N, defter 0.1.0) | `8d550a19df7f518ceaeb19e0d10f21b3c3604807` |
| `image/phase2-app-r12/apps/defter/ana.dil` | `dc619f5e568773efb96a0225ad593538ac938e34` |
| `image/phase2-app-r12/apps/defter/gocler/0001_notlar.sql` | `b5e4f0d7af4c933142cb7fa6d876bbc6f601a42b` |
| `image/phase2-app-r15/apps/defter/proje.dil` (küme N+1, defter 0.2.0) | `87ae0748827983ae1695f9322854a6980d10244a` |
| `image/phase2-app-r15/apps/defter/ana.dil` | `f02f0f54cd6495824a0864f43d590775a0224491` |
| `image/phase2-app-r15/apps/defter/gocler/0001_notlar.sql` (N ile aynı) | `b5e4f0d7af4c933142cb7fa6d876bbc6f601a42b` |
| `image/phase2-app-r15/apps/defter/gocler/0002_etiket.sql` | `c7c3942439dec60ff1e3cbbcee785784cb7ccf99` |

Ürün göç özetleri: `0001_notlar.sql`
`176f1ff5c8b18649f6f6a102bb8ce381cdef62cb97be3b9cdf878afa711b7fdd`,
`0002_etiket.sql`
`32ab16629528052ba1a45c9b8e8e99a72641c1577e9f6fc17b934078086a4678`
(`ADD COLUMN etiket text`, ayrıca `SECURITY DEFINER` işlev ve `GRANT`).
Aynadaki göçler anonim kesittir (yalnız tablo ve sütun); özetleri ürünle
aynı değildir.

## Yeniden üretim (dil `2ad4f746`, PostgreSQL 17, 26 Kasım 2026)

| Adım | Komut | Sonuç (K-407 öncesi) |
|---|---|---|
| a | boş veritabanı, küme N `dil göçür .` | `1 uygulandı, 0 zaten günceldi`, rc 0 |
| b | küme N+1 `dil göçür .` | `1 uygulandı, 1 zaten günceldi`, rc 0 |
| c | küme N `dil göçür .` | C026 "Veritabanında kayıtlı bir migration dosyası proje ağacında yok", rc 1 |
| d | küme N `dil göçür --durum .` | C026 "Migration planı başarısız: …", rc 1 |

K-407 sonrası c ve d bayraksız aynı kalır; `--ileri-kayitlara-izin` ile rc 0,
SQL uygulanmaz, `0002` ileri kayıt olarak raporlanır. Ayna
`defter-geri-donus/` altındadır; adımlar gerçek PostgreSQL kapısında
(`compiler/tests/postgresql_goc_plani_testi.rs`) koşar.
