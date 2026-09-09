# Kelime avı (üçüncü depo içi ürün)

K-179/ADR-074. `dogfood/kelime-avi` beş harfli gizli kelimeyi altı denemede
bulma oyunudur; iş yükü etkileşimli komut satırı (stdin döngüsü, rastgele
seçim, harf ipucu, skor dosyası). Amaç dili büyütmek değil, K-164 sürtünme
sınıflarının bağımsız bir üründe tekrar edip etmediğini ölçmektir.

```bash
cd compiler
cargo run --locked -- çalıştır ../dogfood/kelime-avi Zeynep
```

Sözlük `kaynak/kelimeler.txt` (satır başına bir kelime, `#` yorum), skorlar
`kaynak/skorlar.txt` (`ad|deneme|kelime`, sürüm kontrolünde tutulmaz).
İpucu: `●` doğru yer, `○` kelimede var, `·` yok. Kısa/uzun tahmin hak yakmaz.

| Birim | Sorumluluk |
|---|---|
| `harf_araclari` | harf boyu, sıradaki harf, ipucu üretimi |
| `kelime_secimi` | sözlük ayıklama (yorum/boş/boy), sıradaki kelime |
| `skor_araclari` | skor satırı, oyuncu başına en küçük deneme özeti |
| `ana.dil` | tek IO noktası: sözlük, argüman/soru, oyun döngüsü, skor |

198 kaynak satırı, 7 birim testi; `kelime_avi_testi` kazanma, kaybetme,
kısa tahmin, önceki skor özeti ve boş sözlük senaryolarını hermetik koşar.

## Sürtünme tekrarı (ADR-074 §Sonuçlar)

Yeni: F001 iyelikli döngü adının örtük çoğulu (A003), F003 `metnin satırları`
yok (T028). Tekrar: K-164/F010 sıra erişimi (üç işlemde sayaç), F009 proje
kipinde `../`, F013 iyelikli `-ndeki`, F011 sekme ayracı. Kılavuzla önlenen:
F001/F006/F007/F008/F014. Compiler değişikliği yok; K-180 tasarım maddesi açıldı.

## Komut

```bash
cd compiler
cargo test --locked --test kelime_avi_testi --test dogfood_korpusu_testi
```
