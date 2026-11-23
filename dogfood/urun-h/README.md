# urun-h — e-posta sunucusu dogfood ürünü (K-355)

Ürün H, protokol motorları dahil bütünüyle zee ile yazılması hedeflenen bir
e-posta sunucusudur. Gerçek ürün özel depodadır; bu klasör
[RFC-0039](../../rfcs/0039-ag-sunucusu-yetkinligi.md) ağ sunucusu
yetkinliğinin depo içi kanıt aynasıdır. Köken: [KOKEN.md](KOKEN.md).

| Sürtünme | Kanıt | RFC-0039 katmanı |
|---|---|---|
| TCP dinleyicisi yok, tek bağlantı yüzeyi HTTP rotası (S036) | `korpus/f009-dinleyici-yok.dil` | A |
| Metin kaçışında `\r` yok, çıplak CR denetimi yazılamaz (S040) | `korpus/f004-cr-kacisi-yok.dil` | A (çalışma zamanı değişmezi) |
| Yerleşik büyük harf Türkçe yerel ayarlı (`MAİL`), ASCII katlama yok | `korpus/f001-yerel-ayarli-buyuk-harf.dil`, çözüm `korpus/f001-ascii-katlama-elle.dil` | D adayı |
| Adlı systemd soketi ve `Type=notify` yok; on soket sırasız geliyor (K-376, çözüldü: A1) | `dinleyici-devri/` | A1 (uygulandı) |
| Web arayüzü ek indiremiyor (ZM-F022; K-378, çözüldü: RFC-0041 A) | `ek-akisi/` | RFC-0041 A (uygulandı) |

Sürtünmelerin K-164 kataloğuyla karşılaştırması `kararlar/gunluk.md` K-355
kaydındadır. Korpus vakaları `dogfood/korpus-v1.tsv` içinde exact
beklentiyle kayıtlıdır.
