# Zee performans gözlemi

- Şema: `zee-performans-2`
- Kayıt: `K-255-m4pro-021bff4`
- Git SHA: `021bff47839092706c7698d360ae5face9cd2a19`
- Milestone: `K-255-P19-checkpoint`
- Git çalışma ağacı: **temiz**
- Platform: `macos-aarch64`
- OS: `macOS 26.6.2 (25G83)`
- CPU: `Apple M4 Pro`
- RAM: **25769803776 bayt**
- Rust: `rustc 1.93.1 (01f6ddf75 2026-02-11)`
- Profil: **release**, ölçüm turu: **25**, ısınma: **2**
- Exact ortam karşılaştırma tabanı: `K-207-m4pro-scope-after` / `9e80b692f9a0a37cdd8136783b42f2a0056f2478` / `K-207-scope`; Git temiz; OS `macOS 26.6.2 (25G83)`; CPU `Apple M4 Pro`; RAM `25769803776` bayt; Rust `rustc 1.93.1 (01f6ddf75 2026-02-11)`; profil `release`; 25 örnek + 2 ısınma; `bagimsiz_tur`
- Politika: yalnız gözlem; shared CI hard gate değildir

| Ölçüm | p50 | p95 | En az | En çok | Önceki p50 | Önceki p95 | Eğilim |
|---|---:|---:|---:|---:|---:|---:|---:|
| `parse_gecikmesi` — 2000 satırda lexer + parser | 1.676 ms | 2.286 ms | 1.625 ms | 4.368 ms | 1.712 ms | 1.864 ms | +22.7% |
| `typecheck_gecikmesi` — hazır AST'de resolver + checker ve HIR kanıt toplama | 3.351 ms | 3.515 ms | 3.187 ms | 3.542 ms | 3.639 ms | 3.875 ms | -9.3% |
| `hir_olusturma` — 2000 satır kaynak -> bağlı typed HIR tam ön uç | 4.971 ms | 5.235 ms | 4.703 ms | 8.585 ms | 5.031 ms | 5.610 ms | -6.7% |
| `runtime_baslangici` — önceden derlenmiş boş typed HIR runtime dispatch | 125 ns | 1.417 µs | 83 ns | 1.791 µs | 125 ns | 2.041 µs | -30.6% |
| `yurutme_gecikmesi` — önceden derlenmiş 100 bin turluk sayaç | 22.452 ms | 26.178 ms | 22.229 ms | 27.218 ms | 23.141 ms | 24.212 ms | +8.1% |
| `lsp_engine_initialize` — in-process LSP engine kurma + initialize isteği | 583 ns | 667 ns | 541 ns | 833 ns | 667 ns | 1.167 µs | -42.8% |
| `lsp_process_cold_start` — dillsp process spawn + stdio initialize capabilities yanıtı | 2.712 ms | 3.061 ms | 2.302 ms | 3.129 ms | 2.609 ms | 4.049 ms | -24.4% |
| `lsp_ac` — initialize edilmiş sunucuda 2000 satır didOpen | 5.059 ms | 5.340 ms | 4.743 ms | 9.079 ms | 5.020 ms | 5.116 ms | +4.4% |
| `lsp_degistir_2k` — açık 2000 satır belgede tam metin didChange | 5.037 ms | 7.490 ms | 4.763 ms | 10.701 ms | 5.024 ms | 5.207 ms | +43.8% |
| `lsp_degistir_5k` — açık 5000 satır belgede tam metin didChange | 17.410 ms | 18.227 ms | 16.780 ms | 22.256 ms | 17.315 ms | 18.131 ms | +0.5% |
| `lsp_degistir_10k` — açık 10000 satır belgede tam metin didChange | 50.874 ms | 53.784 ms | 49.973 ms | 60.531 ms | 50.578 ms | 53.105 ms | +1.3% |
| `lsp_degistir_20k` — açık 20000 satır belgede tam metin didChange | 162.821 ms | 177.321 ms | 156.206 ms | 180.611 ms | 162.519 ms | 165.137 ms | +7.4% |
| `tepe_bellek` — koşucu süreç tepe resident set'i | 247136 KiB | 247136 KiB | 247136 KiB | 247136 KiB | 462432 KiB | 462432 KiB | -46.6% |

## LSP tam-metin değişiklik ölçeği

Bu gözlem bir optimizasyon ya da hard CI kapısı değildir. Eşikler ölçümden önce 250/500/1000 ms olarak sabitlenmiştir ve karar p95 üzerinden verilir.

| Satır | p50 | p95 |
|---:|---:|---:|
| 2000 | 5.037 ms | 7.490 ms |
| 5000 | 17.410 ms | 18.227 ms |
| 10000 | 50.874 ms | 53.784 ms |
| 20000 | 162.821 ms | 177.321 ms |

| p95 eşiği | İlk aşım |
|---:|---|
| 250 ms | 20.000 satıra kadar aşılmadı |
| 500 ms | 20.000 satıra kadar aşılmadı |
| 1000 ms | 20.000 satıra kadar aşılmadı |

Shared CI sayıları makine seçimi ve komşu iş yüklerinden etkilenir; doğruluk başarısı veya sürüm engeli değildir. Eşik yalnız aynı exact ortamda sabitlenmiş adanmış benchmark koşucusunda açıkça istenir.
