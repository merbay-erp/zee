# Derleyici hata kataloğu

Her tanı: **kod + Türkçe açıklama + kaynak konumu + işaret + öneri** biçiminde
basılır (RFC-0001 §8). Bu katalog, koddaki her tanının ne zaman doğduğunu ve
nasıl çözüldüğünü listeler. Kaynak: `compiler/src/`; kod↔katalog birebirliği
ve kod↔anlam kimliği CI'da ayrı kapılarla doğrulanır. Yayımlanmış bir kod başka
bir anlam için yeniden kullanılamaz; kaldırılan kod `ayrılmış` mezar taşı olarak
kalır (ADR-025).

Ön ekler: **S** sözdizimi/sözcükleme · **A** ad çözümleme · **T** tür denetimi ·
**C** çalışma zamanı · **D** doğrulama (test) · **P** proje · **Ç** iç akış
(kullanıcıya görünmez).

## S — Sözdizimi ve sözcükleme

| Kod | Ne oldu | Çözüm |
|---|---|---|
| S001 | Beklenmeyen karakter (sembolik işleç, parantez...) | Kalıpları kelimelerle yaz; bkz. anti-örnek A03/A06 |
| S002 | Metin sabiti kapanmadan satır bitti | Kapatan `"` ekle |
| S003 | Girintide sekme (tab) | 4 boşluk kullan (RFC-0003) |
| S004 | Cümle tanınmadı — satır bilinen bir eylemle bitmiyor | Desteklenen kalıplar öneride listelenir; işlem çağrısıysa işlem önce tanımlanmalı; gövde dışına düşen `sayıyı al` parametre satırında öneri satırı işlem başlığının altına almayı söyler |
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
| S032 | **ayrılmış** — eski 9 hane Ondalık sınırı | K-092 keyfî hassasiyet kararıyla üretimden kaldırıldı; kod başka anlamda kullanılamaz |
| S033 | Boşluk-virgül-rakam dizisi belirsiz | Ondalıksa bitişik yaz (`3,14`); liste ayracıysa virgülden sonra boşluk bırak (`3, 14`) |
| S041 | Yönlendirme biçimi | `"/liste" adresine yönlendir` (K-051) |
| S042 | Silme biçimi | `sayılardan 5 i sil` · `defterden "elma" yı sil` (K-059) |
| S040 | Bilinmeyen kaçış dizisi | Metinde yalnız \" (tırnak), \\\\ (ters bölü) ve \\n (yeni satır) geçerli |
| S034 | Birim/paket kullanımı biçimi | `hesaplar birimini kullan` — aynı klasördeki dosyayı; `grafik paketini kullan` — proje bağımlılığını alır |
| S035 | Sunucu açma biçimi | `8080 kapısında sunucu başlat` |
| S036 | Olay/zaman aşımı bloğu biçimi | `"/durum" adresine istek geldiğinde` · `yetişmezse` tek başına satır |
| S037 | Yanıt gönderme biçimi | `"çalışıyor" yanıtını gönder` veya `"hazır değil" yanıtını 503 durumuyla gönder`; durum kaynakta görünen 100..599 literalidir. K-384: adı `gönder` ile biten işlemi cümle olarak çağırınca öneri çakışan işlemi anar — işleme başka ad ver |
| S038 | Eşzamanlı blok biçimi | `eşzamanlı olarak` + girintide `<ad> <ifade>` görev satırları |
| S039 | Işık komutu biçimi | `kırmızı ışığı yak` / `mavi ışığı söndür` |
| S043 | Web güvenlik cümlesi biçimi | Rota önsözünü, oturum açmayı veya kapatmayı tanıda gösterilen tam kalıpla yaz |
| S044 | Yapılandırılmış hata biçimi/kodu geçersiz | `"DOSYA_YOK" kodlu "Dosya bulunamadı" hatasını döndür`; neden önce, veri sonra yazılır |
| S045 | Kaynak metni ya da token akışı güvenli profil sınırını aştı | Kaynağı sorumluluğu açık daha küçük birimlere böl |
| S046 | Görünüm bloğu satırı biçimi geçersiz | Satır `etiket ad değer …`, `"sabit"`, `metin ifade` ya da `ham ifade` olmalı; öznitelikler çift, çocuklar öğe altında (K-212) |
| S047 | Lint: erişilemeyen cümle veya sabit koşul dalı | Kesin `döndür`/`programı bitir` sonrasını ya da literal yanlış dalı gözden geçir; uygun dala taşı veya kaldır. `yanıtını gönder` bloğu sonlandırmaz. Yalnız isteğe bağlı uyarıdır |
| S048 | Lint: rotanın yanıt üretmeyen yolu olabilir | Her dala yanıt veya yönlendirme ekle. Örnek: `değilse` altında `"Geçersiz istek" yanıtını 400 durumuyla gönder`. Belirsiz çağrı etkisinde tahmin yapılmaz; yalnız isteğe bağlı uyarıdır |
| S049 | Sıra eki noktası RFC-0029 kalıbına oturmuyor | Kalıp `<liste>nin <sıra>. öğesi` ya da `… öğesini almayı dene`; sıra tek ad ya da TamSayı sabitidir, hesaplanan sırayı önce bir ada ver: `konum sıra ile 1 in toplamı olsun` (K-386). Bitişik olmayan, `öğe…` öncesi olmayan ya da `3.14` gibi nokta S001 kalır |

## A — Ad çözümleme

| Kod | Ne oldu | Çözüm |
|---|---|---|
| A001 | Ad bu kapsamda tanımlı değil | Önce `<ad> <değer> olsun`; tanı tanımlı adları listeler. K-384: işlemde örtük `bağlantı` Sözlüğü görünmez (alanı parametre olarak geçir); adı yerleşik `yaz` ile biten işlemi cümle olarak çağırınca tanı çakışan işlemi anar — işleme başka ad ver |
| A002 | Ad birden çok köke çözülüyor (belirsizlik) | Adlardan birini değiştir — belirsizlik dilde hatadır. K-398: parametre gövdede sözlük köküyle (`başlık … olsun`) yeniden atandıktan sonra K-398 öncesi yumuşamış yazımla (`başlığ`) başvurmak da A002'dir; parametre için sözlük kökünü yaz, ayrı yerele başka ad ver |
| A003 | `her X için` gezilecek listeyi bulamadı | Kapsamda `Xlar`/`Xler` adlı liste olmalı (örtük çoğul, K-013) |
| A004 | **ayrılmış** — eski ad çözümleme yuvası | Bu kod yeni bir anlam için kullanılamaz |
| A005 | Aynı adla ikinci işlem tanımı | İşlem adları benzersizdir |
| A006 | Aynı adla ikinci yapı tanımı | Yapı adları benzersizdir |
| A007 | `yeni <Ad>` — yapı tanımlı değil | Yapıyı kullanmadan önce tanımla |
| A008 | Ad iki kaynaktan geliyor (birim çakışması) | Sessiz gölgeleme yoktur: adlardan birini değiştir ya da tek kaynakta topla (RFC-0009). Aynı tanımın elmas yoldan iki kez gelmesi çakışma değildir (ADR-072) |
| A009 | Birimler döngüsel kullanıyor | Ortak tanımları üçüncü bir birime taşı |
| A010 | Birim yüklenemedi | Aynı klasörde adı birebir `<ad>.dil` olan dosya olmalı; yalnız büyük/küçük harfi farklı dosya (`Çay.dil`) bağlanmaz (RFC-0009 §4, K-363) |
| A011 | Paket yüklenemedi | Paketi `yerel_bağımlılıklar` listesine ekle, ardından `dil kilitle .` çalıştır |
| A012 | Lint: ad tanımlanıyor ama okunmuyor | Adı kullan veya gereksiz bağlamayı kaldır; değer ifadesinin etkisini incelemeden satırı silme. Örnek: `adet 3 olsun` ardından `adedi yaz`. Yalnız `--lint`/LSP lint açıkken uyarıdır |

## T — Tür denetimi

| Kod | Ne oldu | Çözüm |
|---|---|---|
| T001 | Karşılaştırma tür uyuşmazlığı | Büyüklük sayılar arasında; eşitlik aynı türler arasında |
| T002 | Değerin türü değiştirilmeye çalışıldı | Tür sonradan değişmez; yeni ad kullan. K-398: `başlığı al` parametresi `başlık` adıyla bağlanır; gövdedeki `başlık … olsun` artık parametreye atamadır, öneri bunu söyler — yerel değere başka ad ver |
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
| T016 | Checker semantic kaydı bulunamadı | İşlem/sembol/yapı/HIR iç tutarlılık hatası olabilir — bildir (özyineleme v0.2'de serbest) |
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
| T039 | Birim/paket işleminin public sözleşmesi eksik | Bütün parametre türlerini ve `<Tür> döndürür` / `değer döndürmez` satırını yaz |
| T040 | Açık dönüş türü tanınmadı | Örn. `TamSayı döndürür`, `Metin seçeneği döndürür` ya da `değer döndürmez` |
| T041 | Bildirilen dönüş ile gövde uyuşmuyor | Dönüş bildirimi ve bütün `döndür` dalları aynı türü üretmeli |
| T042 | Değer döndüren işlemin bir yolu gövde sonuna düşebilir | Her koşul/eşleştirme yolunda döndür veya sona ortak dönüş ekle |
| T043 | Eylemin açık girdi/dönüş sözleşmesi eksik | Bütün girdileri türleriyle yaz; dönüş türünü ya da değer döndürmediğini bildir |
| T044 | Eylem HTTP adaptörüne bağımlı | Yanıt/yönlendirme/çerezi rotada bırak; eylem yalnız iş kuralını taşısın |
| T045 | GET/HEAD durum değiştirebilir | Değişikliği POST/PUT/PATCH/DELETE rotasından açık bir eylemle çağır |
| T046 | Rota uygulama durumunu doğrudan ya da yazıcı işlemle değiştiriyor | Yazmayı açık imzalı bir eylem içine taşı |
| T047 | Aynı yöntem ve yol birden çok rotaya bağlı | Her yöntem+yol çiftini yalnız bir kez tanımla |
| T048 | Eylem geri alınamayan ekran/girdi/donanım etkisi taşıyor | Bu etkiyi çağıran adaptöre taşı; eylemde transaction destekli kaynakları kullan |
| T049 | Durum değiştiren rota açık erişim politikası taşımıyor | İlk satıra `herkese açık`, `oturum gerekli` ya da `"rol" yetkisi gerekli` yaz |
| T050 | Rota güvenlik önsözü yanlış yerde veya yinelenmiş | Tek erişim politikasını ilk satıra, zorunlu alanları hemen arkasına koy |
| T051 | Eşzamanlı görev grubunun sözcüksel sahipliği kapanmadı ya da boş birleştirme yapıldı | Her `eşzamanlı olarak` grubunu aynı kapsamda tek `hepsini bekle` ile kapat; açık görevlerle dönme/bitirme |
| T052 | Hata nedeni/verisi yanlış türde ya da yeniden yayma zenginleştiriliyor | Neden `Hata`, veri `Metin sözlüğü` olmalı; zenginleştirmek için yeni kodlu hata ile sar |
| T053 | Gezilen kaynak koleksiyonun kendisi değiştiriliyor ya da aynı kaynak iç içe geziliyor | Öğeyi döngü adıyla güncelle; ekleme/silmeyi ayrı listede toplayıp gezme bitince uygula |
| T054 | Program çalışma politikasında açılmayan yetkinliği ya da outbound origin'i kullanıyor | Yetkiyi ve gerekiyorsa tam şema+host+port hedefini `proje.dil` içinde bilinçli olarak bildir; tek dosya kipinde dış ağ varsayılan kapalıdır, `dil çalıştır --izin ağ dosya.dil` ile açılır (K-387) |
| T055 | Görünüm metni/özniteliği Metin (ya da bayrak için Mantıksal) değil | Sayıyı metne çevir; bayrak öznitelik için `doğru`/`yanlış` ver |
| T056 | Önbelleklenebilir rota politikası GET/HEAD dışı rotada | Durum değiştiren rotada `herkese açık` yaz; önbellek iznini salt-okuma GET/HEAD rotasına taşı (K-335) |
| T057 | Önbelleklenebilir rota oturum, CSRF belirteci ya da çerez kullanıyor | Gövdede ya da çağırdığı işlemde olabilir; oturuma bağlı parçayı ayrı rotaya taşı ya da `önbelleklenebilir` iznini kaldır; paylaşılan yanıt kimliğe bağlanamaz (K-335) |
| T058 | Sıra ile öğe erişiminde tür uyuşmuyor: nesne liste değil, listenin öğe türü belli değil ya da sıra TamSayı değil | Sıra 1, 2, 3 gibi TamSayıdır (metinse önce `metnin sayısı`); metnin harfleri için `metnin harfleri` listesini al; boş başlayan listeye önce öğe ekle ya da `… öğesini almayı dene` kullan (K-386) |
| T059 | Sabit sıra 1'den küçük | Sıra 1 tabanlıdır: ilk öğe `listenin 1. öğesi` (ya da `listenin ilki`), son öğe `listenin sonu`; 0 ve negatif sıra yoktur (K-386) |

## C — Çalışma zamanı

Web rotasında C000 dışındaki her C/D tanısı yalnız o isteği bitirir (K-324,
spec/11 §5): istemci ayrıntısız 500 (geçici/kaynak kodlarında 503, son tarihte
504) alır, worker sonraki isteği kabul eder ve tanı stderr'e tek satırlık
`zee-web istek-hatası` kaydıyla yazılır.

| Kod | Ne oldu | Çözüm |
|---|---|---|
| C000 | İç tutarlılık hatası; AST/HIR invariant ihlalinde faz ve yapısal düğüm yolu da gösterilir | Derleyici hatasıdır — lütfen bildir |
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
| C013 | Dosyaya yazılamadı | Yol/izin, var olan no-clobber hedef ve owner/group/ACL/xattr desteğini kontrol et. `... dene` yaşam döngüsü yüzeyleri de bu kararlı dosya hata ailesini yapılandırılmış Hata olarak döndürür; doğrudan yazma arızasında web adaptörü ayrıntı sızdırmayan 503 ile worker'ı yaşatır. Ayrı bağlama noktaları arasındaki taşıma (EXDEV) hata değildir; runtime kopya + atomik adlandırmayla tamamlar (K-329). İmza okumasında (`ilk N baytını onaltılık okumayı dene`, K-333) 1–64 dışı bayt sayısı, normal dosya olmayan hedef ve proje sınırı reddi de C013'tür. Dosya yanıtında (`… adıyla ek olarak göndermeyi dene`, K-378) kayıtsız içerik türü, geçersiz ad (CR/LF, tırnak, yön karakteri, yol), proje sınırı, sembolik bağ/dizin, 32 MiB aşımı (K-401 öncesi 16 MiB) ve yanıtı zaten belirlenmiş istek yanıt yazılmadan C013 başarısız Sonuç'tur. Aralıklı okumada (`… baytından … baytını metin olarak|base64 okumayı dene`, K-389) negatif konum, 1–1048576 dışı bayt sayısı ve normal dosya olmayan hedef; çözerek yazmada (`… çözerek … dosyasına yazmayı|eklemeyi dene`) var olan hedef (`hedef dosyası zaten var`), 32 MiB toplam tavanı (K-395 öncesi 16 MiB), eylem içinde ekleme ve proje sınırı reddi de C013'tür; kodlama hatası C032. Atomik yazım arızasının iletisi düştüğü adımı (ör. `yazma kilidi`, `ReplaceFileW`) taşır (K-351). Proje kipinde göreli yol giriş dosyasının klasöründen çözülür (spec/07); hedef klasör yoksa ileti `hedef klasör "…" yok`, kaynak yoksa `"…" bulunamadı` der, `dosya yolu güvenle çözülemedi` yalnız yokluk dışı çözüm hatasıdır (K-382). Paylaşılan veri kökünde (`veri:`, K-394) eksik `veri_kökü` ortam/credential değeri, göreli/bağlı/başkalarınca yazılabilir kök, `..`/mutlak/boş bileşenli yol, ara ya da yaprak sembolik bağ, çok hard-link'li okuma, paylaşılan yazılabilir ata klasörü, `veri-*` yetkinlik reddi ve desteklenmeyen işlem (listeleme, görsel küçültme) de C013'tür; ileti kökün mutlak yolunu taşımaz |
| C014 | **ayrılmış** — eski çalışma zamanı yuvası | Bu kod yeni bir anlam için kullanılamaz |
| C015 | CSV biçim hatası (boş dosya ya da sütun sayısı uyuşmazlığı) | Başlık satırı + eşit hücreli veri satırları (K-062: hücreler Metin okunur) |
| C016 | JSON biçim hatası | v0: düz nesne + metin değerler |
| C017 | Sunucu kurulamadı | Kapı boşta mı? Düşük kapılar (<1024) yönetici ister |
| C018 | Ağ isteği başarısız | HTTPS adresini, proje yetkinliğini, tam origin allowlist'ini, DNS sonucunu ve zaman aşımını denetle; düz HTTP yalnız açık `yerel-ağ` içindir |
| C019 | Çağrı derinliği 500'ü aştı | Özyinelemeli adım her seferinde temel duruma yaklaşmalı |
| C020 | Çıkış kodu 0–255 dışında | `programı 0 ile bitir` … `programı 255 ile bitir` (K-069) |
| C021 | Eylem transaction'ı başlatılamadı, tamamlanamadı ya da geri alınamadı | Kalıcı kaynağın yol/izin/bağlantı durumunu denetle; write'ı otomatik tekrarlama. Web adaptörü isteği 503 ile kapatıp worker'ı yaşatır; CLI tanıyı yayar |
| C022 | Web güvenlik adaptörü işlemi reddetti | Güvenli oturum/CSRF desteğini ve başlığa yazılan çerez ya da yerel yönlendirme değerini denetle. Rotadaki ret yalnız o isteği 500 ile bitirir; oturum deposunun yeni kaydı reddetmesi (kapasite, depo IO, kilit) ise C022 değil 503 `web güvenlik durumu güvenle kaydedilemedi` yanıtıdır (spec/12, K-370). İstemcinin yanıt yazımı sırasında kopması C022 değildir: stderr'e tek satır yazılır, worker yaşar (K-327) |
| C023 | Çalışma adımı, çıktı, koleksiyon ya da eşzamanlı görev kaynak bütçesi aşıldı | Sonsuz döngüyü düzelt; işi, çıktıyı veya koleksiyonu daha küçük sonlu parçalara böl |
| C024 | Metin ya da saklanan değer heap bütçesi aşıldı | Zarf canlı saklanan değerleri sayar; düşen değerin payı iade edilir (K-393). Büyük metin ve değer grafiklerini daha küçük çalışma adımlarına böl; canlı tuttuğun liste/sözlük birikimini küçült; döngüde metin biriktirirken parçaları listeye ekleyip sonda `parçaların "" ile birleşmişi` kullan. Web rotasında aşım yalnız o isteği 503 ile bitirir |
| C025 | PostgreSQL sorgusu veya değişikliği başarısız | Hatanın `sqlstate`, `kısıt` ve `tablo` verisini incele; sorgu ve parametreyi düzelt. `db.response_frame_limit` için yanıtı küçült; geçersiz transaction’ı temizleyip yeni bir işlem başlat |
| C026 | PostgreSQL migration veya bağlantı işlemi başarısız | Exact hedefi, `sslmode`, pinned CA ortam değişkenini, havuz kapasitesini ve değişmez migration geçmişini doğrula |
| C027 | COMMIT sonucu istemci tarafından doğrulanamadı; otomatik tekrar güvenli değil | Aynı write'ı körlemesine tekrarlama. Yeni bağlantıda kararlı iş anahtarıyla sonucu uzlaştır; varsa mevcut sonucu benimse, yoksa ürünün idempotency politikasına göre yeni girişim başlat |
| C028 | Kriptografi adaptörü parola özeti ya da TOTP anahtarı üretemedi | `kriptografi` yetkinliğini aç; playground üretmez; web'de giriş kovası aşımı C028 değil 429/503 yanıtıdır (spec/12 §5, K-365); "gizli değeri okunamadı" nedeninde systemd credential dosyasını/dizinini spec/23 §K-339'a göre düzelt |
| C029 | E-posta gönderimi başarısız veya teslim sonucu belirsiz; SMTP/host ayrıntıları maskelenir | `e_posta_bağlantı_değişkeni`, exact `ağ_hedefleri`, alıcı ve ekli iletide readonly dosya/başlık sınırlarını denetle; başarısız `dene` sonucu teslim edilmedi garantisi değildir, otomatik tekrar yapma |
| C030 | Ağ bağlantısı işlemi başarısız ya da bağlantı kalıbı bağlamı dışında | Okuma/yazma ret Sonuç'unda Hata verisinin `tür` değerine bak: `kapandı`, `zaman_aşımı`, `satır_uzun`, `satır_sonu` (çıplak CR/LF), `kodlama`, `kapanıyor` (SIGTERM), `yazılamadı`, `tls` (STARTTLS reddi: aynı pakette gelen düz metin — bağlantı kapanır —, zaten TLS, kimlik yok, el sıkışma başarısız); K-384 blok/bayt/aktarım: `boyut_aşımı` (`bağlantı_blok_sınırı` ya da bellekte satır sınırı; nokta bloğu sonlandırıcıya kadar tüketilir, literal hiç okunmaz; base64 aktarımında ayrıca 32 MiB kaynak dosya tavanı, bayt yazılmaz), `geçersiz` (negatif bayt sayısı, 1 sn'den kısa okuma süresi), `dosya` (blok dosyası açılamadı/yazılamadı, aktarılacak dosya yok, normal değil ya da dosya sınırı dışında); K-396 satırlı base64 aktarımı (`… dosyasını base64 satırlarıyla bağlantıya aktarmayı dene`, aralıklı `… satırlarının K baytından N baytını …`) aynı türleri kullanır, negatif konum/bayt sayısı `geçersiz`, kodlanmış çıktının sonunu aşan konum hata değil 0 bayttır. Zaman aşımı bağlantıyı kapatmaz; yarıda kalan blok okuma yönünü kapatır (sonraki okumalar `kapandı`). Protokolün uygun yanıtını yaz. `bağlantıya … satırını yaz` cümlesinin hatası yalnız o bağlantıyı kapatır. Kayıtsız dinleyici, ikinci blok, web sunucusuyla birlikte kullanım ve blok dışı kullanım yürütmeyi durdurur (K-381). K-397 giden bağlantı (`adresine bağlanmayı dene`, ADR-173): `hedef_yasak` (`ağ` kapalı, hedef `ağ_hedefleri` `tcp://` listesinde değil, joker 25 dışında ya da çözülen IP'lerden biri public değil/IPv4-mapped — hiç bağlantı açılmaz), `çözülemedi`, `bağlanamadı`, `zaman_aşımı` (bağlanma en çok 30 sn), `zaten_bağlı` (görevin açık giden bağlantısı var; önce `bağlantıyı kapat`), `sınır` (süreçte 16 giden bağlantı), `geçersiz` (`<ana>:<kapı>` biçimi değil; dinleyici görevinde giden bağlantı; TLS istemcisinde sunucu adı ASCII DNS adı değil). İstemci TLS'inde `tls` ayrıca STARTTLS yanıtından sonra gelen şifresiz veriyi (bağlantı kapanır) ve el sıkışma hatasını kapsar; sertifika doğrulama hatası ret değildir, başarı sözlüğünde `doğrulandı=hayır` + `doğrulama_hatası` olarak döner |
| C031 | Listede o sırada öğe yok (sıra 1 ile adet aralığı dışında) | Sırayı `listenin adedi` ile sınırla ya da olmayabilecek öğe için `listenin sıra. öğesini almayı dene` kullan; ret Sonuç'unda Hata kodu `SIRA_DISI` (K-386) |
| C032 | Kodlanmış içerik çözülemedi | `Hata` verisinin `tür` değerine bak (K-389, RFC-0043 Dilim 1): `alfabe` (base64 alfabesi dışında ya da QP'de yazdırılabilir ASCII dışı karakter), `dolgu` (ortada/fazla dolgu, kanonik olmayan dolgu bitleri), `uzunluk` (4'e bölünmeyen base64 — dolgu zorunlu), `kaçış` (QP `=` ardından iki onaltılık hane yok), `satır_sonu` (QP'de çıplak CR/LF), `utf8` (aralıklı metin okumada geçersiz UTF-8 ya da bölünen çok baytlı dizi; ikili içerik için `base64 okumayı dene`), `boyut_aşımı` (çağrı başına çözülen çıktı 1 MiB'ı aşıyor; parçalara bölüp `eklemeyi dene` kullan). `konum` ilk hatalı baytın girdi (utf8'de dosya) içi konumudur; mesaj girdiyi taşımaz. Hedef dosyaya dokunulmamıştır |
| C033 | DNS sorgusu geçici olarak başarısız (RFC-0044 B1) | Hata verisinin `tür` değerine bak: `zaman_aşımı` (5 sn ya da son tarihin kalanı), `sunucu_hatası` (SERVFAIL — DNSSEC doğrulaması başarısız yanıt dahil —, FORMERR, NOTIMP), `ret` (REFUSED), `ağ` (yerel çözücüye ulaşılamadı ya da `dns_çözücü` bildirilmedi), `boyut_aşımı` (64 kayıt, 4 KiB TXT/TLSA, 8 CNAME), `bütçe`, `geçersiz_ad` (ASCII harf/rakam/`-`/`_`, etiket 1–63, ≤ 253 oktet; paket gönderilmez), `bozuk_yanıt`. Geçici hata "kayıt yok" değildir: posta akışında `451 4.4.3` gibi yeniden dene yanıtı ver. Kayıt yok/alan yok başarılı boş listedir (K-390) |

## D — Doğrulama

| Kod | Ne oldu | Çözüm |
|---|---|---|
| D001 | `olmalı` koşulu tutmadı | Tanı beklenen/bulunan değerleri gösterir; `dil dene` testte raporlar |

## P — Proje bildirimi

| Kod | Ne oldu | Çözüm |
|---|---|---|
| P001 | `proje.dil` bilinmeyen/tekrarlı alan ya da değer tanımı dışında cümle içeriyor | `proje`, `sürüm`, `morfoloji`, `giriş`, `yetkinlikler`, `ağ_hedefleri`, `yerel_bağımlılıklar`, registry root alanları ve `uzak_bağımlılıklar` alanlarını kullan |
| P002 | Zorunlu proje alanı eksik | Eksik `proje`, `sürüm` veya `giriş` satırını ekle |
| P003 | Proje adı ya da `X.Y.Z` sürümü geçersiz | Boş olmayan ad ve üç sayılı sürüm kullan: `0.1.0` |
| P004 | Giriş mutlak, proje dışına çıkan veya `.dil` olmayan yol | Proje içinde kalan göreli `.dil` yolu kullan |
| P005 | Yerel bağımlılık listesi ya da yollarından biri geçersiz | Göreli proje klasörlerini Metin listesiyle yaz: `yerel_bağımlılıklar "../ortak" listesi olsun` |
| P006 | Yerel bağımlılık klasörü/bildirimi okunamadı | Yolun `proje.dil` taşıyan erişilebilir bir zee projesi olduğunu doğrula |
| P007 | Bağımlılık döngüsü, yinelenen paket adı veya kaynakta kullanılamayan paket adı | Döngüyü kır; her pakete benzersiz, küçük harfli tek Türkçe/Latin tanımlayıcı ad ver (başka alfabe harfi, birleştirici im yok — K-363) |
| P008 | `proje.kilit` eksik ya da kaynak/bildirim grafiğiyle uyuşmuyor | Değişikliği incele, sonra `dil kilitle .` çalıştır |
| P009 | Paket/proje girişi güvenli ve gerçek bir `.dil` kaynağına çözülemedi | Girişi proje içindeki sembolik bağ olmayan bir kaynağa yönelt |
| P010 | Kaldırılmak istenen paket en az bir proje kaynağında hâlâ kullanılıyor | Önce `<ad> paketini kullan` satırını ve pakete bağlı çağrıları kaldır |
| P011 | Projenin sabitlediği morfoloji profili bu derleyicide desteklenmiyor | Derleyiciyi/projeyi uyumlu sürüme getir; v1 için `morfoloji "zee-tr-1" olsun` kullan |
| P012 | Yayıncı anahtarı veya tekrar üretilebilir paket/imza/SBOM/provenance zinciri üretilemedi | Anahtar izin/biçimini, kaynak paket limitlerini ve yerel bağımlılık/sembolik bağ olmadığını denetle; ayrıntı fail-closed nedeni gösterir. Proje adı küçük harfli tek Türkçe/Latin tanımlayıcı olmalı (iç P007); dosya/klasör adı Windows aygıt adı (`CON`, `NUL`, `COM1`…) olamaz, nokta/boşlukla bitemez (K-366) |
| P013 | Registry root/metadata imzası, eşik, sürüm, süre, kanonik biçim, üst rol boyut/özet bağı veya kalıcı geçmiş doğrulanamadı | Registry kök sabitlemesini ve rol metadata zincirini yenile; doğrulanmayan aynayı/cache durumunu kullanma |
| P014 | Exact hedef bulunamadı; targets yayıncı/paket bağı uyuşmadı; sürüm yanked ya da etkin kritik duyurudan etkilendi | Doğru exact sürüm/yayıncıyı seç; yanked/duyuru baypasını yalnız açık gerekçe ve kilit kaydıyla uygula |
| P015 | Yetkinlik listesi/hedefi geçersiz ya da paket üst projenin iznini aşıyor | Yetkinliği ve tam origin'i ana `proje.dil` içinde açıkça onayla; public ağda HTTPS kullan |
| P016 | Registry HTTPS taşıması, içerik-adresli cache veya atomik kalıcı durum işlemi başarısız | Origin/root pin/cache yolunu denetle; bozuk nesneyi kullanma ve güvenilir aynadan yeniden doğrula |
| P017 | Uzak bağımlılık exact `ad@X.Y.Z` değil ya da HTTPS registry/root sabitlemesi eksik/geçersiz | Exact sürüm, HTTPS origin, pozitif root sürümü ve ağ dışı `sha256:` root özetini birlikte bildir |
| P018 | `dinleyiciler` bildirimi `<ad> <adres>:<kapı>` değil; ad/adres yinelenmiş, kapı 0, IPv6 kapsamlı ya da IPv4 eşlemeli, 64'ten fazla öğe | Her dinleyiciyi `"smtp_v4 0.0.0.0:25"` biçiminde, systemd `FileDescriptorName=` ile aynı tekil adla yaz; çift yığında aileye ayrı ad ver |
| P019 | `e_posta_ek_türleri` boş, Metin listesi değil, ek türü kaydında olmayan (makro/yürütülebilir/betik dahil) ya da yinelenen tür | Listeyi kanonik adlarla yaz (pdf, png, jpeg, webp, txt, csv, ics, docx, xlsx, pptx, zip); liste kümeyi tam belirler, varsayılanları da yaz; makro taşıyan ve yürütülebilir türler açılamaz |
| P020 | Bağlantı ayarı (`bağlantı_sınırı`, `uzak_adres_bağlantı_sınırı`, `bağlantı_satır_sınırı`, `bağlantı_boşta_süresi`, `bağlantı_ömrü`, `bağlantı_dolu_yanıtı`, `bağlantı_blok_sınırı`) aralık dışı, sabit değil, kibar ret satırı ASCII/CR-LF kuralını bozuyor ya da `dinleyiciler` bildirimi yok | Aralıkları tanıdaki gibi yaz; uzak adres sınırı genel sınırı aşamaz; ret satırını en çok 512 baytlık yazdırılabilir ASCII yaz |
| P021 | TLS bildirimi bozuk: `tls_sertifika_değişkeni` / `tls_anahtar_değişkeni` eksik, aynı ya da A-Z/0-9/_ dışı ad; `tls_dinleyicileri` kimliksiz, `dinleyiciler`de olmayan ya da yinelenen ad | Zincir ve anahtar için iki ayrı credential/ortam adı yaz (değerin kendisini değil); implicit TLS dinleyicilerini `dinleyiciler`deki adlarla yaz |
| P022 | Boyut ayarı geçersiz: `gövde_aşımı_rotası` ya da `geniş_gövde_rotaları` ögesi `/` ile başlamıyor, 256 baytı aşıyor, sorgu/parça/yüzde/boşluk/denetim karakteri taşıyor ya da yinelenmiş; `geniş_gövde_rotaları` ile `geniş_gövde_sınırı` birlikte değil ya da sınır 65 537–1 048 576 dışında; `veritabanı_parametre_sınırı` 65 536–4 194 304 dışında ya da veritabanı bildirimi yok | Sorgusuz mutlak yol yaz (`gövde_aşımı_rotası "/ileti/cok-buyuk" olsun`; `/` ile biten geniş rota öneklidir); geniş rotaları ve sınırı birlikte, SQL tavanını veritabanı bildirimiyle yaz |
| P023 | Paylaşılan veri kökü bildirimi bozuk: `veri_kökü` A-Z/0-9/_ dışı ya da 128 baytı aşan ad, `veri-okuma`/`veri-yazma` yetkinliği olmadan `veri_kökü`, ana projede `veri_kökü` olmadan `veri-*` yetkinliği ya da paketin `veri_kökü` bildirmesi (kök yalnız ana projeden gelir) | Yolu değil kökün mutlak yolunu taşıyan ortam değişkeninin / systemd credential'ın adını yaz (`veri_kökü "UYGULAMA_DEPO" olsun`); yalnız okuyan hizmete `veri-okuma`, yazan hizmete ayrıca `veri-yazma` ekle; `veri-*` kullanan kütüphane paketi tek başına denetlenirken P023 beklenir: paketi kullanan kök projeden ya da paketi yerel bağımlılık olarak alıp `veri_kökü` bildiren bir test kök projesinden denetle |

## Ç — İç akış

| Kod | Not |
|---|---|
| Ç000 | `programı bitir` iç nöbetçisi; çalıştırıcı yakalar, kullanıcı asla görmez |
| Ç001 | Son tarih iptal nöbetçisi; yalnız sahibi `içinde/yetişmezse` bloğu yakalar, kullanıcıya sızmaz |
| Ç002 | Web isteğinde parola özeti giriş oran reddi nöbetçisi; istek döngüsü 429/503 sabit yanıtına çevirir, kullanıcıya sızmaz |
| Ç003 | Web isteğinde oturum deposu yeni kaydı reddetti (kapasite, depo IO, kilit); istek döngüsü sabit 503 yanıtına çevirir, kullanıcıya sızmaz |

> S032, A004 ve C014 bilinçli mezar taşlarıdır; ayrılmış kodlar başka anlamda
> kullanılamaz. Yeni tanılar ilgili ailedeki sıradaki boş numarayı alır.

K-208: parola doğrulama intrinsic’inde yanlış argüman türü mevcut T034
koduyla `Parola ve parola özeti Metin olmalı.` bildirir. Bcrypt ve
Argon2id aynı Metin imzasını kullanır; bozuk hash/yanlış parola tanı
üretmek yerine `yanlış`tır. K-209 eylem içi IP okuması mevcut T044’tür.

K-330: `oturum_ömrü_saniye` 300–2592000 sn aralığı dışında ya da `web-oturumu` yetkinliği olmadan `oturum_ömrü_saniye`/`oturum_kayar` kullanımı P015, Sayı olmayan ömür ve Mantıksal olmayan `oturum_kayar` P001 verir; yeni tanı kodu yoktur. [Karar](../adr/158-proje-basina-oturum-omru-ve-kayar-yenileme.md).

K-220: yeni HTTP/ortam/JSON işlemlerinin beklenen hatası C018 kodlu Sonuç değeridir; adaptör ayrıntısı gizlenir. T034 argüman türü, T048 eylemde geri alınamaz istek ve P015 proje ortam listesi tanıları mevcut kodları korur. Üst program deadline C020 kalır. [Sözleşme](../adr/091-http-post-baslik-ve-gizli-ortam.md).

K-378: `çok_parçalı_yükleme` bildirimi mantıksal değilse P001, `web-oturumu`
yetkinliği olmadan açılırsa P015'tir. Multipart istek reddi (400 bozuk gövde,
403 CSRF/oturum, 413 tavan) tanı kodu üretmeyen ham HTTP yanıtıdır; rota
çalışmaz. [Karar](../adr/163-ikili-yanit-multipart-ve-ek-turu-kaydi.md).

K-378: `e_posta_ek_türleri` bildirim hataları P019'dur (`ağ` yetkinliği
yoksa P015). Bildirime uymayan, imzası/UTF-8'i türüyle uyuşmayan ya da sınırı
aşan ek çalışma zamanında yine C029'dur, ama ağa çıkmadan verilen bu yerel
ret `MIME e-posta gönderilemedi: ek reddedildi (<neden>)` sabit metniyle
SMTP/ağ hatasının maskesinden ayrılır (K-236 maskesi diğer hatalar için aynen).

K-394: paylaşılan veri kökü bildirim hataları P023'tür; çalışma zamanındaki `veri:` yolu, kök ve yetkinlik redleri C013 ailesinde başarısız `Sonuç`tur ([RFC-0045](../rfcs/0045-paylasilan-veri-koku.md), [ADR-170](../adr/170-paylasilan-veri-koku.md)).

K-381: bağlantı ayarı hataları P020, TLS bildirimi hataları P021'dir (RFC-0039 A2a/A2b); TLS kimliği okunamaz ya da çözülemezse süreç kaynak yürütülmeden `bağlantı TLS'i:` iletisiyle çıkış 2 verir, ileti anahtar içeriği taşımaz. Çalışma zamanı
bağlantı hataları C030'dur: beklenen okuma/yazma reddi yakalanabilir `Sonuç`,
`tür` Hata verisiyle; görev hatası yalnız o bağlantıyı kapatır ve nedeni
operatör günlüğüne `Zee bağlantı #N … kapandı:` satırıyla düşer.
[Karar](../adr/165-ag-dinleyici-baglanti-yuzeyi.md).

K-391: web isteği ve SQL parametresi boyut ayarı hataları P022'dir ([ADR-168](../adr/168-istek-boyut-sinirlari-ve-govde-asimi.md)).
Gövde boyutu 413'ü tanı değildir; `gövde_aşımı_rotası` bildirimli projede
salt okuma GET rotasının sayfası 413 durumuyla gider, rota yoksa ya da 200
metin vermezse ham `istek gövdesi … KiB sınırını aşıyor` iletisi kalır.
K-390: `dns` yetkinliği, `dns_çözücü` ve `dns_dnssec` üçlüsünün eksik ya da
tek başına bildirimi, loopback dışı/adlı/0 kapılı çözücü ve bilinmeyen
`dns_dnssec` kipi P015'tir; yeni proje tanı kodu yoktur. `dns_dnssec
"zorunlu"` iken çözücü kök bölgeyi AD=1 ile yanıtlamazsa süreç kaynak
yürütülmeden `dns_dnssec zorunlu:` iletisiyle çıkış 2 verir. DNS sorgu
hataları C033 kodlu `Sonuç`tur. [Karar](../adr/169-dns-stub-cozucu-ve-tel-bicimi-siniri.md).

K-376: `dinleyiciler` bildirim hataları P018'dir. Çalışma zamanındaki adlı
dinleyici devri ihlalleri (eksik/fazla ad, başka PID, yanlış adres, V6ONLY
kapalı) tanı kodu üretmeyen CLI başlangıç hatalarıdır: `dinleyici devri:`
önekli ileti, `.socket` önerisi ve çıkış 2. [Karar](../adr/162-adli-dinleyici-devri-ve-hizmet-bildirimi.md).

K-234 `zee.ayar` okuma/şema/güvenlik hataları tanı kodu üretmeyen CLI
exit 2 retleridir; anahtar/değer/yol veya JSON alıntısı yazılmaz.
Bilinmeyen olağan alan sabit uyarıyla güvenli varsayılanı korur.

K-232 ekli ileti C029 Hata değeridir: SMTP bağlantı/izin, readonly ek
yol/boyut/tür, başlık/Reply-To ve HTML/düz gövde sınırlarını kontrol edin.
Hata SMTP credential, ileti/ek içeriği veya dosya yolu taşımaz; DATA ack
kaybı gönderilmedi kanıtı değildir ve otomatik retry yapılmaz.

K-236 eski plain ortam/explicit SMTP kalıplarında C029 kodunu korur; native
transport ve host ayrıntıları şu sabit mesajla gizlenir: `E-posta gönderilemedi;
ayar geçersiz veya gönderim sonucu belirsiz; ayrıntılar gizlendi; otomatik tekrar yok`.
AUTH PLAIN echo, URL/credential ve policy ayrıntısı kullanıcı hata metnine girmez.
Bu maske genel sır taint sistemi veya SMTP teslim onayı değildir.
[Kanıt](../adr/099-smtp-hata-maskelemesi.md).

K-263 host supervisor mevcut ownership/cleanup retlerini korur: geçici owned EPERM/WNOWAITNone aynı grace budget içinde yeniden gözlenir; kalıcı belirsizlik ve kaybolmuş child authority failclosed kalır. Compiler tanı kodu, özel argv/customer mesajı veya yeni hostret adı eklenmez. [ADR-122](../adr/122-supervisor-sahipli-eperm-gecisi.md).
