# zee dil turu

Dilin bütün yüzeyi, kolaydan zora, çalışan örneklerle. Buradaki her parça
ya golden korpustan alınmıştır ya da regression testlerinde koşar — süs
örnek yoktur. Denemek için: `playground/zee-playground.html` (çift tıkla)
ya da `dil çalıştır dosya.dil`.

Okuma anahtarı: zee'de anahtar kelime yoktur; **cümlenin ne yaptığını son
kelimesi söyler** (yaz, olsun, tekrarla, döndür...). Türkçenin yüklem-sonlu
doğası, dilin ayrıştırma kuralıdır.

## 1. Yazmak

```
"Dünyaya merhaba" yaz
```

Parçalar `ile` ile birleşir:

```
isim "Ayşe" olsun
"Benim adım " ile isim yaz
```

Metin içinde kaçışlar: `\"` (tırnak), `\\` (ters bölü), `\n` (yeni satır).

## 2. Değerler: `olsun`

```
yaş 10 olsun
şehir "İzmir" olsun
pi 3,14 olsun
hazır doğru olsun
```

- Bir adın türü sonradan değişmez (`yaş "on" olsun` → T002).
- Ondalık sayı **bitişik virgülle** yazılır: `3,14` ondalıktır,
  `3, 14` iki öğeli listedir (RFC-0013). `0,1 ile 0,2 nin toplamı`
  tam olarak `0,3`tür — onluk aritmetik, ikilik sürpriz yok.
- Negatif sabit işaretle bitişiktir: `-3`, `-3,14`.

## 3. Sormak

```
"Adın ne?" diye sor
"Merhaba " ile yanıt yaz
```

Cevap her zaman `yanıt` adındadır. Sayı gerekiyorsa: `yanıtın sayısı`.
Cevabın sayı olacağından emin değilsen §12'deki `dene` kalıbına bak.

## 4. Aritmetik

Genitif kalıp — Türkçedeki gibi "ikisinin toplamı":

```
toplam 3 ile 7 nin toplamı olsun
fark 10 ile 4 ün farkı olsun
çarpım fiyat ile adedin çarpımı olsun
bölüm sayının 2 ye bölümü olsun
```

Cümle biçimleri de var:

```
toplamı notla artır
sayacı 1 azalt
ortalamayı toplamı adede böl
```

TamSayı gerektiğinde Ondalığa kendiliğinden genişler; tersi bilinçlidir:
`ondalığın tam kısmı`, `ondalığın yuvarlanmışı`.

## 5. Koşullar

Koşul da yüklem-sonlu: `...se/...sa` ile biter.

```
yaş 8 veya daha büyükse
    "Programlamaya başlayabilirsin!" yaz
değilse
    "Biraz daha oyun zamanı" yaz
```

Zincir: `değilse tahmin gizliden büyükse ...` — else-if.
Bağlaçlar: `ve` / `veya` (aynı koşulda karıştırmak S030 hatasıdır:
belirsizlik dile giremez; basamakla). Olumsuzlama: `... değilse`.
Sık koşullar: `çiftse/tekse`, `boşsa`, `içeriyorsa`, `eşitse`,
`küçükse/büyükse`.

## 6. Döngüler — dört tane

```
10 kez tekrarla
    "Merhaba" yaz

1 den 100 e kadar her sayı için
    sayı çiftse
        sayıyı yaz

sayaç 0 dan büyük olduğu sürece
    sayacı 1 azalt

bildi doğru olana kadar tekrarla
    "Tahminin?" diye sor
    ...
```

Sayı sabitine gelen ek ayrı yazılır: `1 den`, `100 e` (K-011).
Döngü gövdesinde doğan ad gövdeyle ölür (blok kapsamı, K-034);
dıştaki ada atama kalıcıdır — biriktirme böyle yapılır.

## 7. Listeler

```
sayılar 3, 7, 1, 9 listesi olsun
sayılara 5 ekle

"Adet: " ile sayıların adedi yaz
"İlk: " ile sayıların ilki yaz
"Son: " ile sayıların sonu yaz

her sayı için
    sayıyı yaz
```

`her sayı için` gezerken listeyi **örtük çoğuldan** bulur: `sayı` → `sayılar`
(K-013). Süzme + biriktirme:

```
geçenler boş liste olsun
her not için
    not 50 veya daha büyükse
        geçenlere notu ekle
```

Bir listenin bütün öğeleri aynı türdendir (T011).

## 8. Sözlükler

```
yaşlar boş sözlük olsun
yaşların "Ayşe" değeri 10 olsun

yaşlarda "Ayşe" varsa
    yaşların "Ayşe" değeri yaz

yaşlardaki her ad için
    ad ile ": " ile yaşların ad değeri yaz
```

Sıra korunur: eklediğin sırayla gezersin (determinizm).

## 9. Metin işlemleri

```
cümle "Türkçe düşün, Türkçe yaz" olsun

cümlenin uzunluğu yaz
cümlenin büyük harflisi yaz
cümlenin küçük harflisi yaz

cümle "Türkçe" içeriyorsa
    "Geçiyor" yaz

her kelime için        # cümlenin kelimeleri listesinden
    kelimeyi yaz
```

Büyük/küçük dönüşümü Türkçe kurallıdır: İ↔i, I↔ı.

## 10. İşlemler

Tanım `işlem` ile başlar; parametreler gövdenin başında `... al` satırlarıdır;
değer `döndür` ile çıkar. Çağrı: argümanlar + `için`/`ile` + işlem adı.

```
işlem karesini hesapla
    sayıyı al
    sonucu sayı ile sayının çarpımı olsun
    sonucu döndür

kare 4 için karesini hesapla olsun
```

Çok parametre: `ve` ile ayrılır, her dilim tam bir ifade olabilir (K-038):

```
"Ayşe" ve 10 ile selamla
tabanın tam kısmı için yuvarla
```

İmza ilk çağrıda sabitlenir (v0 monomorfizmi, T017); işlem çağrıdan sonra
da tanımlanabilir (adlar ön-taranır).

### Özyineleme

```
işlem faktöriyelini hesapla
    sayıyı al

    sayı 1 den küçükse
        1 döndür
    bir_eksiği sayı ile 1 in farkı olsun
    alt bir_eksiği için faktöriyelini hesapla olsun
    sonucu sayı ile altın çarpımı olsun
    sonucu döndür
```

Kural: **temel durum önce** — özyinelemeli çağrıdan önce en az bir dal
değer döndürmüş olmalı (T035). Karşılıklı özyineleme serbesttir.
Derinlik 500'ü aşarsa Türkçe tanı gelir (C019), makine taşmaz.

## 11. Seçenek: değer var ya da yok

`null` yok; "olmayabilir" bir türdür. `yok döndür` içeren işlem
Seçenek üretir:

```
işlem ilk çift sayıyı bul
    sayıları al
    her sayı için
        sayı çiftse
            sayıyı döndür
    yok döndür

bulunan sayılar için ilk çift sayıyı bul olsun

bulunan varsa
    "Bulundu: " ile bulunanın değeri yaz
değilse
    "Çift sayı yok" yaz
```

`bulunanın değeri` yalnız `varsa` dalında (ya da `yoksa`nın `değilse`
dalında) yazılabilir — dal dışında derleme hatasıdır (T036). Boş değeri
açmak diye bir çalışma hatası bu dilde yoktur.

## 12. Sonuç: başarabilir ya da başaramaz

Beklenen hatalar `... dene` ile Sonuç'a dönüşür:

```
sonuç "veriler.txt" dosyasını okumayı dene olsun

sonuç başarılıysa
    sonucun değerini yaz
değilse
    "Okunamadı: " ile sonucun hatası yaz
```

`yanıtın sayısını almayı dene` — kullanıcı girdisini güvenle sayıya çevirir.
Kendi işleminden hata döndürmek: `"sıfıra bölünmez" hatasını döndür`.
`değeri`/`hatası` erişimi Seçenek'teki gibi daraltmayla korunur (T036).

## 13. Yapılar

```
yapı Öğrenci
    ad Metin
    yaş TamSayı

ayşe yeni Öğrenci olsun
ayşenin adı "Ayşe" olsun
ayşenin yaşı 10 olsun
```

Alan erişimi iyelik ekiyle: `ayşenin adı`.

## 14. Desen eşleştirme

```
şekle göre
    "daire" ise
        "Köşesi yok" yaz
    "kare" ise
        "4 köşesi var" yaz
    değilse
        "Bu şekli tanımıyorum" yaz
```

## 15. Test yazmak

Testler dilin parçasıdır; `dil dene dosya.dil` koşar, playground da koşar:

```
test "kare doğru hesaplanır"
    kare 4 için karesini hesapla olsun
    kare 16 ya eşit olmalı
```

Testler taze ortamda ve hermetik IO ile koşar: rastgelelik, saat ve
dosyalar test dünyasından gelir — aynı test her makinede aynı sonucu verir.

## 16. Dosyalar, CSV, JSON

```
satırlar "siir.txt" dosyasının satırları olsun
"günlük.txt" dosyasına "Bugün hava güzeldi" yaz
"günlük.txt" dosyasına "Yarın da güzel olsun" ekle

tablo "notlar.csv" dosyasından okunan tablo olsun
tablodaki her satır için
    toplamı satırın "not" değeriyle artır

kişi "kisi.json" dosyasından okunan veri olsun
"Ad: " ile kişinin "ad" değeri yaz
```

## 17. Tarih, saat, süre

```
bugün bugünün tarihi olsun
"Yıl: " ile bugünün yılı yaz
yarın bugünün 1 gün sonrası olsun

yarım saniye bekle
```

Süre birinci sınıftır: `5 saniye`, `yarım saniye`.

## 18. Ağ ve eşzamanlılık

```
cevap "https://ornek.dev/durum" adresinden gelen yanıt olsun
"Durum: " ile cevabın durum kodu yaz

8080 kapısında sunucu başlat
"/durum" adresine istek geldiğinde
    "çalışıyor" yanıtını gönder
```

Eşzamanlı görevler ve zaman aşımı:

```
eşzamanlı olarak
    profil "Ayşe" için profili getir
    faturalar "Ayşe" için faturaları getir

hepsini bekle
profil yaz

5 saniye içinde
    veri "https://ornek.dev/rapor" adresinden gelen yanıt olsun
yetişmezse
    "Zaman aşımı, sonra tekrar dene" yaz
```

Görev adlarına `hepsini bekle`den önce erişim derleme hatasıdır (T033).

## 19. Fiziksel dünya (ESP32)

```
kapı açıksa
    kırmızı ışığı yak
değilse
    yeşil ışığı yak
```

Donanım yokken simülatörle çalışır; testte sensörler sahtedir.

## 20. Birimler

Dosya = birim. `hesap_araclari.dil` içindeki işlemler:

```
hesap_araclari birimini kullan

öde etiket için kdvli fiyatı hesapla olsun
```

Tanımlar görünür, birimin üst düzey cümleleri kapsüllüdür; ad çakışması
sessiz gölgelemez, hatadır (A008).

## 21. Komut satırı

```
argümanlar komut satırından gelenler olsun

argümanlar boşsa
    "Kullanım: selamla <isim> ..." yaz
    programı bitir

her argüman için
    "Merhaba " ile argüman yaz
```

## 22. Hata aldığında

Her tanı: **kod + Türkçe açıklama + konum + öneri**. Örnek:

```
HATA T036
"deneme" başarısız olabilir: değeri ancak "başarılıysa" dalında alınır.
Öneri: Önce kontrol et: deneme başarılıysa
Ayrıntı için: dil hata T036
```

`dil hata <kod>` her kodu açıklar; katalog: [hata-katalogu.md](hata-katalogu.md).

## 23. Araç kutusu

| Komut | Ne yapar |
|---|---|
| `dil yeni <ad>` | testli başlangıç projesi |
| `dil çalıştır <dosya>` | çalıştırır |
| `dil dene <dosya>` | testleri koşar |
| `dil biçimle <dosya>` | resmi biçime getirir |
| `dil denetle <dosya>` | çalıştırmadan bütün hataları listeler (`--json`) |
| `dil hata <kod>` | hata kodunu açıklar |

Editör desteği: `dillsp` LSP sunucusu ([editors/](../editors/)) +
VS Code renklendirme. Kurulumsuz deneme: [playground](../playground/).

---

Sözdiziminin *neden* böyle olduğunu merak edersen: her kararın gerekçesi
[kararlar/gunluk.md](../kararlar/gunluk.md)'de, tasarımın tamamı
[rfcs/](../rfcs/) altında.
