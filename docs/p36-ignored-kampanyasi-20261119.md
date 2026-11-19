# P36 — Güncel master'da yoksayılan Cargo vakaları (19 Kasım 2026)

Bu kayıt [K-298 kabul tablosundaki](urunlesme-k298-kabul-20261016.md) P36
borcunu günceller: K-306 kampanyasında 25 `#[ignore]` vakanın 23'ü kendi
kaynağında koşmamıştı. Burada **tek exact kaynakta** hangi vakaların gerçekten
koştuğu ve hangilerinin neden açık kaldığı yazılır. Farklı SHA sonuçları
toplanmaz; önceki kampanyaların geçen vakaları bu kayda aktarılmaz.

## Kaynak ve ortam

| Alan | Değer |
|---|---|
| Kaynak | `0367cd52741eb80ff83301596005306359fa8d8e` (master, PR #24 birleşmesi) |
| Ağaç | `90aace92ad587470a19fa17916a512fe37751d01` |
| Checkout | ayrı tam klon, `git status --porcelain` boş; koşucular `source_clean=true` ve `sürüm --json` `dirty=false` doğruladı |
| Makine | macOS 26.6.2 (25G83), Apple M4 Pro, 24 GiB |
| Araç zinciri | rustc 1.93.1 (`rust-toolchain.toml`), Python 3.13.3, Docker 27.4.0 |
| PostgreSQL | depo sabitli imaj `postgres@sha256:0ea6700a…7319c4` (16.15, linux/arm64), rastgele host portu |
| Envanter | güncel kaynakta 27 `#[ignore]` vaka (K-306'daki 25 + K-313'ün iki tenant vakası) |

## Sonuç: 14 geçti, 0 başarısız, 13 açık

| Vaka grubu | Komut | Sonuç |
|---|---|---|
| `postgresql_testi` 10 izole TLS vakası (TLS1.2 olumlu, yanlış CA/SAN, NULL/metin, kaynak sınırları, 40P01, 40001, TLS1.1 reddi, iki tenant temizliği) | `python3 scripts/postgresql-tls-ci.py --engine docker` | **10/10 geçti**, 0 ignored; 30,8 sn. Bağımsız TLS1.1 olumlu kontrolü: istemci ve sunucu `TLSv1.1`, `ECDHE-RSA-AES256-SHA`, CA+hostname doğrulandı; ardından native ret. Rapor SHA-256 `6e806989…d842e` |
| `postgresql_goc_plani_testi` 1 vaka | `python3 scripts/postgresql-goc-ci.py --engine docker` | **1/1 geçti**; 13,7 sn. Rapor SHA-256 `e66f7b5a…46c4` |
| `lsp_artimli_testi::yirmi_bin_satir_proje_duzenleme_olcumu` | `cargo test --locked --release --test lsp_artimli_testi yirmi_bin -- --ignored --nocapture` | **Geçti.** 20.081 satır/40 modül; tek açık belge p95 19,5 ms, 41 açık belge p95 122,9 ms (gözlemsel, bütçe 200 ms) |
| `service_fault_testi::service_fault_native` | `ZEE_NATIVE_WAVE_SHA=<kaynak>` + debug `dil` SHA-256 ile `--ignored --exact` | **Geçti**; iki gerçek native worker: SIGKILL sonrası kardeş temizliği/yabancı süreç sağ kalır, bayat kapı reddi. Kanıt SHA-256 `ebe50fcc…bf19` |
| `mixed_revision_testi::mixed_revision_native` | aynı ortam | **Geçti**; dört karışık kod dalgası, ortak iptal/kota, yanlış revizyon/aktivasyon reddi, geçersiz kaynak aktif revizyonu değiştirmez, tek nesil deadline. Kanıt SHA-256 `993cf398…be32` |
| Owned PG16.15 sahipli 9 vaka: `havuz.rs::izole_pg_pool_metrikleri…`, `web_test_koprusu`, `owned_web_kit`, `native_metrik_host`, `native_effect_consumer`, `modul_sql`, `job_dispatcher`, `starter_content`, `sureli_hazirlik` | — | **Açık, koşmadı.** Koşucular kaynaktan derlenmiş, `postgres --version` çıktısı tam `postgres (PostgreSQL) 16.15` olan ikili ister. Bu makinedeki Homebrew 16.11, sabitli Debian imajı `16.15 (Debian 16.15-1.pgdg12+2)` son eki taşır; fixture bu ikisini haklı olarak reddeder. Resmî 16.15 kaynak arşivi (SHA-256 `c1575341…b7ed`) bu oturumda indirilip derlenmedi. |
| `dispatcher_service_native_linux`, `http_service_native_linux` | — | **Açık, koşmadı.** Gerçek Linux ve temiz systemd kullanıcı yöneticisi ister; macOS ve systemd'siz konteyner bu kapıyı sağlamaz. Son geçiş K-306'nın ayrı kaynağındadır ve buraya aktarılmaz. |
| `pdf_host_testi`, `pdf_plugin_testi` | — | **Açık, koşmadı.** Operatörün kurduğu exact ReportLab 4.4.9 / pypdf 6.10.0 / Pillow 12.3.0 arka ucu gerekir; makinedeki Python'da 4.4.10 / 6.16.1 var, pin uymaz. |

İlk `service_fault`/`mixed_revision` denemesi çıktı dizini göreli verildiği
için koşucunun mutlak yol ön koşulunda durdu; vaka koşmadan reddedildi ve
mutlak yolla yeniden koşuldu. Bu, kaynak ya da test arızası değildir.

## Sınır

- Bu kayıt P36'yı kapatmaz: 13 vaka bu exact kaynakta hâlâ koşmamıştır.
- Owned PG16.15 dokuzlusu tek ek girdiyle (resmî kaynak arşivinden özel dizine
  derleme) aynı Mac'te koşulabilir; Linux servis ikilisi Linux VM, PDF ikilisi
  pinli arka uç kurulumu ister.
- Normal (yoksayılmayan) faz matrisi bu kaydın kapsamı dışındadır; CI'da ayrı
  koşar.
- Docker PG imajı TLS/göç koşucularının resmî motorlarından biridir; CI aynı
  vakaları Linux'ta `--engine local` ile koşar.
