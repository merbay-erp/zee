# K-396 — IMAP `BODY[n]` gövdesinin satırlı base64 akışı (ürün H, RFC-0039 A2d)

Bu anonim kesit etkin `urun-h` ürününe (bkz. [KOKEN](../KOKEN.md)) bağlıdır.
Ürün deposu özeldir; sunucu adı, alan adı, adres ve yerel yol taşınmaz
(ADR-087).

| Exact ürün dosyası | Ürün commit'i | Git blob | SHA-256 |
|---|---|---|---|
| Sürtünme kaydı (`ZM-F040` satırı) | `171f6b2a8dc5fcbb2f42c4f769f94a8ced968eca` | `830420b4a41decec07e2d689cb5efafa57a306fc` | `306f8861f14acacf107a40189ebfd1342ef2de6f383e30a8580b11f562300652` |
| Hermetik yeniden üretim (57 baytlık `base64 okumayı dene` döngüsü) | `171f6b2a8dc5fcbb2f42c4f769f94a8ced968eca` | `ebb9164b7089ab84215c3f009f2ff82dc909dd94` | `703729b91ffc0af2660d3c627d5a4f8b02f1e484370f08eb7c6bf9d5676aa244` |
| IMAP FETCH ekli ileti SQL'i (işaret satırları, `{T}` formülü, kısmi aralık) | `c455cc8d1780ade481adc44752700bf6c9fdef8c` | `9b57ae88e804fba8e44fac4b5375de598bd9bdcc` | `32092b6beae80d186642c77b80c37c3d063b55bb169e92c786b7ebf9bbfb6a72` |

## Sürtünme (ZM-F040)

Ürün ekleri K-394 ile paylaşılan veri kökünde çözülmüş tutar
(`veri:ekler/<kiracı>-<sha256>`). IMAP istemcileri (Outlook, Apple Mail,
Thunderbird) `BODY[n]` gövdesini RFC 2045 base64 satırları (76 karakter +
CRLF) olarak bir literal içinde ister; kısmi getirme `BODY[n]<o.n>`
kodlanmış çıktı üzerindedir. Var olan tek yol 57 baytlık `base64 okumayı
dene` döngüsüydü: 16 MB ek 29,6 sn sürüp C024 ile kesiliyordu; 1 MiB'lık tek
base64 satırı 998 bayt satır sınırını aştığından "tek okumada al" da yoktu.
Ürün bu yüzden ekleri IMAP'te hiç göstermiyordu. K-164 kataloğunda sınıf
"yerleşik yüzey eksik".

## Kesit

[ana.dil](ana.dil) — ürünün yanıt iletme döngüsü: veritabanı FETCH yanıtını
satır satır verir; ekli parça yerine `␞D␞<yol>␞<başlangıç>␞<uzunluk>` işaret
satırı bırakır ve Zee bu aralığı `… dosyasının base64 satırlarının … baytından
… baytını bağlantıya aktarmayı dene` ile akıtır. Literal boyu SQL'de
T = e + 2·⌈e/76⌉ (e = 4·⌈b/3⌉) ile hesaplanır. Değişiklik öncesi kesit S015
verirdi; şimdi `denetle` temizdir. Bayt-kesinlik, aralık kenarları, kaçış,
`veri:` kökü ve kopan karşı taraf `compiler/tests/ag_blok_testi.rs` ve
`compiler/tests/ag_baglantisi_surec_testi.rs` ile sınanır.
