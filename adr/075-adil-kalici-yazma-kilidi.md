# ADR-075 — Adil kalıcı yazma kilidi: bekleyen yazar yoklamaz, kuyrukta bloklanır

- **Durum:** kabul
- **Tarih:** 10 Eylül 2026
- **İlgili kayıt:** K-183, K-182, K-346, ADR-032, RFC-0016, spec/08, spec/11

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

**Ayrı K-347 (açık).** Aynı kilit-dışı okuma iki yolda daha var: atomik
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
