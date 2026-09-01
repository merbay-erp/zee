# RFC-0011 — Structured Concurrency

- **Durum:** **geçici kabul** (K-085 son tarih çekirdeği; K-090 deterministik
  görev scheduler'ı, sözcüksel sahiplik ve kardeş iptali gerçeklendi.)
- **Tarih:** 31 Ağustos 2026; K-090 revizyonu 1 Eylül 2026
- **İlgili günlük kayıtları:** K-023, K-085, K-090
- **İlgili golden programlar:** 26 (eşzamanlı görevler), 27 (zaman aşımı)
- **Normatif gerçekleme:** spec/09 ve spec/14

## Özet

zee eşzamanlılığı blok-kapsamlıdır. Her görev bir üst kapsama aittir; kapsam
başarıyla bitmeden görevler birleştirilir, hata/son tarih halinde kardeşler
iptal edilir. Sahipsiz görev yoktur. K-090 scheduler'ı tek iş parçacıklı ve
işbirliklidir: görevler `bekle` noktalarında kaynak sırasıyla el değiştirir.
Bu, gerçek eşzamanlı ilerleme sağlarken data race sınıfını doğurmaz.

## 1. Temel model ve K-023'ün cevabı

```dil
eşzamanlı olarak
    profil müşterinin profilini getir
    faturalar müşterinin faturalarını getir
    cihazlar müşterinin cihazlarını getir

hepsini bekle
"Profil: " ile profil yaz
```

`eşzamanlı olarak` bloğundaki her satır `<ad> <ifade>` biçiminde bir GÖREV
BAĞLAMASIDIR. `hepsini bekle`den sonra bu adlar sıradan sonuç değerleridir.
Birleşik sonuç isteniyorsa açıkça bir yapı/liste kurulur; örtük `sonuç` yoktur.

Görev bildirimi dış ortamın değer kopyasını alır. Sonuç adına birleştirmeden
önce erişim T033'tür. Görevin ilk poll'u `hepsini bekle` cümlesinde başlar;
bildirim ile birleştirme arasındaki ana-kapsam cümleleri önce çalışır. Bu sıra
gözlemlenebilir ve spec/14'te TANIMLIDIR.

## 2. Sahiplik ve kapsam

1. Görev grubu, bildirildiği sözcüksel kapsamın çocuğudur.
2. Aynı kapsamda ikinci grup açılmadan önce ilk grup birleştirilir.
3. `hepsini bekle` yalnız aynı kapsamın açık grubunu kapatabilir.
4. Açık grupla kapsamdan çıkmak, değer/hata döndürmek ya da programı bitirmek
   T051'dir. Açık grup olmadan birleştirme de T051'dir.
5. Ana kapsam başka bir runtime hatasıyla birleştirmeye ulaşamazsa çocuk
   kayıtları iptal edilerek düşer; kapsam dışında çalışmaya devam edemez.

Bu kuralların sonucu yapısaldır: derlenmiş bir programda dangling task üretme
yolu yoktur. İç içe görev grubu ancak çağrılan işlemin kendi sözcüksel
kapsamında kurulabilir; onun sahipliği de aynı kurallarla kapanır.

## 3. Deterministik yürütme

Scheduler bağlayıcı biçimde şöyledir:

1. Hazır görevler kaynak sırasıyla poll edilir.
2. Görev bir `bekle` noktasına, sonuca veya hataya kadar ilerler.
3. Hazır iş yoksa saat en yakın uyanma anına tek adımda ilerler.
4. Aynı anda uyanan görevlerde yine kaynak sırası kazanır.
5. Sonuçlar kaynak sırasıyla görev adlarına bağlanır.

İki görev 2 saniye ve 1 saniye bekliyorsa toplam sanal süre 2 saniyedir;
beklemeler eski sıralı modeldeki gibi toplanmaz. Aynı anda yalnız bir görev
çalışır. Görev yerel ortamları kopyadır; ortak IO etkileri kesin scheduler
sırasıyla görünür. `eylem` transaction'ı savepoint sahipliği karışmasın diye
tek atomik scheduler dilimidir.

Çok çekirdekli paralellik ayrı bir v2+ kararıdır. Bu RFC'nin "eşzamanlı"
sözü, bekleme sürelerinin örtüştüğü gerçek işbirlikli ilerlemedir; işletim
sistemi thread'i ya da paylaşılan bellek yarışı sözü değildir.

## 4. Hata ve iptal

1. İlk yönetilmemiş görev hatası aynı tanı koduyla birleştirmeden dışarı
   yayılır; mesaj görev sahibini ve iptal edilen kardeşleri gösterir.
2. Tamamlanmamış kardeş future'ları hemen düşürülür. `bekle` sonrasındaki
   cümleleri ve yan etkileri çalışmaz.
3. Hata öncesinde tamamlanmış sıradan IO etkileri geri alınmaz. Uygulama
   değişikliği geri alınacaksa RFC-0015 `eylem` transaction'ı kullanılır.
4. Yönetilen hata istenirse görev ifadesi `dene`li bir `Sonuç` üretir; bu
   durumda görev olağan değerle tamamlandığından kardeş iptali olmaz.
5. Birleştirmeden önce ana kapsam runtime hatasıyla biterse henüz poll
   edilmemiş görevler etkisiz biçimde iptal edilir.
6. Görev içindeki `programı N ile bitir`, Ç000 nöbetçisini ve çıkış kodunu
   bozmadan köke taşır; kardeşleri iptal ederek programı olağan bitirir.

## 5. Son tarih

```dil
5 saniye içinde
    eşzamanlı olarak
        profil müşterinin profilini getir
        faturalar müşterinin faturalarını getir
    hepsini bekle
yetişmezse
    "Zaman aşımı, sonra tekrar dene" yaz
```

K-085'in mutlak son tarihi görev ağacına miras kalır. `bekle`, kalan süreye
kırpılır. Son tarih dolunca iç Ç001 sahiplik nöbetçisi metinsel olarak
bozulmadan doğru `içinde/yetişmezse` sahibine ulaşır ve kardeşler iptal edilir.
İç içe tarihlerde en erken tarih kazanır; iç kol dış iptali yakalayamaz.

## 6. İşbirlikli sınır

İşlem çağrısı ne kadar iç içe olursa olsun dilin `bekle` cümlesi scheduler
noktasıdır. HTTP isteği senkron adaptöre girmeden kardeşlere bir tur verir;
adaptörün içi ve platformun diğer senkron/iptal edilemeyen çağrıları (bazı
DNS/dosya işlemleri gibi) dönene kadar atomik dilimdir. Dönüşte son tarih
denetlenir ve süre aşılmışsa sonraki cümle çalışmaz. Bu sınır spec/09 ile
aynıdır. Async host IO ayrı adaptör/API çalışmasıdır; dilin sahiplik modelini
değiştirmez.

## 7. Açık sorular

1. `ilkini bekle`/yarış gerekli mi; kaybedenin sonucu ve iptali nasıl görünür?
2. Görev sonucuna kısmi erişim ve akış/stream v2'de nasıl sahiplenilir?
3. Döngüyle dinamik sayıda görev başlatma hangi sonuç koleksiyonunu üretir?
4. Uzun saf hesaplar için açık bir `sıra ver` noktası gerekli mi?
5. Synchronous host IO için taşınabilir async adaptör sözleşmesi nasıl olmalı?

## Dört soru süzgeci

Doğal ✓ (`hepsini bekle`) · Deterministik ✓ (kaynak sıralı tek-thread
scheduler) · Öğrenilebilir ✓ (kapsam = yaşam süresi) · Savunulabilir ✓
(sahipsiz görev yok, hata/son tarih bütün ağaca yayılır).

## Korpus ve conformance etkisi

Golden 26 görev sonuçlarının birleştirmeden sonra değer olduğunu korur; golden
27 son tarih sahibini kanıtlar. `ag_ve_esz_testi` ayrıca farklı beklemelerde
çıktı izini ve en-uzun-süre saatini, kardeş iptalini, dış deadline yayılımını,
iç görev ağacının dış kardeşe sıra vermesini, T033'ü ve T051 kapsam
olumsuzlarını sabitler. Davranış değişikliği bu testler, spec/14 ve sürüm notu
birlikte güncellenmeden yapılamaz.
