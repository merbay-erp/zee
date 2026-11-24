# urun-h — e-posta sunucusu dogfood ürünü (K-355)

Ürün H, protokol motorları dahil bütünüyle zee ile yazılması hedeflenen bir
e-posta sunucusudur. Gerçek ürün özel depodadır; bu klasör
[RFC-0039](../../rfcs/0039-ag-sunucusu-yetkinligi.md) ağ sunucusu
yetkinliğinin depo içi kanıt aynasıdır. Köken: [KOKEN.md](KOKEN.md).

| Sürtünme | Kanıt | RFC-0039 katmanı |
|---|---|---|
| TCP dinleyicisi yok, tek bağlantı yüzeyi HTTP rotası (S036; K-381 ile çözüldü) | `korpus/f009-dinleyici-yok.dil` | A |
| Metin kaçışında `\r` yok, çıplak CR denetimi yazılamaz (S040) | `korpus/f004-cr-kacisi-yok.dil` | A (çalışma zamanı değişmezi) |
| Yerleşik büyük harf Türkçe yerel ayarlı (`MAİL`), ASCII katlama yok | `korpus/f001-yerel-ayarli-buyuk-harf.dil`, çözüm `korpus/f001-ascii-katlama-elle.dil` | D adayı |
| Adlı systemd soketi ve `Type=notify` yok; on soket sırasız geliyor (K-376, çözüldü: A1) | `dinleyici-devri/` | A1 (uygulandı) |
| Zee kodu adlı dinleyiciden bağlantı kabul edemiyor, satır okuma istem basıyor (F009/F010; K-381, çözüldü: A2a) | `baglanti-yankisi/`, `korpus/f009-dinleyici-yok.dil` | A2a (uygulandı) |
| Web arayüzü ek indiremiyor, JS'siz dosya yükleyemiyor, PDF/görsel dışı ek gönderemiyor (ZM-F022; K-378, çözüldü: RFC-0041 A, B, C) | `ek-akisi/` | RFC-0041 A, B, C (uygulandı) |
| Webmail e-posta istemcisi autoconfig XML'ini ve Apple yapılandırma profilini kayıtlı türle sunamıyor (ZM-F033; K-385, çözüldü: ADR-163 eki) | `istemci-yapilandirma/` | RFC-0041 A (tür kaydı satırı) |
| Türkçe-yoğun uzun ileti urlencoded formda 64 KiB'ı aşıp rota çalışmadan 413 alıyor (ZM-F037; K-391, çözüldü: ADR-168 Dilim 1); tek SQL parametresi 64 KiB (ZM-F036; Dilim 2, açık) | `ileti-boyutu/` | RFC-0041 B (multipart metin alanı) |

Sürtünmelerin K-164 kataloğuyla karşılaştırması `kararlar/gunluk.md` K-355
kaydındadır. Korpus vakaları `dogfood/korpus-v1.tsv` içinde exact
beklentiyle kayıtlıdır.
