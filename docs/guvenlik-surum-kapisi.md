# Güvenlik sürüm kapısı rehberi

K-170/ADR-066'nın işletim sözleşmesi. Politika kök
[SECURITY.md](../SECURITY.md), kayıt
[guvenlik-bulgulari-v1.tsv](guvenlik-bulgulari-v1.tsv), betik
`scripts/guvenlik-kapisi.sh`.

## Kayıt satırı

`kimlik	tarih	kaynak	onem	yuzey	ozet	durum	kapanis	karar`

- `kimlik` ardışık `GB-NNN`; `tarih` ISO `YYYY-MM-DD`.
- `kaynak`: `inceleme|fuzz|dogfood|advisory|drift|saha`.
- `onem`: `kritik|yuksek|orta|dusuk` (sözlük SECURITY.md'dedir).
- `durum`: `acik` (hedef K-işi zorunlu; kritik/yüksek olamaz), `kapali`
  (kapanış K-işi günlük/backlog'da gerçek), `kabul` (kapanış `-`, karar
  normatif spec/ADR yolu; kritik kabul edilemez).
- `ozet` en az 30 karakter; `karar` var olan `adr/`, `spec/`, `rfcs/` ya da
  `docs/` Markdown yolu.

## Akış

- Yeni bulgu: `acik` satırı + hedef K-işi; kritik/yüksekse düzeltme aynı
  dalgada zorunludur, aksi halde CI kırmızıdır.
- Düzeltme: regresyon vakası (ADR-044), K-işi ve satırın `kapali` olması aynı
  incelemede.
- Bilinçli sınır: `kabul` + normatif karar yolu; sınır değişirse satır yeni
  bulguya dönüşmez, yeni satır açılır.

## Komutlar

```bash
bash scripts/guvenlik-kapisi.sh --surekli      # tedarik workflow'u her push'ta
bash scripts/guvenlik-kapisi.sh --surum-adayi  # etiket öncesi, temiz ağaçta
```

Sürüm adayı kipi exact HEAD için `docs/fuzz-rc-gecmisi-v1.tsv` içinde dört
hedefin ≥1800 saniyelik `gecti` satırını ister; K-157 kampanyası aynı
commit'te koşulmadan etiket kesilemez. Etiket sonrası sürüm ikilisi yalnız
[tekrar üretilebilir artefakt](tekrar-uretilebilir-surum.md) akışından gelir.
