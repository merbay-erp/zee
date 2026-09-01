# 14 — Yapılandırılmış eşzamanlılık

Normatif kaynak: RFC-0011 (geçici kabul), K-090. Durum: **TANIMLI**.

## 1. Görev grubu ve birleştirme

```dil
eşzamanlı olarak
    profil "Ayşe" için profili getir
    faturalar "Ayşe" için faturaları getir

hepsini bekle
profil yaz
```

- `eşzamanlı olarak` içindeki her satır `<ad> <ifade>` biçiminde bir görev
  bağlamasıdır. Görev, bildirildiği anda dış ortamın değer kopyasını alır.
- Görevler, bildirildikleri sözcüksel kapsamın çocuklarıdır. Aynı kapsamda
  ikinci grup açılmadan önce ilk grup `hepsini bekle` ile kapatılmalıdır.
- `hepsini bekle`, açık grubun görevlerini çalıştırır ve tamamlanan sonuçları
  adlarına bağlar. Birleştirmeden önce görev adına erişim T033'tür.
- Açık grup olmadan `hepsini bekle`, bir grubu birleştirmeden kapsamdan çıkma
  ve görevler açıkken `döndür`/`programı bitir` T051'dir. Böylece başarılı bir
  programda görev kapsamdan kaçamaz ve sahipsiz görev oluşamaz.
- Görev bildirimi ile `hepsini bekle` arasındaki ana-kapsam cümleleri önce
  çalışır. Çocuk görevlerin ilk poll'u birleştirme cümlesinde başlar. Bu sıra
  TANIMLIDIR; eşzamanlı işlerin yan etkileri birleştirme sırasında görünür.

## 2. Deterministik scheduler

Runtime tek işletim sistemi iş parçacığında işbirlikli çalışır:

1. Hazır görevler kaynak sırasıyla bir kez ilerletilir.
2. Bir görev `bekle`ye gelene, tamamlanana ya da hata verene kadar çalışır.
3. Hazır görev kalmazsa tekdüze saat en yakın uyanma anına ilerletilir.
4. Aynı anda uyanan görevler yine kaynak sırasıyla ilerler.
5. Bütün görevler tamamlanınca sonuçlar kaynak sırasıyla adlarına bağlanır.

Bu sıra aynı program ve aynı IO girdileri için değişmez. İki görev sırasıyla
2 saniye ve 1 saniye beklerse sanal saat 3 değil 2 saniye ilerler. Aynı anda
yalnız bir görev çalıştığından paylaşılan bellekte data race sınıfı yoktur.
Görevlerin yerel ortamları birbirinden ayrıdır; dış dünya etkileri ortak IO
adaptörüne yukarıdaki kesin sırayla gider.

Tekdüze saatin 0 başlangıcı, geriye gitmeme, negatif bekleme ve takvim
saatinden ayrılma kuralları spec/22'deki `zee-io-1` profilidir. Scheduler bu
tek ortak saati en yakın uyanışa taşır; ayrı görev saatleri uydurmaz.

`eylem` transaction'ı tek bir scheduler dilimidir: savepoint açılışı ile
tamamlama/geri alma arasına kardeş görev giremez. Bu, transaction sahipliğini
korur; uzun eylem kardeş görevi ancak eylem bittikten sonra ilerletebilir.

## 3. Hata, iptal ve son tarih

- İlk yönetilmemiş görev hatası birleştirmeden aynı tanı koduyla dışarı yayılır.
  Hata mesajı görev adını ve iptal edilen kardeşleri belirtir.
- Hata görüldüğü anda tamamlanmamış kardeş future'ları düşürülür. İptal edilen
  görev `bekle` sonrasındaki hiçbir cümleyi ve yan etkiyi çalıştıramaz.
- Hata öncesinde tamamlanmış sıradan IO etkileri geri alınmaz. Geri alma
  isteyen uygulama değişikliği `eylem` transaction'ı içinde olmalıdır.
- Bir görev dış `içinde` son tarihini miras alır. Son tarih dolunca aynı Ç001
  sahiplik nöbetçisi görev ağacından dış sahibine ulaşır; kardeşler iptal
  edilir ve yalnız doğru `yetişmezse` kolu çalışır (spec/09).
- Görev içindeki `programı N ile bitir`, Ç000 çıkış nöbetçisini bozmadan köke
  taşır; kardeşleri iptal eder ve program N koduyla olağan biçimde biter.
- Ana kapsam birleştirmeden önce başka bir hatayla biterse henüz
  çalıştırılmamış çocuk kayıtları iptal edilerek düşer; dışarıda iş kalmaz.

## 4. İşbirlikli sınır

`bekle` tam bir scheduler noktasıdır. İşlem çağrıları iç içe olsa da bu nokta
çağrı zincirinden dışarı taşınır. HTTP isteği senkron adaptöre girmeden önce
kardeşlere bir tur verir; adaptör çağrısının içi ise diğer senkron platform
çağrıları (bazı DNS/dosya işlemleri gibi) kadar atomik ve önleyici olmayan bir
dilimdir. Dönüşte son tarih yeniden denetlenir. Çok çekirdekli paralellik,
yarış/`ilkini bekle`, akış ve dinamik görev sayısı bu sürümün sözü değildir.

## 5. Conformance kanıtı

`compiler/tests/ag_ve_esz_testi.rs` en az şunları sabitler:

- farklı süreli görevlerin kaynak sıralı çıktı izi ve en-uzun-süre saati;
- bir görev hatasında bekleyen kardeşin sonraki etkisinin yokluğu;
- dış son tarihin bütün görev ağacını doğru sahibine taşıması;
- iç görev ağacının beklerken dış kardeşe yürütme sırası vermesi;
- görev içindeki atomik `eylem` hatasının rollback edip kardeşi araya almaması;
- beklemeden kapsamdan çıkış ve boş birleştirme için T051;
- birleştirme öncesi sonuç erişimi için T033.
