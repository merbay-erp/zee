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
| PostgreSQL (TLS/göç) | depo sabitli imaj `postgres@sha256:0ea6700a…7319c4` (16.15, linux/arm64), rastgele host portu |
| PostgreSQL (owned) | resmî `postgresql-16.15.tar.bz2` (SHA-256 `c1575341fa7bd40f5274ea465b34390f4dc64cdd0770af327005caaeb9f6b7ed` birebir doğrulandı) özel boş dizinde `--with-openssl --without-icu --without-readline` ile derlendi, OpenSSL 3.6.4; `postgres --version` = `postgres (PostgreSQL) 16.15`; küresel kurulum yok |
| Envanter | güncel kaynakta 27 `#[ignore]` vaka (K-306'daki 25 + K-313'ün iki tenant vakası) |

## Sonuç: 23 geçti, 0 ürün arızası, 4 açık

`0367cd52` üzerinde 22 vaka geçti; `starter_content` bu kaynakta bayat test
kahini yüzünden düştü ve yalnız koşucu düzeltmesini taşıyan `ddd79f0e`
kaynağında geçti (aşağıda).

| Vaka grubu | Komut | Sonuç |
|---|---|---|
| `postgresql_testi` 10 izole TLS vakası (TLS1.2 olumlu, yanlış CA/SAN, NULL/metin, kaynak sınırları, 40P01, 40001, TLS1.1 reddi, iki tenant temizliği) | `python3 scripts/postgresql-tls-ci.py --engine docker` | **10/10 geçti**, 0 ignored; 30,8 sn. Bağımsız TLS1.1 olumlu kontrolü: istemci ve sunucu `TLSv1.1`, `ECDHE-RSA-AES256-SHA`, CA+hostname doğrulandı; ardından native ret. Rapor SHA-256 `6e806989…d842e` |
| `postgresql_goc_plani_testi` 1 vaka | `python3 scripts/postgresql-goc-ci.py --engine docker` | **1/1 geçti**; 13,7 sn. Rapor SHA-256 `e66f7b5a…46c4` |
| `lsp_artimli_testi::yirmi_bin_satir_proje_duzenleme_olcumu` | `cargo test --locked --release --test lsp_artimli_testi yirmi_bin -- --ignored --nocapture` | **Geçti.** 20.081 satır/40 modül; tek açık belge p95 19,5 ms, 41 açık belge p95 122,9 ms (gözlemsel, bütçe 200 ms) |
| `service_fault_testi::service_fault_native` | `ZEE_NATIVE_WAVE_SHA=<kaynak>` + debug `dil` SHA-256 ile `--ignored --exact` | **Geçti**; iki gerçek native worker: SIGKILL sonrası kardeş temizliği/yabancı süreç sağ kalır, bayat kapı reddi. Kanıt SHA-256 `ebe50fcc…bf19` |
| `mixed_revision_testi::mixed_revision_native` | aynı ortam | **Geçti**; dört karışık kod dalgası, ortak iptal/kota, yanlış revizyon/aktivasyon reddi, geçersiz kaynak aktif revizyonu değiştirmez, tek nesil deadline. Kanıt SHA-256 `993cf398…be32` |
| Owned PG16.15: `native_metrik_host` (alt süreçte `havuz.rs::izole_pg_pool_metrikleri_wait_timeout_ve_donus_gercek_sinirlar` dahil) | `ZEE_NATIVE_METRIK_PG_BIN` + taze çıktı, `--ignored --exact` | **2/2 geçti** (havuz vakası 1 passed, gözlem dizisi kayıtlı); 27 sn. Kanıt `6117b2d8…af7b7` |
| `owned_web_kit_dort_native_http_pg_izolasyon_vakasi` | `ZEE_OWNED_WEB_KIT_PG_BIN` | **Geçti**; 14 sn |
| `web_test_koprusu_unit_native_http_pg_ve_kaynak_kapsami_birlikte` | `ZEE_WEB_TEST_PG_BIN` + operatör Python 3.13.3 | **Geçti**; 6 sn |
| `native_effect_onbir_sql_smtp_no_replay_vakasi` | `ZEE_EFFECT_PG_BIN` + taze çıktı | **Geçti**; 9 sn. Kanıt `34efb033…a9551` |
| `modul_sql_native_crud_goc_yasam_dongusu` | `ZEE_MODULE_POSTGRES_BIN_DIR` + taze çıktı | **Geçti**; 6 sn. Kanıt `096a29f0…542e` |
| `dispatcher_owned_pg_heartbeat_commit_ve_grup_sahipligi` | `ZEE_JOB_DISPATCHER_PG_BIN` + taze çıktı | **Geçti**; 52 sn. Kanıt `7270ab6c…eb29` |
| `native_pg_tekrarlanan_hazirlik_ve_sigkill_ttl` | `ZEE_K274_PG_BIN` + temiz debug `dil` | **Geçti**; 38 sn |
| `icerik_baslangici_native_crud_medya_yonetim` (starter) | `ZEE_STARTER_POSTGRES_BIN_DIR` + taze çıktı | `0367cd52`: debug derlemede yanlış parola POST'u koşucunun 2 sn okuma süresini aştı (K-308 kabulü de release ile alınmıştı); release'de bütün HTTP/SQL/medya adımları geçti, son satırda **bayat kahin** düştü: harness kapanışta `.zee/yuklemeler` içinde artık **kalmasını** bekliyordu, oysa K-332/GB-129 tüketilen ya da reddedilen yüklemenin `.parca` dosyasını silmeyi ZORUNLU yaptı. Ürün doğru davranıyor; kahin `.parca` kalmamasını isteyecek şekilde düzeltildi (yalnız `scripts/tests/starter-content-native.py`, compiler değişmez). **`ddd79f0e7ecab29314bebe91218be92e54d382f2` temiz klonunda release ile geçti**; 6 sn. Kanıt `bdb2ed25…542e` |
| `dispatcher_service_native_linux`, `http_service_native_linux` | — | **Açık, koşmadı.** Gerçek Linux ve temiz systemd kullanıcı yöneticisi ister; macOS ve systemd'siz konteyner bu kapıyı sağlamaz. Son geçiş K-306'nın ayrı kaynağındadır ve buraya aktarılmaz. |
| `pdf_host_testi`, `pdf_plugin_testi` | — | **Açık, koşmadı.** Operatörün kurduğu exact ReportLab 4.4.9 / pypdf 6.10.0 / Pillow 12.3.0 arka ucu gerekir; makinedeki Python'da 4.4.10 / 6.16.1 var, pin uymaz. |

İlk `service_fault`/`mixed_revision` denemesi çıktı dizini göreli verildiği
için koşucunun mutlak yol ön koşulunda durdu; vaka koşmadan reddedildi ve
mutlak yolla yeniden koşuldu. Bu, kaynak ya da test arızası değildir.

## Sınır

- Bu kayıt P36'yı kapatmaz: 4 vaka hâlâ koşmamıştır; Linux servis ikilisi
  Linux VM, PDF ikilisi pinli arka uç kurulumu ister.
- Starter sonucu `0367cd52` sonucuna toplanmaz; `ddd79f0e` yalnız bu koşucu
  satırıyla ayrılır, diğer 22 vaka o kaynakta yeniden koşulmadı.
- Starter native koşucusu debug derlemede zaman aşımına düşebilir; release
  derleme kullanılır.
- Normal (yoksayılmayan) faz matrisi bu kaydın kapsamı dışındadır; CI'da ayrı
  koşar.
- Docker PG imajı TLS/göç koşucularının resmî motorlarından biridir; CI aynı
  vakaları Linux'ta `--engine local` ile koşar.
