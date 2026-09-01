# RFC-0021 — İfade grameri büyüme mimarisi

- **Durum:** **geçici kabul** (K-097; mevcut yüzey ve büyüme kuralı testli)
- **Tarih:** 1 Eylül 2026
- **İlgili günlük kayıtları:** K-004, K-008, K-010, K-016, K-027, K-038, K-097
- **İlgili golden programlar:** 04, 05, 12, 14, 30, 32
- **Normatif karşılık:** spec/20
- **Gerçekleme:** `ayristirici.rs` içinde `ile_ifadesi`, `bolge_ifadesi`,
  `yapili_kalip`, `cagri_kalibi`, `kosul_atomu`, `kosul_ifadesi`

## Özet

Zee ifadeleri yeni özellik geldikçe bağımsız `if` dalları eklenen düz bir
kalıp listesi olarak büyümeyecek. Yüzey, en sıkıdan en gevşeğe şu katmanlara
ayrılır:

1. **primary** — sabit, ad, liste/yapı gibi temel değer;
2. **erişim/postfix** — iyelik alanı, koleksiyon özelliği, dönüşüm ve güvenli
   Seçenek/Sonuç erişimi;
3. **çağrı** — K-016'nın geçici işlem çağrısı yüzeyi;
4. **aritmetik** — toplam/fark/çarpım/bölüm/kalan isim kalıpları;
5. **birleştirme** — başka bir tam kalıbın parçası olmayan `ile` zinciri;
6. **karşılaştırma** — yüklem-sonlu eşit/büyük/küçük ve durum yüklemleri;
7. **boolean** — yalnız aynı tür `ve` ya da `veya` zinciri.

Bu RFC yeni kullanıcı sözdizimi eklemez. Bugünkü davranışı adlandırır,
belirsizlik politikasını dondurur ve expression parser'ın büyüme sözleşmesini
kurar. K-016 sonucu veya başka bir ifade yüzeyi ancak bu mimariyi RFC+spec+
conformance testiyle revize ederek yürürlüğe girebilir.

## 1. Bölge ve tam tüketim

Bir **ifade bölgesi**, cümle yüklemi (`olsun`, `yaz`, `döndür` gibi) veya
kalıbın açık ayıracı tarafından sınırlandırılan token dizisidir. Bir katman
yalnız bölgenin **tamamını** tanıyorsa başarı sayılır. Önek eşleşip artan
tokenları başka anlama bırakmak yasaktır; sonuç ya tek AST ya kararlı tanıdır.

Türkçe ekler ayrı operatör tokenı olmadığı için bu model klasik sembolik
önceliğin birebir kopyası değildir. Güçlü katmanlar, zayıf katmanın ayırıcı
gibi görünen kelimelerini kendi tam kalıbında tüketebilir. Örnek:

```dil
değer 2 için biri ekle ile 4 ün çarpımı olsun
```

Burada `2 için biri ekle` önce çağrıdır; dıştaki `... ile 4 ün çarpımı`
aritmetiktir. `ile`, çağrı/aritmetik tam kalıbı oluşmadığında birleştirme olur.

## 2. Normatif katmanlar

Aşağıdaki şema niyet grameridir; Türkçe ek yüzeyleri spec/03'ün sürümlü
morfoloji çözümüne bağlıdır:

```ebnf
değer-ifadesi      = birleştirme ;
birleştirme        = yapısal-ifade { "ile" yapısal-ifade } ;
yapısal-ifade      = aritmetik | çağrı | erişim-postfix | primary ;
aritmetik          = güçlü-ifade aritmetik-kuyruğu ;
güçlü-ifade        = çağrı | erişim-postfix | primary ;
çağrı               = [ argüman { "ve" argüman } ("için" | "ile") ] işlem-adı ;
argüman             = yapısal-ifade ;
koşul-ifadesi       = karşılaştırma { aynı-bağlaç karşılaştırma } ;
aynı-bağlaç         = "ve" | "veya" ;
```

Bir `koşul-ifadesi` içindeki bütün `aynı-bağlaç` seçimleri birebir aynı
olmalıdır; ilk bağlaç zincirin türünü sabitler.

`aritmetik-kuyruğu` bugünkü genitif isim kalıplarıdır: `... toplamı`,
`... farkı`, `... çarpımı`, `... bölümü`, `... bölümünden kalanı`.
`erişim-postfix`, bugünkü yüzeyde tek bir primary ya da desteklenen güçlü
bölge üzerine kurulur; genel ve sınırsız zincir sözü değildir.

## 3. Bağlama göre ayrılan kelimeler

- `ile`, tam çağrı/aritmetik/intrinsic kalıbının içindeyse o kalıbındır;
  aksi halde birleştirme katmanıdır.
- `ve`, çağrının açık argüman bölgesinde argüman ayırır; koşul bağlamında
  boolean VE'dir. Boolean bir çağrı argümanı bugün adlandırılmış ara değer
  ister; gizli öncelik uydurulmaz.
- `veya daha`, karşılaştırmanın `>=`/`<=` kalıbıdır; boolean `veya` diye
  bölünmez.
- `ve` ve `veya` aynı koşul zincirinde karışamaz (S030). Zee bunlardan birine
  örtük öncelik vermez; kullanıcı koşulu adlandırır veya basamaklara böler.
- İşlem adının yalnız **kuyruk eşleşmesi**, daha güçlü ve tam tüketilmiş
  primary/erişim/postfix ifadesini gölgeleyemez. Örneğin `işlem sayısı`
  görünür olsa bile `metnin sayısı` dönüşüm/postfix ifadesidir; çağrı değildir.
  Bütün bölge görünür işlem adıyla birebir aynıysa eski sıfır-argüman çağrısı
  korunur: `işlem metnin sayısı` tanımlıysa çıplak `metnin sayısı` o çağrıdır.

## 4. Bugünkü birleşebilirlik sınırı

| Birleşim | V1 öncesi gerçek | Kural |
|---|---|---|
| primary → erişim/postfix | destekli | tam tanınan özellik/alan kalıpları |
| erişim/postfix → aritmetik | destekli | güçlü sol bölge olarak |
| çağrı → aritmetik | destekli | güçlü sol bölge olarak |
| çağrı sonucu → erişim/postfix | desteklenmiyor | ara değer adlandırılır; S015 |
| aritmetik → karşılaştırma | genel değil | ara değer adlandırılır |
| karşılaştırma → boolean | destekli | yalnız homojen `ve` veya `veya` |
| parantezle serbest gruplama | desteklenmiyor | spec/02; K-016 sonucu beklenir |

Bu tablo eksikleri “gelecekte kesin eklenecek özellik” yapmaz. Yalnız parser'ın
bugün sessizce verdiğinden fazlasını vaat etmemesini ve gelecekteki değişimin
hangi katmanda yapılacağını açıklar.

## 5. Gerçekleme mimarisi ve geçiş

Elle yazılmış yüklem-sonlu recursive descent korunur (ADR-002). Pratt parser
zorunlu değildir; çünkü Zee'de bugünkü işlemler sembolik, serbest infix token
çifti değil, tam tüketilen Türkçe bölgelerdir.

K-097 öncelik düzeltmesinden sonra bootstrap gerçekleyicisi şu girişlere
eşlenir:

- `ile_ifadesi`: değer/birleştirme girişi;
- `bolge_ifadesi` + `yapili_kalip`: primary, erişim/postfix, çağrı ve
  aritmetik uyumluluk yolu;
- `cagri_kalibi`: çağrı katmanı;
- `kosul_atomu`: karşılaştırma katmanı;
- `kosul_ifadesi`: boolean katmanı.

B-005 parçalama turunda bu sınırlar ayrı modül/fonksiyon sahipliklerine
taşınacaktır. Bu RFC'nin kapanışı dosya taşımaya değil, mimari kararın ve
conformance matrisinin sabitlenmesine bağlıdır. Yeni özellik B-005'i beklemeden
gelirse bile `yapili_kalip` içine gelişigüzel dal eklenemez; aşağıdaki uzatma
protokolünü karşılamak zorundadır.

## 6. Uzatma protokolü

Her yeni ifade RFC'si veya RFC revizyonu aynı değişiklikte şunları belirtir:

1. ait olduğu tek katman ve güçlü/zayıf komşuları;
2. bölge sınırı ve tam tüketim kuralı;
3. `ile`, `ve`, `veya`, iyelik ve işlem-adı kuyruğuyla çakışma matrisi;
4. en az bir olumlu birleşim ve iki olumsuz/belirsiz örnek;
5. AST düğümü veya mevcut düğüme normatif lowering;
6. parser+formatter parse-equivalence ve tanı kodu etkisi;
7. RFC, spec, golden/anti-example ve sürüm notu güncellemesi.

Bir özellik bu yedi kanıt olmadan yalnız “mevcut kalıplardan önce denenen” dal
olarak eklenemez.

## 7. Tanı politikası

- Boş değer bölgesi S013; eksik `ile` tarafı S014; yan yana/tanınmayan bölge
  S015'tir.
- Tanınmayan koşul S016; çağrı ayıracı/argüman bölgesi S019/S020'dir.
- Homojen olmayan boolean zincir S030'dur.
- Kısmi kalıp başka AST'ye düşmez. Yeni katman yeni tanı gerektiriyorsa kod
  kataloğu aynı değişiklikte güncellenir; mevcut kodun anlamı değiştirilmez.

## Dört soru süzgeci

1. **Türkçe doğal mı?** Evet — öncelik sembol sırasından değil Türkçe tam
   tamlamanın bölge sınırından doğuyor.
2. **Deterministik mi?** Evet — tam tüketim, en uzun işlem adı ve karışık
   bağlaç yasağı tek AST/tek tanı sağlar.
3. **Öğrenilebilir mi?** Evet — başlangıçta katman adı öğretilmez; kullanıcı
   “önce küçük ifadeyi adlandır” kuralıyla her sınırı aşabilir.
4. **Profesyonel ölçekte savunulabilir mi?** Evet — yeni özellik için açık
   yer, çakışma matrisi ve conformance zorunluluğu parser büyümesini denetler.

## Alternatifler

- **Her yeni kalıbı dev fonksiyonun başına eklemek:** Yerel olarak hızlı,
  küresel önceliği görünmez yaptığı için reddedildi.
- **Hemen Pratt parser:** Sembolik operatörlü dillerde güçlü; bugünkü Türkçe
  tam-bölge kalıplarında tek başına çözüm değil. İleride açık operator
  yüzeyi gelirse ADR-002 yeniden açılır.
- **Her belirsizliği parantezle çözmek:** K-016/C adayının sonucunu peşinen
  seçer ve spec/02'yi kırar; kullanıcı kanıtı olmadan reddedildi.

## Korpus etkisi

Golden dosyalar değişmedi. Katman sırası, postfix/işlem-adı kuyruğu, tam
sıfır-argüman, en-uzun-çağrı ve fail-closed birleşim sınırı
`compiler/tests/ifade_grameri_testi.rs` içinde bağımsız conformance kanıtıdır.

Tek gözlenebilir düzeltme, `sayısı` gibi görünür bir işlem-adı kuyruğunun
`metnin sayısı` doğal postfix ifadesini yanlış S019'a çevirememesidir. Bu
önceden geçerli programı bozmaz: bütün bölgeyle birebir eşleşen sıfır-argüman
işlem çağrısı ayrı erken adımla korunur; daha önce hatalı olan çakışma artık
doğru postfix AST'sini üretir.
