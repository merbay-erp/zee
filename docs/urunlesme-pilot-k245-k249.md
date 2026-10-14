# K-245–K-249 temiz pilot kabulü

Bu dalga POSIX credential ve worker host araçlarını, dar SQL iş kuyruğunu,
anonim TR–EN görünüm yardımcısını ve LSP UTF-8 dosya yolu düzeltmesini
birlikte doğrular. [39 işlik plan](urunlesme-plan-v1.tsv) bütünüyle kapanmadı;
gerçek ürün, insan, platform ve yayın kapıları ayrı açık sınırlar taşır.

Temiz teknik kaynak `c67a69e6ed9c24c4a540a55eeaaa986a348a2de3`;
ilk LSP düzeltmesi `176a35f09926dafdecfc15e30f1e5776714f3361` ve
`introduced_by=9a1f0b7e683bf778d3226aecbe8bd7b5f273740e` exact kayıtlıdır.
Sonraki private test taşıması semantic/core-freeze maintenance beyanına
bağlıdır. Başlangıç `zee` ve gerçek `zee-cekirdek` checkout'ları aynı
SHA'da temiz tutuldu; çalışma onaylı ayrı dalda commitlendi.

| Dilim | Yerel kabul | Açık sınır |
|---|---|---|
| K-245/P25 | Dosya→çocuk env; nofollow, sahip/mode/identity, FIFO/boyut, ambient collision ret; 14 host senaryosu, gerçek Zee SMTP dahil | Windows/systemd, canlı ürün/provider, file-native noninheritance |
| K-246/P13 | Exact build + yeni app readiness; owned WNOWAIT grup cleanup, TERM/KILL ve ardışık restart; 25 senaryo, gerçek Node/native worker/credential birleşimi | Request drain, request/DB metrics, HUP config reload, P14 socket/overlap |
| K-247/P09 | Idempotent SQL lease/generation, toplam 3 deneme, backoff/dead-letter/fencing; 11 PostgreSQL ve 16 validator senaryosu | SMTP dış etki/teslim tekrarı, scheduler, gerçek ürün Python kaldırılması |
| K-248/P19 | Percent octet'leri UTF-8 baytı olarak çözme; olumlu/olumsuz unit ve gerçek LSP stdio disk birimi regresyonu | Authority/UNC/Windows URI ve incremental cache/200 ms kabulü |
| K-249/P21 | Tek anonim TR–EN fragment; plain Metin slotları/sabit sözlük; Python 3.9.6 ve 3.13.3'te 7 native + 4 Zee testi | Genel layout/i18n/list/detail/router, gerçek ürün ve tam P08 |

## Exact kapılar

| Kapı | Sonuç | Kanıt |
|---|---|---|
| Tam faz | 895/901; 0 başarısız, 6 dedicated atlama; 178.603 sn faz toplamı | [Faz](incelemeler/urunlesme-20261014/devam-c67a69e/faz-test-matrisi.md) |
| Dedicated PostgreSQL | 5 TLS + 1 göç + 11 parola/COMMIT + 11 SQL queue; 28/28, 0 atlama | [Native](incelemeler/urunlesme-20261014/devam-c67a69e/native-proof.json), [parola](incelemeler/urunlesme-20261014/devam-c67a69e/parola-goc-cases.json), [queue](incelemeler/urunlesme-20261014/devam-c67a69e/is-kuyrugu-cases.json) |
| Host | Credential 14/14, supervisor 25/25; 0 başarısız/atlama | [Host](incelemeler/urunlesme-20261014/devam-c67a69e/host-proof.json), [25 vaka](incelemeler/urunlesme-20261014/devam-c67a69e/worker-campaign.json) |
| TR–EN | İki gerçek Python sürümünde 7 native + 4 Zee; Cargo wrapper 1/1 | [TR–EN](incelemeler/urunlesme-20261014/devam-c67a69e/tr-en-proof.json) |
| Sürekli kapılar | fmt/spec203/clippy/API/uyumluluk/katman/SCC/tedarik; iki graph deny/fuzz derleme; 443 commit + kaynak ağacı sır taraması | [Kapı özeti](incelemeler/urunlesme-20261014/devam-c67a69e/gate-summary.json) |
| Kanonik ikililer | İki temiz klonda dil/LSP eş hash; a/b/çıktı exact SHA ve dirty=false; SBOM/provenance | [İki klon](incelemeler/urunlesme-20261014/devam-c67a69e/build-proof.json) |
| Kaynak ZIP | 1.190 Git kaynağı; exact comment, kaynak kümesi, manifest ve byte eşliği | [Arşiv](incelemeler/urunlesme-20261014/devam-c67a69e/source-archive.json) |

Python ve host senaryoları Cargo toplamına ayrıca eklenmez. Normal fazdaki
altı dedicated PostgreSQL vakası gerçek owned runtime koşusunda ayrıca
yürütülür. Her native kabul actual BuildJSON, başlangıç/son kaynak temizliği,
girdi/ikili hash eşliği ve owned cleanup ile kaydedilir. PostgreSQL 16.15
resmî kaynak hash'i ve bağlı OpenSSL/runtime byte'ları ayrıca doğrulandı.
Zee/Rust/Go kanıt özeti aynı sayfayı bayt bayt üretti; resmi karşılaştırma
betiğinin temiz `64f141d` üç tur kaydı [tarihçededir](dogfood-karsilastirma.md).

## Önce/sonra ve kalıcı kayıt

İlk çalışma fazındaki modül bütçesi/bayat özet, readiness testinin eski
olay eşleşmesi, public launcher PID/grup sahipliği ve expired lease-token
belirsizliği ayrı başarısız kanıtlarıyla korundu. Python 3.11-only hash
adaptörü gerçek Python 3.9 hatası üzerine streaming SHA-256 ile düzeltildi;
[bağımsız inceleme](incelemeler/urunlesme-20261014/devam-c67a69e/k249-independent-review.json)
ürün/P08 sınırlarını açık tutar.

İlk temiz faz 894/901, bir başarısız ve altı dedicated atlama kaydetti:
dış CARGO_TARGET_DIR, mevcut Rust eşdeğerini testin beklediği manifest
target yolundan başka yere yazdı. Yalnız dış koşucu ortamı düzeltildi;
kaynak değişmeden kendi ignored clone target'ında tam kapı yeniden alındı.
PostgreSQL ilk toplama doğrulayıcısı parola ve queue raporlarını aynı şema
saydı; actual dört runner zaten başarılıydı. Ayrı schema-aware doğrulama
16 ham dosyayı tekrar hash'ledi; ilk KeyError raporu saklandı, native testler
yeniden etiketlenmedi. Ham günlükler ve byte snapshot'ları kaynak deposu
dışında kalıcı özel kanıt dizininde SHA-256 indekslidir.

## Etki incelemesi ve teslim sınırı

ADR-105–109, güvenlik GB-068–071, kaynak/regresyon beyanı, dogfood/faz,
kanıt haritası, README/sürüm/backlog/master/kapı ve ürünleşme kayıtları birlikte
günceldir. Hata kataloğu, tanı kalitesi, normatif spec/RFC, public API,
production bağımlılık/katman/SCC, iki Cargo lock ve action pinleri incelendi;
bu dalga onların yüzeyini genişletmez. Yeni uzun ömürlü compiler state/cache
veya sabit benchmark iş yükü yoktur; performans v2/soak/RC geçmişine sonuç
icat edilmez. K-248 grammar veya özellik genişletmesi değildir.

Yerel macOS ARM64 ikilileri ignored `dist/urunlesme-20261014/macos-arm64-c67a69e/`
altında kendi kaynak kimliğini korur. Son belge/kanıt commit'inin kaynak ZIP'i
ve geri yüklenip fsck doğrulanan Git bundle'ı aynı teslim manifestinde ayrıca
izlenir; belge commit'i c67 ikili kimliği diye sunulmaz. İkili release profili
yerel tekrar üretimdir; yayıncı imzası veya release etiketi değildir.

Lisans kararı bekler, sınırlı pilot desteği seçilidir. P08 beş tam anonim
site/çekirdek korpusu, K-161/K-162 insan kayıtları, temiz fiziksel
Windows/macOS/minimum OS, GB-046 GitHub destek yanıtı ve exact RC ASan/Miri
kapıları açık kalır. Yeni uzak CI artefaktı indirilmedi; GitHub destek
taslağını kullanıcı gönderir. Canlı ürün değişikliği, push, PR, deployment,
yayın veya genel ürün kabulü yapılmış gösterilmez.
