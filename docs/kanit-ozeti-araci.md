# Kanıt özeti aracı (ikinci gerçek Zee ürünü)

K-164/ADR-072'nin işletim sözleşmesi. Kaynak `dogfood/kanit-ozeti/`, çıktı
[`docs/kanit-ozeti.md`](kanit-ozeti.md), kapı `compiler/tests/kanit_ozeti_testi.rs`.

## Ne yapar

Dokuz kayıt defterini (regresyon, güvenlik bulguları, spec maddeleri,
dogfood ürünleri/korpusu, deprecation, soak tarihçesi, compiler değişiklik ve
core freeze beyanları) okur; faz/tanı/önem/durum sayımlarını Türk alfabesi
sırasıyla Markdown tablolarına döker; güvenlik kapısını (açık kritik/yüksek 0)
ve spec kanıt yüzdesini hesaplar.

## Komutlar

```bash
cd compiler && cargo run --locked -- çalıştır ../dogfood/kanit-ozeti/kaynak/ana.dil
```

```bash
cd compiler && cargo run --locked -- dene ../dogfood/kanit-ozeti
```

```bash
cd compiler && cargo test --locked --test kanit_ozeti_testi --test dogfood_korpusu_testi
```

Üretim dosya kipinde koşar: proje kipinde dosya sınırı ürün köküdür (C012,
K-164/F009) ve depo kökündeki kayıt defterlerine ulaşamaz. Ürün politikası
(`dosya-okuma`, `dosya-yazma`) hermetik testte doğrulanır.

## Yapı

| Birim | İş |
|---|---|
| `tsv_araclari` | başlık satırı (son sekmeli yorum), veri satırı, sekmeyle bölme, başlık→alan eşleme, sütun toplama, çift birleştirme |
| `sayim_araclari` | sayaç sözlüğü, Türk alfabesi sıralı anahtar, tam sayı yüzde |
| `markdown_araclari` | hücre/başlık/sayım tablosu, bölüm başlığı |
| `regresyon_raporu` … `beyan_raporu` | her kayıt defteri için bir bölüm üreticisi (saf işlem + testler) |
| `kaynak/ana.dil` | tek IO noktası: okur, bölümleri birleştirir, sayfayı yazar |

## Ad seçimi kılavuzu (morfoloji `zee-tr-1`)

Ürün yazılırken sabitlenen kurallar (K-164/F006–F008, F013):

- Parametre adı sert ünsüzle bitmesin (`başlığı al` → "başlığ"): `adı al`,
  `yolu al`, `değerleri al`.
- Ünlü düşmesi yapan kök kullanma (`kayıt` → `kaydın` çözülmez): `bilgi`.
- Tekil ve çoğulu birlikte tanımlama (`satır` + `satırlar` → `satırlara` A002).
- `-daki` gezme kaynağı iyeliksiz yalın ad olsun (`tablodaki`, `sıradaki`).
- Postfix ve çağrı sonucu önce ada bağlanır; test satırı düz adla doğrular.

## Sürtünme kaydı

On altı sürtünme günlükte (K-164/F001–F016), on bir ret/çözüm çifti
`dogfood/kanit-ozeti/korpus/` altında korpus kapısındadır. İki compiler
düzeltmesi: elmas birim içe alımı (F002) ve birim kökenli tanı (F012).
