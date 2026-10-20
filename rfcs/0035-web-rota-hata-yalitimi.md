# RFC-0035 — Web rotasında çalışma hatası yalıtımı ve operatör günlüğü

- **Durum:** geçici kabul — 20 Ekim 2026 insan onayıyla (500 gövdesinde olay
  kimliği yok, C018/C025 500, günlük `anahtar=değer`) K-324'da gerçeklendi;
  normatif metin spec/11 §5, karar ADR-157.
- **Tarih:** 20 Ekim 2026
- **İş:** K-324 (kaynak: K-323 açık bırakılan iki konu; GB-122, GB-123)
- **İlgili günlük kayıtları:** kararlar/gunluk.md K-323, K-324
- **İlgili belgeler:** spec/06 §hata sınıfları, spec/11 §5–6, spec/24,
  ADR-033, ADR-034, ADR-060 (F029 istek-yerel hata sınırı), ADR-007
- **Freeze sınıfı (önerilen):** `security` — grammar, kalıp, tanı kimliği ve
  `dil::api::v1` facade'ı değişmez.

## Özet

V1 web modeli süreç başına tek worker'dır (ADR-034). Rota gövdesinde
C013/C021/C027 (eylem/dosya) ve K-323 sonrası C023/C024 (kaynak bütçesi)
istek düzeyinde 503'e çevrilir; geri kalan bütün çalışma hataları (C003 sıfıra
bölme, C004 sayıya çevirme, C010 sözlükte anahtar yok, C019 derinlik, D001 …)
`calistir_program_kodla` içinde `return Err` ile **bütün süreci** bitirir.
Kullanıcı girdisine bağlı tek bir istek worker'ı düşürebilir. Ayrıca istek
düzeyine çevrilen tanılar operatöre hiç görünmez.

Bu RFC iki bağlı karar önerir:

1. **K1 — Rota çalışma hatası istek-yereldir.** İzinli küme yerine ret kümesi:
   rota yürütmesinden dönen her tanı, taslak geri alındıktan sonra istemciye
   ayrıntı sızdırmayan sabit gövdeli yanıta çevrilir ve worker sonraki isteği
   kabul eder. Kaynak/geçici sınıf 503 (bugünkü beş kod), diğerleri 500.
   Yalnız iç tutarlılık (C000) süreci bitirmeye devam eder.
2. **K2 — İstek düzeyine çevrilen her tanı operatör günlüğüne yazılır.**
   İç (`#[doc(hidden)]`) `GirdiCikti` trait'ine varsayılanı boş bir
   `istek_tanisini_gunlukle` yöntemi eklenir; gerçek CLI adaptörü tek satırlık,
   denetim karakterleri kaçışlanmış kaydı stderr'e yazar.

K2, K1'in önkoşuludur: spec/06 “beklenmeyen çalışma hatasında sessiz devam
YASAK” der. İstemciye 500 verip devam etmek ancak tanı operatöre ulaşıyorsa
“sessiz” değildir. Bugünkü 503 yolları da bu bakımdan eksiktir (GB-123).

## Motivasyon

- **Güvenlik reddi bile süreci düşürüyor.** Yönlendirme hedefini sorgudan
  alan rota, CR/LF taşıyan tek istekle C022 üretir; reddetme doğrudur ama
  `return Err` worker'ı kapatır. Saldırganın elinde “güvenlik kontrolünü
  tetikle, sunucuyu kapat” düğmesi vardır.
- **Kullanıcı girdisiyle süreç düşürme (GB-122, orta).** Rota `istek`
  alanını `sayısı` ile çevirip bölen bir ürün, `?n=0` ya da `?n=abc` alan tek
  istekle sürecini kaybeder. Süreç yöneticisi yeniden başlatsa bile yeniden
  başlatma süresince bütün kullanıcılar bağlantı hatası alır; saldırgan her
  yeniden başlatmadan sonra tek istekle döngüyü sürdürür. Kimlik doğrulaması
  gerekmeyen herkese açık rotada bu kimliksiz DoS sınıfıdır (GB-001'in
  sınıfıyla aynı).
- **Parça parça izinli küme büyüyor.** Aynı sorun F029'da C013 için
  (ADR-060), K-163'te C021/C027 için, K-323'te C023/C024 için ayrı ayrı
  çözüldü. Her yeni kod sahada süreci düşürdükten sonra listeye ekleniyor.
  ADR-060 bu sınırı açıkça “request-local failure sınırı” diye adlandırmış ama
  genel kural yazılmamıştır.
- **Operatör körlüğü (GB-123, düşük).** K-323 sonrası C024 aşan sayfa
  istemciye 503 döner; süreç yaşar ve stderr'de hiçbir iz kalmaz. Operatör
  sorunu ancak istemci şikâyetinden öğrenir. Eskiden süreç ölümü en azından
  tanıyı stderr'e basıyordu; K-323 gözlenebilirliği farkında olmadan azalttı.

## Yalıtımın neden güvenli olduğu (doğrulanan durum)

İstek-yerel bitirmenin doğru olması için rota hatası süreç düzeyinde bozuk
durum bırakmamalıdır. Kod incelemesi (K-323 tabanı) şunu gösterir:

| Süreç düzeyi durum | Hata yolunda | Kanıt |
|---|---|---|
| Global ortam | rotalar `&ortam` alır; her istek taze `rota_ortami` kurar, T046 rotanın uygulama durumunu yazmasını derlemede reddeder | `web_istek.rs::web_istegini_calistir` |
| Eylem transaction'ı | çalışma hatasında eylem geri alınır (spec/11 §6) | `yorumlayici.rs` eylem çağrısı, `eylem_calisma_hatasinda_tum_dosyalari_geri_alir` |
| Çerez/oturum/yanıt taslağı | `istek_islemini_geri_al` her `Err` dalında çağrılır | `rota_hatasi_tamponlu_yaniti_cerezi_ve_oturum_mutasyonunu_geri_alir` |
| Adım/çıktı/heap bütçesi | her istek başında `calistirma_butcesini_yenile` | K-323, `istek-butcesi-yalitimi` |
| Son tarih yığını | `SonTarihNobetcisi` Drop ile; scheduler eski yığını sonuçtan bağımsız geri koyar | Ç001→504 yolu zaten süreç yaşatıyor |
| Çağrı derinliği | C019, doğal yığın taşmadan (K-178) üretilir; yığın açılmış durumda döner | `kaynak_sinirlari_testi` |
| PostgreSQL bağlantısı | hata sonrası client yeniden kullanılmaz (ADR-060) | `postgresql-recovery-v1.tsv` |

Yani Ç001 (504) ve beş 503 kodu için bugün zaten güvenilen temizlik yolu,
geri kalan çalışma hatalarında da aynı yoldur; tanı kodu temizliği
değiştirmez. Kalan risk iç tutarlılık ihlalidir (C000): yorumlayıcının kendi
değişmezi bozulmuşsa süreç durumu güvenilir değildir.

Prototip kanıtı (K-323 tabanında, commitlenmedi): `Err` dalı “C000
değilse 500” yapıldığında `cargo test --locked --no-fail-fast` içinde yalnız
bu davranışı bilinçli sabitleyen iki test kırıldı:
`rota_hatasi_tamponlu_yaniti_cerezi_ve_oturum_mutasyonunu_geri_alir` (C003)
ve `yonlendirme_baslik_enjeksiyonu_c022_ile_reddedilir` (C022). Golden,
regression, IO izi, kaynak sınırı ve eşzamanlılık testleri değişmedi.

## Tasarım

### K1 — İstek-yerel hata sınırı (spec/11 §5'e eklenecek normatif metin)

> Rota seçildikten sonra gövde yürütmesi bir tanıyla biterse istek
> yanıtı/çerez/oturum taslağı geri alınmak ZORUNDADIR ve worker sonraki
> bağımsız isteği kabul eder. İstemciye tanı kodu, mesajı, satırı ya da değer
> içeriği SIZDIRILAMAZ; yanıt aşağıdaki sabit gövdelerden biridir:
>
> | Tanı | Durum | Gövde |
> |---|---|---|
> | Ç001 son tarih | 504 | bugünkü metin |
> | C013, C021, C027 | 503 | bugünkü metinler (spec/11 §6) |
> | C023, C024 | 503 | bugünkü metin (spec/24) |
> | diğer bütün C/D tanıları | 500 | “istek işlenemedi; sunucu hatası” |
> | C000 iç tutarlılık | — | süreç tanıyla sonlanır (bugünkü davranış) |
>
> Kurtarma rota gövdesini, eylemi ya da yazımı otomatik TEKRARLAYAMAZ.
> `programı N ile bitir` rota içinde bugünkü gibi sunucuyu kapatır; geçersiz N
> (C020) 500'dür.

Notlar:

- **Ret kümesi, izinli küme değil.** Yeni bir çalışma tanısı varsayılan olarak
  istek-yerel olur; süreci bitirmesi gereken bir kod bilinçli olarak C000
  kümesine eklenir. Böylece “sahada süreç düştü → listeye ekle” döngüsü biter.
- **Neden 500, 503 değil?** 503 “geçici, sonra yeniden dene” anlamı taşır;
  C003 gibi deterministik program hatası aynı girdiyle tekrar edilirse yine
  oluşur. 500 istemciye ve reverse proxy'ye doğru sinyaldir. Bugünkü beş 503
  kodu değişmez (geriye uyum ve “otomatik tekrar yok” metinleri korunur).
- **C022 ikiye ayrılır.** Rota gövdesi içinde adaptörün güvensiz çerez ya
  da yönlendirme değerini reddetmesi (C022) diğer tanılar gibi 500'dür:
  bugün `?hedef=%2Fiyi%0D%0AX-Sahte%3A+evet` gibi tek bir sorgu, başlık
  enjeksiyonu doğru biçimde reddedildiği hâlde **sunucuyu sonlandırır**
  (`yonlendirme_baslik_enjeksiyonu_c022_ile_reddedilir` bunu sabitliyor).
  Gövde başarıyla bittikten sonra `istek_islemini_tamamla` üretilen C022 ise
  yanıt socket'e yazılırken oluşur; istemciye ikinci yanıt yazılamaz. Bu
  ikinci yol bugünkü gibi kalır ve ayrı değerlendirmeye bırakılır.
- **CLI değişmez.** Sunucu kurulmamış programda ve sunucu kurulmadan önceki
  üst düzey kodda tanı bugünkü gibi yayılır; `dil dene` hermetik testleri de
  değişmez.
- **Sabit HTTP durum yüzeyi.** 500 yeni bir *dil* durumu değil, adaptör
  durumudur; ADR-063'teki kullanıcı kontrollü sabit durum kümesine girmez.

### K2 — Operatör günlüğü

İç trait'e (modül `yorumlayici` `lib.rs`'te `#[doc(hidden)]`; `dil::api::v1`
facade'ı `GirdiCikti`'yı yeniden dışa vermez) varsayılanı boş bir yöntem:

```rust
/// Rota tanısı istek düzeyinde yanıta çevrildiğinde operatör içindir.
/// İstemciye ve programa görünmez; varsayılan adaptör hiçbir şey yapmaz.
fn istek_tanisini_gunlukle(&mut self, _durum: u16, _tani: &Tani) {}
```

- Yorumlayıcı `Err` dalında `istek_islemini_geri_al`dan sonra, durum
  yanıtından önce çağırır (504/503/500 hepsi).
- `GercekIo` stderr'e tek satır yazar:
  `zee-web istek-hatası durum=500 kod=C003 satır=12 yöntem=GET yol=/bol mesaj="…"`.
  `yol` sorgu dizgesi olmadan yazılır (sorgu kişisel veri taşıyabilir);
  `mesaj` ve `yol` içindeki CR/LF, denetim karakterleri ve `"` kaçışlanır
  (günlük enjeksiyonu). Satır 4 KiB ile kırpılır.
- `PaylasilanIo`, `PolitikaliIo`, `IzKaydedenIo` iç adaptöre iletir.
  `IzKaydedenIo` olayı **kaydetmez**: günlük program tarafından gözlenemez,
  replay determinizmini etkilememeli. `IzYenidenOynatici` ve `PlaygroundIo`
  varsayılanı kullanır.
- `ToplayanIo` çağrıları bir alanda biriktirir; testler buradan doğrular.
- ADR-007 (araçlar veri toplamaz) ile çelişmez: kayıt yalnız yerel stderr'e
  gider, ağ ya da dosya yazımı yoktur; dağıtım hedefi operatörün süreç
  yöneticisidir (systemd journal vb.).

Neden yorumlayıcıda doğrudan `eprintln!` değil: çekirdek IO adaptör sınırını
deler (ADR-046 katman yönü), wasm hedefinde stderr yoktur ve hermetik
testler tanıyı doğrulayamaz.

Neden `durum_yaniti_gonder`'e parametre değil: imza değişikliği bütün
adaptörleri kırar ve istemci yanıtı ile operatör kaydını aynı kanala bağlar.

### Uyumluluk yüzeyi etkisi

- `dil::api::v1` ve `public_api_testi` allowlist'i: değişmez (trait
  facade dışında).
- `dil-yuzeyi-v1.tsv`: tanı kodu, kalıp, CLI komutu eklenmez; değişmez.
- Gözlenebilir davranış değişikliği: web rotasında C000 dışı tanılar artık
  süreci bitirmez. Bu, `docs/surumler.md` göç notu ister: süreç ölümünü
  sağlık sinyali olarak kullanan dağıtımlar artık stderr kaydını izlemelidir.

## Dört soru süzgeci

1. Türkçe doğal mı? — Kaynak yüzeyi değişmez; gövde ve günlük metinleri
   Türkçedir.
2. Deterministik mi? — Durum eşlemesi tanı kodunun saf işlevidir; günlük
   program tarafından gözlenmez ve izde yer almaz.
3. Öğrenilebilir mi? — “Rotadaki hata yalnız o isteği bozar” tek cümlelik
   kuraldır; bugünkü beş istisnalı liste yerine geçer.
4. Profesyonel ölçekte savunulabilir mi? — Yerleşik web çatılarının (istek
   başına 500, süreç yaşar, ayrıntı yalnız sunucu günlüğünde) davranışıdır.

## Alternatifler

- **A — Bugünkü izinli kümeyi korumak, gerektikçe kod eklemek.** Reddedildi:
  her ekleme saha arızası sonrası geliyor; kullanıcı girdisine bağlı
  deterministik hatalar (C003/C004/C010) kimliksiz DoS bırakıyor.
- **B — Yalnız “girdiye bağlı” kodları (C003/C004/C010/C019) yalıtmak.**
  Reddedildi: hangi kodun girdiye bağlı olduğu programa göre değişir; ayrım
  belgelenemez ve test edilemez.
- **C — Süreç düşsün, süreç yöneticisi yeniden başlatsın.** Reddedildi:
  yeniden başlatma süresince bütün istemciler etkilenir, saldırgan döngüyü
  tek istekle sürdürür; ADR-034 çok süreçli modeli kapasite içindir, hata
  yalıtımı için değil.
- **D — Her şeyi 503 yapmak.** Reddedildi: deterministik hata için yanlış
  “yeniden dene” sinyali; proxy'ler 503'ü yeniden deneyebilir.
- **E — Panik yakalama (`catch_unwind`) ile Rust panik'lerini de yalıtmak.**
  Kapsam dışı: panik yorumlayıcı hatasıdır, süreç bitmesi doğru davranıştır.
- **F — Günlüğü yeni bir public API/geri çağırma ile açmak.** Gereksiz:
  gömme kullanıcısı bugün `GirdiCikti`'yı facade üzerinden zaten alamıyor;
  facade'a açmak ayrı SemVer kararı olur.

## Uygulama planı (onaydan sonra; tek toplu iş)

1. `compiler/src/yorumlayici.rs`: `Err` dalı ret kümesi + 500; trait yöntemi;
   `PaylasilanIo` iletimi. `web_istek.rs`: durum/gövde eşlemesi tek işlevde
   (`istek_hatasi_yaniti(kod) -> Option<(u16, &str)>`; `None` = C000).
2. `toplayan_io.rs`, `yetkinlik.rs`, `io_izi.rs`: iletim/kayıt; `main.rs`
   `GercekIo` stderr satırı ve kaçış işlevi.
3. Testler (olumlu/olumsuz):
   - `rota_hatasi_tamponlu_…` yeniden yazılır: aynı koşuda 500, sabit gövde,
     çerez/oturum geri alındı, sonraki istek 401.
   - C003/C004/C019/D001 rota → 500 + worker yaşar; C024 → 503 + günlük;
     C000 → süreç biter (iç hata enjeksiyonuyla).
   - Günlük kaçışı: CR/LF içeren yol ve mesaj tek satır kalır; sorgu yazılmaz.
   - `IzKaydedenIo` günlük olayını izlemez (iz baytı değişmez).
   - Gerçek TCP (`--deneysel-web` ve `--web-proxy`): 500 ardından 200,
     stderr'de tek satır.
4. Regression: `regression/security/rota-hatasi-yalitimi.dil` + `v2.tsv`.
5. Belgeler: spec/11 §5 (yukarıdaki metin), spec/24 son paragraf (“diğer
   çalışma hataları spec/11'deki yayılma kuralını korur” cümlesi yeni kurala
   bağlanır), spec/06 §3'e web notu, ADR-157 (istek-yerel hata sınırı ve
   operatör günlüğü), hata kataloğu C003/C019 öneri sütunları değişmez, yalnız
   C024/C013 satırlarındaki web notu genel kurala bağlanır; `surumler.md`
   göç notu; `spec-madde-kaniti-v1.tsv` + `spec_drift --rapor-yaz`;
   GB-122/GB-123 kapanış; backlog K-324.
6. Beyanlar: compiler-degisiklik (`semantic-change`, kanıt spec/11 +
   ADR-157), core-freeze (`security`), katman fixture etkisi (yeni kenar yok
   beklenir), `faz_test_matrisi`, `tani_kalitesi` (tanı metni değişmez).

## İnsan kararları (20 Ekim 2026)

1. 500 gövdesi “istek işlenemedi; sunucu hatası”dır; olay kimliği eklenmedi.
2. C018/C025 dış servis/DB hataları 500'dür; 502/503 ayrımı yapılmadı.
3. Günlük biçimi bağımlılıksız `anahtar=değer` satırıdır; JSON değil.

Uygulama planındaki sapmalar: 504 (Ç001) yolu da günlüğe düşer; C000'a ek
olarak iç `Ç` nöbetçileri (Ç001 hariç, ayrı 504 dalı) ret kümesindedir.
Gerçek TCP kanıtı `--web-proxy` worker'ıyla alındı; `--deneysel-web` aynı
`GercekIo` adaptörünü kullanır. `yorumlayici/yetkinlik.rs` satır bütçesi
iletim yöntemi için 250→260'a çıkarıldı; biçimleyici ayrı
`yorumlayici/istek_gunlugu.rs` modülündedir.
