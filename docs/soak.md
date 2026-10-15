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


K-207 kapsam dilimi kapalı: temiz exact 9e80b69 release/25 örnek/2 ısınma
20k LSP p95 165,137 ms; aynı exact kaynakta 1800 sn/264.343 döngü soak
geçti. Derleyici RSS medyanı 8.144→8.160 KiB, LSP 19.088→17.568 KiB.
Ham örnek/çevre ve tarihçe [ölçümlerde](olcumler.md). 728/728 faz,
26.624 ad oracle ve 4.096 kapsam işlemi oracle karşılaştırması geçti.
K-207 KISMEN; full didChange ve incremental/cache kapasitesi açıktır.

K-214 geçmiş düzeltmesi: yukarıdaki kaynak referansları anonim dal kimliğine
eşlenmiştir. Ham ölçüm/log/ikili/provenance kayıtları ölçülen özgün SHA
kimliğini korur; bu eşleme yeni ölçüm veya yeni ikili kabulü değildir.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.
