# K-016 — İşlem çağrısı V1 karar paketi

- Durum: **KARAR AÇIK — gerçek katılımcı verisi bekleniyor**
- Protokol sürümü: **K016-1 (1 Eylül 2026; sayım başlamadan donduruldu)**
- Backlog: B-001 · RFC: RFC-0006 · Normatif geçici yüzey: spec/02
- Deney uygulaması: [usability-kiti.md](usability-kiti.md)
- Anonim sonuçlar: [usability-sonuclari/](usability-sonuclari/README.md)

Bu paket, çağrı sözdiziminin alışkanlıkla veya yalnız parser kolaylığıyla
kalıcılaşmasını engeller. Buradaki teknik envanter tamamdır; gerçek çocuk ve
profesyonel cevabı uydurulamaz. Ham formlar ve önceden ilan edilmiş eşik
olmadan RFC-0006 tam kabul edilmez.

## 1. V1'in tek yüzeyle karşılaması gereken bağlamlar

Seçilecek genel çağrı biçimi en az şu bağlamların tamamını deterministik
karşılamalıdır:

1. **Değere bağlama:** çağrı sonucunu `ortalama` adına bağlama.
2. **Cümle çağrısı:** değer döndürmeyen `selamla` işlemini iki argümanla çağırma.
3. **İç içe ifade:** bir çağrı sonucunu başka çağrının argümanı veya aritmetik/
   koşul parçası yapma.
4. **Dönüş:** çağrı sonucunu işlemden doğrudan döndürme.
5. **Özyineleme:** aynı biçimin temel durumdan sonraki özyinelemeli adımda
   çalışması.
6. **Tanım sırası:** başlık ön-taraması sayesinde çağrının tanımdan önce ya da
   sonra aynı AST/semantiği üretmesi.
7. **Çok-tokenli argüman:** `sayıların adedi`, `yanıtın ondalığı` veya başka
   çağrı gibi bir bölgenin tek argüman kalması.

Yalnız ilk bağlamda doğal görünen bir kart, tek V1 çağrı yüzeyi olmaya yeterli
değildir.

## 2. Bugünkü gerçek

Geçici A yüzeyi şöyledir:

```dil
ortalama notlar için ortalamayı hesapla olsun
"Ayşe" ve 10 ile selamla
```

Parser bütün işlem başlıklarını dosya/birim ayrıştırmasından önce toplar, en
uzun işlem adını önce eşler ve işlem adından hemen önce `için|ile` ister.
Argümanlar `ve` ile ayrılan tam ifade bölgeleridir. Bu yapı ifade, cümle,
özyineleme ve iç içe çağrıda çalışır; ancak `için` döngüyle, `ile` birleştirme/
aritmetikle yüklüdür ve değer bağlamadaki `... hesapla olsun` ritmi doğal
bulunmayabilir.

Tür denetimi kararı çağrı yazımından ayrıdır. Public birim/paket işlemi açık
parametre+dönüş sözleşmesiyle çağrı sırasından bağımsızdır. Yerel çıkarımlı
işlemlerin çağrı sırası etkisi B-007'nin ayrı işidir; K-016 sonucu onu sessizce
çözülmüş saymaz.

## 3. Kör gösterilecek adaylar

Katılımcı kartlarında “mevcut”, “önerilen”, A/B/C gibi değer yüklü etiket
bulunmaz; yalnız rastgele `Kart 1/2/3` yazılır.

### Aday A — argüman önce, noktalamasız

```dil
ortalama notlar için ortalamayı hesapla olsun
```

Geneldir ve bugün çalışır. Ana risk, ayraç yükü ve `hesapla olsun` okunuşudur.

### Aday B — çağrı sonra sonuç bağlama

```dil
notlar için ortalamayı hesapla, sonucu ortalama olsun
```

Sesli okunuşu güçlüdür; fakat tek başına iç içe ifade, koşul ve doğrudan dönüş
biçimi vermez. Bu aday 15 kişide çok güçlü çıkarsa doğrudan “ek sözdizimi”
olarak alınmaz: K-097/RFC-0021 katmanları yeniden açılır ve aynı zihinsel
modeli genel bir ifade biçimine dönüştüren ikinci tasarım turu yapılır.

### Aday C — açık ifade sınırı

```dil
ortalama (notlar için ortalamayı hesapla) olsun
```

İç içe ifadeyi genel çözer; fakat noktalama-minimum ve ilk öğrenme hedefinde
parantez maliyeti taşır. C seçilirse parantezin bütün expression grammar
katmanındaki anlamı RFC-0021 revizyonuyla birlikte dondurulur.

“Hiçbiri; ben şöyle yazardım: ____” her kartta gerçek bir seçenektir.

## 4. Deney tasarımı

- **Katılımcı:** 10 öğrenci (8–14) + 5 profesyonel. İsim/e-posta alınmaz;
  `C01..C10`, `P01..P05` anonim kimliği kullanılır. Çocuk oturumu veli/onam ve
  kurum kurallarına uyar; açık izin yoksa ses/video kaydı yapılmaz.
- **Önce serbest üretim:** Hiç Zee çağrı örneği göstermeden değere bağlama ve
  cümle çağrısı yazdırılır. İcat edilen yazım birebir korunur.
- **Sonra kör kart:** Beşer katılımcı `A→B→C`, `B→C→A`, `C→A→B` sırasını alır.
  Uygulayıcı kartın mevcut olup olmadığını söylemez.
- **Her aday için üç görev:** değere bağlama; satırı kendi cümlesiyle açıklama;
  iç içe sonuç kullanımını yazma veya “bu kartla yazamıyorum” deme.
- **Öğretim sonrası tekrar:** Tek cümlelik kural okunur, aynı türde yeni örnek
  verilir. İlk sezgi ile öğrenilebilirlik ayrı kaydedilir.
- **Son tercih:** En doğal, en anlaşılır ve yazarken seçilecek kart ayrı ayrı
  sorulur; bunların aynı olmak zorunda olmadığı belirtilir.

Golden 12/14, K-016 serbest üretim ve kör kartlar tamamlanmadan gösterilmez;
aksi halde mevcut A yüzeyi katılımcıya öğretilmiş olur.

Dile adını veren 4 yaşındaki çocuk, isterse yalnız “Merhaba” ve sesli okuma
için sıcak/formative pilot olabilir; 8–14 yaş karar örneklemine sayılmaz.
Pilot protokolde değişiklik gerektirirse K016-2 önce belgelenip dondurulur;
farklı protokol sürümlerinin sayıları tek eşikte birleştirilmez.

## 5. Önceden taahhütlü karar eşiği

Bir aday tek V1 yüzeyi olarak önerilebilmek için bütün koşulları sağlamalıdır:

1. Öğretim sonrası değere bağlama + gösterilen satırı doğru açıklamada en az
   **12/15**, çocuklarda **7/10**, profesyonellerde **4/5** doğru zihinsel model.
2. Son “yazarken seçerim” tercihinde toplam en az **9/15** ve iki grupta da
   salt çoğunluk.
3. Teknik envanterdeki yedi bağlamın tamamı tek genel grammar kuralıyla,
   bağlama göre ikinci bir çağrı sözdizimi eklemeden ifade edilebilmeli.
4. Golden/anti-example prototipinde tek AST, güvenilir tanı ve formatter
   round-trip kanıtı bulunmalı.

Sonuç:

- **A tüm eşikleri geçerse:** RFC-0006/spec-02 tam kabul adayı olur; K-016
  golden/anti-example ve öğretim metniyle dondurulur.
- **C tüm eşikleri geçerse:** RFC-0021 expression grammar revizyonundan sonra
  migration yapılır; A aynı ana sürümde sessizce silinmez.
- **B doğal tercih eşiğini geçerse:** B teknik genellik koşulunu bugün
  sağlamadığı için karar kapanmaz; RFC-0021 yeniden açılır ve genel ikinci
  tur yapılır.
- **Hiçbiri geçmezse veya gruplar ayrışırsa:** K-016 açık kalır; veri/tasarım
  turu genişletilir. Çocuk sonucu profesyonel sonucu adına veya tersi yönde
  ezilmez.

## 6. Karar sonrası atomik değişiklik listesi

Sonuç geldiğinde tek toplu değişiklikte:

- anonim ham formlar ve sayım özeti;
- `kararlar/gunluk.md` yeni kayıt;
- RFC-0006 durum/gerekçe/migration;
- spec/02 grammar;
- parser + formatter + LSP;
- golden/anti-example ve olumlu/olumsuz testler;
- README, sürüm notu, v1 kapısı ve bu backlog

birlikte güncellenir. Gerçeklenmeyen aday belgede seçilmiş gösterilmez.
