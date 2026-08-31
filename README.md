# zee — Türkçe Programlama Dili

> "Türkçe düşün. Türkçe yaz. Makine kesin olarak anlasın."

*Zeynep Eliz Erbay için. Bu dil, bir babanın kızına bıraktığı mirastır:
kendi dilinde düşünüp kendi dilinde inşa edebilsin diye.*

İlkokuldaki bir çocuğun başlayabileceği, profesyonelin bırakmak zorunda kalmayacağı,
Türkçenin doğal akışına göre tasarlanmış **deterministik** genel amaçlı programlama dili.
Bu bir çeviri katmanı değildir (`if→eğer` makyajı yok); AI semantiğin parçası değildir.

- Geçici dosya uzantısı: **`.dil`**
- Master plan: [docs/master-plan.md](docs/master-plan.md) (kaynak: [docs/Turkce_Programlama_Dili_Master_Proje_Dokumani.docx](docs/Turkce_Programlama_Dili_Master_Proje_Dokumani.docx))
- İlkeler: [MANIFESTO.md](MANIFESTO.md)

## Durum: Faz 1 — Interpreter (Faz 0 tamamlandı)

**31 Ağustos 2026 — proje doğdu.** `merhaba.dil` kendi lexer → parser → tür
denetimi → yorumlayıcı zincirimizden geçip çalıştı (master plan bölüm 41'deki
ilk milestone). Bootstrap derleyici: [compiler/](compiler/) (Rust, sıfır bağımlılık).

```bash
cd compiler && cargo build && ./target/debug/dil çalıştır ../merhaba.dil
```

Çalışan golden programlar (regression testte): **01–09, 12, 13, 14** —
**v0.1 kabul listesindeki 4 program da çalışıyor** (Merhaba Dünya, hesap
makinesi, sayı tahmini, not ortalaması). Desteklenen yüzey: `olsun`, `yaz`,
`ile`, `ise/değilse` zinciri, dört döngü (`kez tekrarla`, aralık, `olduğu
sürece`, `olana kadar`), `artır/azalt`, `diye sor`/`yanıt`, rastgele sayı,
genitif aritmetik, listeler (`listesi`, `boş liste`, `ekle`, `adedi/ilki/sonu`,
örtük çoğulla `her ... için`), **işlem tanımı ve çağrısı** (çok kelimeli adlar,
parametreler, `döndür` — K-016 geçici sözdizimi), ek ayıklamalı ad çözümleme
("sayacı" → "sayaç") ve Türkçe kodlu tanılar (S/A/T/C).

Tarih arşivi: ilk README'ler [docs/tarih/](docs/tarih/) altında dondurulmuştur;
`baslangic` ve `dogum` git etiketleri ilk commit'leri kalıcı işaretler.

### v0.1 kabul kriterleri durumu (master plan bölüm 33)

| Kriter | Durum |
|---|---|
| Merhaba Dünya, hesap makinesi, not ortalaması, sayı tahmini çalışır | ✅ hepsi regression testte |
| Türkçe tanımlayıcılar sorunsuz | ✅ |
| Girinti blokları deterministik | ✅ (sekme/karışık girinti hatası testli) |
| Temel type errors Türkçe ve kaynak konumlu | ✅ (S/A/T/C kodları + öneri) |
| Formatter idempotent | ✅ `dil biçimle` — 30 golden dosyada idempotentlik testli |
| Windows/macOS/Linux interpreter/CLI | 🔄 macOS'ta doğrulandı; CI matrisi hazır ([ci.yml](.github/workflows/ci.yml)) |
| Golden corpus CI'da | ✅ cargo test + CI tanımı (uzak repoya itilince aktif) |
| Kaynak kodda İngilizce keyword gerekmez | ✅ |

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
