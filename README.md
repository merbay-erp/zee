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
- **Web'i üretime hazırla:** [docs/web-production-profili.md](docs/web-production-profili.md) — güvenilir proxy, ortak durum ve N worker süreci
- **Derleyici tedarik zinciri:** [docs/tedarik-zinciri.md](docs/tedarik-zinciri.md) — RustSec/lisans/kaynak ve gerçek offline vendor kapısı
- **Playground host sözleşmesi:** [docs/wasm-c-abi.md](docs/wasm-c-abi.md) — sürümlü, kayıtlı ve fuzz kanıtlı WASM C ABI
- **RC fuzz güveni:** [docs/fuzz-rc.md](docs/fuzz-rc.md) — dört hedefte uzun AddressSanitizer kampanyası ve seçili Miri kapısı
- **Rust gömme API'si:** [docs/public-api-politikasi.md](docs/public-api-politikasi.md) — desteklenen `dil::api::v1` facade ve SemVer sınırı
- **Derleyici büyüme eğilimi:** [docs/islev-egilimi.md](docs/islev-egilimi.md) — kritik işlevlerde incelenmiş Clippy tabanı ve CI gözden geçirme payı
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
Projeler `yerel_bağımlılıklar` ile başka zee projelerini, `uzak_bağımlılıklar`
ile exact `ad@X.Y.Z` registry paketlerini alabilir. Uzak kaynak HTTPS origin ve
ağ dışı root sürüm+SHA-256 kimliğine sabitlenir. `dil kilitle .` bütün geçişli
grafiği deterministik `proje.kilit` v3 dosyasına bağlar (K-078/K-136); normal
derleme sessiz ağ açmadan doğrulanmış proje-local cache'den çalışır.

Çalışan golden programlar (regression testte): **NUMARALI KORPUSUN TAMAMI**
(orijinal seri + birimler, ondalık market ve açık işlem imzası) —
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
0,1+0,2=0,3; yalnız sonsuz bölüm 34 anlamlı hane; binary float'a örtük FFI
köprüsü yoktur),
**Süre** (yarım saniye), **birimler** (`X birimini kullan`), **Sonuç dönüşü**
(`hatasını döndür`), `ve/veya/değilse` mantığı, **HTTP istemcisi** ve
**deneysel web sunucusu** (localhost TCP; testlerde sahte), **yapılandırılmış
eşzamanlılık** (K-090: deterministik tek-thread scheduler, gerçek
`hepsini bekle`, T033/T051 sahiplik ve kardeş iptali; K-124
[`zee-esz-1` gözlenebilir conformance profili](docs/eszamanlilik-conformance.md)),
  **işbirlikli son tarih
iptali** (`... içinde/yetişmezse`, görev ağacına yayılır; K-133 dış etkileri
tam çağrı öncesi yeniden denetler),
**ESP32
simülatörü**, **özyineleme** (T035 "temel durum önce", C019 derinlik sınırı),
**blok kapsamı** (K-034), **akış-duyarlı daraltma** (T036: korumasız
değer/hata erişimi derleme hatası), metin kaçışları ve negatif sabitler,
**uygulama eylemleri ve yöntemli web adaptörü** (`eylem`, GET/HEAD salt-okuma
kanıtı, POST/PUT/PATCH/DELETE, 404/405/413/504, iç içe dosya savepoint'i),
**merkezî dış dünya yetkinlikleri** (K-127: proje/paket→compile→runtime,
dosya kökü, public HTTPS, exact origin, DNS sonrası özel-IP/geçiş öneki SSRF
ve kapalı redirect),
**güvenli web profili** (Argon2id, 256 bit CSPRNG sunucu oturumu/rol,
rotation/revoke/ömür, otomatik CSRF, `__Host-` çerez, HTTPS proxy Origin
kapısı; K-137 kalıcı ortak oturum/rate-limit deposu, kanonik `Forwarded`
istemci kimliği ve N ayrı tek-worker süreç modeli; K-139 ortak kanonik
DNS/IPv6/port origin tipi ve loopback bind+peer değişmezi; K-140 byte tabanlı
CRLF/target/TE-CL/UTF-8 HTTP framing kapısı; K-176 strict `%xx` ve exact UTF-8
form çözümü), form/istek sözlüğü,
yönlendirme, html güvenlisi ve
önekli rotalar, **metin cerrahisi** (parçala/birleştir/
değiştir/kırp/harfler), **JSON/CSV yazma**, **Türk alfabesiyle sıralama**,
**silme** ve **çıkış kodu** (`programı 2 ile bitir`), bölümden **kalan**,
geri sayan aralık, para biçimi `kuruşlusu`, evrensel `metni`, çerez
üçlemesi (oku/yaz/**sil**), **sürümlü `zee-tr-1` morfolojisi** (tek kaynak ek
tablosu, zamir n'si, ikizleşme, iki katmanlı çözüm↔üretim ve
[sürekli property/fuzz kanıtı](docs/morfoloji-dogrulama.md),
[immutable profil kaydı](docs/morfoloji-profil-uyumlulugu.md) ve
[bağımsız conformance korpusu](docs/morfoloji-conformance.md)), **açık işlem
parametre türleri**
(`sayıyı Ondalık olarak al`) ve public **dönüş sözleşmesi**
(`Ondalık döndürür` / `değer döndürmez`), **proje bildirimi** (`proje.dil`,
`yetkinlikler` + `ağ_hedefleri`, klasörden
çalıştır/denetle/dene), **yerel ve exact registry paketleri**
(`X paketini kullan`; K-136), süreçler
arası kilitli ve owner/group/ACL/xattr güvenli **atomik dosya yazma** (K-128,
ADR-032) ve sürüm kimlikli Türkçe tanı kataloğu.
Araçlar: `dil çalıştır(--güvenli/--deneysel-web/--web-proxy)/iz(kaydet/oynat)/parola-özeti/denetle(--json)/dene/biçimle/ekle/çıkar/kilitle/paketler/anahtar(üret)/paketle/hata/belge/morfoloji/yeni`
+ **dillsp** LSP sunucusu — tanılar, hover, HIR/semantic kimlikli tanıma git,
**kapsam güvenli morfolojili yeniden adlandırma (F2)** ([editors/](editors/));
K-138/ADR-035 ile RFC 8259 sayı/duplicate alan denetimli, kayıpsız kimlikli
[protokol-kesin JSON-RPC sınırı](docs/lsp-json-rpc-profili.md).
Determinizm testlerde tam: rastgelelik, saat, dosyalar, HTTP, sunucu istekleri,
sensörler ve an ölçümü IO soyutlamasından gelir; bütün testler hermetik koşar.
Lexer/parser panic-free ve morfoloji üret→çöz sözlerini ayrıca kalıcı saldırı
korpusları, deterministik üretim ve gecelik [libFuzzer hattı](docs/fuzzing.md)
denetler. K-156 her hedefin coverage öğrenimini cache'ten bağımsız, SHA-256 ve
run/commit provenance'lı 90 günlük artefakta taşır; repoya giriş küçültme,
stable replay ve review ister. Parser sonrası AST ile checker sonrası typed HIR arasındaki iç
sözleşme de debug/test hattında yürütülebilir
[invariant kapısıyla](docs/ast-hir-invariantleri.md) doğrulanır.

**Playground:** `playground/olustur.sh` derleyiciyi WebAssembly'e derler ve
tek dosyalık `playground/zee-playground.html` üretir — çift tıkla aç, tarayıcıda
yaz-çalıştır; kurulum ve internet gerekmez. Host sınırı K-142/ADR-039 ile exact
pointer/uzunluk, strict UTF-8 ve açık tampon ömrü; K-143/ADR-040 ile 8 MiB
kaynak ve 1 MiB/4.096 satır soru ön-tahsis bütçesi taşıyan
[WASM C ABI v3](docs/wasm-c-abi.md)'tür. Aynı tohum + aynı girdi = her zaman
aynı çıktı (`zee-io-1` [deterministik IO profili](docs/deterministik-io-profili.md),
K-039/K-116).

**Gömülü kitaplık** (RFC-0014): `matematik` ve `liste_araclari` birimleri
zee'yle yazılıdır ([kitaplik/](kitaplik/)) ve ikiliye gömülüdür — playground
dahil her yerde kurulumsuz çalışır. **Çocuk modu:** `dil çalıştır --güvenli`
(ağ kapalı, dosyalar klasörle sınırlı, K-047). Sınıf için:
[docs/ogretmen-rehberi.md](docs/ogretmen-rehberi.md).

Uzak depo: **github.com/merbay-erp/zee** (özel; lisans seçilmeden — bölüm 26 —
herkese açılmayacak, K-032). Tarih arşivi: ilk README'ler [docs/tarih/](docs/tarih/)
altında dondurulmuştur; `baslangic` ve `dogum` etiketleri GitHub'a da itildi.
Paylaşılabilir kaynak paketi çalışma klasörünün ZIP'i değildir; yalnız izlenen
`HEAD` içeriğini alan [temiz kaynak arşivi](docs/temiz-kaynak-arsivi.md)
komutuyla üretilir.

### v0.1 kabul kriterleri durumu (master plan bölüm 33)

| Kriter | Durum |
|---|---|
| Merhaba Dünya, hesap makinesi, not ortalaması, sayı tahmini çalışır | ✅ hepsi regression testte |
| Türkçe tanımlayıcılar sorunsuz | ✅ |
| Girinti blokları deterministik | ✅ (sekme/karışık girinti hatası testli) |
| Temel type errors Türkçe ve kaynak konumlu | ✅ (S/A/T/C kodları + öneri) |
| Formatter idempotent | ✅ `dil biçimle` — idempotentlik + 33 golden üzerinde tam parser-token eşdeğerliği testli |
| Windows/macOS/Linux interpreter/CLI | ✅ üç platformda CI yeşil (31 Ağu 2026) |
| Golden corpus CI'da | ✅ her push'ta 3 platformda koşuyor |
| Kaynak kodda İngilizce keyword gerekmez | ✅ |

Grammar masa başında tek seferde dondurulmaz. Önce **30 golden program** yazılır;
sözdizimi bu gerçek kullanım örneklerinden çıkarılır. Her syntax değişikliği bu
korpus üzerinde regression testine girer.

| Ne | Nerede | Durum |
|---|---|---|
| Manifesto ve değişmez ilkeler | [MANIFESTO.md](MANIFESTO.md) | ✅ ilk sürüm |
| Golden programlar | [golden/](golden/) | ✅ tamamı regression testte; sözdizimi RFC'lerle geçici kabulde |
| Anti-örnekler | [anti-ornekler/](anti-ornekler/) | ✅ geçersiz yüzeyler kalıcı korpusta |
| Syntax karar günlüğü | [kararlar/gunluk.md](kararlar/gunluk.md) | ✅ işleniyor |
| RFC süreci | [rfcs/](rfcs/) | ✅ durum dağılımı aşağıdaki canlı tabloda |
| ADR süreci | [adr/](adr/) | ✅ kabul sayısı aşağıdaki canlı tabloda; 004/005 faz verisi bekliyor |
| Hata kataloğu | [docs/hata-katalogu.md](docs/hata-katalogu.md) | ✅ kaynak ve sürüm kimliği testli |
| Spesifikasyon | [spec/](spec/) | ✅ başladı — normatif çekirdek (RFC'lere bağlar) |
| Kanıt haritası | [docs/depo-butunlugu.md](docs/depo-butunlugu.md) | ✅ bütün RFC/ADR/spec → test yolları ve canlı sayılar CI'da |
| Faz test matrisi | [docs/faz-test-matrisi.md](docs/faz-test-matrisi.md) | ✅ gerçek Cargo/libtest envanteri, Tier-1 pass/fail/süre artefaktı |
| Semantic regresyon | [docs/semantic-regresyon-korpusu.md](docs/semantic-regresyon-korpusu.md) | ✅ 17 geçmiş bug → minimal `.dil` + faz/tanı/span/exit/çıktı |

<!-- ZEE-DEPO-SAYILARI:BEGIN -->
<!-- `cd compiler && cargo run --bin depo_sayilari -- --yaz` üretir. Elle değiştirme. -->
### Canlı depo sayıları

| Ölçüm | Tek kaynaklı değer |
|---|---:|
| Golden program | **33/33** |
| Rust + doctest vakası | **632** |
| Tanı kimliği | **154 etkin + 3 ayrılmış** |
| RFC | **26** (2 kabul, 22 geçici kabul, 2 taslak) |
| ADR | **58** (58 kabul) |
| Normatif spec bölümü | **25** |
<!-- ZEE-DEPO-SAYILARI:END -->

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
  `zee-tr-1` morfolojisi profil/snapshot/property korpusu ve immutable semantic
  SHA-256 kaydıyla sabitlendi;
  deterministik scheduler, görev sahipliği ve hata/iptal yayılımı gerçeklendi.
  Sonuç'un hata tarafı kod/mesaj/neden/veri taşıyan, eski çıktıyı koruyan
  Hata değeridir. Ondalık keyfî hassasiyetlidir; gezme derin değer kopyası,
  değer-sonuç imleci ve T053 kaynak sabitliğiyle tanımlıdır. Gezme usability
  kapısı gerçek insan formlarını bekler. K-094 paket yayın çekirdeği aynı
  kaynaktan deterministik `.zep`, Ed25519 imzalı yayın, SPDX 3.0.1 SBOM ve
  SLSA v1 provenance üretir. K-095 ağ dışı sabit root, eşik ve çift eşikli
  rotasyon, timestamp/snapshot/targets, rollback/expiry/mix-and-match, yanlış
  yayıncı, yanked ve kritik duyuru metadata doğrulamasını kurar. K-135 yalnız
  HTTPS origin kullanan redirect'siz statik taşıma, 64 ardışık root rotasyonu,
  atomik monoton durum ve yalnız tam zincirden sonra yayımlanan salt-okunur
  içerik-adresli cache/offline hit-miss katmanını ekler. K-136 exact
  `proje.dil` bildirimi, `proje.kilit` v3 kimliği, atomik ve salt-okunur kaynak
  kurulumu ile açık ağ kullanan `ekle/kilitle/paketler --yenile` CLI zincirini
  tamamlayarak V1-P1-07'yi kapatır. K-117 kaynak
  paketinin NFC yolunu, üç platformlu byte fixture'ını ve Unicode saldırı
  korpusunu kapatır.
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
  beklenirken K-101/ADR-014 yapı, işlem ve yerel sembolleri çıplak
  indeks/adlardan ayıran `YapiId`/`IslemId`/`SymbolId`
  [semantic kimlik modelini](docs/semantic-kimlik-modeli.md) kurdu.
  K-102/ADR-015 kaynak, token, parsed AST, bağlanmamış ve bağlanmış programı
  ayrı [faz tiplerine](docs/derleyici-faz-modeli.md) taşıdı. K-103/K-104 ve ADR-016
  checker'ın ifade türleri ile sembol/işlem/yapı bağlarını zorunlu
  [typed HIR'a](docs/typed-hir-modeli.md) indirdi; standart runtime ve `dene`
  hattı semantic kararlarını yalnız bu bağlardan alır. K-105/ADR-017 native
  HTTP istemcisine varsayılan 30 saniye + 8 MiB yanıt sınırı, yerel sunucuya
  bayt damlatmayla uzamayan 10 saniyelik mutlak istek okuma sınırı ekledi.
  K-106/ADR-018 process içi web deposunu 4096 toplam/1024 anonim oturumla
  sınırladı; anonim LRU tahliyesi ve kaymayan mutlak ömür sözleşmesini bağladı.
  K-137/ADR-034 production profilini proje kökündeki CAS-korumalı kalıcı ortak
  depoya taşıdı; login/revoke/expiry ile endpoint, CSRF ve Argon2id oran
  pencereleri restart ve worker geçişinde ortaktır. Güvenilir proxy tek-hop
  `Forwarded` içindeki kanonik IP'yi kurar. Process başına tek worker gerçeği,
  farklı loopback portlu N ayrı süreç ve reverse proxy modeliyle açıkça
  sözleşmeye bağlandı.
  K-139/ADR-036 CLI'a özel origin parser'ını kaldırdı; outbound allowlist,
  `--web-proxy`, `Host`, `Forwarded host` ve unsafe `Origin` artık aynı
  `AgHedefi` DNS/IPv6/port kimliğini kullanır. Listener sabit loopback'e bind
  eder ve kabul edilen peer'i ayrıca loopback olarak doğrular; B-052 kapandı.
  K-140/ADR-037 request-line, header ve gövdeyi metne geçmeden önce tek byte
  parser'da doğrular. Bare-LF/CR, obs-fold, NUL, absolute/authority/asterisk
  target, TE, duplicate/bozuk CL, exact olmayan veya UTF-8 dışı gövde
  fail-closed'dur; ayrı libFuzzer hedefi ve kalıcı korpus B-053'ü kapattı.
  K-141/ADR-038 iki Cargo grafiğini sabit `cargo-deny` ile RustSec, lisans,
  duplicate/wildcard ve kaynak politikasından geçirir. Bütün CI Cargo
  komutları kilitlidir; iki ayrı vendor üretiminin SHA-256 manifesti eşitlenir
  ve boş Cargo home ile compiler+fuzz gerçekten offline derlenir. B-054
  kapandı; bu bağımlılık allowlist'i Zee'nin bekleyen ürün lisansını seçmez.
  K-142/ADR-039 playground köprüsünü ABI v2'ye taşıdı. Yalnız kayıtlı başlangıç
  pointer'ı+exact boy çekirdeğe kopyalanır; invalid UTF-8 görünür hatadır,
  sonuç boyu kayıttan sorgulanır, yanlış/çift bırakma durumu bozamaz. Native,
  gerçek wasm32 Node hostu ve 1.745.134 çağrılık libFuzzer kampanyası B-055'i
  kapattı. K-143/ADR-040 ABI v3'ün limit dışa aktarımlarıyla kaynak metnini
  8 MiB, soru girdisini 1 MiB/4.096 satırda sınırlar. Rust köprüsü sahipli
  kopya/satır tablosundan, tarayıcı UTF-8 byte dizisi ve WASM tahsisinden önce
  reddeder; 1.709.869 çağrılık kaynak+soru fuzz kampanyasıyla B-056 kapandı.
  K-144/ADR-041 üretim `lib`+`dil` yüzeyindeki 48 kritik işlevi sabit Clippy
  ölçüsü, incelenmiş taban, kademeli büyüme payı ve bayat olmayan
  [eğilim raporuyla](docs/islev-egilimi.md) CI'a bağladı; B-035 kapandı.
  K-145/ADR-042, 53 Rust dosyasındaki eski biçim borcunu sabit `rustfmt` ile
  tek mekanik dilimde temizledi; üç platformlu `cargo fmt --all -- --check`
  CI kapısıyla B-037 kapandı. K-146/ADR-043 bütün gerçek Cargo/libtest
  vakalarını 21 birincil faza sahipletip her Tier-1 işletim sisteminde
  test/pass/fail/ignored/süre, regression, fuzz ve conformance ilişkisini
  raporlayarak B-038'i kapattı. K-158/ADR-057 birincil sahipliği koruyup kritik
  test gruplarının gerçekten yürüttüğü ek fazları exact seçici metadatasıyla
  görünür kıldı; blast radius dosya adından tahmin edilmez. K-147/ADR-044 bunu
  ayrı bir 22. rapor fazıyla tamamladı: `regression/` altındaki 17 minimal `.dil` vaka bug kimliği,
  birincil faz, kesin tanı spanı, exit ve stdout beklentisiyle birebir
  sahiplenir; [bakım protokolü](docs/semantic-regresyon-korpusu.md) yeni her
  compiler bug düzeltmesinde aynı kaydı zorunlu kılar. K-155/ADR-052'nin v2
  manifesti her vakaya exact `fixed_by`, kanıtlıysa `introduced_by` ve
  `guaranteed_since=0.8.0-dev` ekledi. K-155A/ADR-053 ilk başlık regex'ini
  kaldırdı: `1c73298…` sonrasındaki her `compiler/src` commit'i artık mesajdan
  bağımsız tekil semantic sınıf, kanıt ve gerekçe taşır; `semantic-bugfix`
  exact fixture olmadan geçemez. CI provenance yeniden yazımını da reddeder.
  B-039/B-063/B-064 kapandı.
  K-148/ADR-045 eski terminal medyanını dokuz ayrı parse/checker/HIR/runtime/
  yürütme/LSP/bellek yüzeyinde 25 turluk ham örnek+p50/p95 gözlemine çevirdi.
  K-152/ADR-049 tarihçedeki her satırı tam Git SHA, ayrı milestone, temiz
  çalışma ağacı ve OS/CPU/RAM/Rust/release+örnekleme provenance'ına bağladı;
  RSS açıkça tek süreç-tepe görüntüsüdür. [Sürümlü tarihçe](docs/performans-gecmisi-v2.tsv), JSON ve Markdown her
  Linux CI koşusunda summary+90 günlük artefakttır; shared CI hard gate
  değildir, eşik yalnız adanmış runner'da açıkça etkinleşir. B-040 kapandı.
  K-153/ADR-050, eski nanosaniyelik LSP ölçümünü `lsp_engine_initialize`
  diye doğru adlandırdı ve gerçek `dillsp` process spawn→stdio→capabilities
  yanıtını `lsp_process_cold_start` olarak ayırdı. Tier-1 entegrasyon ve CI
  yolu hazırdır; exact `a2693d6…` commit'indeki 25 örnek process p50 1,557 ms,
  p95 1,997 ms tabanını verdi. K-153/B-061 kapandı.
  K-154/ADR-051, aynı gözlem hattına açık `--lsp-olcek` ile 2k/5k/10k/20k
  satır tam-metin `didChange` eğrisini ve p95 250/500/1000 ms ilk-aşım
  raporunu ekledi. Mevcut LSP her değişimde tam belgeyi saklayıp klonlar ve
  lexer/parser/resolver/checker/typed-HIR hattını baştan kurar; incremental
  cache yoktur. Exact temiz `59580cd…` uygulama commit'indeki 25 örnekte
  2k/5k/10k/20k p95 167,281 ms / 1.081,746 ms / 4.669,372 ms /
  20.159,636 ms'dir; 250/500/1000 ms eşiklerinin üçü de 5k'da aşılır.
  K-154/B-062 kapandı; bu dilimde optimizasyon yapılmadı.
  K-149/ADR-046 bütün production Rust ağacını 35 sorumluluk sahibine, exact
  doğrudan kenar tabanına ve izinli katman yönüne bağladı. Yeni/kayıp modül,
  yeni/kaldırılmış kenar ve ters katman geçişi fail-closed'dur; SHA-256 ile
  takvim ilkellerinin iki gerçek ters bağımlılığı aşağı taşındı. Ayrıntı
  [katman mimarisi rehberindedir](docs/katman-mimarisi.md); B-057 kapandı.
  K-150/ADR-047 exact graph'a SCC kapısı ekledi: tanı↔kaynak bütçesi ile
  checker↔HIR çevrimleri bağımsız `tani_politikasi` ve `semantic_model`
  sahipleriyle kırıldı. K-160/ADR-059 son paket/registry/tedarik SCC'sini
  davranışsız model, resolver, registry taşıması, artefakt doğrulama ve yayın
  sahiplerine ayırdı. Geçici C001 silindi; production SCC ve izin sayısı
  sıfırdır. CORE FREEZE etkindir: yeni compiler özelliği gerçek dogfood
  ihtiyacı kanıtlanmadıkça varsayılan olarak reddedilir. K-160A bu sözü
  executable yaptı: CI exact freeze sınıfını ve dogfood feature için ürün,
  K-işi, reproducer, etkilenen proje, minimalite ve ADR/spec/RFC kanıtını ister.
  B-073 bu bağı append-only ürün/provenance kaydıyla sertleştirdi: K-işi gerçek
  backlog/günlük kaydı, etkilenen Zee dosyası etkin ürün kökü ve karar belgesi
  iş/ürün referansı taşımadan kapı açılmaz.
  K-163 ilk gerçek ürün hattı da başladı: Çatlı/ITWISE Admin'in temiz dogfood
  dalındaki `43171b9 → cef5ae3` hattı 495 satır Zee ile yönetici oturumu,
  CSRF korumalı duyuru CRUD'u, JSON site ayarları ve slug doğrulamalı
  taslak/yayında sayfa yaşam döngüsünü çalıştırdı. 7/7 test ile gerçek TCP
  içerik+ayar+sayfa provası geçer. K-163'ün 500–1500 satır, medya ve production
  dayanıklılığı gibi geniş ürün yüzeyi kapısı açıktır.
  K-163'ün dördüncü dilimi bu ilk gerçek runtime eksiğini ürün kanıtıyla
  açtı: RFC-0026/ADR-060/spec-25 altında secretsiz exact loopback bildirim,
  ayrı TEXT parametre bind'i, C025 içinde SQLSTATE/constraint, eylem
  transaction/savepoint'i ve SHA-256 geçmişli dil göçür komutu çalışır.
  PostgreSQL 16.11 gerçek provada apply→skip→değişmiş-hash reddi, injection
  benzeri slug'ın aynen saklanması, 23505 dalı ve outer rollback geçti.
  Production TLS/pool/multi-process hâlâ açıkça kapsam dışıdır.
  Beşinci dilimin `43d04fc` ürün kanıtı ilk dayanıklılık kırığını buldu:
  başlangıç bağlantısı kapasite dönünce iyileşiyor, öldürülmüş canlı client ise
  süreç içinde yeniden bağlanmıyordu. K-163 correctness dilimi transaction dışı
  salt okumada kapalı client'ı atıp tek reconnect + tek SELECT tekrarı ekledi;
  aynı gerçek PostgreSQL 16.11 backend-sonlandırma provası process restart
  olmadan iyileşti; exact ürün kanıtı `78cce11` commit'indedir. Transaction içi
  okuma, SQL reddi, write ve COMMIT otomatik
  tekrar edilmez; COMMIT kaybı “sonuç belirsiz”dir. Write olumsuzunda satır
  oluşmadı fakat uygulama C021 ile sonlandı. Ürünün DB hatasını boş liste gibi
  göstermesi görünür arıza kartına çevrildi. F027 doğruluk dilimi transaction
  sınırı hatasını 503'e çevirdi, client/savepoint durumunu geçersiz kıldı ve
  worker'ı sonraki bağımsız read/write için ayakta tuttu; başarısız write tekrar
  edilmedi. Gerçek pool ve wire-level COMMIT ambiguity açık kaldı. 492 LOC bakım tabanı 1000+ satır
  karşılaştırması için exact yöntemle kaydedildi.
  K-151/ADR-048 bütün GitHub Actions `uses:` referanslarını incelenmiş 40
  haneli commit SHA'lara sabitledi. Sürümlü pin kaydı workflow'larla birebir,
  haftalık Dependabot yalnız inceleme PR'ı açar; hareketli `@v4`, `@stable`
  ve kayıt dışı action tedarik kapısından geçemez. B-059 kapandı.
  K-107/ADR-019 `dillsp` girdisini 8 KiB başlık, 8 MiB gövde, 128 JSON
  derinliği ve 100 bin düğümle sınırlayıp Unicode parser olumsuzlarını kapattı.
  K-138/ADR-035 sayı ayrıştırmasını RFC 8259 durum makinesine taşıdı; sayısal
  kimliği float'a çevirmeden korudu, duplicate anahtarı reddetti ve parse,
  request, method, params hata kodlarıyla notification sessizliğini bağladı;
  B-051 kapandı.
  K-108/ADR-020 her semantic typed-HIR ifadesine zorunlu kaynak aralığı
  ekledi; kesin token konumu olmayan eski AST düğümleri uydurma sütun yerine
  kaynak satırı zarfı taşır. K-126/ADR-030 bu geçişi tamamladı: artık her
  parser AST yaprağı ve bileşiği ayrı kesin `AstKaynakAraligi` taşır, checker
  aynı aralığı HIR'a aktarır ve spansiz/uyuşmayan kayıt invariantta reddedilir.
  K-109/ADR-021 production'daki 46 doğrudan panic noktasını tanı/sonuca
  çevirdi; `lib`, `dil`, `dillsp` ve `olcum` yeni unwrap/expect/panic
  kullanımını test dışı Clippy kapısında reddeder.
  K-110/ADR-022 lexer ve iki parser yolunu Unicode/girinti/sayı/virgül
  korpusu, 4.096 deterministik UTF-8 bileşimi ve gecelik libFuzzer ile bağladı.
  K-111 `zee-tr-1` üret→çöz değişmezini 4.096 geniş kök, bütün adaylarda
  fail-closed A002, NFC/NFD sınırı ve ayrı gecelik morfoloji fuzz hedefiyle
  sertleştirdi.
  K-112/ADR-023 parser AST'sinin semantic bağ taşımamasını ve her bağlı AST
  ifadesinin tam bir typed-HIR kaydıyla eşleşmesini yürütülebilir invariant
  kapısına bağladı. Bu kapı özellik→alan dönüşümündeki gerçek bir yetim HIR
  kaydını buldu; dönüşüm artık alt düğümü klonlamak yerine taşıyor.
  K-113/ADR-024 kurtarmalı parser'ı cümle sonu+dengeli girinti sınırlarına
  bağladı. İç hata sağlam kardeşi bloktan dışarı sızdırmıyor; yapı, `göre` ve
  eşzamanlı bloklar sonraki sağlam satırı koruyor. CLI/LSP tanıları kaynak
  sırasında ve belge başına en çok 20 kayıtla yayımlanıyor.
  K-114/ADR-025 yayımlanmış 145 etkin tanıyı ve 3 ayrılmış mezar taşını
  sürümlü kimlik fixture'ına bağladı; aynı kod artık sessizce başka bir anlam
  için kullanılamıyor. Bakım protokolü
  [tanı kimliği rehberindedir](docs/tani-kimligi.md).
  K-115/RFC-0022 bütün `GirdiCikti` olaylarını işlem+argüman+sonuç+sıra olarak
  kanonik şema-1 izine bağladı. `dil iz kaydet/oynat`, gerçek koşuyu atomik
  kaydedip dış dünyaya yeniden dokunmadan fail-closed oynatır; ayrıntılar
  [IO izi rehberindedir](docs/io-izi.md).
  K-116/RFC-0023 rastgele tohum dizisini, tam i64 aralık eşlemesini, sanal
  saati ve hermetik adaptör gözlemlerini `zee-io-1` profiline sabitledi;
  [profil rehberi](docs/deterministik-io-profili.md) kırıcı değişikliği yeni
  kimliğe zorlar.
  K-117/ADR-028 `.zep` dosya yollarını NFC'ye kanonikledi; Türkçe Unicode
  adlı sabit fixture üç Tier-1 işletim sisteminde aynı byte'ı arar ve
  [80 vakalık saldırı korpusu](docs/zep-conformance.md) ayraç benzerleriyle
  görünmez bidi yollarını fail-closed reddeder.
  K-118/ADR-010 her RFC/ADR/spec'i test dosyalarına bağlayan
  [kanıt haritasını](docs/depo-butunlugu.md) ve README canlı sayı üreticisini
  tazelik testine bağladı; sayılar artık elle tutulmuyor.
  K-119 resmî formatter'ın satır ve girinti tokenları dahil parser girdisini
  değiştirememesini production C011 güvencesine ve 33 golden programın
  dağınık-boşluk property kapısına bağladı; B-042 kapandı.
  K-120 LSP definition/rename'i `SymbolId`/`IslemId`/`YapiId` typed-HIR
  bağlarına geçirdi; ayrı kapsamdaki aynı adlar ayrılır, hatalı belgede metin
  tahmini yapılmaz ve başka dosyadaki tanım için eksik rename üretilmez. Bakım sözleşmesi
  [semantic gezinme rehberindedir](docs/lsp-semantic-gezinme.md).
  K-121 yerel `<ad> al` işlemlerinde bütün erişilebilir çağrı kısıtlarını
  gövde ve HIR üretiminden önce birleştirdi; dar/geniş çağrıların kaynak
  sırası artık program türünü değiştiremez. Bakım sözleşmesi
  [çağrı çıkarımı rehberindedir](docs/cagri-cikarimi.md).
  K-122 `zee-tr-1` davranışını 53.248 üretim/çözüm vektörlü semantic kayda
  bağladı; kod davranışı değişirse test, eski fixture değişirse Git-geçmişli
  CI koruğu kırılır. Yeni anlam yalnız yeni profil kimliğiyle açılır. Bakım
  sözleşmesi [profil uyumluluk rehberindedir](docs/morfoloji-profil-uyumlulugu.md).
  K-123 aynı profili kök `conformance/` alanında sürümlü JSON Schema, 27
  çözüm/karar ve 21 üretim vakasıyla Rust'tan bağımsız yayımladı. Gelecekteki
  derleyici gerçeklemeleri aynı sıralı adayları ve üretimleri bu veriyle
  sınayabilir; bakım ve tüketici protokolü
  [morfoloji conformance rehberindedir](docs/morfoloji-conformance.md).
  K-124 scheduler'ın çıktı/ortak IO sırası, sanal süre, sonuç bağlama ve iptal
  etkilerini `zee-esz-1` JSON Schema + 10 kaynak programla derleyiciden
  bağımsız kilitledi. Çok çekirdekli bir gelecek runtime bu gözlemleri aynen
  korumadıkça uyumlu sayılamaz; bakım sınırı
  [eşzamanlılık conformance rehberindedir](docs/eszamanlilik-conformance.md).
  K-125/ADR-029 artık var olmayan `GerçekSayı↔double` FFI taslak eşlemesini
  kaldırdı. Ondalık'ın binary32/binary64'e örtük dönüşümü yasaktır; gelecekteki
  köprü açık `kayıplı` işareti, `Sonuç` ve normatif IEEE 754 ayrıntıları
  olmadan gerçeklenemez. FFI'nın kendisi Faz 4/5 taslağı olarak kalır.
  K-126/ADR-030 B-050'yi kapattı: bileşik ve yaprak ifadeler kesin
  satır+sütun+uzunluk taşır; eski `1:1` checker işaretleri gerçek AST
  ifadesine yükselir, örtük çoğul kaynak döngü adının tokenına bağlanır ve LSP
  işlem/yapı adını yalnız semantic ifadenin kesin aralığında seçer.
  K-127/RFC-0024/ADR-031 B-023/B-049'u kapattı: manifestteki sekiz kararlı
  yetkinlik paketlerin üst sınırı ve compile/runtime kapısıdır. Native
  outbound exact `ureq 3.4.0` + rustls ile HTTPS, tam origin, DNS sonrası
  IP/özel-kullanım-geçiş öneki, sıfır redirect/proxy ve sabit zaman/bellek
  zarfı taşır; ayrıntılı
  kullanım [yetkinlik ve ağ güvenliği rehberindedir](docs/yetkinlik-ve-ag-guvenligi.md).
  K-128/ADR-032 B-048'i kapattı: atomik replace Linux/macOS'ta mode+uid+gid
  ve desteklenen ACL/xattr'ı, Windows'ta DACL/security resource/named stream'i
  korur; metadata taşınamıyorsa eski hedefe dokunmadan fail-closed durur.
  K-129/RFC-0025/ADR-033 B-025'in ilk ortak kaynak profilini kurdu: kaynak,
  token, proje toplamı, çalışma adımı, çıktı, koleksiyon, görev, dosya okuma
  ve LSP toplam belleği/yanıtı aynı değişmez `KaynakSinirlari` değerinden
  sınırlanır. K-130 buna 16 MiB bütçeli metin, yaklaşık 64 MiB saklanan değer
  zarfı ve 64 süreç-geneli ağ bağlantısı ekledi; aşım S045/C023/C024 veya
  kontrollü ağ reddiyle durur. K-131 HTTP/ağ, web oturumu, IO izi, LSP,
  paket/registry, tanı, atomik metadata ve kilit sürelerinin eski yerel
  sayılarını davranış değiştirmeden aynı profilin domain görünümlerine taşıdı.
  K-132 LSP initialize, diagnostics, completion, hover, definition, rename ve
  hata gövdelerini 8 MiB sınırlı akış yazıcısına taşıdı; rename/diagnostics
  artık bütçesiz ara JSON listeleri kurmaz. Böylece B-025/V1-P0-31 kapandı.
  K-133 çıktı, dosya, HTTP, web/oturum, eyleyici ve rastgelelik sınırlarına
  tam etki öncesi deadline kapısı ekledi; görev HTTP öncesi sıra verdikten
  sonra süreyi yeniden hesaplar ve dolmuş isteği hiç başlatmaz. K-134 ilk web
  yanıtıyla session/cookie mutation'ını aynı istek transaction'ında tamponlar;
  timeout/runtime/socket hatası veya yanıtsız rota hepsini geri alır. B-026
  kapandı.
  Uygulama sırası
  [öncelikli backlog](docs/oncelikli-backlog.md) ile sabittir.
  Bağlayıcı liste: [docs/v1-surum-kapilari.md](docs/v1-surum-kapilari.md).

## İlk gerçek milestone

Tek dosya: `merhaba.dil` — tek satır:

```
"Dünyaya merhaba" yaz
```

Bu dosya kendi lexer/parser/type checker zincirimizden geçip çalıştığında proje doğmuş sayılır.
