# Spec maddesi drift rehberi

K-171/ADR-065'in işletim sözleşmesi. Güncel görünüm
[spec-drift-raporu.md](spec-drift-raporu.md), makine-okunur kaynak
[spec-madde-kaniti-v1.tsv](spec-madde-kaniti-v1.tsv), araç
`compiler/src/bin/spec_drift.rs`.

## Madde nedir

`spec/NN-*.md` içinde kod bloğu ve tablo satırı dışında kalan ve büyük harfli
`ZORUNLU*`, `ZORUNDA*`, `YASAK*`, `TANIMLI*` ya da `AÇIK*` taşıyan paragraf,
liste öğesi (devam satırları dahil) veya başlık. Kimlik
`spec/<dosya>#<16 hex>`; parmak izi dosya adı + boşlukları sadeleştirilmiş
metinden üretilir. `AÇIK değil` olumsuzlaması normatif sayılır.

## Kayıt satırı

`madde	ozet	durum	kanit	not`

- `ozet`: metnin ilk 72 karakteri (+`…`); araç bayat özeti reddeder.
- `durum`: `kanitli` (yalnız seçici), `kismi` (seçici + ≥20 karakter not),
  `acik` (kanıt `-`, ≥20 karakter not). AÇIK işaretli madde yalnız `acik`.
- `kanit`: `;` ile ayrılmış `compiler/tests/x.rs::islev` ya da
  `compiler/src/.../y.rs::islev`; dosya `#[test]` taşımalı, işlev var olmalı.

## Akış

```bash
cd compiler
cargo run --locked --bin spec_drift -- --taslak     # kayıtsız maddeleri basar
# satırları docs/spec-madde-kaniti-v1.tsv içine kanıtla birlikte ekle
cargo run --locked --bin spec_drift -- --rapor-yaz  # raporu yeniler
cargo run --locked --bin spec_drift -- --denetle    # CI kapısı
```

Spec metni değişince eski kimlik BAYAT, yeni kimlik KAYITSIZ görünür: eski
satır yeni kimliğe taşınır ve kanıt yeniden incelenir. Kısmi maddeler K-166/
K-172 için açık iş listesidir; bir boşluk kapanınca satır `kanitli`ye çekilir.
