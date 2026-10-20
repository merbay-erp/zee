# ADR-157 — Web rotasında istek-yerel hata sınırı ve operatör günlüğü

- **Durum:** kabul
- **Tarih:** 20 Ekim 2026
- **İlgili kayıt:** K-324, K-323, RFC-0035, ADR-033, ADR-034, ADR-046, ADR-060,
  ADR-007, spec/06, spec/11 §5, spec/24, GB-122, GB-123

## Bağlam

V1 web modeli süreç başına tek worker'dır (ADR-034). Rota yürütmesinden dönen
tanılardan yalnız C013/C021/C027 (F029, K-163) ve C023/C024 (K-323) istek
düzeyinde 503'e çevriliyordu; geri kalanı (`?n=abc` ile C004, sıfıra bölme
C003, CR/LF taşıyan yönlendirme hedefinde C022 …) `return Err` ile süreci
bitiriyordu. Kimliksiz bir istemci tek istekle worker'ı düşürüp yeniden
başlatma döngüsü kurabiliyordu (GB-122). İzinli küme her seferinde sahada süreç
düştükten sonra büyüdü. Ayrıca istek düzeyine çevrilen tanılar hiçbir kanaldan
operatöre görünmüyordu (GB-123); bu, spec/06'nın sessiz devam yasağına
aykırıydı.

## Karar

1. **Ret kümesi, izinli küme değil.** Rota tanısı yalnız o isteği bitirir;
   taslak geri alınır, worker sonraki isteği kabul eder. Durum tanı kodunun saf
   işlevidir (`yorumlayici::istek_hatasi_yaniti`): Ç001 504, bugünkü beş
   geçici/kaynak kodu 503, diğer bütün tanılar sabit gövdeli 500. Süreci yalnız
   iç tutarlılık (C000 ve iç Ç nöbetçileri) bitirir; yorumlayıcının kendi
   değişmezi bozulmuşsa durumu güvenilir değildir. Yanıt commit'inden sonra
   gelen C022 ayrı yoldur ve değişmedi.
2. **Neden güvenli.** Rota global ortamı yalnız okur (T046), her istek taze
   ortam, taze bütçe ve kendi son tarih nöbetçisiyle koşar; eylem, çerez,
   oturum ve yanıt taslağı her hata dalında geri alınır; PostgreSQL bağlantısı
   hata sonrası yeniden kullanılmaz (ADR-060). Ç001 ve beş 503 kodu bu
   temizliğe zaten güveniyordu; tanı kodu temizliği değiştirmez.
3. **Operatör günlüğü iç IO sınırındadır.** `GirdiCikti` `#[doc(hidden)]`
   `yorumlayici` modülündedir ve `dil::api::v1` facade'ı onu dışa vermez. Trait'e
   varsayılanı boş `istek_tanisini_gunlukle(durum, tani)` eklendi; yorumlayıcı
   yanıttan önce çağırır. `PaylasilanIo`, `PolitikaliIo` ve `IzKaydedenIo`
   iletir; `IzKaydedenIo` olay KAYDETMEZ, çünkü program günlüğü gözleyemez ve
   replay değişmemelidir. `GercekIo` `istek_tani_gunluk_satiri` ile tek satırı
   stderr'e yazar: sorgusuz yol, kaçışlanmış mesaj, 4 KiB sınır. Biçim
   bağımlılıksız ve insan okunur `anahtar=değer`dir.
4. **Gövde sabit.** 500 gövdesi “istek işlenemedi; sunucu hatası”dır; olay
   kimliği taşımaz (deterministik test için IO'dan rastgele çekmek gerekirdi).
   C018/C025 dış servis hataları da 500'dür; 502/503 ayrımı yüzeyi büyütürdü.

## Reddedilen seçenekler

- **İzinli kümeyi büyütmek:** her kod saha arızasından sonra ekleniyor.
- **Yalnız “girdiye bağlı” kodları yalıtmak:** hangi kodun girdiye bağlı
  olduğu programa göre değişir; ayrım test edilemez.
- **Süreç yöneticisine bırakmak:** yeniden başlatma süresince bütün
  istemciler etkilenir; saldırgan döngüyü tek istekle sürdürür.
- **Her şeyi 503 yapmak:** deterministik hataya yanlış “yeniden dene” sinyali.
- **Yorumlayıcıda `eprintln!`:** temel→adaptör yönünü (ADR-046) deler, wasm
  hedefinde stderr yoktur ve hermetik test tanıyı doğrulayamaz.

## Sonuçlar

- Rota hatası artık süreç çıkış kodu üretmez; süreç ölümünü sağlık sinyali
  sayan dağıtım stderr'deki `zee-web istek-hatası` satırlarını izlemelidir
  (`docs/surumler.md` göç notu).
- CLI ve `dil dene` değişmez; sunucu kurulmadan önceki üst düzey hata yine
  süreci tanıyla bitirir.
- Kanıt: `web_testi` (C003/C004/C019/D001/C022 → 500, C023 → 503, Ç001 → 504,
  ret kümesi, günlük kaçışı), `web_cok_surec_testi` (gerçek `--web-proxy`
  worker'ı, 500 ardından 200, tek stderr satırı), `io_izi_testi` (günlük ize
  girmez), regression `security/rota-hatasi-yalitimi`.

## Ek — K-327: yanıt yazımı sırasında istemci kopması (20 Ekim 2026)

Karar 1'deki “yanıt commit'inden sonra gelen C022” yolu sahada ikinci bir
DoS sınıfı çıkardı (GB-125): istemci ya da önündeki nginx büyük bir yanıtın
(2,3 MB site haritası) yazımı sırasında bağlantıyı RST ile keserse
`write_all` EPIPE/ECONNRESET döndürüyor, `istek_islemini_tamamla` bunu C022'ye
çeviriyor ve worker çıkıyordu; üç worker'ın üçü de aynı istekle düştü. Küçük
yanıtlarda soket tamponu yazımı yuttuğu için görünmüyordu.

- **Karar.** Socket yazım hatası tanı değil, istemcinin yanıtı almadığı
  bilgisidir. `web_yanit_taslagini_yaz` yazım hatasında `IstemciKoptu`
  döndürür; GercekIo stderr'e `yanit_kopmasi_gunluk_satiri` ile tek satır
  (`zee-web istemci yanıt sırasında koptu: …`) yazar, native metrikte
  `final_write_errors` (`yanit_yazilamadi`) artar, çerez/oturum commit'i geri
  alınır ve worker sonraki isteği kabul eder. Bağlantı izni ve socket aynı
  istekte bırakılır. PostgreSQL istek transaction'ı yazımdan önce tamamlanmış
  olduğu için geri alınmaz; bu bugünkü davranıştır.
- **Kalan C022.** Yalnız iç tutarlılık (bekleyen bağlantı yok) ve kopmadan
  sonra web oturum commit'inin geri alınamaması süreci bitirir: güvenlik
  durumu belirsizse fail-closed kalınır.
- **Reddedilen:** yazımı tekrar denemek (TCP'nin yazılmış kısmı geri alınamaz),
  oturum commit'ini kopmada korumak (istemci Set-Cookie'yi almadı) ve yeni
  metrik alanı eklemek (`zee-native-metrics-1` şeması K-167 uyumluluk
  yüzeyidir; mevcut `final_write_errors` sayacı tam bu olaydır).
- **Kanıt:** `web_cok_surec_testi` (gerçek worker, 2 MiB yanıt + yazım
  ortasında RST, sonraki istek 200; 64'lük bağlantı izni sınırını aşan 100
  ardışık kopma sonrası 200 ve RSS artışı ≤ 32 MiB; her kopma tek stderr
  satırı, C022 yok), regression `security/yanit-yazimi-kopmasi`
  (`native-web-kopma` kipi).
