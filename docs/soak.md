# Uzun soak rehberi

K-173/ADR-069'un işletim sözleşmesi. Araç `compiler/src/bin/soak.rs`,
tarihçe [`soak-gecmisi-v1.tsv`](soak-gecmisi-v1.tsv), workflow
`.github/workflows/soak.yml`.

## Koşu

```bash
cd compiler
cargo build --locked --release --bin dillsp
cargo run --locked --release --bin soak -- --sure-sn 1800 --pencere-sn 60 \
  --dillsp target/release/dillsp --rapor target/soak.md \
  --gecmis ../docs/soak-gecmisi-v1.tsv     # yalnız temiz çalışma ağacında
```

Koşucu süre boyunca golden korpusunu derler, sabit programı yürütür ve gerçek
`dillsp` sürecine içeriği değişen 2.000 satırlık `didChange` gönderir; her 5
saniyede iki sürecin RSS'ini örnekler.

## Karar

Isınma penceresi atılır. Isınma sonrası ilk pencere medyanı ile son pencere
medyanı karşılaştırılır; bir süreç hem %10 hem 32 MiB'ı aşarsa **KALDI**.
Rapor bütün örnekleri tablo olarak taşır; tarihçe satırı exact Git SHA,
platform, rustc, süre, pencere, dönem sayıları, medyanlar ve sonucu taşır.

## Sınır

Kapı sızıntı eğilimini ölçer, mutlak bellek bütçesini değil (spec/24).
Süreler shared CI'da gürültülüdür ve karar girdisi değildir. Windows'ta `ps`
yoktur; koşu Unix Tier-1 platformlarında tanımlıdır.
