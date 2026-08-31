# Türkçe Programlama Dili — çalışma adı: *zee* (belirlenecek)

> "Türkçe düşün. Türkçe yaz. Makine kesin olarak anlasın."

İlkokuldaki bir çocuğun başlayabileceği, profesyonelin bırakmak zorunda kalmayacağı,
Türkçenin doğal akışına göre tasarlanmış **deterministik** genel amaçlı programlama dili.
Bu bir çeviri katmanı değildir (`if→eğer` makyajı yok); AI semantiğin parçası değildir.

- Geçici dosya uzantısı: **`.dil`**
- Master plan: [docs/master-plan.md](docs/master-plan.md) (kaynak: [docs/Turkce_Programlama_Dili_Master_Proje_Dokumani.docx](docs/Turkce_Programlama_Dili_Master_Proje_Dokumani.docx))
- İlkeler: [MANIFESTO.md](MANIFESTO.md)

## Durum: Faz 0 — Felsefe ve grammar discovery

Grammar masa başında tek seferde dondurulmaz. Önce **30 golden program** yazılır;
sözdizimi bu gerçek kullanım örneklerinden çıkarılır. Her syntax değişikliği bu
korpus üzerinde regression testine girer.

| Ne | Nerede | Durum |
|---|---|---|
| Manifesto ve değişmez ilkeler | [MANIFESTO.md](MANIFESTO.md) | ✅ ilk sürüm |
| 30 golden program | [golden/](golden/) | ✅ ilk taslak — hepsi **geçici** sözdizimi |
| 10 anti-örnek | [anti-ornekler/](anti-ornekler/) | ✅ ilk sürüm |
| Syntax karar günlüğü | [kararlar/gunluk.md](kararlar/gunluk.md) | ✅ işleniyor |
| RFC süreci | [rfcs/](rfcs/) | 📋 şablon + planlanan liste |
| ADR süreci | [adr/](adr/) | 📋 şablon + planlanan liste |
| Spesifikasyon | [spec/](spec/) | ⏳ korpus olgunlaşınca |

### Golden korpus hakkında

- Programlar kolaydan zora doğru numaralıdır (`01`–`30`) ve her biri başındaki
  yorumda **neyi sınadığını** ve varsa **açık kararları** belirtir.
- Buradaki her sözdizimi **önerdir, söz değildir.** Kesinleşme RFC ile olur;
  gerekçeler [kararlar/gunluk.md](kararlar/gunluk.md)'de tutulur.
- Master plandaki kategorilerden PostgreSQL/SQLite, basit 2D oyun ve paket
  oluşturma bilinçli olarak korpus dışıdır: bunlar stdlib/CLI API tasarımı
  gerektirir ve korpusun sonraki genişletmesinde eklenecektir.

## Sonraki adımlar (90 günlük plan)

- **Hafta 1–2 (bu faz):** korpus revizyonu — golden programları gerçek kullanıcılara
  okutup karar günlüğünü güncelle; anti-örnekleri netleştir.
- **Hafta 3–4:** Rust bootstrap: lexer + Unicode kuralları (NFC) + girinti
  tokenları + parser iskeleti.
- **Hafta 5–8:** AST, isim çözümleme, temel türler, koşul/döngü/işlem, interpreter.
- **Hafta 9–12:** Türkçe diagnostic framework, formatter, CLI, VS Code minimum, v0.1 demo.

## İlk gerçek milestone

Tek dosya: `merhaba.dil` — tek satır:

```
"Dünyaya merhaba" yaz
```

Bu dosya kendi lexer/parser/type checker zincirimizden geçip çalıştığında proje doğmuş sayılır.
