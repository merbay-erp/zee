# RFC-0042 — PostgreSQL sorgusunun son tarihe bağlanması

- **Durum:** **kabul edildi, uygulandı** — 23 Kasım 2026 (K-375). Taslak
  K-371'de açıldı; core freeze (K-160A/ADR-059) altında `correctness`
  sınıfıyla seçenek 1'in en küçük hâli uygulandı.
- **Tarih:** 23 Kasım 2026
- **İş:** K-371 (taslak), K-375 (karar ve uygulama)
- **İlgili belgeler:** [spec/09](../spec/09-son-tarih-ve-iptal.md),
  [spec/11](../spec/11-uygulama-eylemleri-ve-web-adaptoru.md),
  [spec/25](../spec/25-postgresql-veri-erisimi.md),
  [ADR-060](../adr/060-postgresql-adaptor-siniri.md),
  [ADR-077](../adr/077-request-bound-postgresql-tenant-baglami.md),
  [ADR-103](../adr/103-native-commit-belirsizligi-kaniti.md),
  [K-371 denetimi](../docs/k371-dis-etki-hata-eslemesi.md)
- **Freeze sınıfı adayı:** `correctness`; grammar, tanı kodu, CLI ve
  `dil::api::v1` yüzeyi açmaz.

## Sorun

spec/09 son tarihi blok/cümle sınırlarında ve bekleme noktalarında denetler;
HTTP ve SMTP adaptörleri kalan süreyi socket bütçesine taşır. PostgreSQL
adaptörü ise yalnız bağlantı kurmada 5 sn, havuz checkout'unda 2 sn sınır
taşır; sorgu yürütmesi süresizdir. K-371 ölçümü (loopback PostgreSQL 16):

- `1 saniye içinde` bloğunda `SELECT pg_sleep(4)` → blok 4,1 sn sürer,
  `yetişmezse` 4. saniyede çalışır;
- web rotasında `pg_sleep(33)` → istek 30 sn son tarihine rağmen 504'ü
  33,0 sn'de verir; işçi o süre boyunca başka isteğe yanıt vermez.

Kilit bekleyen ya da planı bozulmuş tek sorgu bu yüzden tek işçili web
dağıtımını son tarihin ötesinde kilitler. Davranış bugün spec/25'te AÇIK
madde olarak kayıtlıdır; yanlış HTTP kodu üretmez (504 doğru), yalnız geç
üretir.

## Seçenekler

1. **Sorgu başına `statement_timeout`.** Okuma/değişiklikten önce kalan
   süre (ms) ile `SET LOCAL statement_timeout` (transaction içinde) ya da
   transaction dışı okumada tek ifadelik oturum ayarı. Sunucu `57014` ile
   iptal eder. Artı: sürücü değişmez. Eksi: ek round trip; transaction
   dışında `SET`'in havuzdaki bağlantıya sızmaması için `RESET` disiplini;
   iptal açık transaction'ı abort eder.
2. **İstemci tarafı süre + iptal isteği.** Socket okuma süresi kalan bütçe;
   dolunca `CancelRequest` gönderilir ve bağlantı geçersiz sayılır. Artı:
   sunucu ayarına dokunmaz. Eksi: iptal ile sonuç yarışı; bağlantı kaybı
   sınıfı C027 ile karışabilir.
3. **Yalnız belge.** spec/09 sınırına PostgreSQL'i ekleyip ürünlere
   `lock_timeout`/`statement_timeout`'u göç ya da rol ayarıyla vermelerini
   söylemek.

## Taslak karar ve açık sorular (K-371)

Seçenek 1, `lock_timeout` ile birlikte öne çıkıyor. Kabul öncesi yanıtlanması
gerekenler:

- İptal edilen sorgu `C025` (`sqlstate=57014`) Hata değeri mi, yoksa
  dönüşteki `son_tarihi_denetle` ile doğrudan `Ç001` mi? HTTP/SMTP modeli
  ikincisidir: süre dolmuş sonuç değere dönüşmez.
- Eylem içinde iptal savepoint'i abort eder; yerel geri alma `C021`'e mi,
  `Ç001`'e mi öncelik verir? COMMIT süresince iptal C027 belirsizlik sınıfına
  girer mi (spec/25 COMMIT kuralı otomatik tekrar YASAK)?
- Transaction dışı okuma yeniden bağlanma kuralı (spec/25) iptal sonrası
  tekrar DENEMEZ olmalı.
- IO izi (`zee-io-izi`) süre bütçesini kaydetmemeli; replay deterministik kalır.

## Kabul kanıtı

Gerçek PostgreSQL ile: `1 saniye içinde` + `pg_sleep(4)` ≤ 1,5 sn ve
`yetişmezse`; web `pg_sleep(33)` → 504 ≤ 31 sn ve sonraki istek 200; eylem
içinde iptal sonrası satır yazılmamış ve havuz bağlantısı geçersiz; transaction
dışı okumada otomatik tekrar yok; `postgresql-recovery-v1.tsv` karar matrisi
yeni satırla; spec/25 AÇIK maddesinin ZORUNLU'ya çevrilmesi.

## Karar (K-375)

Seçenek 1 en küçük hâliyle kabul edildi. Grammar, tanı kodu, CLI, IO izi
şeması ve `dil::api::v1` değişmez; yalnız iç `GirdiCikti` adaptör sözleşmesi
HTTP'deki gibi `zaman_asimi_ms: Option<i64>` taşır.

1. **Bütçe.** Yorumlayıcı her okuma/değişiklikten hemen önce etkin son
   tarihin kalanını hesaplar; dolmuşsa çağrı hiç yapılmaz (Ç001). Adaptör
   kalanı monoton bitiş anına çevirir ve ifadeden hemen önce yeniden
   hesaplar: milisaniyeye yukarı yuvarlanmış, en az 1 (0 PostgreSQL'de
   "sınırsız"dır), `i32` ile kırpılmış değer. Yukarı yuvarlama sunucu
   iptalinin son tarihten önce gelmemesini sağlar.
2. **Transaction içinde** `SET LOCAL statement_timeout = n; SET LOCAL
   lock_timeout = n; SAVEPOINT …` tek batch'tir. Değişiklik zaten
   `zee_intrinsic` savepoint'i taşıyordu; son tarihli okuma da `zee_sure`
   savepoint'i alır. Ek maliyet okumada iki, değişiklikte sıfır round trip.
3. **Transaction dışı okuma** `BEGIN; SET LOCAL …` → sorgu → `COMMIT`
   kullanır; oturum düzeyinde `SET` hiç yapılmaz, bu yüzden `RESET`
   disiplinine ve havuza sızıntıya gerek kalmaz. Hata yolunda bağlantı
   açıksa `ROLLBACK` onaylanır; kapalıysa r2d2 onu bozuk sayıp atar.
4. **Son tarih yoksa** davranış ve round trip sayısı değişmez. Aynı
   transaction'da daha önce bütçe ayarlanmışsa (son tarih bloğu eylemin
   içinde bitti) sonraki süresiz ifadeden önce bir kez `SET LOCAL … TO
   DEFAULT` gönderilir; `ROLLBACK TO SAVEPOINT` bu dönüşü geri alabileceği
   için bayrak transaction boyunca korunur.

Açık soruların yanıtları:

- **C025 mi Ç001 mi?** Ç001. Sunucunun `57014`/`55P03` iptali yalnız kendi
  bitiş anımız geçtiyse `hata_sinifi=db.deadline` taşır ve yorumlayıcı onu
  `C025` değerine değil son tarihin sahibine Ç001 olarak verir; dönüşte
  süresi dolmuş başarılı sorgu da değere dönüşmez (HTTP/SMTP modeli).
  Bitişten önce gelen `55P03` (`NOWAIT`) ya da yönetici iptali olağan `C025`
  kalır; sınıf hiçbir zaman kullanıcı değerine sızmaz çünkü yalnız son tarih
  varken üretilir.
- **Eylem içinde iptal:** sorgu savepoint'i yalnız iptal edilen ifadeyi geri
  alır; transaction abort olmaz. Ç001 eylem gövdesini terk eder ve eylem
  sınırı olağan geri almayı yapar (ROLLBACK onayı başarılı → yalnız Ç001,
  `yetişmezse`/504). Geri alma da başarısızsa var olan C021 kuralı
  uygulanır; Ç001 ile C021 arasında yeni bir öncelik kuralı gerekmez çünkü
  savepoint iptali geri almayı bozmaz. `içinde` eylemin içindeyse
  `yetişmezse` sonrası eylem aynı transaction'la sürebilir.
- **COMMIT:** son tarih bütçesiyle iptal **edilmez**. Senkron replikasyon
  beklemesinde iptal edilen COMMIT yerelde gerçekleşmiş olabilir; bu,
  ADR-103'ün kaçındığı belirsizliği bütçeyle üretirdi. Adaptör COMMIT
  öncesi zaman aşımlarını varsayılana döndürür; COMMIT hatası olduğu gibi
  C027/`db.commit_unknown`'dır. Gerçek kapıda 0,8 sn'lik ertelenmiş
  tetikleyici 300 ms bütçeden sonra COMMIT'te başarılıdır. Son tarih COMMIT
  sırasında dolarsa commit kalıcıdır; sonraki cümle sınırı Ç001 verir
  (spec/09: `içinde` transaction değildir).
- **Yeniden bağlanma:** iptal bağlantı kaybı değildir; transaction dışı okuma
  iptalden sonra tekrar **DENENMEZ** (`postgresql-recovery-v1.tsv`
  `son_tarih_iptali` sütunu). Kapalı bağlantı yüzünden yapılan tek tekrar
  kalan süreyi yeniden hesaplar; süre bittiyse tekrar yapılmaz.
- **IO izi:** bütçe iz argümanı değildir; kayıt ve replay değişmez.

## Kanıt (K-375)

Yerel PostgreSQL 16 (loopback, debug ikili), ısınmış bağlantı:

| Senaryo | Önce | Sonra |
|---|---:|---:|
| `1 saniye içinde` + `pg_sleep(4)` okuma | 4,00 sn | 1,00 sn |
| Eylem içinde INSERT + `pg_sleep(4)`, `1 saniye içinde` | 4,06 sn, 0 satır | 1,04 sn, 0 satır |
| Web `pg_sleep(33)` | 504, 33,0 sn | 504, 30,0 sn |
| İptal sonrası aynı havuzda sonraki sorgu / istek | başarılı / 200 | başarılı / 200 |

Gerçek TLS kapısı (`scripts/postgresql-tls-ci.py`, 11/11): 57014 iptali
<3 sn ve tek deneme; havuz bağlantısında `statement_timeout`/`lock_timeout`
`0/0`; eylem içi savepoint sonrası transaction sağlıklı, geri almadan sonra
kalıcı satır 0; kilit bekleyen INSERT son tarihte kesilir; `NOWAIT` 55P03
C025 kalır; ertelenmiş tetikleyicili COMMIT başarılı; yeni oturum sağlıklı.
Regression `runtime/postgresql-sorgu-son-tarihi` (kip `postgresql`),
spec/25 ZORUNLU/YASAK maddeleri, ADR-060 K-375 eki, GB-149 kapalı.

Bilinçli sınır: bütçe sunucu tarafındadır; ağ bölünmesinde yanıt gelmeyen
socket okuması istemci tarafında sınırlı değildir (seçenek 2 kapsam dışı).
Bağlantı kurma 5 sn ve havuz checkout'u 2 sn sınırları ayrıca geçerlidir.
