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
  tam olarak `0,3`tür — onluk aritmetik, ikilik sürpriz yok. Ondalıkta
  dokuz-hane/makine-sayısı sınırı yoktur; çok büyük para ve çok küçük ölçüm
  değerleri aynı türde kayıpsız taşınır. Yalnız sonsuz açılımlı bölme
  deterministik 34 anlamlı haneye yuvarlanır (K-092/spec-16).
- Negatif sabit işaretle bitişiktir: `-3`, `-3,14`.

### Ekli adlar ve `zee-tr-1`

Tanımlayıcı eki doğal Türkçe gibi bitişik alır: `sayaç → sayacı`,
`kitap → kitabı`, `fiyat → fiyatıyla`. Derleyici tahmin etmez; `zee-tr-1`
profilindeki bütün kök adaylarını kapsamdaki adlarla karşılaştırır. Tek aday
çözülür, birden fazlası A002'dir. Örneğin hem `sayac` hem `sayaç` tanımlıysa
`sayacı` sessizce birini seçmez.

İyelik+hâl zinciri iki katmana kadar tanımlıdır: `zarından`, `fiyatıyla`.
Editörde adı değiştirdiğinde ekler de yeni köke uyar; `fiyatıyla` adı `elma`
olursa `elmasıyla` üretilir. Bir kelimenin bütün yapısal çözümlerini görmek
için `dil morfoloji sayacı`, etkin tabloyu görmek için `dil morfoloji` yaz.
Yeni projeler profili `proje.dil` içinde sabitler; böylece gelecekteki bir
derleyici güncellemesi eski kaynağın anlamını sessizce değiştiremez.

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

Kalan (mod) da okul diliyle: `17 nin 5 e bölümünden kalanı` → `2`.
Kalan hiç negatif olmaz (okul kuralı, K-046).

TamSayı gerektiğinde Ondalığa kendiliğinden genişler; tersi bilinçlidir:
`ondalığın tam kısmı`, `ondalığın yuvarlanmışı`. Para gösterimi:
`tutarın kuruşlusu` → `"1824,50"`; binlikli: `tutarın binlikli kuruşlusu`
→ `"1.234.567,89"` (Türk yazımı, K-075). Her değerin
resmî metin hali: `sayının metni` → `"42"` (K-066).

### İfadeler nasıl bağlanır?

Günlük kullanımda öncelik tablosu ezberlemen gerekmez. Dilin kesin düzeni,
küçük parçadan büyük cümleye şöyledir: temel değer → özellik/erişim → işlem
çağrısı → aritmetik → metin birleştirme → karşılaştırma → `ve`/`veya`.
Örneğin `2 için biri ekle ile 4 ün çarpımı` önce çağrıyı, sonra çarpımı kurar.

Bugün tanımlanmamış iki uzun ifadeyi parantezle zorlamak yerine sonucu bir ada
bağla ve sonraki satırda kullan. `ve` ile `veya` aynı koşulda karışmaz; dil
gizli öncelik uydurmaz. Bu büyüme sözleşmesinin ayrıntısı
[RFC-0021](../rfcs/0021-ifade-grameri-mimarisi.md) ve
[spec/20](../spec/20-ifade-grameri.md) içindedir.

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
`küçükse/büyükse`. Mantıksal bir ad tek başına koşuldur: `hazır ise` /
`hazır değilse` (K-044).

## 6. Döngüler — dört tane

```
10 kez tekrarla
    "Merhaba" yaz

1 den 100 e kadar her sayı için        # 5 ten 1 e: geri sayar (K-068)
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
(K-013). Döngü adı öğenin yazılabilir imlecidir; yeniden bağlarsan kaynak
listedeki aynı sıra değişir (K-093):

```dil
her sayı için
    sayı 0 olsun

sayıların json metni yaz       # [0,0,0,0,0]
```

zee değerleri bağımsız kopyalardır: `yedek sayılar olsun` sonrasında yedeği
değiştirmek asıl listeyi değiştirmez. Gezme sırasında kaynak kabın kendisini
büyütmek, silmek, yeniden bağlamak veya aynı kaynağı iç içe gezmek T053'tür;
değişiklikleri ayrı listede toplayıp gezme bitince uygula. Bu tek kural sıra
kaymasını ve “hangi öğeye geri yazıldı?” belirsizliğini engeller.

Süzme + biriktirme:

```
geçenler boş liste olsun
her not için
    not 50 veya daha büyükse
        geçenlere notu ekle
```

Sıralama ve arkadaşları (K-056/058):

```
sıralı adların sıralanmışı olsun    # Metinler TÜRK ALFABESİYLE sıralanır
ters sıralının tersi olsun
sayılarda 5 varsa                   # liste üyeliği
    "beş listede" yaz
sayılardan 5 i sil                  # silme — yoksa sessiz (K-059)
defterden "elma" yı sil
```

Bir listenin bütün öğeleri aynı türdendir (T011). `boş liste`nin öğe
türü İLK eklemeyle belli olur (K-045): `adlara "Zeynep" ekle` → metin
listesi. Eklemeden okumaya kalkarsan derleyici "önce öğe ekle" der.

## 8. Sözlükler

```
yaşlar boş sözlük olsun
yaşların "Ayşe" değeri 10 olsun

yaşlarda "Ayşe" varsa
    yaşların "Ayşe" değeri yaz

yaşlardaki her ad için
    ad ile ": " ile yaşların ad değeri yaz
```

Sıra korunur: eklediğin sırayla gezersin (determinizm). `boş sözlük`ün
değer türü de ilk atamayla belli olur (K-045) — metin değerli şifre
defterleri serbest (bkz. projeler/gizli-dil.dil).

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

Metin cerrahisi (K-053):

```
parçalar satırın ";" ile parçaları olsun      # bölme (boş ayraç = harflere)
birleşik parçaların " ve " ile birleşmişi olsun
yeni cümlenin "kedi" yerine "köpek" değişmişi olsun
temiz cümlenin kırpılmışı olsun
harfler adın harfleri olsun                   # Liste<Metin>

dosya ".dil" ile bitiyorsa
    "zee kaynağı" yaz
```

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

Başlangıçta tür yazmak gerekmez; bu durumda imza çağrıdan çıkarılır. Birim,
paket ve uzun ömürlü API'de ise tam sözleşme zorunludur:

```
işlem yarısını bul
    sayıyı Ondalık olarak al
    Ondalık döndürür
    sonuç sayının 2 ye bölümü olsun
    sonucu döndür
```

Açık imzalı gövde hiç çağrılmasa bile denetlenir; imza sonraki çağrıyla
değişmez. Değer üretmeyen public işlem `değer döndürmez` yazar. Bildirilen dönüş
bütün gövde yollarıyla uyuşur ve hiçbir değer yolu sessizce sona düşemez
(T039–T042, K-086). TamSayı, Ondalık beklenen çağrıda hem statik olarak hem
runtime değeriyle Ondalığa genişler. `Ondalık listesi`, `Metin sözlüğü`,
`TamSayı seçeneği`, `Metin sonucu` ve yapı adları geçerli tür yazımlarıdır.
v1 public modeli bilinçli olarak monomorfiktir. İşlem çağrıdan sonra da
tanımlanabilir (adlar ön-taranır).

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
Bu eski biçim aynı mesajı göstermeye devam eder; arka planda `GENEL` kodlu
yapılandırılmış Hata üretir. Profesyonel biçim kod, neden ve veri taşır:

```zee
bilgi boş sözlük olsun
bilginin "dosya" değeri "ayarlar.dil" olsun
"AYAR_OKUNAMADI" kodlu "Ayarlar yüklenemedi" hatasını alt_hata nedeniyle bilgi verisiyle döndür
```

Başarısız dalda `hata sonucun hatası olsun` denebilir. Ardından `hatanın kodu`,
`hatanın mesajı`, `hatanın nedeni` (Seçenek\<Hata\>) ve `hatanın verisi`
okunur; kod `göre / ise` ile eşlenebilir. `hata yaz` yalnız anlaşılır mesajı
gösterir, `hatanın json metni` bütün zinciri taşır. `değeri`/`hatası` erişimi
Seçenek'teki gibi daraltmayla korunur (T036).

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

Yapı LİSTELERİ kayıt tablosudur (K-060):

```
öğrenciler boş liste olsun
öğrencilere ayşeyi ekle          # öğe türü: Öğrenci

her öğrenci için
    öğrencinin adı yaz

öğrencilerin json metni yaz      # [{"ad":"Ayşe",...},...]

her öğrenci için                 # gezerken değiştir → listeye yansır (K-074)
    öğrencinin yaşı 11 olsun
```

Alan yazma ile `öğrenci başka_öğrenci olsun` yeniden bağlaması aynı
değer-sonuç kuralına tabidir: turun son `öğrenci` değeri kaynak listedeki aynı
sıraya geri yazılır. Önceden başka ada alınmış liste/yapı kopyaları değişmez.

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
    kare 17 ye eşit olmamalı        # olumsuz doğrulama (K-071)
    çıktı "16" içermeli             # metin içerme doğrulaması
```

Testler taze ortamda ve hermetik IO ile koşar: rastgelelik, saat ve
dosyalar test dünyasından gelir — aynı test her makinede aynı sonucu verir.
Tohum, sanal saat ve sahte dünya davranışı
[`zee-io-1` profilinde](deterministik-io-profili.md) sürümlüdür.

## 16. Dosyalar, CSV, JSON

```
satırlar "siir.txt" dosyasının satırları olsun
"günlük.txt" dosyasına "Bugün hava güzeldi" yaz
"günlük.txt" dosyasına "Yarın da güzel olsun" ekle

tablo "notlar.csv" dosyasından okunan tablo olsun
tablodaki her satır için                       # hücreler METİN (K-062)
    puan satırın "not" değerinin sayısı olsun  # sayıya bilinçli çevir
    toplamı puanla artır

kişi "kisi.json" dosyasından okunan veri olsun
"Ad: " ile kişinin "ad" değeri yaz

"veri.json" dosyasına kişinin json metni yaz   # yazma (K-054)
"yedek.csv" dosyasına tablonun csv metni yaz   # CSV yazma (K-058)
```

Gerçek dosya sisteminde `yaz` ve `ekle` K-084/RFC-0016 ile atomiktir:
okuyucu yarım dosya görmez, iki zee sürecinin eklediği satırlar kaybolmaz ve
commit öncesi hata eski içeriği korur. Runtime aynı klasördeki gizli
`.zee-yazma-kilidi` dosyasını kendine ayırır.

## 17. Tarih, saat, süre

```
bugün bugünün tarihi olsun
"Yıl: " ile bugünün yılı yaz
yarın bugünün 1 gün sonrası olsun

yarım saniye bekle
```

Süre birinci sınıftır: `5 saniye`, `yarım saniye`.

Playground'da görünen tohum `zee-io-1` profilini besler; aynı tohum ve aynı
rastgele çağrı sırası her platformda aynı diziyi verir. Takvim saati ile
deadline için kullanılan, beklemeyle ilerleyen tekdüze saat ayrıdır. Bu
rastgelelik oyun/simülasyon içindir; güvenlik belirteçleri OS CSPRNG'si kullanır.

## 18. Ağ ve eşzamanlılık

```
cevap "http://127.0.0.1:8081/durum" adresinden gelen yanıt olsun
"Durum: " ile cevabın durum kodu yaz

8080 kapısında sunucu başlat
GET "/durum" adresine istek geldiğinde
    "çalışıyor" yanıtını gönder
```

Gerçek TCP dinleyicisi production sözleşmesi değildir ve güvenli varsayılanla
kapalıdır. Yalnız localhost eğitim/prototipi için açıkça
`dil çalıştır --deneysel-web program.dil` denir; CLI görünür uyarı verir.
Native bootstrap istemcisi bugün yalnız `http://` destekler; deadline yazılmasa
bile 30 saniyede durur ve başlıklar dahil 8 MiB'dan büyük yanıtı reddeder.
TLS istemci backend'i henüz verilmiş söz değildir; Zee TLS'yi elle yazmayacaktır.

Deneysel eşzamanlılık ve süre yüzeyi:

```
eşzamanlı olarak
    profil "Ayşe" için profili getir
    faturalar "Ayşe" için faturaları getir

hepsini bekle
profil yaz

5 saniye içinde
    veri "http://127.0.0.1:8081/rapor" adresinden gelen yanıt olsun
yetişmezse
    "Zaman aşımı, sonra tekrar dene" yaz
```

Görev adlarına `hepsini bekle`den önce erişim derleme hatasıdır (T033).
K-090 ile `hepsini bekle` gerçek bir işbirlikli birleştirmedir. Görevler tek
iş parçacığında kaynak sırasıyla ilerler; `bekle`ye gelen görev uyurken hazır
kardeşi çalışır. Örneğin 2 ve 1 saniyelik iki görev toplam 2 saniyede biter;
aynı girdiler aynı çıktı sırasını verir ve data race oluşmaz. İlk yönetilmemiş
görev hatası bekleyen kardeşleri iptal eder. Her grup aynı sözcüksel kapsamda
tek `hepsini bekle` ile kapanır; aksi T051'dir. K-085 son tarihi bütün görev
ağacına yayılır ve yalnız doğru `yetişmezse` sahibi çalışır. Ayrıntı:
RFC-0011, spec/09 ve spec/14.

Uygulama eylemi ve web adaptörü (K-087): iş kuralı HTTP bilmeyen, açık imzalı
bir `eylem`dir. Aynı eylem CLI, web, görev ya da test bağlamından çağrılır.
Rota yöntemi açıkça bağlar; GET/HEAD üzerinden ulaşılabilen yazma derlemede
reddedilir. Form ve sorgu verisi örtük `istek` sözlüğündedir:

```
eylem notu kaydet
    notu Metin olarak al
    değer döndürmez
    "notlar.txt" dosyasına notu ekle

POST "/kaydet" adresine istek geldiğinde
    "yönetici" yetkisi gerekli
    "not" alanı gerekli
    yeni isteğin "not" değeri olsun
    yeni ile notu kaydet
    "/" adresine yönlendir

# listede: satırın html güvenlisi  ← kullanıcı verisi kaçışlanır
```

Her eylem çalışma hatasında ya da başarısız `Sonuç` dönüşünde dosya
savepoint'ini geri alır. Süreç çökmesinde çok-dosyalı tek commit sözü yoktur;
dosya başına K-084 atomikliği geçerlidir. Ayrıntı:
[spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md) ve
[spec/12](../spec/12-web-guvenlik-profili.md).

Önekli rota (K-055/K-087): `GET "/yazi/" önekli adrese istek geldiğinde` — kimliği
metinden çıkar: `kimlik yolun "/yazi/" yerine "" değişmişi olsun`.

Güvenli oturum profili (K-088): form GET'i sunucu oturumuna bağlı CSRF
belirteci üretir; giriş Argon2id doğrular ve oturum kimliğini döndürür:

```
GET "/giris" adresine istek geldiğinde
    csrf csrf belirteci olsun

POST "/giris-yap" adresine istek geldiğinde
    herkese açık
    "parola" alanı gerekli
    verilen parola_özeti ile doğrulanıyorsa
        "Zeynep" kullanıcısını "yönetici" rolüyle oturuma al

POST "/cikis" adresine istek geldiğinde
    oturum gerekli
    oturumu kapat
```

Unsafe yöntemlerde `_csrf` otomatik zorunludur. Production kipinde
`dil çalıştır --web-proxy https://panel.example ...` kullanılır; runtime
yalnız loopback'teki HTTPS proxy'ye güvenir. `girisli-panel.dil` tam akışı,
`panel-not-defteri.dil` ise kimliksiz ama CSRF korumalı temel formu gösterir.
K-106 ile process içi depo 4096 toplam/1024 anonim kayıtla sınırlıdır; kota
dolunca en uzun süredir kullanılmayan anonim kayıt tahliye edilir. Anonim
10 dakika ve kimlikli 30 dakika ömür mutlaktır, erişim süreyi kaydırmaz. Bu
profil tek runtime process'i içindir; çok süreçli production ortak depo ister.

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

Projeler arası paylaşım pakettir. Uygulamanın `proje.dil` bildirimi:

```text
yerel_bağımlılıklar "../hesap" listesi olsun
```

Bağımlı projenin adı `hesap` ise uygulama kaynağı:

```text
hesap paketini kullan

sonuç toplam ver olsun
sonucu yaz
```

Listeyi güvenli biçimde eklemek için `dil ekle ../hesap` kullanılır: komut
yeni grafiği yazmadan önce çözer, yorumları korur, yolları sıralar ve kilidi
aynı işlemde yeniler. `dil kilitle .` ise var olan doğrudan ve geçişli yerel paketleri `proje.kilit` içinde
göreli yol, sürüm ve SHA-256 kaynak özetiyle sabitler. Yalnız doğrudan
bildirilen paket kullanılabilir; geçişli bağımlılığa gizlice uzanılmaz.
Paket içindeki birimler kendi kaynak klasöründen çözülür ve paketin üst düzey
cümleleri de birimlerde olduğu gibi kapsüllüdür (K-078). Dışa açılan bütün
işlemler parametre ve dönüş türünü kaynakta taşır; çağrı sırası paket API'sinin
anlamını değiştiremez (K-086/spec-10).

Ana kaynağın başlangıç düzeyi `<ad> al` işlemlerinde de K-121 ile bütün
erişilebilir çağrı kısıtları gövde ve HIR üretiminden önce birleştirilir.
TamSayı+Ondalık Ondalığa genişler; skaler, sayısal kapsayıcı ve çok parametreli
çağrılarda kaynak sırası yerel işlem imzasını değiştiremez (spec/04).

`dil paketler .` grafiğin doğrudan/geçişli ayrımını, sürümünü, göreli yolunu
ve kilit özetini görünür kılar. `dil çıkar hesap .` yalnız doğrudan paketi
kaldırır; ana projenin herhangi bir kaynağında `hesap paketini kullan`
kalıyorsa P010 verir ve hiçbir dosyayı değiştirmez. Başarılı kaldırmada bildirim
ile kilit birlikte güncellenir; başka bir paketin bağımlılığı olan paket
grafikte geçişli olarak kalabilir (K-080).

Yayınlanabilir bir proje `dil anahtar üret` ve `dil paketle` ile deterministik
`.zep`, SPDX SBOM, SLSA provenance ve Ed25519 imzalı bildirim üretir. Dosya
sistemi yolları NFC'ye kanoniklenir; normalizasyon çakışması, Unicode ayraç
benzerleri ve görünmez bidi yolları reddedilir. Üç Tier-1 platformdaki byte
fixture'ı ve saldırı korpusu [`.zep` conformance rehberinde](zep-conformance.md)
açıklanır (K-117/ADR-028/spec-18).

**Gömülü standart kitaplık** (RFC-0014, deneysel): `matematik` (mutlak, üs,
tam karekök, obeb, okek), `liste_araclari` (toplam, uçlar, ortalama) ve
`metin_araclari` (tersi, ünlü sayımı, baş harf) ve `sozluk_araclari`
(`en çok geçeni bul`)
derleyicinin içindedir — kurulum gerektirmez, playground'da bile çalışır:

```
matematik birimini kullan

x 48 ve 36 ile obebini hesapla olsun
```

## 21. Komut satırı

```
argümanlar komut satırından gelenler olsun

argümanlar boşsa
    "Kullanım: selamla <isim> ..." yaz
    programı 2 ile bitir      # çıkış kodu kabuğa gider (K-069)

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
| `dil çalıştır <dosya\|proje>` | dosyayı ya da `proje.dil` taşıyan klasörü çalıştırır |
| `dil çalıştır --güvenli <dosya\|proje>` | çocuk modu: ağ kapalı, dosyalar klasörle sınırlı |
| `dil iz kaydet <iz> <program> [argümanlar]` | gerçek koşunun bütün runtime IO protokolünü atomik kaydeder |
| `dil iz oynat <iz> <program>` | kaydı dış etki uygulamadan birebir yeniden oynatır |
| `dil dene <dosya\|proje>` | testleri koşar |
| `dil biçimle <dosya\|proje>` | tek kaynağı veya bütün projeyi resmi biçime getirir |
| `dil denetle <dosya\|proje>` | çalıştırmadan bütün hataları listeler (`--json`) |
| `dil hata <kod>` | hata kodunu açıklar |
| `dil belge <birim>` | birimin işlemlerini listeler |
| `dil morfoloji [kelime]` | sürümlü ek tablosunu veya kelimenin bütün kök+ek çözümlerini gösterir |

IO izi kullanıcı girdisi, dosya/ağ gövdesi, argüman, çerez veya token
taşıyabilir; hex alanları şifreli değildir. `*.zee-io-izi` Git dışında tutulur
ama yine de özel veri gibi saklanmalıdır. Kayıt, replay ve sınırların tamamı
[IO izi rehberindedir](io-izi.md).

Editör desteği: `dillsp` — tanılar, hover, semantic kimliğe bağlı tanıma git,
**kapsam güvenli morfolojili yeniden adlandırma (F2)**
ve dış tanımda eksik düzenlemeyi reddeden tek-belge güvenlik sınırı
([editors/](../editors/); [semantic sözleşme](lsp-semantic-gezinme.md)). Gelen tek JSON-RPC
çerçevesi 8 KiB başlık/8 MiB gövde; JSON 128 iç içelik/100 bin düğüm sınırı
taşır. Geçersiz Unicode fail-closed reddedilir. Kurulumsuz deneme:
[playground](../playground/).

---

Sözdiziminin *neden* böyle olduğunu merak edersen: her kararın gerekçesi
[kararlar/gunluk.md](../kararlar/gunluk.md)'de, tasarımın tamamı
[rfcs/](../rfcs/) altında.
