# RFC-0013 — Ondalık Sayılar

- **Durum:** **geçici kabul** (31 Ağu 2026 — bitişik virgül kuralı, onluk
  aritmetik, S032/S033, biçimleyici desteği gerçeklendi ve testli; onay
  kapısı: usability + korpus genişletmesi)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** RFC-0002 §6.3, RFC-0007 §4.1
- **Gerçekleme:** sözcükleyici (bitişik virgül, S032/S033), tür sistemi
  (Tur::Ondalik + genişleme kuralları), yorumlayıcı (i128 onluk çekirdek),
  biçimleyici; testler `ondalik_testi.rs` (18) — günlük kaydı K-028

## Özet

Türkçede ondalık ayracı virgüldür (3,14); dilde virgül liste ayracıdır. Bu RFC
çakışmayı **bitişiklik kuralıyla** çözer ve tek kullanıcı-türü önerir:
**Ondalık** — tam onluk aritmetikli (binary float sürprizsiz) sayı.

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
  yalnız Ondalık sunulur; ikili kayan nokta gerekirse ileri düzey paket işi).
- **Gösterim:** onluk sabit hassasiyet — 64 bitlik tam sayı gövde + onluk
  ölçek (ölçek üst sınırı 9 hane). Gerekçe: `0,1 ile 0,2 nin toplamı` TAM
  OLARAK `0,3`tür — çocuğa "bilgisayar 0,30000000000000004 dedi" sürprizi
  yaşatılmaz (determinizm + öğrenilebilirlik ilkeleri).
- **Aritmetik:** mevcut genitif kalıplar aynen (`a ile b nin toplamı/farkı/
  çarpımı`, `a nın b ye bölümü`, `artır/azalt`, `böl`). Taşma C002; ölçek
  taşması da C002 ailesi.
- **Bölme:** Ondalık bölme onluk 9 haneye yuvarlar (yarımdan yukarı —
  "bankacı yuvarlaması" değil, okulda öğretilen kural; spesifikasyonda sabit).
  TamSayı bölmesi TAM KALIR (mevcut davranış değişmez; golden 04/12/14 korunur).
- **Karışım:** TamSayı → Ondalık genişlemesi kayıpsızdır ve örtük SERBESTTİR
  (`3,14 ile 2 nin çarpımı` çalışır — "tehlikeli implicit yok" ilkesini
  bozmaz çünkü kayıp imkânsız). Ters yön açık kalıp ister:
  `x in tam kısmı` (kırpma) ve `x in yuvarlanmışı` (en yakına).
- **Karşılaştırma:** değer üzerinden (`0,5 1 den küçükse` doğru); ölçek farkı
  eşitliği etkilemez (`1,50` = `1,5`).
- **Gösterim/basım:** gereksiz sondaki sıfırlar atılır (`1,50` → `1,5`;
  tam değer `2,0` → `2` DEĞİL: Ondalık kimliği korunur, `2,0` basılır —
  tür yazımda görünür kalır).
- **Dönüşümler:** `yanıtın sayısı` TamSayı verir (mevcut); yeni kalıp
  `yanıtın ondalığı` Ondalık verir (C004 ailesi hata).

## 3. Bağlı türler (yol açılır)

- **Süre** (RFC-0011 `5 saniye içinde`): `yarım saniye` = `0,5 saniye` —
  Ondalık + birim kelimesi.
- **Para:** onluk taban parayı floating hatasız taşır: `19,99` tam saklanır.
  Para ayrı RFC (birim/kur), ama temeli budur.

## 4. Açık sorular

1. Ölçek üst sınırı 9 hane yeterli mi (bilimsel iş için değil — hedef kitle
   kararı; bilimsel hesap paketi f64'ü `tehlikeli`siz ayrı türle sunabilir).
2. Binlik ayracı (1.000.000 yazımı Türkçede nokta!) — v1'de YOK; sayılar
   ayraçsız yazılır. İleride `1 milyon` sözel kalıbı düşünülebilir.
3. `yüzde 20` kalıbı (`0,2`ye açılım) — eğitimde çok değerli, v1.1 adayı.

## Dört soru süzgeci

Doğal ✓ (3,14 okuldaki yazımın kendisi) · Deterministik ✓ (bitişiklik kuralı
+ biçimleyici garantisi + özel tanılar) · Öğrenilebilir ✓ (0,1+0,2=0,3) ·
Savunulabilir ✓ (onluk sabit nokta; para/süre yolu açık).

## Korpus etkisi

Kabulde: golden 04'e Ondalık bölme örneği, yeni golden "31-ondalik.dil"
(market hesabı: 2,5 kilo × 19,99), günlük K-028 kaydı. Anti-örneklere
`3.14` nokta kullanımı eklenir (özel tanılı).
