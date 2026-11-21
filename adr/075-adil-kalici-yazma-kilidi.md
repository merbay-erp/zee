# ADR-075 — Adil kalıcı yazma kilidi: bekleyen yazar yoklamaz, kuyrukta bloklanır

- **Durum:** kabul
- **Tarih:** 10 Eylül 2026
- **İlgili kayıt:** K-183, K-182, K-346, K-347, K-361, ADR-032, RFC-0016, spec/08, spec/11

## Bağlam

Kalıcı dosya yazması aynı klasördeki `.zee-yazma-kilidi` üzerinde süreçler
arası özel kilit alır; spec/08 beş saniyede alınamayan kilidi hata sayar.
Kilit `flock(LOCK_NB)` / `LockFileEx(FAIL_IMMEDIATELY)` ile 5 ms aralıkla
yoklanıyordu. İlk gerçek Tier-1 koşuları (K-182) iki kez aynı sınıfta düştü:
ubuntu-latest'ın yavaş diskinde kilidi ardışık yeniden alan bir yazar
(10 yazar × 10 rate-limit artışı; sonra 2 yazar × 40 fsync'li ekleme) 5 ms
yoklayan diğerini beş saniye boyunca dışarıda bıraktı. Bu disk hızı değil,
yazar açlığıdır: yoklayan taraf boş pencereyi kaçırır, tutan taraf fsync'ten
döner dönmez yeniden alır. Ürün analoğu çok worker'lı web'de sürekli ekleyen
bir worker'ın diğerini C013'e düşürmesidir.

## Karar

1. **Hızlı yol aynı.** Boş kilit `LOCK_NB`/`FAIL_IMMEDIATELY` ile iş parçacığı
   açmadan alınır.
2. **Bekleyen bloklanır.** Kilit doluysa yardımcı iş parçacığı bloklayan
   `flock(LOCK_EX)` (EINTR yeniden dener) ya da `LockFileEx` (FAIL_IMMEDIATELY
   yok) ile çekirdeğin kilit kuyruğuna girer; çağıran `KILIT_BEKLEME` (5 sn)
   kadar kanaldan bekler. Süre dolarsa aynı C013 metniyle hata döner; geç
   gelen kilit yardımcı iş parçacığında anında bırakılır, sızmaz.
3. **Söz değişmedi.** spec/08'in beş saniye sınırı ve dosya korunması aynıdır;
   eklenen tek söz bekleyenin aç bırakılmamasıdır. Kilit desteği olmayan
   platform yine reddeder.
4. **Kanıt.** `iki_yazar_satir_kaybetmez` 2 × 40'a geri döndü;
   `kilidi_ardisik_yeniden_alan_yazar_bekleyeni_ac_birakmaz` A'nın 120 ardışık
   eklemesi sırasında B'nin tek eklemesinin 2 sn içinde tamamlanmasını ister.
   Platform katmanı `kalici_dosya/platform.rs`e ayrıldı (mimari satır bütçesi).

## Reddedilen seçenekler

- **Testleri küçültmek:** kayıpsızlık kanıtlanır ama açlık gizlenir; CI'ın iki
  kez gösterdiği davranış ürün sorunudur.
- **Beş saniyeyi uzatmak:** spec sözünü gevşetir, açlığı çözmez.
- **Kullanıcı alanında FIFO bilet kilidi:** süreçler arası bilet sayacı ek
  dosya ve kendi yarışını getirir; çekirdek kuyruğu yeterli ve daha basit.
- **Yoklama aralığını kısaltmak:** CPU yakar, açlığı yalnız olasılık olarak
  azaltır.

## Sonuçlar

- Bekleyen yazar CPU harcamaz; en kötü bekleme diğer yazarın tek bir fsync'li
  yazması kadardır. Çekirdek kuyrukları katı FIFO sözü vermez; kanıt ölçülen
  üst sınırdır, teorik adalet değil.
- Her çekişmeli alım bir kısa ömürlü iş parçacığı açar; fsync maliyeti
  yanında ihmal edilebilir, hızlı yolda yoktur.
- K-183 kapandı; `kilit_yeniden_dene_ms` limiti API'de kalır ama resmî yol
  artık yoklamaz.

## Ek — K-346: eşzamanlı ekleme ve eylem geri alması (19 Kasım 2026)

**Bağlam.** K-336 çok işçi profiliyle aynı proje kökünde birden çok süreç
olağanlaştı. "İki süreç aynı dosyaya eklerse satır kaybolur mu?" sorusu iki
yolda ölçüldü. Düz `ekle` kayıpsızdı: 4 süreç × 1000 ekleme 4000/4000
(oku-değiştir-yaz bu ADR'nin kilidi altındadır). Kayıp eylem yolundaydı:
gerçek adaptör eylem başı yedeğini yazımdan **önce**, eylemin bıraktığı
içeriği yazımdan **sonra** kilit dışında okuyordu. Araya giren süreç ekleme
yaparsa yedek ya da "beklenen" onun satırını içerir/atlar; geri almanın
karşılaştırması geçer ve eylem başı içeriği geri konunca o sürecin başarılı
satırı silinir. Ölçüm: eylem içinde ekleyip başarısız Sonuç döndüren
süreçlerle yarışan düz yazar 1000 satırından 48'ini kaybetti.

**Karar.** Kilit + aynı klasörde geçici dosya + atomik replace korunur.
`kalici_dosya` yazım ve silmenin hedefin önceki ve yayımlanan bütün içeriğini
**aynı kilit penceresinden** döndüren `_izli` biçimlerini sunar
(`atomik_satir_yaz_izli`, `atomik_sil_izli`, `KilitliDegisim`). Eylem defteri
ilk dokunuşta yedeği bu önceki içerikten alır; sonraki dokunuşta önceki içerik
savepoint'in en son bıraktığından farklıysa kaydı çatışmalı işaretler ve geri
alma eylem başını geri koymak yerine C021 verir (spec/11). Eylem dışındaki
yazımlar ek okuma yapmaz.

**Reddedilen: `O_APPEND` tek-write.** Ekleme maliyetini O(dosya)'dan
O(satır)'a indirirdi, ama (1) spec/08'in "okuyucu eski ya da yeni bütünü
görür" sözü bozulur: eşzamanlı okuyucu süren `write`'ın önekini, yani yarım
satırı görebilir; (2) çökme/güç kaybında POSIX kısa ya da sıfır dolu kuyruk
bırakabilir; kurtarma bir "yarım son satır" sezgisi ister ve sonu `\n`
olmayan meşru dosyayı ayırt edemez; (3) Windows'ta `FILE_APPEND_DATA` aynı
kilit, metadata (spec/08 DACL/stream) ve K-342 okuyucu penceresi sözlerini
taşımaz; (4) eylem geri alması yine önceki içeriği ister, kayıp sınıfı
çözülmezdi. Tek kazanç performanstır; büyük günlük dosyalarında ekleme
maliyeti açık sınır olarak kalır (her ekleme bütün içeriği yeniden yazar ve
fsync eder). Ölçüm: macOS'ta 4 süreç × 1000 ekleme ~35 sn.

**Ayrı K-347 (kapandı, aşağıdaki ek).** Aynı kilit-dışı okuma iki yolda daha vardı: atomik
taşımanın eylem yedeği kaynak içeriğini kilit dışında okur ve yalnız hedef
klasörünü kilitler; `dil paket ekle/çıkar` `proje.dil`i karşılaştırmasız
yazar, eşzamanlı iki komutta biri diğerinin bağımlılığını ezer. Kalıcı web
deposu (`atomik_karsilastir_ve_yaz`) ve registry durumu zaten CAS'lıdır.
Kullanıcı programının kendi oku → hesapla → `yaz` dizisi (ör. sayaç dosyası)
dil düzeyinde atomik değildir; spec/08 baştan yazmada son yazarın kazandığını
söyler. Bunun için ürün kanıtı olmadan yeni dil yüzeyi açılmaz (core freeze).

**Kanıt.** Regression `eszamanli-ekleme-geri-alma`
(`native-eszamanli-ekleme`: iki düz yazar süreci × 1000 + iki geri alan
süreç; düzeltmesiz b 996–999/1000, düzeltmeli 1000/1000, bozuk satır 0, üç
platform), `eylem_geri_almasi_araya_giren_yazarin_eklemesini_ezmez`
(deterministik iki dokunuş arası yazar), `izli_yazim_ve_silme_kilit_altindaki_onceki_ve_sonrakini_dondurur`.
GB-137.

K-217 Gitsrv kabul incelemesi: paylaşılan disk üzerinde bekleyen yazar
3,593 sn'de tamamlandı. Eski testin 2 sn varsayımı taşınabilir performans
garantisi değildir; kapı normatif 5 sn kilit bütçesini kullanır. Bloklayan
OS kilidi ve süre aşımında fail-closed dönüş değişmedi. OS kuyruğu tüm
platformlarda FIFO sırası garantilemez; bu karar böyle bir söz vermez.

## Ek — K-347: taşıma yedeği ve `proje.dil` karşılaştırması (21 Kasım 2026)

**Bağlam.** K-346 taramasında kalan iki kilit-dışı okuma. (1) Eylem içindeki
`atomik_tasi` kaynağın içeriğini ve eylem yedeğini kilit dışında okuyor,
kilit olarak yalnız hedef klasörünü alıyordu. Eylem kaynağa ekleyip taşırken
araya giren yazarın satırı "beklenen" içeriğe girer, yedeğe girmez; geri
alma hedefi silip kaynağa eylem başı içeriğini koyar ve satır kaybolur.
Ölçüm: eylem içinde ekleyip taşıyan ve başarısız dönen iki süreçle yarışan
düz yazar 1000 satırdan 4–160'ını kaybetti (macOS debug, 4/4 koşu). (2) `dil
ekle/çıkar` `proje.dil`i okuyup grafiği çözdükten sonra karşılaştırmasız
yazıyordu; 4 tur × 6 eşzamanlı `dil ekle`de başarı bildiren 24 komuttan 20'sinin
bağımlılığı ezildi.

**Karar.** (1) `atomik_tasi` kaynak ve hedef klasör kilitlerini birlikte
alır: aynı klasörde tek kilit (aynı süreçte ikinci kilit kendini 5 sn
bekletirdi), ayrı klasörlerde kanonik klasör yolu sırasıyla; ters yönlü iki
taşıma kilitlenmeye girmez. `atomik_tasi_izli` taşınan bütün içeriği
adlandırmayla aynı kilit penceresinden döndürür ve eylem defteri bunu
K-346 `KilitliDegisim` gibi kaydeder (kaynakta önceki içerik → yok, hedefte
yok → içerik); önceki dokunuştan sonra kaynak değiştiyse kayıt çatışmalıdır.
Geri alma çatışmayı hiçbir yolu değiştirmeden önce denetler ve içerik geri
koymalarını silmelerden önce yapar. (2) `kalici_dosya::karsilastirip_cift_yaz`
bildirimi okunan byte'larla karşılaştırır ve bildirim ile türetilmiş
`proje.kilit`i tek kilit penceresinde yayımlar; kilit yazılamazsa iki dosya
aynı kilit altında, karşılaştırmasız (araya kimse giremez) önceki byte'larına
döner. Okuma yolu (`kaynak_dosyasi_oku`) byte'ları dönüştürmeden UTF-8 çözer;
karşılaştırma bu yüzden kesindir. Windows'ta ikinci dosyanın yarım
değiştirmesi de aynı kilit altında kurtarılır.

**Windows kaynak toplama.** PR CI'ında Windows koşularından birinde bir
eşzamanlı `dil ekle` karşılaştırma kaybı dışında bir hatayla düştü; yeniden
koşuda üretilemedi ve stderr kaydedilmemişti (harness artık yazar). En olası
neden incelemeyle bulundu: proje kaynak toplayıcısı her `.dil` dosyasını,
`proje.dil` dahil, okumadan önce `canonicalize` ediyordu; Windows'ta bu dosyayı
açar ve başka komutun `ReplaceFileW` penceresinde düşebilir, oysa ardından
gelen okuma pencereyi bekler. Klasör zaten kanonik ve sembolik bağlar
atlandığından çağrı kaldırıldı. `proje_testi::kaynak_toplama_proje_dil_degistirilirken_dusmez`
bu yarışı sürekli yeniden yazılan `proje.dil` üstünde 300 çözümle zorlar.

**Reddedilen: komutu kilit altında yeniden çözmek.** K-353
`kilit_altinda_guncelle` tek dosyalık oku-değiştir-yaz için uygundur; burada
dönüşüm paket grafiğini (ağ dahil registry zinciri) yeniden çözmek demektir
ve kilidi saniyelerce tutardı. CLI komutu kaybedince dosyaya dokunmadan
yeniden çalıştırılabilir; ürün yolunda (web/eylem) kullanılmaz.

**Kanıt.** `main.rs::eylem_tasimasi_araya_giren_yazarin_satirini_geri_almada_silmez`,
`yasam_dongusu::tests::tasima_icerigi_kilit_altinda_yakalanir`,
`yasam_dongusu::tests::ters_yonlu_tasimalar_kilitlenmez`,
`kilitli::tests::cift_yazim_karsilastirir_ve_ikincil_hatasinda_geri_alir`,
`proje_testi::bildirim_yazimi_araya_giren_komutun_bagimliligini_ezmez`;
regression `eszamanli-tasima-geri-alma` (`native-eszamanli-tasima`) ve
`eszamanli-paket-ekle` (`native-eszamanli-paket-ekle`). GB-144.

## Ek — K-361: registry kalıcı durumunun kilit içi uzlaştırması (21 Kasım 2026)

**Bağlam.** K-353 iyimser oku+CAS döngüsünü web deposunda kaldırdı; registry
istemcisi (`registry/istemci.rs`) aynı deseni yeniden denemesiz kullanıyordu.
`durum-v1.json` kilit dışında okunur, zincir ağdan doğrulanır, durum kilit
altında tek CAS ile yayımlanırdı. Aynı başlangıç durumunu okuyan ikinci
çevrimiçi çözüm aynı zinciri getirse bile P016 ile düşüyordu (yarış
regresyonunda 4 senaryonun 4'ünde).

**Karar.** Okuma kilit dışında kalır: doğrulama ağ ister ve kilit spec/08'in 5
sn bekleme sınırını aşacak süre tutulamaz. Yayın `kilit_altinda_guncelle` ile
yapılır ve CAS değil uzlaştırmadır: başlangıç byte'ları yerindeyse yeni durum
yazılır; değilse güncel durum aynı pencerede kapalı/kanonik şemayla
ayrıştırılır, ağ dışı root sabitlemesi aynı olmalıdır (aksi P016) ve
root/timestamp/snapshot/targets rolleri sürüm+özetle karşılaştırılır. Her
rolde eşit → dosyaya dokunulmaz; eşit ya da ileri → yeni durum yazılır;
eşit ya da geri → durum korunur ve sonuç yeni kalıcı durumla çevrimdışı
yeniden doğrulanır, böylece yanked/kritik duyuru eski zincirle atlanamaz.
Aynı sürümde farklı özet ya da karışık sıra çelişkidir (P013). Yeniden deneme
döngüsü yoktur; ilerlemeyi ADR-075 kilit kuyruğu verir.

**Reddedilen.** (1) Bütün güncellemeyi kilit altında koşmak: ağ süresi kilit
bekleyenleri 5 sn sınırında düşürür. (2) CAS kaybında bütün güncellemeyi
yeniden denemek: K-353'ün açlık sınıfını ve ağ tekrarını geri getirir. (3)
Geride kalan sonucu eski zincirle döndürmek: yeni durumun gördüğü duyuru
atlanırdı (freeze).

**Kanıt.** `registry::istemci::testler::eszamanli_guncellemeler_durumu_uzlastirir_ve_geri_gitmez`
(aynı/ileri/geri/duyuru/çelişki), `zincir_sirasi_karisik_gorunumu_celiski_sayar`;
regression `registry-durum-cekismesi` (`native-registry-durum-cekismesi`). GB-145.
