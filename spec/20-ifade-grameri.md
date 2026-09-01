# 20 — İfade grameri ve katman önceliği

Normatif kaynak: RFC-0021, ADR-002, RFC-0006, RFC-0005. Tanı kodları:
S013–S016, S019–S020, S030.

## Temel sözleşme

Bir ifade bölgesi ya bütünüyle tek bir AST olarak ayrıştırılır ya kararlı bir
tanı üretir. Bir kalıbın yalnız önekini tüketip kalan tokenları başka bir
yoruma bırakmak **YASAK**tır. Aynı kaynak ve aynı görünür işlem adı kümesi
daima aynı AST/tanıyı üretir (**TANIMLI**).

## Katman sırası

En güçlüden en zayıfa sıra şöyledir:

| Sıra | Katman | Bugünkü yüzey örneği |
|---:|---|---|
| 1 | primary | `3`, `3,14`, `"metin"`, `ad`, `boş liste` |
| 2 | erişim/postfix | `yanıtın sayısı`, `listenin adedi`, `sonucun değeri` |
| 3 | çağrı | `notlar için ortalamayı hesapla` |
| 4 | aritmetik | `a ile b nin toplamı`, `x in y ye bölümü` |
| 5 | birleştirme | `"Merhaba " ile ad` |
| 6 | karşılaştırma | `yaş 8 den büyükse`, `sonuç başarılıysa` |
| 7 | boolean | `A ve B`, `A veya B` |

“Güçlü” katman, zayıf katmanın ayıracı gibi görünen kelimeyi kendi tam
kalıbında tüketebilir. Bu nedenle çağrı/aritmetik içindeki `ile`, otomatik
birleştirme değildir. Bütün bölge görünür işlem adıyla birebir aynıysa bu,
geriye uyumlu sıfır-argüman **çağrı atomu**dur; aşağıdaki postfix önceliği
yalnız işlem-adı kuyruğunun önde artan token bırakması halinde uygulanır.

## Ayrıştırma yordamı

1. Cümle yüklemi kendi değer veya koşul bölgesini ayırır.
2. Bütün bölge görünür işlem adıyla birebir aynıysa sıfır-argüman çağrısı
   kurulur; en uzun tam ad kazanır.
3. Aksi halde değer bölgesi önce tamamını tüketen primary/postfix, sonra
   parametreli çağrı, ardından aritmetik kalıplarına sınanır. İşlem-adının
   yalnız kuyruk eşleşmesi primary/postfix'i gölgeleyemez.
4. Bunların hiçbiri bütün bölgeyi tüketmezse dış `ile` parçaları ayrı yapısal
   bölgeler olarak ayrıştırılır ve birleştirme AST'si kurulur.
5. Koşul bölgesinde `ve` ya da `veya` parçaları önce ayrılır; her parça tam
   karşılaştırma olmalıdır. `veya daha` karşılaştırma kalıbının içidir.
6. Aynı zincirde `ve` ve `veya` karışırsa öncelik uydurmak **YASAK**tır;
   S030 üretilir.
7. Çağrı tanıma görünür işlem adlarını en uzun addan kısaya dener. Argüman
   bölgesinin sonunda `için`/`ile`, argümanların arasında `ve` bulunur.

## Birleşebilirlik

- Erişim/postfix ve çağrı, desteklenen aritmetik kalıbın güçlü operandı
  olabilir (**TANIMLI**).
- Karşılaştırmalar homojen boolean zincir oluşturabilir (**TANIMLI**).
- Çağrı sonucuna doğrudan postfix ekleme ve genel aritmetik sonucu üzerinde
  doğrudan karşılaştırma bugün **AÇIK** değildir; **DESTEKLENMEZ**. Kullanıcı
  sonucu önce bir ada bağlar. Bu bölgeler sessizce farklı AST'ye çevrilemez.
- Parantezle serbest gruplama spec/02 gereği **YASAK**tır. K-016 bunu
  değiştirirse RFC-0006, RFC-0021, bu bölüm ve conformance aynı committe
  değişir.
- Çağrı argümanı içindeki `ve` argüman ayırıcıdır. Boolean değeri argüman
  yapmak için bugün ara ad **ZORUNLU**dur.

## Conformance örnekleri

```dil
# çağrı, dış aritmetikten güçlüdür
değer 2 için biri ekle ile 4 ün çarpımı olsun

# erişim/postfix, dış aritmetikten güçlüdür
değer metnin sayısı ile 2 nin çarpımı olsun

# `sayısı` adlı işlem görünür olsa da postfix anlamı değişmez
değer metnin sayısı olsun

# bütün bölge tam işlem adıysa sıfır-argüman çağrısı korunur
seçilen metnin sayısı olsun

# karşılaştırmalar boolean zincirden önce kurulur
x 3 den büyükse ve y 20 den küçükse
```

Şu biçimler geçerli bir yeni anlam kazanmaz:

```dil
# S030 — ve/veya arasında örtük öncelik yok
x 1 e eşitse ve y 2 ye eşitse veya z 3 e eşitse

# S015 — çağrı sonrası genel postfix henüz tanımlı değil
adet sayıları ver adedi olsun
```

Bağlayıcı yürütülebilir kanıtlar:
`compiler/tests/ifade_grameri_testi.rs` ve
`compiler/tests/bicimleyici_testi.rs`. İkinci kapı, 33 golden programın
dağınık-boşluk varyantında biçimleme öncesi/sonrası tam parser token izini
(`SatirSonu` ve girinti yapısı dahil) eşitler ve iki tarafı da ayrıştırır.

## Yeni ifade yüzeyi kapısı

Yeni yüzey; katmanını, tam bölge sınırını, ayıraç çakışmalarını, AST/lowering
sonucunu, olumlu/olumsuz conformance'ı ve formatter parse-equivalence etkisini
RFC-0021 §6 uyarınca aynı atomik değişiklikte göstermeden yürürlüğe giremez.
