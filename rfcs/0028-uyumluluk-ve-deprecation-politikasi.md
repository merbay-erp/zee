# RFC-0028 — Uyumluluk ve deprecation politikası

- **Durum:** geçici kabul
- **Tarih:** 5 Eylül 2026
- **İş:** K-167 (B-072 dahil)
- **Karar:** ADR-064
- **Normatif yüzey:** spec/27

## Özet

Zee'nin kullanıcıya verdiği uyumluluk sözü sekiz yüzeyden oluşur ve her biri
makine-okunur bir kayıtla dondurulur. Ekleme uyumlu ve serbest; davranış
değişikliği RFC + spec + sürüm notu; kaldırma ise duyuru sürümü, kaldırma
sürümü ve göç yolu taşıyan `DEP-NNN` kaydı ister. 1.0 sonrasında grammar
kırılması yalnız yeni ana sürüm ya da edition ile olur.

## Motivasyon

Bir çocuğun bugün yazdığı `"Merhaba" yaz` dosyası yıllar sonra da çalışmalı
(MANIFESTO, master plan §23). Bu söz, kelime kelime hangi yüzeyin korunduğunu
ve bir şeyin nasıl kaldırılabileceğini yazmadan tutulamaz. Tanı, morfoloji,
paket ve Rust API katmanları zaten donuktu; kalıp kelimeleri, koşul yüklemleri
ve CLI komutları için tek kaynaklı bir envanter ile tek bir deprecation süreci
yoktu. Sürüm kimliği de etiket sonrası geliştirme serisini yansıtmıyordu.

## Tasarım

### 1. Sürüm kimliği

Sürüm SemVer `X.Y.Z`dir. Etiket kesildikten hemen sonra çalışma ağacı
`X.(Y+1).0-dev` serisine geçer; `-dev` aynı sayılı yayından önce sıralanır.
`dil sürüm`, LSP `serverInfo.version` ve SBOM builder kaydı aynı Cargo
sürümünü basar. Yayımlanmış bir sürüm kimliği asla yeniden kullanılmaz.

### 2. Dondurulmuş yüzeyler

| Yüzey | Kayıt | Değişmezlik kaynağı |
|---|---|---|
| `kalip` — kalıp kelimeleri | `dil-yuzeyi-v1.tsv` | LSP tamamlama listesi = ayrıştırıcı yüzeyi |
| `kosul` — koşul yüklemleri | `dil-yuzeyi-v1.tsv` | `kosul_kelimesi` |
| `komut` — CLI alt komutları (Türkçe ve ASCII biçimleri) | `dil-yuzeyi-v1.tsv` | `dil` dağıtım tablosu |
| `tani` — tanı kodları ve anlamları | `tani-kimlikleri-v1.tsv` | ADR-025 |
| `bicim` — `proje.kilit` v3, `ZEEZEP` v1, `zee-yayin-v1` zarfı | `dil-yuzeyi-v1.tsv` exact kayıt | RFC-0009/0020, ADR-028 |
| `profil` — `zee-tr-1`, `zee-io-1`, `zee-esz-1` | SHA-256/JSON korpusları | RFC-0018/0022/0023 |
| `abi` — WASM C ABI v3 | `DIL_ABI_SURUMU` | ADR-039/040 |
| `api` — `dil::api::v1` | exact allowlist | ADR-058 |

Her `aktif` satır giriş sürümünü taşır; `0.7.0` son etiketli sürümde var olan,
`0.8.0-dev` bu seride eklenen yüzeydir.

### 3. Değişiklik sınıfları

- **Ekleme** uyumludur: fixture'a `aktif` satırı ve giriş sürümü eklenir;
  spec/RFC gereği ayrıca RFC-0021 katman ve K-160A freeze kuralları geçerlidir.
- **Davranış değişikliği** (`davranis`): aynı yüzeyin anlamı değişiyorsa RFC,
  spec ve sürüm notu zorunludur; kırıcıysa deprecation kaydıyla duyurulur ve
  eski davranış süre bitene kadar korunur.
- **Yeniden adlandırma** (`yeniden-adlandirma`) ve **kaldırma** (`kaldirma`):
  yalnız `DEP-NNN` kaydıyla.

### 4. Deprecation protokolü

1. Kayıt `duyuruldu` durumuyla açılır: yüzey, öğe, duyuru sürümü, hedef
   kaldırma sürümü, göç yolu ve karar belgesi.
2. Desteklenen yüzeyde kaldırma sürümü duyurudan **en az bir alt sürüm
   serisi** sonradır (`0.8` → `0.9`). Tanı mezar taşları ve `ic` yüzeyler
   süre şartı taşımaz; `ic` desteklenen söz değildir (ADR-058).
3. Kaldırma commit'inde kayıt `kaldirildi` olur; kelime fixture'da mezar taşı
   olarak kalır ve yeniden kullanılamaz. Eski biçim sessiz kalmaz: derleyici
   tanı üretir ve önerisi göç yolunu adlandırır; bu davranış `regression/`
   altında kalıcı `.dil` vakasıyla kanıtlanır (ADR-044).
4. Sürüm notu (`docs/surumler.md`) kaydı kimliğiyle duyurur.

### 5. 1.0 ve edition

1.0 etiketinden sonra sekiz yüzeyde kırıcı değişiklik yalnız yeni ana sürüm
ya da `proje.dil` içinde açıkça seçilen edition ile mümkündür; eski edition
derlenmeye devam eder. Edition alanının sözdizimi **AÇIK**tır ve ayrı RFC
ister; bu RFC yalnız çerçeveyi bağlar.

### 6. Paket uyumluluğu

Registry'de yayımlanmış `.zep` ve metadata değişmez (spec/18–19); aynı ad+
sürüm ikinci kez yayımlanamaz. Yeni kilit sürümü eski kilidi `dil kilitle`
göçüyle yeniler; sessiz yeniden yazma YASAKTIR. Morfoloji profili değişimi
RFC-0018 gereği yeni kimlik ve ana sürüm/edition ister.

## Dört soru süzgeci

1. Türkçe doğal mı? — Politika kelime yüzeyine dokunmaz; yalnız korur.
2. Deterministik mi? — Fixture ve kayıt metin dosyasıdır; kapı CI'da aynı
   sonucu verir.
3. Öğrenilebilir mi? — Öğretmen yazdığı programın hangi sürüme kadar
   çalışacağını ve göç yolunu tek tablodan okur.
4. Profesyonel ölçekte savunulabilir mi? — SemVer + süreli deprecation +
   makine kapısı; sürüm kimliği geliştirme serisini yayımlanmış sanmaz.

## Alternatifler

- **Golden korpusa güvenmek:** kullanılmayan kelimeler korunmaz.
- **Her kaldırma için ayrı ADR:** kayıt dağılır; süre ve göç yolu tek yerde
  denetlenemez.
- **1.0 öncesi hiç deprecation süresi tutmamak:** 0.x'te de gerçek ürün
  (K-163) ve ders içeriği vardır; bir alt sürüm serisi asgari süredir.

## Korpus etkisi

Golden ve anti-örnek korpusu değişmez. `dil::tedarik` kaldırması (DEP-004)
Zee kaynağına dokunmaz. Kapı `uyumluluk_testi` ile CI'dadır.
