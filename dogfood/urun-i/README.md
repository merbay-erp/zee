# urun-i — ZeeOS dogfood ürünü (K-407)

ZeeOS, zee uygulamalarını A/B kök dilimleriyle güncelleyen işletim imajıdır.
Gerçek ürün özel depodadır; bu klasör depo içi kanıt aynasıdır. Köken:
[KOKEN.md](KOKEN.md). Sunucu adı, adres ve yerel yol taşınmaz (ADR-087).

| Sürtünme | Kanıt | Karar |
|---|---|---|
| Yeni dilimin uyguladığı göç kaydı eski dilimin ağacında yok; geri dönüşte `dil göçür` C026 ile düşüyor, eski dilim açılamıyor (K-407, çözüldü) | `defter-geri-donus/n/`, `defter-geri-donus/n1/` | [ADR-181](../../adr/181-goc-ileri-kayit-toleransi.md) |

K-164 kataloğu (F001–F016) dil/tanı sürtünmeleridir; bu CLI göç işletimi
sürtünmesi onlarla örtüşmez. Korpus vakaları `dogfood/korpus-v1.tsv`
içindedir.
