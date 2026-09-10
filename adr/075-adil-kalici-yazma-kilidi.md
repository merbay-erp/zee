# ADR-075 — Adil kalıcı yazma kilidi: bekleyen yazar yoklamaz, kuyrukta bloklanır

- **Durum:** kabul
- **Tarih:** 10 Eylül 2026
- **İlgili kayıt:** K-183, K-182, ADR-032, RFC-0016, spec/08

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
