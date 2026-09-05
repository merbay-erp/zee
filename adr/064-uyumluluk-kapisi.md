# ADR-064 — Yürütülebilir uyumluluk ve deprecation kapısı

- **Durum:** kabul
- **Tarih:** 5 Eylül 2026
- **İlgili kayıt:** K-167, B-072, RFC-0028, spec/27, V1-P1-21

## Bağlam

Tanı kodları (ADR-025), morfoloji profili (RFC-0018), kanonik `.zep`
(ADR-028), Rust facade'ı (ADR-058) ve WASM ABI'si (ADR-039) ayrı ayrı
dondurulmuştu; fakat dilin kalıp kelimeleri, koşul yüklemleri ve CLI komutları
yalnız golden korpus ve LSP listesiyle örtük korunuyordu. Bir kelimenin sessizce
kaldırılması golden'ı kırmadan mümkündü; hangi yüzeyin hangi sürümde girdiği,
neyin ne zaman ve hangi göç yoluyla kaldırılacağı tek bir kayıtta yoktu.
Çalışma ağacı ise etiketlenmiş `0.7.0` kimliğini 126 commit boyunca taşıdı;
`dil sürüm`, LSP `serverInfo` ve SBOM builder'ı yayımlanmamış yüzeyi
yayımlanmış sürüm sanıyordu. Eski `dil::tedarik` cephesinin ömrü de B-072
olarak açıktı.

## Karar

1. **Dondurulmuş yüzey envanteri.** `compiler/tests/fixtures/dil-yuzeyi-v1.tsv`
   her kalıp kelimesini, koşul yüklemini ve CLI komutunu `aktif|kaldirildi`
   durumu ve giriş sürümüyle; kilit/ZEP/yayın zarfı biçimlerini, morfoloji/IO/
   eşzamanlılık profillerini, WASM ABI'sini, Rust facade'ını ve tanı kimlik
   şemasını exact kaynak kaydıyla listeler. Kelime kümeleri Rust kaynağından
   türetilip fixture ile birebir karşılaştırılır; diğer satırların exact kaydı
   kaynak dosyada aynen bulunmak zorundadır.
2. **Deprecation kaydı.** `docs/deprecation-kayitlari-v1.tsv` append-only
   `DEP-NNN` kimlikli kayıt taşır: yüzey, öğe, sınıf, duyuru sürümü, kaldırma
   sürümü, en az 20 karakterlik göç yolu, karar belgesi ve durum. Desteklenen
   yüzeyde kaldırma sürümü duyurudan en az bir alt sürüm serisi sonradır; tanı
   mezar taşları ve iç (`ic`) yüzeyler süre şartı taşımaz. Kaldırılan kelime
   fixture'da mezar taşı olarak kalır; ayrılmış her tanı kodu kayıt taşır;
   kaldırılan iç modül `lib.rs`te bulunamaz.
3. **Sürüm kimliği.** Etiket kesildikten sonra Cargo sürümü bir sonraki
   `X.Y.Z-dev` serisine çekilir; kapı çalışma ağacının en yüksek `v*` etiketinden
   büyük olmasını ister. `dil sürüm`, LSP ve SBOM aynı kaynağı okur.
4. **B-072 kapanışı.** `dil::tedarik` cephesi kaldırıldı (DEP-004). Kurulum
   katmanı fiziksel sahibi `artefakt_dogrulama/kurulum.rs` altına taşındı;
   production sahip sayısı 37'dir ve `tedarik` adı katman graph'ında yeniden
   doğamaz.

`uyumluluk_testi` bu dört maddeyi fail-closed doğrular; sentetik bozuk
fixture ve süresiz kaldırma reddi ayrı regresyondur.

## Reddedilen seçenekler

- **Yalnız golden korpus:** korpusun kullanmadığı kelime sessizce kaybolabilir.
- **Yalnız sürüm notu:** metin makinece doğrulanamaz; kaldırma ile duyuru
  arasındaki süre ve göç yolu kayıt dışı kalır.
- **Kaldırılan kelimeyi fixture'dan silmek:** yeniden kullanım ve tarih kaybı;
  ADR-025 mezar taşı ilkesi kelime yüzeyine de uygulanır.
- **`tedarik` cephesini süreli deprecation'a almak:** depo içi ve bilinen dış
  tüketicisi sıfırdır, facade zaten `doc(hidden)` internal sınıfıydı; süre
  tutmak yalnız ölü kodu ve bir katman sahibini yaşatırdı.

## Sonuçlar

- Yeni kalıp/koşul/komut eklemek fixture'a `aktif` satırı ve giriş sürümü
  ister; kaldırmak deprecation kaydı olmadan derlenemez.
- Kapı yalnız envanter ve süre sözüdür; kaldırılan biçimin tanı ve önerisiyle
  reddedildiği kanıt regression fixture'ında verilir (RFC-0028 §4).
- 1.0 sonrası edition/major geçişi RFC-0028'de tanımlıdır; bu ADR mekanizmayı
  kurar, grammar değişikliğine yetki vermez. CORE FREEZE aynen sürer.
