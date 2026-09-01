# zee — Türkçe Programlama Dili

> "Türkçe düşün. Türkçe yaz. Makine kesin olarak anlasın."

*Zeynep Eliz Erbay için. Bu dil, bir babanın kızına bıraktığı mirastır:
kendi dilinde düşünüp kendi dilinde inşa edebilsin diye.*

**Ad resmîdir:** zee (okunuşu "ze") — ADR-009. Uzantı `.dil`, CLI `dil`.

İlkokuldaki bir çocuğun başlayabileceği, profesyonelin bırakmak zorunda kalmayacağı,
Türkçenin doğal akışına göre tasarlanmış **deterministik** genel amaçlı programlama dili.
Bu bir çeviri katmanı değildir (`if→eğer` makyajı yok); AI semantiğin parçası değildir.

- **Hemen başla:** [docs/baslangic.md](docs/baslangic.md) — 5 dakikada kurulum, ilk proje, araç kutusu
- **Dili gez:** [docs/dil-turu.md](docs/dil-turu.md) — bütün yüzey, çalışan örneklerle
- **Oynayarak öğren:** [projeler/](projeler/) — çocuk proje kitaplığı (hepsi regression testte)
- Dosya uzantısı: **`.dil`** (kalıcı — ADR-009)
- Master plan: [docs/master-plan.md](docs/master-plan.md) (kaynak: [docs/Turkce_Programlama_Dili_Master_Proje_Dokumani.docx](docs/Turkce_Programlama_Dili_Master_Proje_Dokumani.docx))
- İlkeler: [MANIFESTO.md](MANIFESTO.md)

## Durum: **v0.7.0** (1 Eylül 2026) — Faz 2+ sürüyor

Sürüm geçmişi ve ayrıntılar: [docs/surumler.md](docs/surumler.md).

**31 Ağustos 2026 — proje doğdu.** `merhaba.dil` kendi lexer → parser → tür
denetimi → yorumlayıcı zincirimizden geçip çalıştı (master plan bölüm 41'deki
ilk milestone). Bootstrap derleyici: [compiler/](compiler/) (Rust, sıfır bağımlılık).

```bash
cd compiler && cargo build && ./target/debug/dil çalıştır ..
```

Kök klasörün `proje.dil` bildirimi vardır: `dil çalıştır <klasör>`,
`dil denetle <klasör>` ve `dil dene <klasör>` giriş dosyasını buradan bulur.
Bildirim de sıradan zee sözdizimidir (K-076); ayrı bir yapılandırma dili yoktur.

Çalışan golden programlar (regression testte): **32'de 32 — KORPUSUN TAMAMI**
(orijinal 30 + genişletme: 31 birimler, 32 ondalık market) —
**v0.1 kabul listesindeki 4 program da çalışıyor.** Desteklenen yüzey: `olsun`,
`yaz` (ekrana ve `dosyasına`), `ile`, `ise/değilse` zinciri, dört döngü,
`artır/azalt`, `diye sor`/`yanıt`, rastgele sayı, genitif aritmetik, listeler
(örtük çoğulla `her ... için`), sözlükler (sıra korumalı), metin işlemleri
(Türkçe İ/ı kurallarıyla `büyük/küçük harflisi`), `Seçenek` (`var/yok`),
`Sonuç` (`dosyasını okumayı dene`, `başarılıysa`), dosya satırları/yazma,
**işlem tanımı ve çağrısı** (K-016 geçici sözdizimi), **yapılar** (`yapı`,
`yeni`, iyelik ekiyle alan erişimi), **desen eşleştirme** (`göre / ise`),
CSV/JSON okuma, tarih/saat, komut satırı argümanları, **test blokları**
(`dil dene`), **Ondalık** (3,14 — onluk tam aritmetik: 0,1+0,2=0,3),
**Süre** (yarım saniye), **birimler** (`X birimini kullan`), **Sonuç dönüşü**
(`hatasını döndür`), `ve/veya/değilse` mantığı, **HTTP istemcisi** ve
**web sunucusu** (gerçek TCP; testlerde sahte), **eşzamanlı bloklar**
(RFC-0011: bekle-öncesi erişim derleme hatası), **zaman aşımı**, **ESP32
simülatörü**, **özyineleme** (T035 "temel durum önce", C019 derinlik sınırı),
**blok kapsamı** (K-034), **akış-duyarlı daraltma** (T036: korumasız
değer/hata erişimi derleme hatası), metin kaçışları ve negatif sabitler,
**web uygulama katmanı** (form/istek sözlüğü, çerezli oturum, yönlendirme,
html güvenlisi, önekli rotalar), **metin cerrahisi** (parçala/birleştir/
değiştir/kırp/harfler), **JSON/CSV yazma**, **Türk alfabesiyle sıralama**,
**silme** ve **çıkış kodu** (`programı 2 ile bitir`), bölümden **kalan**,
geri sayan aralık, para biçimi `kuruşlusu`, evrensel `metni`, çerez
üçlemesi (oku/yaz/**sil**), çok katmanlı morfoloji (zamir n'si, ikizleşme,
iki katmanlı ek zinciri), **proje bildirimi** (`proje.dil`, klasörden
çalıştır/denetle/dene) ve 113 Türkçe kodlu tanı.
Araçlar: `dil çalıştır(--güvenli)/denetle(--json)/dene/biçimle/hata/belge/yeni`
+ **dillsp** LSP sunucusu — tanılar, hover, tanıma git, **morfolojili
yeniden adlandırma (F2)** ([editors/](editors/)).
Determinizm testlerde tam: rastgelelik, saat, dosyalar, HTTP, sunucu istekleri,
sensörler ve an ölçümü IO soyutlamasından gelir — 255 test hermetik koşar.

**Playground:** `playground/olustur.sh` derleyiciyi WebAssembly'e derler ve
tek dosyalık `playground/zee-playground.html` üretir — çift tıkla aç, tarayıcıda
yaz-çalıştır; kurulum ve internet gerekmez. Aynı tohum + aynı girdi = her zaman
aynı çıktı (determinizm tarayıcıda da geçerli, K-039).

**Gömülü kitaplık** (RFC-0014): `matematik` ve `liste_araclari` birimleri
zee'yle yazılıdır ([kitaplik/](kitaplik/)) ve ikiliye gömülüdür — playground
dahil her yerde kurulumsuz çalışır. **Çocuk modu:** `dil çalıştır --güvenli`
(ağ kapalı, dosyalar klasörle sınırlı, K-047). Sınıf için:
[docs/ogretmen-rehberi.md](docs/ogretmen-rehberi.md).

Uzak depo: **github.com/merbay-erp/zee** (özel; lisans seçilmeden — bölüm 26 —
herkese açılmayacak, K-032). Tarih arşivi: ilk README'ler [docs/tarih/](docs/tarih/)
altında dondurulmuştur; `baslangic` ve `dogum` etiketleri GitHub'a da itildi.

### v0.1 kabul kriterleri durumu (master plan bölüm 33)

| Kriter | Durum |
|---|---|
| Merhaba Dünya, hesap makinesi, not ortalaması, sayı tahmini çalışır | ✅ hepsi regression testte |
| Türkçe tanımlayıcılar sorunsuz | ✅ |
| Girinti blokları deterministik | ✅ (sekme/karışık girinti hatası testli) |
| Temel type errors Türkçe ve kaynak konumlu | ✅ (S/A/T/C kodları + öneri) |
| Formatter idempotent | ✅ `dil biçimle` — 30 golden dosyada idempotentlik testli |
| Windows/macOS/Linux interpreter/CLI | ✅ üç platformda CI yeşil (31 Ağu 2026) |
| Golden corpus CI'da | ✅ her push'ta 3 platformda koşuyor |
| Kaynak kodda İngilizce keyword gerekmez | ✅ |

Grammar masa başında tek seferde dondurulmaz. Önce **30 golden program** yazılır;
sözdizimi bu gerçek kullanım örneklerinden çıkarılır. Her syntax değişikliği bu
korpus üzerinde regression testine girer.

| Ne | Nerede | Durum |
|---|---|---|
| Manifesto ve değişmez ilkeler | [MANIFESTO.md](MANIFESTO.md) | ✅ ilk sürüm |
| 32 golden program | [golden/](golden/) | ✅ tamamı regression testte; sözdizimi RFC'lerle geçici kabulde |
| 11 anti-örnek | [anti-ornekler/](anti-ornekler/) | ✅ (A11: nokta-ondalık) |
| Syntax karar günlüğü | [kararlar/gunluk.md](kararlar/gunluk.md) | ✅ işleniyor |
| RFC süreci | [rfcs/](rfcs/) | ✅ 13 RFC: 2 kabul, 10 geçici kabul, 1 taslak (K-043) |
| ADR süreci | [adr/](adr/) | ✅ 6 kabul (001-003, 007-009); 004/005/006 faz verisi bekliyor |
| Hata kataloğu | [docs/hata-katalogu.md](docs/hata-katalogu.md) | ✅ 113 kod, kaynakla tutarlılığı testli |
| Spesifikasyon | [spec/](spec/) | ✅ başladı — normatif çekirdek (RFC'lere bağlar) |

### Golden korpus hakkında

- Programlar kolaydan zora doğru numaralıdır (`01`–`30`) ve her biri başındaki
  yorumda **neyi sınadığını** ve varsa **açık kararları** belirtir.
- Buradaki her sözdizimi **önerdir, söz değildir.** Kesinleşme RFC ile olur;
  gerekçeler [kararlar/gunluk.md](kararlar/gunluk.md)'de tutulur.
- Master plandaki kategorilerden PostgreSQL/SQLite, basit 2D oyun ve paket
  oluşturma bilinçli olarak korpus dışıdır: bunlar stdlib/CLI API tasarımı
  gerektirir ve korpusun sonraki genişletmesinde eklenecektir.

## Sonraki adımlar

90 günlük başlangıç planının makine tarafı 2 günde kapandı (v0.1 → v0.2 →
v0.3 sürüm notlarına bak). Şimdiki kapılar:

- **Usability oturumları (kurucu):** K-016 çağrı sözdiziminin onayı ve
  10 "geçici kabul" RFC'nin tam kabulü — kit hazır: docs/usability-kiti.md.
- **Lisans (bölüm 26, kurucu):** seçilmeden depo herkese açılmaz; site ve
  topluluk (bölüm 30) bunun arkasında.
- **Makine tarafı sıradaki:** K-076 proje modelinin üstüne yerel bağımlılık +
  deterministik kilit dosyası; RFC-0014'ün usability sonrası kesinleşmesi,
  ADR-004 ölçümleri ve Faz kapıları geldikçe native/paralellik.

## İlk gerçek milestone

Tek dosya: `merhaba.dil` — tek satır:

```
"Dünyaya merhaba" yaz
```

Bu dosya kendi lexer/parser/type checker zincirimizden geçip çalıştığında proje doğmuş sayılır.
