# RFC-0019 — Değer Semantiği ve Gezme İmleci

- **Durum:** **geçici kabul** (K-093 makine sözleşmesi ve conformance kanıtı;
  tam kabul için çocuk/profesyonel usability oturumu bekleniyor.)
- **Tarih:** 1 Eylül 2026
- **İlgili günlük kayıtları:** K-034 (blok kapsamı), K-060 (yapı listeleri),
  K-074 (gezmede geri yazma), K-093 (değer-sonuç modeli)
- **Gerçekleme:** `cozumleyici.rs` (T053), `yorumlayici.rs` (gezme imleci)
- **Normatif kaynak:** [spec/17](../spec/17-deger-semantigi-ve-gezme.md)

## Özet

zee'de kullanıcı değerlerinin gizli, paylaşılan nesne kimliği yoktur. Bir adı
başka bir ada vermek, bir işlemi çağırmak veya listeye öğe eklemek değeri
**derin kopyalar**. Bu nedenle bir kopyayı değiştirmek ötekini uzaktan
değiştirmez.

`her X için` liste gezmesi bu genel kuralın tek açık yazma köprüsüdür: `X`
her turda öğenin kopyasını alır ve turun sonunda son değeri aynı liste sırasına
geri konur. Bu model programlama dilleri literatüründeki **değer-sonuç
(copy-in/copy-out)** parametresine denktir. Alan yazma ile yeniden bağlama aynı
sonucu verir:

```dil
her kutu için
    kutunun adedi 9 olsun   # alan yazma: kaynak öğe değişir

her sayı için
    sayı 0 olsun           # yeniden bağlama: kaynak öğe değişir
```

## 1. Neden paylaşılan referans değil?

Çocuk için “öteki adı değiştirdim, burası da değişti” görünmez bir tuzaktır;
profesyonel için ise sahiplik, alias analizi, eşzamanlılık ve serileştirmede
ek bir belirsizlik kaynağıdır. zee v1'in koleksiyon ve yapı kapasitesi için
nesne kimliği zorunlu değildir. Açık bir kimlik/paylaşımlı başvuru türü ileride
gerekirse adlandırılmış yeni tür ve ayrı RFC ister; sıradan değerlerin anlamını
geriye dönük değiştiremez.

Değer semantiği özellikle şunları sabitler:

- `yedek asıllar olsun` sonrasında iki liste bağımsızdır;
- yapı, sözlük, liste, Seçenek, Sonuç ve Hata içeriği de derin kopyalanır;
- işlem argümanı çağıranın değerine gizlice yazamaz;
- görev snapshot'ı RFC-0011/spec-14 ile aynı değer kopyası ilkesine dayanır.

## 2. Liste gezmesinin kesin algoritması

Kaynak, dilbilgisi gereği çözülmüş tek bir addır. Çalışma zamanı:

1. Kaynak listenin giriş anındaki öğelerini ve sırasını bir kez snapshot alır.
2. Her özgün sıra `i` için `X`, snapshot'taki `i` öğesinin derin kopyasına
   bağlanır.
3. Gövde yazılı sırayla çalışır. `X` aynı türde yeniden bağlanabilir veya bir
   yapıysa alanına yazılabilir.
4. Tur normal bittiğinde ya da `döndür` ile ayrıldığında `X`in son değeri canlı
   kaynak listenin `i` sırasına kopyalanır. `döndür` ifadesi önce değerlendirilir,
   sonra geri yazma yapılır ve akış çağırana yayılır.
5. Döngü adı blokla ölür; genel kapsam kuralı RFC-0004'tür.

Bu algoritma referans değildir: `X` ayrı değerdir. Ancak açık ve sınırlı geri
yazma adımı, “gezerken değiştirdim ama kayboldu” tuzağını kapatır.

## 3. Kaynak koleksiyon kilidi — T053

Snapshot sıralarını canlı koleksiyonla güvenle eşlemek için kaynak koleksiyon
gezme boyunca biçimsel olarak sabittir. Gövde içinde aynı kaynak:

- yeniden bağlanamaz (`sayılar ... listesi olsun`),
- büyütülemez (`sayılara ... ekle`),
- öğe/anahtar silemez,
- sözlük değerine yazamaz,
- ikinci kez iç içe gezilemez.

Bunlar çalışma zamanında sıra kaydırmak veya son yazanı rastlantısal seçmek
yerine derlemede **T053** verir. Tanı, eklenecek/silinecek değerleri ayrı bir
listede toplamayı ve gezme bitince uygulamayı önerir. Aynı değerden daha önce
alınmış bağımsız bir kopya bu kilide dahil değildir; onun değişmesi kaynağı
etkilemez.

İç içe aynı kaynak gerekiyorsa açık değer kopyası kullanılır:

```dil
ötekiler sayılar olsun
her sayı için
    ötekilerdeki her öteki için
        # iki bağımsız değer üzerinde çalışır
```

## 4. Sözlük gezmesi

Sözlük gezmesi giriş anındaki anahtarları ekleme sırasıyla snapshot alır ve
döngü adına Metin anahtar kopyası verir. Anahtar kaynak sözlüğe geri yazılmaz.
Kaynak sözlüğün kendisine yazma/silme ve aynı kaynağı iç içe gezme yine
T053'tür. Böylece liste ve sözlük için “gezerken kabın biçimi sabittir” diye
tek öğretilebilir kural vardır.

## 5. Seçeneklerin değerlendirilmesi

### A — yalnız alan yazma geri yansısın

Profesyonel referans dillerine tanıdıktır; ancak `her sayı için / sayı 0 olsun`
satırının sessizce kaybolması, K-074'te kapatılan tuzağı başka türde geri
getirir. Elendi.

### B — gerçek paylaşılan referans/nesne kimliği

Alan güncellemesini doğrudan kılar; fakat kopya/alias, yaşam süresi,
eşzamanlılık ve JSON kimliği gibi birbirine bağlı yeni sözler doğurur. v1 için
gereksiz ve başlangıç modeline ağırdır. Elendi.

### C — değer-sonuç imleci + sabit kaynak

Hem “bu öğeyi değiştiriyorum” zihinsel modelini korur hem genel değer
semantiğini bozmaz. T053 ile sıra kayması önceden ve öğretici biçimde
engellenir. Geçici kabul edilen model budur.

## 6. Usability kapısı

Makine kanıtı insan kanıtı değildir. `docs/usability-kiti.md` içindeki K-093
kartları, alan yazma ile yeniden bağlamanın beklenen etkisini ayrı ayrı ölçer.
Önceden belirlenen karar eşiği sağlanıp gerçek formlar depoya girene kadar
V1-P1-05 **AÇIK** kalır. Sonuç mevcut modeli reddederse RFC, spec ve testler
aynı değişiklikte revize edilir; veri sonradan kurala uydurulmaz.

## Dört soru süzgeci

Doğal — aday, usability bekliyor · Deterministik ✓ (snapshot + T053) ·
Öğrenilebilir — tek “öğe geri gider, kap sabit kalır” kuralı; ölçüm bekliyor ·
Savunulabilir ✓ (derin değer, sınırlı açık yazma köprüsü, alias yok).
