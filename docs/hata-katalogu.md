# Derleyici hata kataloğu

Her tanı: **kod + Türkçe açıklama + kaynak konumu + işaret + öneri** biçiminde
basılır (RFC-0001 §8). Bu katalog, koddaki her tanının ne zaman doğduğunu ve
nasıl çözüldüğünü listeler. Kaynak: `compiler/src/` (kodlar gerçeklemeden
türetilmiştir; yeni kod eklenince bu dosya güncellenir — CI'a bağlanması planlı).

Ön ekler: **S** sözdizimi/sözcükleme · **A** ad çözümleme · **T** tür denetimi ·
**C** çalışma zamanı · **D** doğrulama (test) · **P** proje · **Ç** iç akış
(kullanıcıya görünmez).

## S — Sözdizimi ve sözcükleme

| Kod | Ne oldu | Çözüm |
|---|---|---|
| S001 | Beklenmeyen karakter (sembolik işleç, parantez...) | Kalıpları kelimelerle yaz; bkz. anti-örnek A03/A06 |
| S002 | Metin sabiti kapanmadan satır bitti | Kapatan `"` ekle |
| S003 | Girintide sekme (tab) | 4 boşluk kullan (RFC-0003) |
| S004 | Cümle tanınmadı — satır bilinen bir eylemle bitmiyor | Desteklenen kalıplar öneride listelenir; işlem çağrısıysa işlem önce tanımlanmalı |
| S005 | Girinti hizası hiçbir açık blokla eşleşmiyor | Satırı bloklardan birinin hizasına getir |
| S006 | Sayı TamSayı sınırından büyük | Daha küçük değer kullan (i64 sınırı) |
| S007 | Blok bekleyen satırdan sonra girinti yok | Alt satırları 4 boşluk içeriden yaz |
| S008 | `olsun` tanımında ad ya da değer eksik | Örnek: `isim "Ayşe" olsun` |
| S009 | Döngü biçimi tanınmadı | `10 kez tekrarla` ya da `... olana kadar tekrarla` |
| S010 | `için` döngüsü tanınmadı | `her sayı için` ya da `1 den 100 e kadar her sayı için` |
| S011 | Koşullu döngü biçimi | `<koşul> olduğu sürece` |
| S012 | Artır/azalt eksik parça | `toplamı sayıyla artır` · `sayacı 1 azalt` |
| S013 | Değer bekleniyordu | İfade konumu boş kalmış |
| S014 | `ile`nin önü/arkası boş | `a ile b` biçiminde iki taraf da dolu olmalı |
| S015 | İki değer yan yana, kalıp yok | Araya `ile` koy ya da bilinen kalıp kullan (`sayıların adedi`) |
| S016 | Koşul tanınmadı | Örnekler: `yaş 8 veya daha büyükse` · `sayı çiftse` |
| S017 | Soru biçimi | `"Adın ne?" diye sor` |
| S018 | Ekleme cümlesi eksik parça | `sayılara 5 ekle` |
| S019 | İşlem çağrısında ayraç yok | Argümanlardan sonra `için` ya da `ile` gelir |
| S020 | Çağrı argümanı tek değer değil | Argümanları `ve` ile ayır: `"Ayşe" ve 10 ile selamla` |
| S021 | işlem/yapı/test tanımı yanlış yerde | Tanımlar en dış düzeyde olmalı |
| S022 | İşlem adı geçersiz | Ad yalnız kelimelerden oluşur: `işlem ortalamayı hesapla` |
| S023 | Bölme cümlesi eksik parça | `sonucu toplamı adede böl` |
| S024 | Eşleştirme biçimi | Başlık `şekle göre`; kollar `"kare" ise` / `değilse` |
| S025 | Yapı tanımı biçimi | `yapı Öğrenci` + alanlar `yaş TamSayı` |
| S026 | Test bloğu biçimi | `test "toplama doğru çalışır"` |
| S027 | Sonlandırma biçimi | `programı bitir` ya da `programı 1 ile bitir` (çıkış kodu, K-069) |
| S028 | Türkçe/Latin dışı karakter (homoglyph koruması) | Tanı kod noktasını gösterir (örn. Kiril а = U+0430); yalnız Türkçe/Latin harf kullan |
| S029 | Birleştirici im (U+0300–U+036F) | Birleşik karakteri kullan: g + ˘ değil ğ (RFC-0002 §2) |
| S030 | `ve` ile `veya` aynı koşulda karıştı | Öncelik parantezsiz belirsizdir: tek tür bağlaç kullan ya da koşulu ayrı `ise` basamaklarına böl (K-027) |
| S031 | `değilse` tek başına | `değilse` bir `... ise` bloğunun hemen ardından, aynı hizada gelir |
| S032 | Ondalık kısım 9 haneden uzun | Ondalık hassasiyeti en çok 9 hanedir (RFC-0013) |
| S033 | Boşluk-virgül-rakam dizisi belirsiz | Ondalıksa bitişik yaz (`3,14`); liste ayracıysa virgülden sonra boşluk bırak (`3, 14`) |
| S041 | Yönlendirme biçimi | `"/liste" adresine yönlendir` (K-051) |
| S042 | Silme biçimi | `sayılardan 5 i sil` · `defterden "elma" yı sil` (K-059) |
| S040 | Bilinmeyen kaçış dizisi | Metinde yalnız \" (tırnak), \\\\ (ters bölü) ve \\n (yeni satır) geçerli |
| S034 | Birim/paket kullanımı biçimi | `hesaplar birimini kullan` — aynı klasördeki dosyayı; `grafik paketini kullan` — proje bağımlılığını alır |
| S035 | Sunucu açma biçimi | `8080 kapısında sunucu başlat` |
| S036 | Olay/zaman aşımı bloğu biçimi | `"/durum" adresine istek geldiğinde` · `yetişmezse` tek başına satır |
| S037 | Yanıt gönderme biçimi | `"çalışıyor" yanıtını gönder` |
| S038 | Eşzamanlı blok biçimi | `eşzamanlı olarak` + girintide `<ad> <ifade>` görev satırları |
| S039 | Işık komutu biçimi | `kırmızı ışığı yak` / `mavi ışığı söndür` |

## A — Ad çözümleme

| Kod | Ne oldu | Çözüm |
|---|---|---|
| A001 | Ad bu kapsamda tanımlı değil | Önce `<ad> <değer> olsun`; tanı tanımlı adları listeler |
| A002 | Ad birden çok köke çözülüyor (belirsizlik) | Adlardan birini değiştir — belirsizlik dilde hatadır |
| A003 | `her X için` gezilecek listeyi bulamadı | Kapsamda `Xlar`/`Xler` adlı liste olmalı (örtük çoğul, K-013) |
| A005 | Aynı adla ikinci işlem tanımı | İşlem adları benzersizdir |
| A006 | Aynı adla ikinci yapı tanımı | Yapı adları benzersizdir |
| A007 | `yeni <Ad>` — yapı tanımlı değil | Yapıyı kullanmadan önce tanımla |
| A008 | Ad iki kaynaktan geliyor (birim çakışması) | Sessiz gölgeleme yoktur: adlardan birini değiştir ya da tek kaynakta topla (RFC-0009) |
| A009 | Birimler döngüsel kullanıyor | Ortak tanımları üçüncü bir birime taşı |
| A010 | Birim yüklenemedi | Aynı klasörde `<ad>.dil` dosyası olmalı (RFC-0009 §4) |
| A011 | Paket yüklenemedi | Paketi `yerel_bağımlılıklar` listesine ekle, ardından `dil kilitle .` çalıştır |

## T — Tür denetimi

| Kod | Ne oldu | Çözüm |
|---|---|---|
| T001 | Karşılaştırma tür uyuşmazlığı | Büyüklük sayılar arasında; eşitlik aynı türler arasında |
| T002 | Değerin türü değiştirilmeye çalışıldı | Tür sonradan değişmez; yeni ad kullan |
| T003 | Tekrar adedi TamSayı değil | `10 kez tekrarla` |
| T004 | Aralık uçları TamSayı değil | `1 den 100 e kadar` |
| T005 | ise/döngü/olmalı koşul istiyor | Koşul kalıbı kullan (K-010) |
| T006 | Artır/azalt sayı istiyor | Hedef ve miktar TamSayı olmalı |
| T007 | Çift/tek sorgusu TamSayı ister | — |
| T008 | Aritmetik sayılar arasında | Metni sayıya çevir: `<metnin> sayısı` |
| T009 | `sayısı` kalıbı Metin ister | — |
| T010 | Rastgele uçları TamSayı değil | `1 ile 100 arasında rastgele sayı` |
| T011 | Liste öğe türü uyuşmazlığı | Bir listenin bütün öğeleri aynı türden |
| T012 | Ekleme hedefi liste değil | Önce `boş liste olsun` |
| T013 | `her ... için` kaynağı liste/sözlük değil | — |
| T014 | Özellik bu türe uygulanamaz | adedi/ilki/sonu → liste; uzunluğu/kelimeleri → metin; yılı → tarih |
| T015 | Çağrı argüman sayısı yanlış | İşlemin parametre sayısına bak |
| T016 | İşlem kaydı bulunamadı | Derleyici iç hatası olabilir — bildir (özyineleme v0.2'de serbest) |
| T017 | Argüman türleri işlem imzasına uymuyor | Çıkarımlı imzayı aynı türle çağır; public API'de `<ad> <Tür> olarak al` sözleşmesine uy |
| T018 | Dönüş türleri tutarsız | Tek tür döndür; `yok` + tür → Seçenek olur |
| T019 | Değer döndürmeyen işlem ifade konumunda | İşleme `... döndür` ekle ya da cümle olarak çağır |
| T020 | `döndür` işlem dışında | Yalnız işlem gövdesinde geçerli |
| T021 | Sözlük işlemi tür uyuşmazlığı | v0: anahtar Metin; değer türü sözlüğün türüne uymalı |
| T022 | Metin işlemi Metin ister | harflisi/içeriyorsa |
| T023 | `varsa` Seçenek ister | Seçenek, `yok döndür` içeren işlemden doğar |
| T024 | değeri/hatası/başarılıysa yanlış türde | Seçenek ya da Sonuç üzerinde kullan |
| T025 | Dosya yolu Metin değil | — |
| T026 | `göre` eşleştirme türleri | Konu TamSayı/Metin; kollar konuyla aynı türde |
| T027 | Yapı alanının türü tanınmadı | Kullanılabilir: TamSayı, Ondalık, Metin, Mantıksal |
| T028 | Yapı alanı yok / yanlış tür | Tanı mevcut alanları listeler |
| T029 | `gün sonrası` Tarih + TamSayı ister | — |
| T030 | `boşsa` liste/sözlük/metin ister | — |
| T031 | ve/veya parçası ya da `değilse` içi koşul değil | Her parça Mantıksal olmalı |
| T032 | Hata mesajı Metin değil | `"sıfıra bölünmez" hatasını döndür` (RFC-0008 §4.1) |
| T033 | Görev sonucuna erken erişim | Eşzamanlı görev adları `hepsini bekle`den sonra kullanılır (RFC-0011 §1) |
| T034 | Ağ/süre/sunucu kalıbı tür uyuşmazlığı | Kapı TamSayı, yol/adres Metin, `içinde`/`bekle` Süre ister |
| T035 | Özyinelemeden önce temel durum yok | Temel durumu üste yaz: önce bir dalda döndür, sonra özyinelemeli adım |
| T036 | Korumasız Seçenek/Sonuç erişimi | `değeri` ancak `varsa`/`başarılıysa`, `hatası` ancak `başarısızsa` dalında (RFC-0008 §4.2) |
| T037 | Bir işlemde açık ve çıkarımlı parametre türleri karışık | Bütün parametreleri `<ad> <Tür> olarak al` yaz ya da hepsini çıkarımlı bırak |
| T038 | Açık parametre türü tanınmadı | Örn. `TamSayı`, `Ondalık listesi`, `Metin sözlüğü` ya da tanımlı bir yapı adı kullan |

## C — Çalışma zamanı

| Kod | Ne oldu | Çözüm |
|---|---|---|
| C000 | İç tutarlılık hatası | Derleyici hatasıdır — lütfen bildir |
| C001 | Değer bulunamadı | (İç duruma yakın; görülmesi beklenmez) |
| C002 | TamSayı taşması | Değerleri küçült; taşma sessizce sarmalanmaz (RFC-0001 §7) |
| C003 | Sıfıra bölme | Önce böleni kontrol et |
| C004 | Metin sayıya çevrilemedi | `sayısı` yalnız rakam içeren metni çevirir |
| C005 | Soruya verilecek girdi kalmadı | Etkileşimsiz koşuda girdi sayısı sorulardan az |
| C006 | Rastgele aralığı ters | Alt uç üstten büyük olamaz |
| C007 | Boş listenin ilki/sonu | Önce `adedi` ile kontrol et |
| C008 | Boş Seçenek'in değeri (çalışma zamanı savunması) | Normalde T036 derlemede yakalar; görülürse bildir |
| C009 | Sonuç'un yanlış tarafı (iç savunma — korumasız erişim derlemede T036 ile yakalanır) | başarılıysa → değeri; değilse → hatası |
| C010 | Sözlükte anahtar yok | Önce `sözlükte <anahtar> varsa` |
| C011 | Biçimleyici token güvencesi bozuldu | Dosya yazılmadı; derleyici hatasıdır — bildir |
| C012 | Dosya okunamadı (düz biçim) | Hata yönetilecekse `... okumayı dene` ile Sonuç al |
| C013 | Dosyaya yazılamadı | Yol/izin kontrolü |
| C015 | CSV biçim hatası (boş dosya ya da sütun sayısı uyuşmazlığı) | Başlık satırı + eşit hücreli veri satırları (K-062: hücreler Metin okunur) |
| C016 | JSON biçim hatası | v0: düz nesne + metin değerler |
| C017 | Sunucu kurulamadı | Kapı boşta mı? Düşük kapılar (<1024) yönetici ister |
| C018 | Ağ isteği başarısız | v0 yalnız http:// destekler (TLS yok); adresi ve bağlantıyı kontrol et |
| C019 | Çağrı derinliği 500'ü aştı | Özyinelemeli adım her seferinde temel duruma yaklaşmalı |
| C020 | Çıkış kodu 0–255 dışında | `programı 0 ile bitir` … `programı 255 ile bitir` (K-069) |

## D — Doğrulama

| Kod | Ne oldu | Çözüm |
|---|---|---|
| D001 | `olmalı` koşulu tutmadı | Tanı beklenen/bulunan değerleri gösterir; `dil dene` testte raporlar |

## P — Proje bildirimi

| Kod | Ne oldu | Çözüm |
|---|---|---|
| P001 | `proje.dil` bilinmeyen/tekrarlı alan ya da değer tanımı dışında cümle içeriyor | `proje`, `sürüm`, `giriş` ve isteğe bağlı `yerel_bağımlılıklar` alanlarını kullan |
| P002 | Zorunlu proje alanı eksik | Eksik `proje`, `sürüm` veya `giriş` satırını ekle |
| P003 | Proje adı ya da `X.Y.Z` sürümü geçersiz | Boş olmayan ad ve üç sayılı sürüm kullan: `0.1.0` |
| P004 | Giriş mutlak, proje dışına çıkan veya `.dil` olmayan yol | Proje içinde kalan göreli `.dil` yolu kullan |
| P005 | Yerel bağımlılık listesi ya da yollarından biri geçersiz | Göreli proje klasörlerini Metin listesiyle yaz: `yerel_bağımlılıklar "../ortak" listesi olsun` |
| P006 | Yerel bağımlılık klasörü/bildirimi okunamadı | Yolun `proje.dil` taşıyan erişilebilir bir zee projesi olduğunu doğrula |
| P007 | Bağımlılık döngüsü, yinelenen paket adı veya kaynakta kullanılamayan paket adı | Döngüyü kır; her pakete benzersiz, küçük harfli tek tanımlayıcı ad ver |
| P008 | `proje.kilit` eksik ya da kaynak/bildirim grafiğiyle uyuşmuyor | Değişikliği incele, sonra `dil kilitle .` çalıştır |
| P009 | Paket/proje girişi güvenli ve gerçek bir `.dil` kaynağına çözülemedi | Girişi proje içindeki sembolik bağ olmayan bir kaynağa yönelt |
| P010 | Kaldırılmak istenen paket en az bir proje kaynağında hâlâ kullanılıyor | Önce `<ad> paketini kullan` satırını ve pakete bağlı çağrıları kaldır |

## Ç — İç akış

| Kod | Not |
|---|---|
| Ç000 | `programı bitir` iç nöbetçisi; çalıştırıcı yakalar, kullanıcı asla görmez |

> Boşluklar bilinçlidir: A004 ve C014 ayrılmış ama kullanılmamıştır; yeni
> tanılar sıradaki boş numarayı alır.
