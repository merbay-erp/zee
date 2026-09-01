# Sürüm notları

Her sürüm: ne eklendi, ne değişti, neyin sözü verildi. Kırıcı değişiklik
sessizce yapılamaz (master plan bölüm 23) — burada duyurulur.

## Yolda (v0.4.0'a birikenler)

- **Ünsüz ikizleşmesi morfolojisi** (K-049): `üssü`, `affı`, `zammı`,
  `reddi` (sertleşmeyle) çözülür; matematik biriminin doğal `üssü al`
  parametresi geri geldi.
- **`dil belge <birim>`**: işlem başlıkları + test sayısı (RFC-0014 §8.2).
- Playground'a "Kitaplık (obeb)" örneği eklendi.
- **zee ile web sitesi** (K-050): sunucu HTML'i text/html olarak servis
  eder; projeler/mini-site.dil — rotalar + stil + gömülü kitaplık hesabı.

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
