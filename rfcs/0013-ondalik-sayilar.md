# RFC-0013 — Ondalık Sayılar

- **Durum:** **geçici kabul** (K-092 revizyonu — keyfî katsayı/ölçek ve
  deterministik sonsuz bölüm bağlamı gerçeklendi; onay kapısı usability)
- **Tarih:** 31 Ağustos 2026; K-092 revizyonu 1 Eylül 2026
- **İlgili günlük kayıtları:** K-028, K-092; RFC-0002 §6.3, RFC-0007 §4.1
- **Gerçekleme:** sözcükleyici (bitişik virgül, S033), tür sistemi
  (`Tur::Ondalik` + genişleme), `BigInt` katsayılı onluk çekirdek,
  biçimleyici; testler `ondalik_testi.rs` + çekirdek birim testleri

## Özet

Türkçede ondalık ayracı virgüldür (3,14); dilde virgül liste ayracıdır. Bu RFC
çakışmayı **bitişiklik kuralıyla** çözer ve tek kullanıcı-türü tanımlar:
**Ondalık** — keyfî hassasiyetli, ikilik kayan nokta sürprizi olmayan onluk
sayı. K-092 ile ilk bootstrap'ın dokuz hane/i64 sınırı dil sözleşmesinden
çıkarılmıştır.

## 1. Sözdizimi önerisi: bitişik virgül kuralı

> Rakam , rakam — aralarında BOŞLUK YOKSA ondalık sayıdır: `3,14`
> Virgülden sonra boşluk varsa liste ayracıdır: `3, 14`

```
pi 3,14 olsun
fiyatlar 2,5, 7,25, 10 listesi olsun     # üç öğe: 2,5 · 7,25 · 10
```

- Kural sözcükleyicidedir ve DETERMİNİSTİKTİR: `rakam ,\S rakam` → Ondalık
  tokenı; başka her virgül ayraçtır.
- Biçimleyici garantisi: liste virgüllerinden sonra HEP boşluk basılır
  (bugün de öyle), ondalık virgülünde asla — iki kullanım görsel olarak da
  ayrışır. `dil biçimle` sonrası belirsiz görünüm kalmaz.
- Yazım hatası koruması: `3 ,14` (virgülden önce boşluk, sonra bitişik rakam)
  yeni S-tanısı alır: "ondalık mı liste mi? Ondalıksa 3,14 bitişik yaz;
  listeyse virgülden sonra boşluk bırak."
- Türkçe imla korunur: çocuk okulda gördüğü `3,14`ü aynen yazar. Nokta
  (`3.14`) kabul edilMEZ — S001 tanısı özel öneriyle: "ondalık ayracı
  Türkçede virgüldür: 3,14".

**Elenen alternatifler:** (a) nokta ayracı — Türkçe imlaya aykırı, manifesto 1
ihlali; (b) `3 virgül 14` sözel biçimi — okunuşu doğal ama yazımı uzun ve
`virgül` kelimesini ayırırdı; okuma-yazma simetrisi bozuluyor; (c) liste
ayracını değiştirmek (`;`) — noktalama ekleme, daha kötü.

## 2. Tür ve anlam

- **Adı:** Ondalık (master plandaki GerçekSayı/Ondalık ikilisinden kullanıcıya
  yalnız Ondalık sunulur; ikinci bir örtük ikilik kayan nokta türü yoktur).
- **Gösterim:** imzalı keyfî uzunlukta tam sayı katsayı + onluk ölçek.
  Kaynaktaki tam ve kesir haneleri i64/i128 makine sınırına bağlı değildir.
  `0,1 ile 0,2 nin toplamı` TAM OLARAK `0,3`tür — çocuğa "bilgisayar
  0,30000000000000004 dedi" sürprizi yaşatılmaz.
- **Kanonik biçim:** gereksiz sağ sıfırlar atılır; ölçek en az birdir.
  Böylece `1,50 = 1,5`, ama Ondalık kimliği taşıyan tam değer `2,0` basılır.
- **Aritmetik:** mevcut genitif kalıplar aynen (`a ile b nin toplamı/farkı/
  çarpımı`, `a nın b ye bölümü`, `artır/azalt`, `böl`). Toplama, çıkarma ve
  çarpma tamdır. Paydası sadeleştirilince yalnız 2 ve 5 asal çarpanlarını
  taşıyan bölme de sonlu onluk olarak tamdır (`1,0 / 8 = 0,125`).
- **Sonsuz bölüm bağlamı:** onluk açılım sonsuzsa sonuç **34 anlamlı haneye**
  yuvarlanır. Yarımlar sıfırdan uzağa gider; bankacı yuvarlaması yoktur.
  Anlamlı hane baştaki sıfırları saymaz: `1,0 / 3` virgülden sonra 34 üç,
  `10,0 / 3` ise tam kısımdaki `3` dahil 34 hane üretir. Bu bağlam platform,
  debug/release ve işlemciye göre değişmez. 34; para/ERP kapasitesini,
  bilimsel ölçümlerde geniş güven payını ve makul kaynak kullanımını tek açık
  varsayılanda birleştirir. Daha uzun bir bağlam ileride ancak görünür bir API
  ile eklenebilir; sessiz küresel ayar olamaz.
- **TamSayı bölmesi:** iki TamSayı arasındaki mevcut tam bölüm davranışı
  değişmez (golden 04/12/14 korunur).
- **Karışım:** TamSayı → Ondalık genişlemesi kayıpsızdır ve örtük SERBESTTİR
  (`3,14 ile 2 nin çarpımı` çalışır — "tehlikeli implicit yok" ilkesini
  bozmaz çünkü kayıp imkânsız). Ters yön açık kalıp ister:
  `x in tam kısmı` (kırpma) ve `x in yuvarlanmışı` (en yakına).
- **Karşılaştırma:** değer üzerinden (`0,5 1 den küçükse` doğru); ölçek farkı
  eşitliği etkilemez (`1,50` = `1,5`).
- **Taşma sınırı:** Ondalık exact işlemleri sabit makine kelimesi taşması
  üretmez. C002; TamSayı i64 işlemi, TamSayıya açık daraltma, Süre i64 sınırı
  veya temsil ölçeğinin teorik metadata sınırı içindir. Kaynak/bellek tükenmesi
  dil içinde sessiz yuvarlama gerekçesi değildir.
- **Dönüşümler:** `yanıtın sayısı` TamSayı verir (mevcut); yeni kalıp
  `yanıtın ondalığı` Ondalık verir (C004 ailesi hata).
- **Tanı göçü:** S032 (dokuz kesir hanesi sınırı) üretimden kaldırılmış ve
  başka anlama verilememek üzere katalogda ayrılmıştır. S033 yazım
  belirsizliğini korur.

## 3. Bağlı türler (yol açılır)

- **Süre** (RFC-0011 `5 saniye içinde`): `yarım saniye` = `0,5 saniye` —
  Ondalık + birim kelimesi.
- **Para:** onluk taban parayı floating hatasız taşır: `19,99` tam saklanır.
  Para ayrı RFC (birim/kur), ama temeli budur.

## 4. Açık sorular

1. Bilimsel gösterim (`1,2 × 10⁻³` benzeri Türkçe yüzey) ayrı sözdizimi
   kararıdır; Ondalık kapasitesini değiştirmez.
2. Görünür, işlem-yerel özel bölüm bağlamı gerekirse v1 sonrasında ayrı RFC
   ister. Varsayılan 34 hane sessiz küresel ayarla değiştirilemez.
3. Binlik ayracı (1.000.000 yazımı Türkçede nokta!) — v1'de YOK; sayılar
   ayraçsız yazılır. İleride `1 milyon` sözel kalıbı düşünülebilir.
4. `yüzde 20` kalıbı (`0,2`ye açılım) — eğitimde çok değerli, v1.1 adayı.

## Dört soru süzgeci

Doğal ✓ (3,14 okuldaki yazımın kendisi) · Deterministik ✓ (bitişiklik kuralı
+ biçimleyici garantisi + özel tanılar) · Öğrenilebilir ✓ (0,1+0,2=0,3) ·
Savunulabilir ✓ (keyfî onluk katsayı; exact işlemler; tek açık sonsuz bölüm
bağlamı; para, ERP ve ölçüm aynı türde).

## Korpus etkisi

Golden 32 market hesabı (`2,5 × 19,99`) ve `ondalik_testi.rs` temel yüzeyi
korur. K-092 regresyonları 9+ ve 30+ haneli sabit, büyük katsayı, çok küçük
değer, sonlu/sonsuz bölüm, negatif metin dönüşümü, JSON/para gösterimi,
karşılaştırma ve i64'e açık daraltma taşmasını kapsar. Anti-örnek `3.14`
özel virgül önerisini korur.
