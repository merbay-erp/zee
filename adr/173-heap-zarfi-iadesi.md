# ADR-173 — C024 heap zarfı düşen değerin payını iade eder

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-393, GB-167, ADR-033 (iade-yok ilkesini değiştirir),
  ADR-165 (bağlantı birimi bütçesi), ADR-059 (core freeze), RFC-0025,
  RFC-0043 §seçenek (b), spec/24

## Bağlam

ADR-033 çalışma/istek başına yaklaşık 64 MiB "iade edilmeyen saklanan değer
tahsis zarfı" kurdu: ortam/koleksiyon yazımı ve görev ortamı klonu zarftan
düşer, değer düşünce geri verilmez. Zarf canlı grafiğe üst sınır olsun diye
bilinçli olarak fazla sayıyordu.

Bağımsız Rust/Go karşılaştırması (v2, B4) bu fazla sayımın üst sınır değil
**kümülatif tahsis sayacı** olduğunu gösterdi:

| Program | Canlı RSS | Önce |
|---|---|---|
| 700 öğelik listeyi bir işleme 5.000 kez vermek | ~15 MB | C024 "~67 MB" |
| 1.000 öğelik kabarcık sıralama (1.000 tur) | ~9–15 MiB | C024 "~67 MB" |

Kök neden: zarf yalnız düşüyordu. İşlem çerçevesine geçen her argüman
kopyası, her `her … için` gezme kopyası, üzerine yazılan her değişken ve
sözlük öğesi, blok kapsamında doğup ölen her ad zarfı kalıcı olarak
tüketiyordu. Eşik bu yüzden canlı bellekle değil iş miktarıyla (≈ öğe
kopyası × tur) ölçekleniyordu ve tanı ("Çalışma değeri yaklaşık N bayt")
kullanıcıya canlı değer boyutu gibi okunuyordu. Bu bir doğruluk hatasıdır:
spec/24'ün güvenlik niyeti canlı belleği sınırlamaktır, iş miktarını değil.

## Karar

1. **Düşen değer iade edilir.** Zarf canlı saklanan değerlerin muhafazakâr
   üst sınırıdır. İade noktaları:
   - ortamda üzerine yazılan değer (`x … olsun`, `artır/azalt`, `böl`,
     gezme değişkeni, görev sonucu, `yanıt`);
   - blok kapsamı bitince düşen ad (ad uzunluğu + yuva payı + değer);
   - işlem çerçevesi bitince bütün parametre ve yerel adlar;
   - `her … için` gezme kopyası döngü bitince;
   - `eşzamanlı` görev ortamı klonları `hepsini bekle` tamamlanınca;
   - koleksiyonda yerinde değişen öğe (sözlük değeri, yapı alanı, gezme
     imlecinin kaynak listeye geri yazımı) ve `sil` ile çıkan öğe.
2. **İade tüketimi aşamaz (güvenlik değişmezi).** Saklanan her değer için
   `ölçü(değer) ≤ o değer için tüketilen` korunur; iade ölçü kadar yapılır.
   Bunun için yerinde büyüyen her yol ölçüdeki artışın tamamını tüketir:
   - `ekle` ve yeni sözlük anahtarı öğenin ölçüsü + anahtar kapasitesi +
     tamponun **gerçek** kapasite artışını tüketir (`reserve` sonrası ölçülür;
     bütçe tükenirse kapasite geri bırakılır, koleksiyon değişmez);
   - `x x ile y olsun` yerinde büyümesi ayırıcının verdiği gerçek kapasite
     artışını tüketir (önceden istenen artış);
   - gezme imlecinin geri yazımı artık kopyanın ölçüsünü tüketir (önceden
     hiç tüketmiyordu — iade açılınca fazla iadeye dönüşürdü);
   - döngü sayacı ve `(sunucu)`/`(bağlantı)` iç adları da yuva payını tüketir
     ki kapsam sonu iadesi tüketilmemiş payı geri vermesin.
   Ek olarak kalan zarf başlangıç zarfını **hiçbir zaman** aşamaz
   (`min(kalan + iade, 64 MiB)`); başka bir bütçeden (ör. önceki bağlantı
   birimi ya da görev ortamı klonu) tüketilmiş değerin iadesi bu yüzden
   sınırı genişletmez.
3. **Önce tüketim, sonra iade.** Üzerine yazmada yeni değer zarftan düşer,
   eski değer ondan sonra iade edilir: geçiş anında iki kopya da gerçekten
   canlıdır ve zarfa sığmak zorundadır.
4. **Eksik iade fazla sayımdır, TANIMLIDIR.** Hata ile yarıda kalan çerçeve
   ya da döngü payını iade etmeyebilir; bu yalnız güvenli yönde sayımdır.
5. **Davranış sınırı değişmez.** 64 MiB, 16 MiB metin, tek değer sınırı,
   C024 kimliği ve öneri metni aynıdır. Web isteği ve bağlantı birimi taze
   zarfla başlar; rotada C023/C024 yine yalnız o isteği taslağı geri
   alınmış 503 ile bitirir (K-323). Tanıdaki sayı tüketilen zarftır,
   allocator/RSS ölçümü değildir.

## Reddedilen seçenekler

- **Yalnız tanı/katalog/spec dilini dürüst yapmak** (asgari): kullanıcı
  programları yine iş miktarıyla düşerdi; karşılaştırma B4 ve ZeeMail
  birim içi birikimi (RFC-0043) açık kalırdı.
- **Argüman/gezme kopyasını hiç saymamak** (ara): kopya gerçekten canlıdır;
  saymamak üst sınır sözünü bozar ve DoS savunmasını zayıflatır.
- **Çerçeve iadesini bütçe farkıyla yapmak** (giriş − çıkış): `eşzamanlı`
  görevler aynı bütçeyi paylaştığı için iç içe geçen çerçeveler birbirinin
  tüketimini iade ederdi. Ölçü tabanlı iade görev sırasından bağımsızdır.
- **Gerçek allocator/RSS telemetrisi:** taşınabilir değil, deterministik
  değil, WASM'da yok; ADR-033'teki gerekçe geçerli.

## Sonuçlar

- Maliyet: üzerine yazma/kapsam/çerçeve sonu düşen değeri bir kez daha
  ölçer (tüketimdeki ölçümle aynı karmaşıklık); ek durum yok.
- ADR-033'ün "silme veya yeniden bağlamada bütçeyi geri vermez" açık sınırı
  ve ADR-165/K-388'in "birim içi atılan değer iadesi yoktur" notu bu kararla
  kapanır. Birimler arası yaşayan değerin yeni zarfa yeniden sayılmaması
  (K-381) açık kalır.
- Canlı biriktirme (liste/sözlük büyütmek, dönen değeri saklamak) eskisi gibi
  64 MiB'da C024'tür.

## Kanıt

- `kaynak_sinirlari_testi::atilan_degerin_zarf_payi_iade_edilir`: çerçeve,
  üzerine yazma, kapsam, gezme kopyası, sözlük/yapı öğesi ve `sil` yolları
  ~1 MiB canlı değerle 200–100.000 tur çalışır (önce hepsi C024);
- `kaynak_sinirlari_testi::iade_canli_birikimi_zarftan_kacirmaz`: düz canlı
  birikim, iade yollarıyla karıştırılmış birikim, gezme geri yazımıyla
  büyüyen kaynak ve dönen değer birikimi C024 ile durur;
- `baglanti_butce_yenileme_testi`: birim yenilemesi geçici çöp yerine canlı
  birikimle sınanır (3 olumlu, 3 olumsuz);
- `web_testi::istek_heap_butcesi_asimi_503_olur_ve_worker_temiz_butceyle_yasar`
  değişmeden geçer;
- regression `runtime/heap-zarfi-cerceve-iadesi`,
  `runtime/heap-zarfi-uzerine-yazma-iadesi` (`run`);
- K-173 soak 1800 sn.
