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
- V1 öncesi sıralı mühendislik backlog'u: [docs/oncelikli-backlog.md](docs/oncelikli-backlog.md)
- İlkeler: [MANIFESTO.md](MANIFESTO.md)

## Durum: **v0.7.0** (1 Eylül 2026) — Faz 2+ sürüyor

Sürüm geçmişi ve ayrıntılar: [docs/surumler.md](docs/surumler.md).

**31 Ağustos 2026 — proje doğdu.** `merhaba.dil` kendi lexer → parser → tür
denetimi → yorumlayıcı zincirimizden geçip çalıştı (master plan bölüm 41'deki
ilk milestone). Bootstrap derleyici: [compiler/](compiler/) (Rust; denetlenmiş,
kilitli bağımlılıklar).

```bash
cd compiler && cargo build && ./target/debug/dil çalıştır ..
```

Kök klasörün `proje.dil` bildirimi vardır: `dil çalıştır <klasör>`,
`dil denetle <klasör>` ve `dil dene <klasör>` giriş dosyasını buradan bulur.
Bildirim de sıradan zee sözdizimidir (K-076); ayrı bir yapılandırma dili yoktur.
Projeler `yerel_bağımlılıklar` ile başka zee projelerini doğrudan paket olarak
alabilir; `dil kilitle .` bütün geçişli grafiği göreli yol, sürüm ve SHA-256
içerik özetiyle deterministik `proje.kilit` dosyasına sabitler (K-078).

Çalışan golden programlar (regression testte): **33'te 33 — KORPUSUN TAMAMI**
(orijinal 30 + 31 birimler, 32 ondalık market, 33 açık işlem imzası) —
**v0.1 kabul listesindeki 4 program da çalışıyor.** Desteklenen yüzey: `olsun`,
`yaz` (ekrana ve `dosyasına`), `ile`, `ise/değilse` zinciri, dört döngü,
`artır/azalt`, `diye sor`/`yanıt`, rastgele sayı, genitif aritmetik, listeler
(örtük çoğulla `her ... için`; K-093 değer-sonuç imleci ve T053 güvenli kaynak
sabitliği), sözlükler (sıra korumalı), metin işlemleri
(Türkçe İ/ı kurallarıyla `büyük/küçük harflisi`), `Seçenek` (`var/yok`),
`Sonuç` (`dosyasını okumayı dene`, `başarılıysa`) ve **yapılandırılmış Hata**
(K-091: kod, mesaj, neden zinciri, veri ve geriye uyumlu gösterim), dosya satırları/yazma,
**işlem tanımı ve çağrısı** (K-016 geçici sözdizimi), **yapılar** (`yapı`,
`yeni`, iyelik ekiyle alan erişimi), **desen eşleştirme** (`göre / ise`),
CSV/JSON okuma, tarih/saat, komut satırı argümanları, **test blokları**
(`dil dene`), **keyfî hassasiyetli Ondalık** (3,14 — tam onluk aritmetik:
0,1+0,2=0,3; yalnız sonsuz bölüm 34 anlamlı hane),
**Süre** (yarım saniye), **birimler** (`X birimini kullan`), **Sonuç dönüşü**
(`hatasını döndür`), `ve/veya/değilse` mantığı, **HTTP istemcisi** ve
**deneysel web sunucusu** (localhost TCP; testlerde sahte), **yapılandırılmış
eşzamanlılık** (K-090: deterministik tek-thread scheduler, gerçek
`hepsini bekle`, T033/T051 sahiplik ve kardeş iptali), **işbirlikli son tarih
iptali** (`... içinde/yetişmezse`, görev ağacına yayılır),
**ESP32
simülatörü**, **özyineleme** (T035 "temel durum önce", C019 derinlik sınırı),
**blok kapsamı** (K-034), **akış-duyarlı daraltma** (T036: korumasız
değer/hata erişimi derleme hatası), metin kaçışları ve negatif sabitler,
**uygulama eylemleri ve yöntemli web adaptörü** (`eylem`, GET/HEAD salt-okuma
kanıtı, POST/PUT/PATCH/DELETE, 404/405/413/504, iç içe dosya savepoint'i),
**güvenli web profili** (Argon2id, 256 bit CSPRNG sunucu oturumu/rol,
rotation/revoke/ömür, otomatik CSRF, `__Host-` çerez, HTTPS proxy Origin
kapısı), form/istek sözlüğü, yönlendirme, html güvenlisi ve
önekli rotalar, **metin cerrahisi** (parçala/birleştir/
değiştir/kırp/harfler), **JSON/CSV yazma**, **Türk alfabesiyle sıralama**,
**silme** ve **çıkış kodu** (`programı 2 ile bitir`), bölümden **kalan**,
geri sayan aralık, para biçimi `kuruşlusu`, evrensel `metni`, çerez
üçlemesi (oku/yaz/**sil**), **sürümlü `zee-tr-1` morfolojisi** (tek kaynak ek
tablosu, zamir n'si, ikizleşme, iki katmanlı çözüm↔üretim), **açık işlem
parametre türleri**
(`sayıyı Ondalık olarak al`) ve public **dönüş sözleşmesi**
(`Ondalık döndürür` / `değer döndürmez`), **proje bildirimi** (`proje.dil`, klasörden
çalıştır/denetle/dene), **yerel paketler** (`X paketini kullan`), süreçler
arası kilitli **atomik dosya yazma** ve 144 etkin Türkçe kodlu tanı
(1 tarihsel kod ayrılmıştır).
Araçlar: `dil çalıştır(--güvenli/--deneysel-web/--web-proxy)/parola-özeti/denetle(--json)/dene/biçimle/ekle/çıkar/kilitle/paketler/hata/belge/morfoloji/yeni`
+ **dillsp** LSP sunucusu — tanılar, hover, tanıma git, **morfolojili
yeniden adlandırma (F2)** ([editors/](editors/)).
Determinizm testlerde tam: rastgelelik, saat, dosyalar, HTTP, sunucu istekleri,
sensörler ve an ölçümü IO soyutlamasından gelir — 398 test hermetik koşar.

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
| 33 golden program | [golden/](golden/) | ✅ tamamı regression testte; sözdizimi RFC'lerle geçici kabulde |
| 11 anti-örnek | [anti-ornekler/](anti-ornekler/) | ✅ (A11: nokta-ondalık) |
| Syntax karar günlüğü | [kararlar/gunluk.md](kararlar/gunluk.md) | ✅ işleniyor |
| RFC süreci | [rfcs/](rfcs/) | ✅ 21 RFC: 2 kabul, 17 geçici kabul, 2 taslak |
| ADR süreci | [adr/](adr/) | ✅ 11 kabul (001-003, 006-013); 004/005 faz verisi bekliyor |
| Hata kataloğu | [docs/hata-katalogu.md](docs/hata-katalogu.md) | ✅ 144 etkin kod + 1 ayrılmış kod, kaynakla tutarlılığı testli |
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

- **Usability oturumları (kurucu):** K-016 çağrı sözdizimi için serbest üretim,
  kör A/B/C kartları ve karar eşikleri önden bağlandı; gerçek 10 çocuk + 5
  profesyonel verisi bekleniyor. Uygulama:
  [oturum kiti](docs/usability-kiti.md) · bağlayıcı
  [karar paketi](docs/k016-cagri-karar-paketi.md).
- **Lisans (bölüm 26, kurucu):** seçilmeden depo herkese açılmaz; site ve
  topluluk (bölüm 30) bunun arkasında.
- **Makine tarafı:** K-081–K-093 ile v1'in altı P0 kapısı ve beş P1
  kapısı kapandı; gezme kapısının makine yarısı da tamamlandı:
  normatif otorite, public işlem sözleşmesi, atomik kalıcılık, gerçek son
  tarih iptali, uygulama eylemi ve production oturum/CSRF/proxy profili.
  `zee-tr-1` morfolojisi profil/snapshot/property korpusuyla sabitlendi;
  deterministik scheduler, görev sahipliği ve hata/iptal yayılımı gerçeklendi.
  Sonuç'un hata tarafı kod/mesaj/neden/veri taşıyan, eski çıktıyı koruyan
  Hata değeridir. Ondalık keyfî hassasiyetlidir; gezme derin değer kopyası,
  değer-sonuç imleci ve T053 kaynak sabitliğiyle tanımlıdır. Gezme usability
  kapısı gerçek insan formlarını bekler. K-094 paket yayın çekirdeği aynı
  kaynaktan deterministik `.zep`, Ed25519 imzalı yayın, SPDX 3.0.1 SBOM ve
  SLSA v1 provenance üretir. K-095 ağ dışı sabit root, eşik ve çift eşikli
  rotasyon, timestamp/snapshot/targets, rollback/expiry/mix-and-match, yanlış
  yayıncı, yanked ve kritik duyuru metadata doğrulamasını kurar; taşıma,
  kalıcı cache/offline ve CLI tamamlanana kadar kapı açık kalır.
  K-096, çalışan A çağrı yüzeyini nihai seçim saymadan tek-genel-sözdizimi
  kapısını ve anonim sonuç arşivini hazırladı. K-097 ifade parser'ını primary
  → postfix → çağrı → aritmetik → birleştirme → karşılaştırma → boolean
  katmanlarına ve tam tüketim kuralına bağladı. K-098/ADR-011 HTTP, sensör,
  CSRF ve parola alan varyantlarını core AST'den çıkarıp tür/yetkinlik/etkisi
  merkezi kayıtlı tek intrinsic düğümüne indirdi; ayrıntılı genişletme
  protokolü [buradadır](docs/intrinsic-yetkinlik-modeli.md). K-099/ADR-012
  parser, checker ve runtime'ın cümle/ifade/çağrı handler'larını fiziksel faz
  modüllerine ayırdı; [mimari bütçe](docs/derleyici-faz-sinirlari.md) yeniden
  tek dosyada büyümeyi testte durdurur. K-100/ADR-013 checker kökünü yalnız
  orkestrasyona indirip tür, bağlam, sembol, akış, çağrı, sözleşme,
  etki/yetkinlik ve dönüş kurallarını tek sahipli
  [semantik katmanlara](docs/checker-katmanlari.md) ayırdı. İnsan kanıtı
  beklenirken uygulama sırası [öncelikli backlog](docs/oncelikli-backlog.md)
  ile sabittir.
  Bağlayıcı liste: [docs/v1-surum-kapilari.md](docs/v1-surum-kapilari.md).

## İlk gerçek milestone

Tek dosya: `merhaba.dil` — tek satır:

```
"Dünyaya merhaba" yaz
```

Bu dosya kendi lexer/parser/type checker zincirimizden geçip çalıştığında proje doğmuş sayılır.
