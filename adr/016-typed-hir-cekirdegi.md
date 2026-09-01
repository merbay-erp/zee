# ADR-016 — Typed HIR çekirdeği ve aşamalı runtime geçişi

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-103, B-019, V1-P0-14

## Bağlam

ADR-014 semantic kimlikleri kurdu, ADR-015 derleyici veri fazlarını ayrı
tiplerde görünür yaptı. Buna rağmen `BaglanmisProgram` yalnız checker
tarafından yerinde değiştirilmiş AST taşıyordu. Bir sonraki geçiş veya runtime,
bir ifadenin kanıtlanmış türünü yeniden çıkarımlamak ve kaynak adı ile semantic
kimlik arasından seçim yapmak zorunda kalabilirdi.

## Karar

Başarılı checker geçişi artık AST'den ayrı bir `HirProgram` üretir. Her
denetlenmiş ifade şu bilgileri taşır:

- program içi kararlı `HirDugumId`,
- açık `Tur`,
- varsa `SymbolId`, `IslemId` veya `YapiId` bağı.

`BaglanmisProgram` doğrudan `Program` değil, zorunlu `HirProgram` sahibidir.
Kaynak AST; tanı, formatter/LSP ve v0 Rust API uyumluluğu için HIR içinde
salt-okunur korunur. HIR ayrıca kimlikten canonical sembol adına, işlem
tanımına ve yapı tanımına güvenli erişim sunar.

Checker sırasında kullanılan düğüm adresi yalnız AST düğümü ile HIR kaydını
aynı süreç içinde eşleyen private locator'dır; semantic identity veya public
API değildir. AST, checker sonrasında kutulu HIR sahipliğine taşınır ve bağlı
program klonlanamaz; böylece locator HIR ömrü boyunca yer değiştirmez.

## Aşamalı geçiş sınırı

K-103 HIR üretimini ve faz sahipliğini kurar. B-019 ancak bağlı runtime
değişken, işlem ve yapı kararlarını kaynak adından değil HIR bağından aldığında
kapanır. Bu nedenle K-103 sonunda B-019 ve V1-P0-14 **kısmen** durumundadır.
Raw `Program` runtime'ı v0 embedding uyumluluğu olarak ad-temelli kalabilir;
standart bağlı hat HIR tüketicisine dönüştürülecektir.

B-020 her semantic düğümde zorunlu source span'i ayrıca kurar. Span eksikliği,
HIR'ın tür ve bağ gerçeğini AST'ye geri itmek için gerekçe değildir.

## Değişmezler

1. Başarılı checker bilgisi olmadan `HirProgram` kurulamaz.
2. Her kaydedilmiş ifade açık `HirDugumId` ve `Tur` taşır.
3. Değişken/çağrı/yapı ifadelerinin HIR bağı çıplak kaynak adı değildir.
4. `BaglanmisProgram`, HIR sahipliğini atlayıp yalnız AST taşıyamaz.
5. Eski `Program` adaptörü ancak HIR üretildikten sonra faz bilgisini siler.
6. Runtime HIR geçişi tamamlanmadan B-019 kapalı gösterilmez.

## Sonuçlar

- Checker tür sonucu artık geçici dönüş değeri olmaktan çıkıp sonraki fazın
  kalıcı girdisidir.
- Semantic bağlar AST alanlarında uyumluluk için dursa da tek gelecek yönü HIR'dır.
- İki davranış ve bir mimari test HIR tür/bağ kayıtlarını ve zorunlu faz
  sahipliğini korur.
- Zee kaynak semantiği değişmediğinden yeni normatif spec gerekmez.
