# ADR-028 — Kanonik `.zep` yol ve Unicode güvenlik profili

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-117, B-030, B-031, V1-P1-08
- **Normatif ayrıntı:** RFC-0020 §3/9, spec/18

## Bağlam

`.zep` kimliği arşivin tam byte dizisidir. Aynı görünen Türkçe dosya adı bir
dosya sisteminde NFC, diğerinde NFD olarak dönerse yalnız sıralama ve metadata
dışlama aynı paket kimliğini üretmeye yetmez. Gelen paketi sessizce normalize
etmek de iki farklı imzalı byte dizisini tek mantıksal yola dönüştürerek
tekillik ve denetim sınırını bulanıklaştırır.

Unicode ayrıca `/`, `\\`, `.` ve `:` gibi yol metakarakterlerine benzeyen
işaretler ile görünmez yön denetleyicileri taşır. Bugün zararsız görünen bir
arşiv girdisi gelecekte farklı platform/çıkarıcı katmanında ayraç, traversal
veya görsel aldatma olarak yorumlanabilir. Bu güvenlik sözünün tek tek dağınık
testlerle değil, sürümlü ve incelenebilir bir korpusla korunması gerekir.

## Karar

1. Paket üreticisi her gerçek dosya yolu bileşenini UAX #15 NFC'ye çevirir,
   sonra `/` ile birleştirir ve UTF-8 byte sırasıyla sıralar. NFC sonrası aynı
   yola inen iki dosya fail-closed reddedilir.
2. Paket tüketicisi yol byte'larını değiştirmez. Arşivdeki yol zaten NFC değilse
   paket reddedilir; böylece tek mantıksal yolun tek kanonik byte gösterimi olur.
3. `.zep` v1, Unicode 17.0 UTS #39 verisinde `/`, `\\`, `.`, `:` iskeletine
   giden karakterleri, tam genişlikli `/`/`.` eşlerini ve görünmez bidi/biçim
   denetleyicilerini yolun herhangi bir yerinde yasaklar. ASCII `/` yalnız
   bileşen ayıracıdır; ASCII `.` yalnız normal dosya adının parçası olabilir,
   tek başına `.`/`..` bileşeni olamaz.
4. Unicode güvenlik kümesi `.zep` v1 sözleşmesinin parçasıdır. Unicode veri
   sürümü veya kabul kümesi sessiz dependency güncellemesiyle değiştirilemez;
   RFC/spec, korpus ve conformance fixture'ı birlikte incelenir.
5. `compiler/tests/fixtures/zep-kanonik-v1.hex`, Türkçe Unicode adlı gerçek bir
   proje ağacının beklenen `.zep` byte'ıdır. Aynı test GitHub CI'da Linux,
   macOS ve Windows üzerinde koşar. `zep-saldiri-korpusu/yollar-v1.tsv` 80
   kalıcı yol saldırısını, ayrı yapısal testler de limit/sıra/tekillik/fazladan
   byte vakalarını korur.
6. UAX #15'i yeniden gerçeklemek yerine `unicode-normalization 0.1.25`
   (Unicode 17.0) `Cargo.lock` ile sabitlenir ve native+WASM kapılarından geçer.

## Reddedilen seçenekler

- **Gelen paketi NFC'ye çevirip kabul etmek:** İmzalanmış byte kimliğini
  mantıksal kimlikten ayırır ve normalizasyon çakışmasını saklar.
- **Bütün yolu NFKC yapmak:** Uyumlu ama kanonik olarak farklı karakterleri de
  dönüştürür; meşru dosya adını gereksiz yere yeniden yazar. NFC + açık
  güvenlik reddi daha dar ve denetlenebilirdir.
- **Yalnız çıkarma anında denetlemek:** Doğrulanmış cache'e tehlikeli yolu
  kabul eder ve her çıkarıcının aynı güvenlik politikasını yinelemesini ister.
- **Platformun dosya adı davranışına güvenmek:** Linux/macOS/Windows arasında
  normalizasyon ve yasak karakter davranışı aynı değildir; paket kimliği host
  davranışına bırakılamaz.

## Sonuçlar

- Aynı Türkçe kaynak ağacı Tier-1 platformlarda aynı `.zep` byte'ına iner.
- Üretici kullanıcıya görünür adı NFC'ye kanonikler; yalnız normalizasyon
  çakışmasında yayın durur ve hangi dosyanın seçileceğini tahmin etmez.
- Çok dilli bazı görsel-confusable karakterler paket yolunda bilinçli olarak
  kullanılamaz. Kaynak içeriği ve dil tanımlayıcıları bu ADR'nin kapsamı değildir.
- Yeni Unicode sürümüne geçiş sıradan crate bump'ı değil, paket uyumluluk
  incelemesidir.

## Dayanaklar

- [Unicode UAX #15 — Unicode Normalization Forms](https://www.unicode.org/reports/tr15/)
- [Unicode UTS #39 — Unicode Security Mechanisms](https://www.unicode.org/reports/tr39/)
- [UTS #39 Unicode 17.0 confusables verisi](https://www.unicode.org/Public/security/latest/confusables.txt)
