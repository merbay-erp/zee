# Performans ölçüm arşivi

K-148/ADR-045 ile performans tek terminal medyanı olmaktan çıktı; K-152/
ADR-049 her satırı gerçek kaynak commit'ine ve tam koşu ortamına bağladı.
K-153/ADR-050, in-process engine süresiyle gerçek dillsp process cold-start'ını
ayırdı. Release
koşucusu varsayılan 25 turdan ham örnek, min/max ve nearest-rank p50/p95
üretir; makine bağlamını `zee-performans-2` JSON'una, insan raporuna ve
incelenebilir `zee-performans-gecmisi-2` TSV'sine bağlar. İş yükleri koşucu
içinde sabittir; kapsam değişirse yeni sonuç eski sayıyla sessizce eşdeğer
sayılmaz ve bu belgede gerekçelenir.

## Güncel sözleşme

| Kimlik | Gözlenen sınır |
|---|---|
| `parse_gecikmesi` | 2.000 satırın lexer + parser geçişi |
| `typecheck_gecikmesi` | Hazır AST'de resolver/checker ve HIR kanıt toplama |
| `hir_olusturma` | 2.000 satır kaynak→bağlı typed-HIR tam ön ucu |
| `runtime_baslangici` | Önceden derlenmiş boş HIR'ın runtime dispatch'i |
| `yurutme_gecikmesi` | Önceden derlenmiş 100 bin turluk sayaç |
| `lsp_engine_initialize` | Aynı süreçte `Sunucu::yeni` + initialize işleme |
| `lsp_process_cold_start` | Process spawn → stdio framing → tam initialize capabilities yanıtı |
| `lsp_ac` | Initialize edilmiş sunucuda 2.000 satır `didOpen` |
| `lsp_degistir` | Açık belgede tam metin `didChange` |
| `tepe_bellek` | Unix sürecinin tepe resident set'i (KiB) |

`typecheck_gecikmesi`, checker'ın bugün typed-HIR kanıtını da ürettiğini
bilerek adlandırılır; saf tür çıkarım süresi iddiası değildir. Tepe bellek
süreç düzeyi ve kümülatiftir; alt-faz tahsis profili değildir. Zaman ölçümleri
iki ısınmadan sonra birbirinden bağımsız 25 örnektir. `tepe_bellek` ise bu
turların sonunda aynı koşucu sürecinden alınan **tek** `ru_maxrss` anlık
görüntüsüdür: `sample_count=1`, `warmup_count=0`,
`sampling_semantics=surec_tepe_anlik_goruntusu` taşır. p50=p95 olması 25 ayrı
RSS örneği alındığı anlamına gelmez.

## Provenance sözleşmesi

Her tarihçe satırı şu alanları eksiksiz taşır: benzersiz kayıt kimliği, tam 40
haneli `git_sha`, ayrı `milestone`, `git_dirty=false`, platform+OS sürümü,
CPU, fiziksel RAM baytı, Rust sürümü, `build_profile=release`, gerçek
`sample_count`, `warmup_count` ve `sampling_semantics`. Kısa SHA, K-numarasını
SHA yerine kullanma, bilinmeyen/boş ortam, sıfır RAM/örnek, debug profil veya
kirli çalışma ağacı fail-closed reddedilir. `--gecmis-cikti` ayrıca verilen
SHA'nın ölçülen `HEAD` ile aynı olmasını zorunlu tutar. Böylece tarihçedeki her
satır tekrar checkout edilebilir bir kaynak ağacı ve açıklanmış koşuya gider;
kirli yerel duman raporu JSON/Markdown'da açıkça `git_dirty=true` görünür ama
kalıcı tarihçeye giremez.

Yerel hızlı duman:

```bash
cd compiler
cargo build --locked --release --bin dillsp
cargo run --locked --release --bin olcum -- --hizli
```

İncelenecek tam kayıt ve karşılaştırma artefaktı:

```bash
cd compiler
cargo build --locked --release --bin dillsp
cargo run --locked --release --bin olcum -- \
  --tur 25 \
  --gecmis ../docs/performans-gecmisi-v2.tsv \
  --json target/performans.json \
  --rapor target/performans.md \
  --gecmis-cikti target/performans-gecmisi.tsv \
  --kayit K-NNN-makine --git-sha GIT_SHA --milestone K-NNN \
  --dillsp target/release/dillsp
```

Yeni TSV doğrudan izlenen dosyanın üstüne yazılmaz. Ölçüm temiz ve exact
commit checkout'unda çalıştırılır. Makine/araç zinciri,
iş yükü ve dağılım incelendikten sonra yeni kayıt kod ve belgelerle aynı
committe eklenir. Şema; CRLF, eksik provenance, bilinmeyen birim/örnekleme,
sıfır örnek, p50>p95, RSS çoklu-örnek yanılsaması, yinelenen ölçüm ve aynı
kayıt içindeki metadata ayrışmasını reddeder.

Shared CI her Linux koşusunda JSON, Markdown ve birleşik TSV'yi job summary
ile 90 günlük artefakta koyar; **hard performans kapısı değildir**. Sabit
makineli adanmış bir koşucu kurulursa p95 sınırı bilinçli olarak örneğin
`--esik-yuzde 20` ile açılabilir. Aynı platformda karşılaştırılabilir geçmiş
yoksa eşikli koşu başarı sayılmaz.

## K-148 başlangıç tabanı — 2 Eylül 2026 · Apple M4 Pro, macOS arm64, Rust 1.93.1

Kanonik sayılar [makine-okunur tarihçededir](performans-gecmisi-v2.tsv).
Kaynak ağacı `df737f643c4ee9c8525ce7e972660230e75f5f45` (milestone K-148),
24 GiB RAM ve release profildir. Süreler iki ısınma ardından 25 örnektir; RSS
yukarıdaki tek süreç-tepe görüntüsü semantiğini taşır.

| Yüzey | p50 | p95 |
|---|---:|---:|
| lexer + parser | 1,776 ms | 2,152 ms |
| resolver/checker + HIR kanıtı | 157,654 ms | 168,605 ms |
| tam kaynak→typed-HIR | 159,668 ms | 165,212 ms |
| boş runtime başlangıcı | 125 ns | 1,292 µs |
| 100 bin tur yürütme | 22,010 ms | 22,772 ms |
| LSP engine initialize (eski `lsp_soguk`) | 541 ns | 584 ns |
| LSP `didOpen` | 159,480 ms | 165,116 ms |
| LSP `didChange` | 161,143 ms | 172,991 ms |
| süreç tepe RSS | 17.888 KiB | 17.888 KiB |

Bu ilk kayıt regresyon hükmü değil, sonraki exact-ortam gözlemlerinin
tabanıdır. Eşik kararı shared CI'dan değil, sabitlenmiş adanmış runner
dağılımından verilir.

## K-153 gerçek cold-start sınırı

Eski nanosaniye ölçekli kayıt process cold-start değildir; kimliği bu nedenle
`lsp_engine_initialize` olarak düzeltildi. Yeni `lsp_process_cold_start`, saati
gerçek `dillsp` process spawn'ından önce başlatır ve istemci stdout'tan tam
çerçeveli `id=1` capabilities yanıtını okuyunca durdurur. Süreç sonlandırma/
wait ölçüm penceresinin dışındadır. Mevcut LSP initialize sırasında workspace
taramaz; dolayısıyla bugün ayrı workspace-load metriği yoktur. Bu davranış
eklendiğinde process cold-start'a gizlenmeden ayrı ölçülecektir.

Gerçek yol ve Tier-1 entegrasyon testi hazırdır. İlk 25 örneklik exact temiz
commit tabanı, K-153 uygulama commit'i checkout edilerek sonraki provenance
commit'inde bu bölüme eklenecektir; o ana kadar K-153 **kısmen açık** tutulur.

## K-148 öncesi elle tutulmuş legacy kayıtlar

Aşağıdaki tablolar eski altı iş yükünün yalnız medyanını taşır. İş yükleri ve
istatistik şeması K-148 ile değiştiği için güncel yüzeylerle doğrudan trend
hesabına katılmaz; tarihsel mühendislik notu olarak korunur.

## v0.8.0 birikimi / K-092 — 1 Eylül 2026 · Apple M4 Pro, macOS 26.5, Rust 1.93.1

| Yük | Medyan | v0.6.0'a göre |
|---|---|---|
| derleme | 14,6 ms | +%8 |
| özyineleme | 39,8 ms | **+%121** |
| döngü | 12,3 ms | **+%116** |
| liste | 1,8 ms | **+%125** |
| ondalık | 6,8 ms | **+%467** |
| metin | 0,8 ms | **+%60** |

Okuma: K-087–K-092 birikimi yorumlayıcıya async/structured scheduler,
yapılandırılmış Hata ve keyfî hassasiyetli Ondalık çekirdeklerini ekledi.
Ondalık artışı K-092'nin bilinçli exact `BigInt` kapasite maliyetidir: 20 bin
işlem hâlâ 6,8 ms'de biter ve para/ölçüm doğruluğu için kabul edilmiştir.
TamSayı sıcak yolu K-092 içinde BigInt'ten ayrıldı; tabloda bu düzeltme
sonrasındaki değer vardır. Diğer >%50 sapmalar kapasite gerekçesiyle kalıcı
kabul edilmiş sayılmaz; v0.8 optimizasyon işinin ölçülü başlangıç noktasıdır.

## v0.6.0 — 1 Eylül 2026 · Apple M4 Pro, macOS 26.5, Rust 1.93.1

| Yük | Medyan | v0.4.0'a göre |
|---|---|---|
| derleme | 13,5 ms | — |
| özyineleme | 18,0 ms | −%5 |
| döngü | 5,7 ms | **−%14 (izleme kapandı)** |
| liste | 0,8 ms | — |
| ondalık | 1,2 ms | — |
| metin | 0,5 ms | — |

Okuma: v0.4.0'da izlemeye alınan döngü sapması kayboldu — v0.2 tabanının
bile altında. K-050..K-070 dalgaları çekirdek maliyeti büyütmedi.

## v0.4.0 — 1 Eylül 2026 · Apple M4 Pro, macOS 26.5, Rust 1.93.1

| Yük | Medyan | v0.2.0'a göre |
|---|---|---|
| derleme | 13,6 ms | +%10 |
| özyineleme | 19,0 ms | +%5 |
| döngü | 6,6 ms | +%12 |
| liste | 0,9 ms | — |
| ondalık | 1,2 ms | — |
| metin | 0,5 ms | — |

Okuma: iki sürümde eklenen büyük yüzey (web katmanı, metin dalgası,
sıralama, yapı listeleri) çekirdek maliyetleri bütçe (%50) içinde tuttu;
döngüdeki +%12, cümle eşlemesine eklenen yeni kollardan — izlenecek.

## v0.2.0 — 31 Ağustos 2026 · Apple M4 Pro, macOS 26.5, Rust 1.93.1

| Yük | Medyan | Açıklama |
|---|---|---|
| derleme | 12,4 ms | 2000 satırlık programın derlenmesi (yalnız denetim) |
| özyineleme | 18,1 ms | fibonacci(22) — ~28 bin çağrı |
| döngü | 5,9 ms | 100 bin turluk sayaç döngüsü |
| liste | 0,9 ms | 5 bin öğe ekle + gezerek topla |
| ondalık | 1,2 ms | 20 bin onluk toplama (0,1 adımlı) |
| metin | 0,5 ms | 2 bin birleştirme |

Okuma: "küçük projede anlık typecheck" hedefi (bölüm 22) bu ölçekte
sağlanıyor (2000 satır ≈ 12 ms). Ağaç-yürüyen yorumlayıcının (ADR-003)
çağrı maliyeti görünür durumda (~0,6 µs/çağrı) — native backend (ADR-005,
Faz 4) kararına veri birikiyor.
