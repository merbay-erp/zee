# Sürüm notları

Her sürüm: ne eklendi, ne değişti, neyin sözü verildi. Kırıcı değişiklik
sessizce yapılamaz (master plan bölüm 23) — burada duyurulur.

## v0.7.0 — 1 Eylül 2026

**Tema: editör zekâsı — dil, düzenleyicide de Türkçe düşünüyor.**

(Üstteki K-069..K-072 kalemleri bu sürümündür: çıkış kodu, kitaplık
olgunlaşması, zengin doğrulamalar, morfolojili yeniden adlandırma.)

## Yolda (v0.8.0'a birikenler)

- **Normatif otorite ve v1 kapıları** (K-081, ADR-010): geçerli dilin kesin
  davranışını spec anlatır; RFC değişikliği yetkilendirir ama spec+conformance
  testi aynı değişiklikte güncellenmeden yürürlüğe girmez. Kaynak denetimli
  [v1.0 sürüm kapıları](v1-surum-kapilari.md) bağlayıcıdır. RFC-0006 ve
  RFC-0011 güncel gerçeklemeyle uzlaştırıldı; RFC-0015 production web/eylem
  sınırını taslağa aldı.
- **Deneysel web korkuluğu** (K-082): gerçek TCP sunucusu artık sıradan
  `dil çalıştır` ile açılmaz; yalnız açık `--deneysel-web` opt-in'i ve görünür
  production uyarısıyla localhost'ta çalışır. Panel örnekleri eğitim demosu
  olarak yeniden etiketlendi; giriş/kaydet/sil/çıkış durum değişiklikleri POST
  kontrolüne alındı. GET ile silmeme regression testidir.
- **Açık işlem imzası** (K-083): başlangıç için `sayıyı al` aynen kalır;
  public/paket API'si `sayıyı Ondalık olarak al` yazabilir. Açık gövde hiç
  çağrılmasa da denetlenir, imza çağrıyla değişmez. Liste/sözlük/Seçenek/Sonuç
  ve yapı tür yazımları desteklenir; TamSayı→Ondalık runtime değeri de
  genişler. T037/T038, golden 33 ve çağrı sırası permütasyonlarıyla 270 test.
- **Atomik kalıcı dosya** (K-084, RFC-0016): `dosyasına ... yaz/ekle`
  aynı klasörde geçici dosya + disk eşzamanlama + atomik replace kullanır.
  Unix `flock` / Windows `LockFileEx` süreç kilidi iki yazarın
  güncellemesini korur; ani süreç sonu kilidi bırakır. Biçimleyici, paket
  bildirimi ve `proje.kilit` tek-dosya yazmaları da aynı çekirdeğe taşındı.
  Hata enjeksiyonu, eşzamanlı okuyucu ve iki bağımsız CLI yazarıyla 276 test.
- **Gerçek son tarih iptali** (K-085, RFC-0011/spec-09): `içinde` artık
  gövdeyi bitirip geç kaldığını sonradan söylemez. Mutlak deadline
  blok/işlem/döngü sınırlarında denetlenir; `bekle` kalan süreye kırpılır,
  HTTP bağlantı/yazma/okuma tek bütçeyi kullanır. İç içe bloklarda yalnız
  deadline sahibi `yetişmezse` kolu çalışır; iptal sonrası yan etki yoktur.
  Ç001 iç nöbetçisiyle 123 katalog kodu ve toplam 279 test; playground'da
  ardışık sanal beklemeler de aynı son tarih anlamını taşır.

- **Proje modeli** (K-076): geçerli zee sözdizimli `proje.dil` (`proje`,
  `sürüm`, `giriş`); `dil çalıştır/denetle/dene <klasör>`; `dil yeni`
  bildirimi hazır üretir. P001–P004 Türkçe proje tanıları. Doğrudan dosya
  kullanımı geriye uyumludur. Göreli dosya IO'su giriş klasörüne sabitlendi;
  `--güvenli` bayrağının program argümanına sızması da kapandı.
- **Yerel paketler ve kilit** (K-078): `yerel_bağımlılıklar` başka zee
  projelerini bağlar; `X paketini kullan` yalnız doğrudan bağımlılığı alır.
  Kaynak kökeni paket içi birimlerde korunur. `dil kilitle` geçişli grafiği
  göreli yol, sürüm, kenar ve SHA-256 `.dil` özetiyle deterministik
  `proje.kilit`e sabitler; eksik/bayat kilit P008'dir. Döngü, yinelenen ad,
  proje dışına çıkan sembolik bağ ve geçişli bağımlılığa gizli erişim testli.
- **Güvenli paket ekleme** (K-079): `dil ekle <yerel-yol> [proje]` yolu
  proje köküne göre taşınabilir biçime çevirir; yorumları koruyup listeyi
  sıralar. Yeni bağımlılık grafiği yazmadan önce bütünüyle doğrulanır. Başarılı
  işlem bildirimi ve kilidi birlikte günceller; yinelenen gerçek kök ikinci kez
  eklenmez, öz-bağımlılık P007'dir.
- **Paket grafiği ve güvenli kaldırma** (K-080): `dil paketler [proje]`
  doğrudan/geçişli paketleri sürüm, göreli yol ve kilit özetiyle listeler.
  `dil çıkar <paket> [proje]` yalnız doğrudan paketi kaldırır; kaynakta kalan
  `X paketini kullan` P010 ile işlemi değişiklik yapmadan durdurur. Başarıda
  bildirim/kilit birlikte güncellenir ve yazma hatasında geri alınır.
- **Proje çapında biçimleme** (K-077): `dil biçimle <klasör>` bütün `.dil`
  kaynaklarını deterministik yol sırasında biçimler; önce tamamını doğrular,
  tek hata varsa hiçbir dosyaya dokunmaz.

- **Gezmede yazma yansır** (K-074): `her kutu için / kutunun adedi ...`
  listeye geri yazılır — kopya tuzağı kapandı (TANIMLI).
- **Türk para yazımı** (K-075): `binlikli kuruşlusu` → "1.234.567,89".
- **Çerez silme** (K-073): `"oturum" çerezini sil` — Max-Age=0; panel
  çıkışı gerçek silmede.

## v0.6.0 — 1 Eylül 2026

**Tema: tip sistemi olgunlaştı, dil ayrıntıda medenileşti.**

- **Aralık iki yönde** (K-068): `5 ten 1 e kadar` geri sayar (sessiz
  boş dönüş tuzağı kapandı). roket.dil projesi.
- **Çıkış kodu** (K-069, K-024 kapanışı): `programı 1 ile bitir` —
  süreç kodu kabuğa gider (0–255, C020); CLI otomasyon kapısı.
- **Yeniden adlandırma** (K-072): dillsp + VS Code F2 — morfoloji
  farkındalıklı: ekler yeni köke Türkçe uyumla giydirilir (sayacı→puanı,
  renk→rengi). Metin/yorum dokunulmaz; tek katman ek kapsamı.
- **Doğrulamalar** (K-071): `içermeli`, genel `olmamalı`, çıplak `boş`
  atomu — test kültürü zenginleşti. Ölçüm: döngü izlemesi kapandı
  (v0.6.0 satırı; v0.2 tabanının altında).
- **Kitaplık** (K-070): `toplamını/ortalamasını hesapla` artık DAİMA
  Ondalık döner (deneysel-kırıcı; TamSayı listeleri genişlemeyle girer).
  Yeni gömülü birim: `sozluk_araclari` (`en çok geçeni bul`).

- **Para biçimi** (K-065): `tutarın kuruşlusu` — daima iki hane.
- **Sayısal genişleme çağrıda + imza terfisi** (K-067): Liste<TamSayı> →
  Liste<Ondalık> parametre; dar imza geniş argümanla terfi eder. Sözlük
  değerleri Ondalık olabilir. Kitaplığa medyan girdi.
- **Evrensel metin hali** (K-066): `değerin metni`; `dil belge` artık
  işlem açıklamalarını basar; playground'da Envanter vitrini.
- JSON okuma hoşgörüsü (K-063), alan-özellik gölgelemesi (K-064),
  envanter projesi; işlemden liste dönüşü testle sabitlendi; biçim
  hijyeni projeler+kitaplığa genişledi.

## v0.5.0 — 1 Eylül 2026

**Tema: kayıtlar ve koleksiyonlar — dil, veri işlerinin dili oldu.**

- **JSON okuma hoşgörüsü** (K-063): sayı/bool/null değerler Metin gelir
  (nokta → virgül); yalnız iç içe yapı C016.
- **Alan, özelliği gölgeler** (K-064): `ürünün adedi` alan okur —
  denetleyici yeniden yazımı. Envanter projesi eklendi (stok defteri).

- **Sıralama** (K-056): `sıralanmışı` — Metinler Türk alfabesi sırasıyla
  (ç, ğ, ı, i, ö, ş, ü yerli yerinde; TANIMLI); `tersi`.
- **Tarih farkı** (K-057): `X ile Y arasındaki günler` (işaretli).
- **Liste üyeliği + CSV yazma** (K-058): `sayılarda 5 varsa`;
  `tablonun csv metni`.
- **Morfoloji: iki katmanlı ek** (K-061): `kitabın fiyatıyla artır`
  (iyelik+araç zinciri) çözülür.
- **CSV hücreleri Metin** (K-062): isimli sütunlar birinci sınıf;
  sayı `değerin sayısı` ile bilinçli çevrilir. Golden 19 revize.
- **Yapı listeleri** (K-060): kayıt tabloları — `boş liste`ye yapı ekle,
  gez, alan oku; `json metni` nesne listesi üretir. (T027 belgesi düzeltildi:
  Ondalık alan zaten vardı.)
- **Silme** (K-059): `sayılardan 5 i sil` / `defterden "elma" yı sil` —
  yoksa sessiz (idempotent). Girişli panel demosuna silme akışı eklendi
  (dosya-satırı silme saf zee: süz + birleştir + yaz).

## v0.4.0 — 1 Eylül 2026

**Tema: dil, kurucunun gerçek projelerine hazırlanıyor — web ve metin.**

- **Metin dalgası** (K-053): `parçaları`, `birleşmişi`, `yerine ...
  değişmişi`, `kırpılmışı`, `harfleri`, `ile başlıyorsa/bitiyorsa` —
  harf düzeyine ilk iniş. Gömülü `metin_araclari` birimi (saf zee).
- **JSON yazma** (K-054): `değerin json metni` — sıra-korumalı,
  deterministik serileştirme.
- **Deneysel web uygulaması katmanı** (K-050..K-052, K-055): HTML servis, örtük
  `istek` sözlüğü (sorgu + POST form), `adresine yönlendir` (303),
  `html güvenlisi` (XSS), örtük `çerezler` + `çerezine yaz` (oturum),
  önekli rotalar. Kanıtlar: mini-site, panel-not-defteri, girisli-panel
  (parola+oturum mantığı SAF ZEE) — üçü de localhost'ta canlı + hermetik.
- **Sınır (değişmedi):** parola düz metin, HTTPS yok — internete açık
  üretim Faz 5 güvenlik dalgasını bekler.

### v0.3.0 sonrası küçükler

- **Ünsüz ikizleşmesi morfolojisi** (K-049): `üssü`, `affı`, `zammı`,
  `reddi` (sertleşmeyle) çözülür; matematik biriminin doğal `üssü al`
  parametresi geri geldi.
- **`dil belge <birim>`**: işlem başlıkları + test sayısı (RFC-0014 §8.2).
- Playground'a "Kitaplık (obeb)" örneği eklendi.
- **Deneysel zee web sitesi** (K-050): sunucu HTML'i text/html olarak servis
  eder; projeler/mini-site.dil — rotalar + stil + gömülü kitaplık hesabı.
- **Web uygulaması dalgası** (K-051): örtük `istek` sözlüğü (sorgu + POST
  form, UTF-8 yüzde çözümü), `adresine yönlendir` (303, S041),
  `html güvenlisi` (XSS kaçışlaması). Kanıt: panel-not-defteri projesi —
  formlu, dosyada saklayan, gizli yollu eğitim paneli (localhost'ta canlı +
  hermetik tam-döngü testi). Sınır: HTTPS/hash Faz 5'te.
- **Oturum kapısı** (K-052): örtük `çerezler` sözlüğü + `çerezine yaz`
  (Set-Cookie, HttpOnly). Oturum mantığı saf zee'de: girisli-panel projesi
  (parola → dosyada oturum kimliği → korumalı rotalar; sahte çerez reddi
  testli). Parola düz metin — hash Faz 5.

## v0.3.0 — 1 Eylül 2026

**Tema: dil günlük Türkçeye yaklaştı; standart kitaplığın tohumu atıldı.**

- **Gömülü standart kitaplık** (RFC-0014 taslak + çalışan prototip, K-048):
  `matematik` (mutlak, üs, tam karekök, obeb-Öklit, okek) ve
  `liste_araclari` (toplam, uçlar, Ondalık ortalama) — zee'yle yazıldı,
  ikiliye gömülü, playground dahil her yerde kurulumsuz; kendi test
  blokları CI'da. Çözüm: yerel klasör → gömülü.
- **Öğretmen rehberi** (docs/ogretmen-rehberi.md): internetsiz sınıf
  kurulumu, 10 oturumluk ders sırası, hata kültürü.
- Proje kitaplığı 10 projeye çıktı (kelime sayacı, gün sayar).

- **Mantıksal ad tek başına koşul** (K-044): `hazır ise` / `hazır değilse`.
- **Boş koleksiyon tür çıkarımı** (K-045): `boş liste`/`boş sözlük` ilk
  eklemeyle türlenir; metin listeleri ve metin sözlükleri artık kurulabilir.
  KIRICI: hiç eklenmemiş boş listenin `ilki` artık derleme hatası (T014;
  eskiden çalışma anında C007).
- **Çocuk modu** (K-047): `dil çalıştır --güvenli` — ağ/sunucu kapalı,
  dosyalar çalışma klasörüyle sınırlı; hata `dene` ile yönetilebilir.
- **Kalan işlemi** (K-046): `17 nin 5 e bölümünden kalanı` — okul kuralı,
  kalan hiç negatif olmaz.
- **Zamir n'si morfolojisi** (K-041): `bilgisayarın_zarından` çözülür.
- **VS Code**: elle yazılmış LSP istemcisi (npm'siz) — canlı tanılar,
  hover, tanıma git, tamamlama editörde. ADR-008 (self-hosting aşamaları) kabul.
- dillsp: hover (Türkçe açıklama) + tanıma git. Performans arşivi
  (docs/olcumler.md) ve `olcum` koşucusu. Proje kitaplığı 8 projeye çıktı.

## v0.2.0 — 31 Ağustos 2026

**Tema: dil derinleşti, derleyici tarayıcıya taşındı.**

### Yeni dil yüzeyi

- **Özyineleme** (K-035): işlem kendini ve karşılıklı olarak birbirini
  çağırabilir; tanım sırası serbest (adlar ön-taranır). Kural: temel durum
  özyinelemeli çağrıdan ÖNCE gelir (T035) — tür çıkarımı "o ana dek görülen
  dönüşler" üzerinden yapılır, dil iyi alışkanlığı kendisi öğretir.
  Çalışma zamanı derinlik sınırı 500 (C019, K-040: her platformda aynı).
- **Blok kapsamı** (K-034, RFC-0004 kapanışı): gövdede doğan ad gövdeyle
  ölür (döngü değişkeni dahil); dıştaki ada atama kalıcıdır; gölgeleme
  yapısal olarak yoktur.
- **Akış-duyarlı daraltma** (K-037, T036): `varsa` / `başarılıysa` /
  `başarısızsa` dalları ve `değilse` tersinmeleri içinde `değeri` / `hatası`
  erişimi statik güvenli; dal dışında korumasız erişim artık DERLEME
  hatasıdır. "Boş değeri açmak" hatası çalışma zamanından derleme zamanına
  taşındı; C008/C009 iç savunmaya dönüştü.
- **Metin kaçışları** (K-036): `\"`, `\\`, `\n`; bilinmeyen kaçış S040.
- **Negatif sayı sabitleri** (K-036): `-3`, `-3,14` — işaret rakama bitişik.
- **Çok-tokenli çağrı argümanları** (K-038): `tabanın tam kısmı için yuvarla`
  — her `ve`/`ile` dilimi tam bir ifade bölgesi.

### Playground (K-039)

- `playground/zee-playground.html`: derleyicinin tamamı WebAssembly olarak
  tek HTML dosyasında. Çift tıkla açılır; kurulum ve internet gerekmez.
- 7 hazır örnek, girdiler alanı, görünür tohum — **aynı tohum + aynı girdi
  = her zaman aynı çıktı** (determinizm tarayıcıda da geçerli).
- ADR-001 korunur: wasm-bindgen yok; elle yazılmış C-ABI köprüsü
  (`compiler/src/wasm_api.rs`), çekirdek doğal derlemede de testli.
- Yeniden üretim: `playground/olustur.sh`.

### Tanılar

- Yeni: T035 (temel durum önce), T036 (korumasız değer/hata erişimi),
  S040 (bilinmeyen kaçış), C019 (çağrı derinliği).
- Yeniden konumlanan: C008/C009 artık iç savunma (derlemede T036 yakalar).

### Mühendislik

- CI üç platformda yeşil; wasm hedefi de derlenir (ubuntu).
- CLI, işi 32 MB yığınlı iş parçacığında koşar (Windows ana iş parçacığı
  1 MB'dır); sınıra dokunan test kendi yığınını getirir (K-040).
- Toplam 154 hermetik test (`cargo test --no-fail-fast`), clippy temiz,
  uyarısız derleme.

### Kırıcı değişiklikler

- Korumasız `değeri`/`hatası` erişimi olan programlar artık derlenmez (T036).
  Düzeltme tanının önerisindedir: erişimi `varsa`/`başarılıysa`/`başarısızsa`
  dalına al. 32 golden programın hiçbiri etkilenmedi.
- Gövde içinde tanımlanan ada gövde dışından erişim artık A001 (K-034).
  Korpusta buna dayanan program yoktu.

## v0.1.0 — 31 Ağustos 2026

İlk sürüm — proje doğdu. 32/32 golden korpus; v0.1 kabul kriterlerinin
8/8'i (master plan bölüm 33). Tam yüzey listesi: [README](../README.md).
Etiketler: `baslangic` (ilk taahhüt), `dogum` (merhaba.dil çalıştı),
`v0.1.0` (kabul kriterleri kapandı).
