# Performans ölçüm arşivi

Master plan bölüm 22: performans sürekli ölçülür, sonuçlar sürümler arası
arşivlenir. Koşucu: `cargo run --release --bin olcum` (5 tur, medyan;
`--hizli` CI dumanı). İş yükleri koşucunun içinde sabittir — sürümler arası
karşılaştırma ancak aynı iş yüküyle anlamlıdır; yük değişirse burada not düşülür.

Kural: arşive yalnız **release** ölçümleri girer ve makine bağlamı yazılır.
Regression bütçesi (v0.3 hedefi): bir sürüm, bir önceki arşiv satırına göre
herhangi bir yükte %50'den fazla yavaşlıyorsa sürüm notunda gerekçelenmek
**ZORUNDA**dır.

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
