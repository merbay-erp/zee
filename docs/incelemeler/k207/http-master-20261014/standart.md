# Zee performans gözlemi

- Şema: `zee-performans-2`
- Kayıt: `K221-on-inceleme-20261014`
- Git SHA: `c7433123ac1755bb81c476e2698ffee661500a69`
- Milestone: `K221-on-inceleme`
- Git çalışma ağacı: **temiz**
- Platform: `macos-aarch64`
- OS: `macOS 26.6.2 (25G83)`
- CPU: `Apple M4 Pro`
- RAM: **25769803776 bayt**
- Rust: `rustc 1.93.1 (01f6ddf75 2026-02-11)`
- Profil: **release**, ölçüm turu: **3**, ısınma: **2**
- Exact ortam karşılaştırma tabanı: `K-207-m4pro-scope-after` / `9e80b692f9a0a37cdd8136783b42f2a0056f2478` / `K-207-scope`; Git temiz; OS `macOS 26.6.2 (25G83)`; CPU `Apple M4 Pro`; RAM `25769803776` bayt; Rust `rustc 1.93.1 (01f6ddf75 2026-02-11)`; profil `release`; 25 örnek + 2 ısınma; `bagimsiz_tur`
- Politika: yalnız gözlem; shared CI hard gate değildir

| Ölçüm | p50 | p95 | En az | En çok | Önceki p50 | Önceki p95 | Eğilim |
|---|---:|---:|---:|---:|---:|---:|---:|
| `parse_gecikmesi` — 2000 satırda lexer + parser | 1.828 ms | 1.879 ms | 1.668 ms | 1.879 ms | 1.712 ms | 1.864 ms | +0.8% |
| `typecheck_gecikmesi` — hazır AST'de resolver + checker ve HIR kanıt toplama | 3.555 ms | 3.903 ms | 3.287 ms | 3.903 ms | 3.639 ms | 3.875 ms | +0.7% |
| `hir_olusturma` — 2000 satır kaynak -> bağlı typed HIR tam ön uç | 5.140 ms | 5.169 ms | 4.706 ms | 5.169 ms | 5.031 ms | 5.610 ms | -7.9% |
| `runtime_baslangici` — önceden derlenmiş boş typed HIR runtime dispatch | 166 ns | 208 ns | 125 ns | 208 ns | 125 ns | 2.041 µs | -89.8% |
| `yurutme_gecikmesi` — önceden derlenmiş 100 bin turluk sayaç | 22.977 ms | 23.364 ms | 21.969 ms | 23.364 ms | 23.141 ms | 24.212 ms | -3.5% |
| `lsp_engine_initialize` — in-process LSP engine kurma + initialize isteği | 750 ns | 875 ns | 708 ns | 875 ns | 667 ns | 1.167 µs | -25.0% |
| `lsp_process_cold_start` — dillsp process spawn + stdio initialize capabilities yanıtı | 3.056 ms | 3.253 ms | 2.465 ms | 3.253 ms | 2.609 ms | 4.049 ms | -19.7% |
| `lsp_ac` — initialize edilmiş sunucuda 2000 satır didOpen | 4.684 ms | 4.962 ms | 4.680 ms | 4.962 ms | 5.020 ms | 5.116 ms | -3.0% |
| `lsp_degistir_2k` — açık 2000 satır belgede tam metin didChange | 4.884 ms | 4.937 ms | 4.562 ms | 4.937 ms | 5.024 ms | 5.207 ms | -5.2% |
| `lsp_degistir_5k` — açık 5000 satır belgede tam metin didChange | 16.873 ms | 17.403 ms | 16.035 ms | 17.403 ms | 17.315 ms | 18.131 ms | -4.0% |
| `lsp_degistir_10k` — açık 10000 satır belgede tam metin didChange | 49.351 ms | 51.791 ms | 49.346 ms | 51.791 ms | 50.578 ms | 53.105 ms | -2.5% |
| `lsp_degistir_20k` — açık 20000 satır belgede tam metin didChange | 161.961 ms | 165.903 ms | 158.573 ms | 165.903 ms | 162.519 ms | 165.137 ms | +0.5% |
| `tepe_bellek` — koşucu süreç tepe resident set'i | 140560 KiB | 140560 KiB | 140560 KiB | 140560 KiB | 462432 KiB | 462432 KiB | -69.6% |

## LSP tam-metin değişiklik ölçeği

Bu gözlem bir optimizasyon ya da hard CI kapısı değildir. Eşikler ölçümden önce 250/500/1000 ms olarak sabitlenmiştir ve karar p95 üzerinden verilir.

| Satır | p50 | p95 |
|---:|---:|---:|
| 2000 | 4.884 ms | 4.937 ms |
| 5000 | 16.873 ms | 17.403 ms |
| 10000 | 49.351 ms | 51.791 ms |
| 20000 | 161.961 ms | 165.903 ms |

| p95 eşiği | İlk aşım |
|---:|---|
| 250 ms | 20.000 satıra kadar aşılmadı |
| 500 ms | 20.000 satıra kadar aşılmadı |
| 1000 ms | 20.000 satıra kadar aşılmadı |

Shared CI sayıları makine seçimi ve komşu iş yüklerinden etkilenir; doğruluk başarısı veya sürüm engeli değildir. Eşik yalnız aynı exact ortamda sabitlenmiş adanmış benchmark koşucusunda açıkça istenir.
