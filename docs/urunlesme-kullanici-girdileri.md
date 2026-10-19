# Ürünleşme: kullanıcı girdileri ve açık kapılar

15 Ekim 2026 başlayan çalışma. [K-298 dar yerel kabulü](urunlesme-k298-kabul-20261016.md),
[tarihsel S4 kabulü](urunlesme-s4-kabul-20261015.md),
[tarihsel S3 teknik kabulü](urunlesme-s3-kabul-20261015.md) ve
[39 maddelik plan](urunlesme-plan-v1.tsv) için teslim listesi. Bu belge yeni
ürün, yayın veya destek kararı vermez; eksik kanıtı tamamlanmış göstermez.

Beş tam anonim proje 19 Ekim 2026'da hazırlandı ve ölçüldü (P08 kanıtlı).
P29'un macOS kısmı 19 Ekim 2026'da proje sahibinin fiziksel Mac'inde kabul edildi ([kayıt](urunlesme-p29-macos-fiziksel-20261019.md)); **fiziksel Windows cihazı** bulunmadığı için Windows kısmı açıktır. Sınırlı pilot desteği seçilmiştir; lisans 19 Ekim 2026'da MIT olarak belirlendi.
Bu iki mevcut karar teknik çalışmaya devam etmek için yeniden sorulmaz.

## Öncelik ve teslim özeti

| Öncelik | Kapı | Kullanıcıdan gereken | Hazır olduğunda yapılacak |
|---|---|---|---|
| 1 | P08 gerçek ürün korpusu | Beş anonim tam site ve çekirdek kökünün erişilebilir yerel paket yolları; exact ürün kökeni ve compiler pini; çalıştırma/beklenti bilgisi | Paket içeriği ve kökeni incelenir; her ürün için `denetle`/`dene` N/N, süre ve RSS kanıtı ayrı kaydedilir. |
| Paralel | P27 gerçek insan çalışması | 10 öğrenci/çocuk ve 5 profesyonelin protokole uygun anonim formları | Önceden ilan edilen eşiklerle toplam ve alt grup sayımları yapılır; ilgili RFC ve V1 kapısı gerçek sonuçla güncellenir. |
| Paralel | P29 fiziksel platform | Temiz fiziksel Windows ve temiz/minimum OS macOS cihazı erişimi veya bu cihazlarda yürütülmüş kabul kayıtları | Exact ikili/kaynak ve ortam bilgisiyle kurulum, program, LSP ve kaldırma doğrulanır. |
| Ürün seçilince | P09–P16, P20–P25 ürün kabulü | Seçilen pilot ürünün anonim kaynağı; ilgili sağlayıcı/staging kapsamı ve beklenen davranış | İlgili mevcut teknik dilimin gerçek üründeki olumlu/olumsuz kabulü ve geri dönüş sınırı doğrulanır. |
| Yayın aşamasında | P06/P24/P30/P39 | Yayıncı/imza ve dağıtım kanalı kararı (lisans: MIT); ilgili yayın işlemi yetkisi | [Sürüm runbook'u](surum-runbook.md) sırasıyla bütün kapılar tamamlandıktan sonra yayın hazırlanır. |

P08 en çok işi açan veri teslimidir: ürün uyumluluğu, kuyruk/SMTP/PDF,
login/form, gerçek proje LSP ölçümü, test kapsamı ve starter kabulünün ürün
kanıtı bu hatta bağlıdır. Diğer paralel hatlar P08'i beklemek zorunda değildir.
P30 bütün gerekli ürün, insan, platform, güvenlik ve takvim kapılarını bekler.

## P08: beş site ve çekirdek teslim kartı

**19 Ekim 2026: teslim edildi ve ölçüldü — [P08 kabulü](urunlesme-p08-kabul-20261019.md).**
Aşağıdaki tablo bir teslim formudur; yer tutucular sonuç veya provenance
sayılmaz. Her ürün için gerçek değerlerle doldurulur. Kaynak ve veriler
anonimleştirilmiş olmalı; müşteri/alan/sunucu kimliği veya canlı sır içermez.
Özgün ürüne ilişkin exact köken bağı özel ürün kaydında korunur.

| Anonim ürün kimliği | Tam paket/kök yolu | Exact ürün commit'i | Compiler pini | Giriş/proje kökü | Beklenen `denetle` ve `dene` sonucu |
|---|---|---|---|---|---|
| site-a | `<mutlak-yerel-yol>` | `<40-haneli-SHA>` | `<exact-SHA/kilit>` | `<paket-içi-kök>` | `<exit/tanı; N/N>` |
| site-b | `<mutlak-yerel-yol>` | `<40-haneli-SHA>` | `<exact-SHA/kilit>` | `<paket-içi-kök>` | `<exit/tanı; N/N>` |
| site-c | `<mutlak-yerel-yol>` | `<40-haneli-SHA>` | `<exact-SHA/kilit>` | `<paket-içi-kök>` | `<exit/tanı; N/N>` |
| site-d | `<mutlak-yerel-yol>` | `<40-haneli-SHA>` | `<exact-SHA/kilit>` | `<paket-içi-kök>` | `<exit/tanı; N/N>` |
| site-e | `<mutlak-yerel-yol>` | `<40-haneli-SHA>` | `<exact-SHA/kilit>` | `<paket-içi-kök>` | `<exit/tanı; N/N>` |
| cekirdek | `<mutlak-yerel-yol>` | `<40-haneli-SHA>` | `<exact-SHA/kilit>` | `<paket-içi-kök>` | `<exit/tanı; N/N>` |

Her paket için ek çalışma notu:

| Alan | Doldurulacak değer |
|---|---|
| Kaynak kökeni | `<anonim paket ile exact ürün commit'inin ilişkisi; özel köken kaydının konumu>` |
| Kurulum/üretim adımı | `<gerekli gerçek komut veya yok; üretilen katalog/medya dahil>` |
| Test komutları | `<çalışma diziniyle exact denetle/dene komutu>` |
| Gerekli servis/veri | `<PostgreSQL sürümü, anonim şema/fixture veya yok>` |
| Ortam sözleşmesi | `<değişken adları ve test değerlerinin nasıl sağlandığı; canlı sır yok>` |
| Beklenen sonuç | `<başarı/ret, tanı veya çıktı, gerçek N/N sayısı; bilinen sınırlar>` |
| Ürün davranışı | `<bu paketin hangi gerçek kullanımını doğrulayacağız>` |

Mevcut küçük örnekler veya eski test toplamları beş tam ürün kabulüne
aktarılmaz. Süre/RSS burada tahminle doldurulmaz; gerçek koşuda ölçülür.
Tam dış ürün kaynağı doğrudan `dogfood/` altına kopyalanmaz.
[Dogfood rehberi](dogfood-korpusu.md) tam dış kaynakla depo içi sürtünme
kanıtını ayırır; [ürün kaydı](dogfood-projeleri-v1.tsv) exact kökeni,
[korpus manifesti](../dogfood/korpus-v1.tsv) tekil kanıt vakalarını izler.

## P27: insan çalışması

[Oturum kiti](usability-kiti.md) kişi başı yaklaşık 33–38 dakika için hazırdır.
Hedef örneklem 8–14 yaş **10 öğrenci/çocuk + 5 profesyoneldir**.

| Teslim | Kullanılacak dosya | Beklenen içerik |
|---|---|---|
| 15 anonim katılımcı formu | [Katılımcı şablonu](usability-sonuclari/katilimci-sablonu.md) | `C01..C10`, `P01..P05`; tarih, rol/yaş grubu, kart sırası, protokol sürümü, ilk/öğretim sonrası yanıtlar ve birebir gerekçe |
| Oturum özeti | [Özet şablonu](usability-sonuclari/ozet-sablonu.md) | Tamamlanan/eksik form, toplam ve alt grup sayımları; önceden ilan edilen eşik sonucu |
| Gizlilik ve kayıt düzeni | [Sonuç arşivi kuralları](usability-sonuclari/README.md) | Çocuk için gerekli onam/izin uygulanması; kişisel veri içermeyen yazılı kayıt |

Örnek dosya adı `YYYY-MM-DD-C01.md` olur. İsim, e-posta, okul, yüz veya ses/video
kaydı depoya konmaz. Sonuçlar görülünce protokol/eşik değiştirilmez; eksik veri
başarı lehine varsayılmaz. Gerçek katılımcı olmadan bu kapı kapanmaz.

## P29: fiziksel cihaz kaydı

macOS kabul edildi; Windows cihazı şu anda yoktur. Cihaz
hazır olunca aşağıdaki bilgiler yeterli başlangıç girdisidir:

| Alan | Doldurulacak değer |
|---|---|
| Platform ve mimari | `<Windows/macOS; x86_64/arm64>` |
| Exact işletim sistemi | `<sürüm ve build; sınanan minimum sürüm iddiası>` |
| Temiz ortam durumu | `<kurulumda mevcut araçlar ve PATH; geliştirici araçları önkoşulu>` |
| Yürütme | `<sahipli test erişimi veya kullanıcı çalıştıracak>` |
| Test edilecek artefakt | `<exact kaynak SHA; CLI/LSP hashleri ve provenance konumu>` |
| Kabul kayıtları | `<kur/program/LSP/kaldır komutları, exit/çıktı ve ortam raporu>` |

Kabul sırası [sürüm runbook'unda](surum-runbook.md), araç zinciri ve açık
minimum OS sınırları [derleme rehberinde](derleme-ve-bagimliliklar.md) bulunur.
[Platform defteri](derleme-platform-kaniti-v1.tsv) tarihsel CI kanıtıdır;
mevcut container/VM veya geliştirici Mac sonucu fiziksel temiz/minimum OS
kabulünün yerine yazılmaz.

## P32 ve ürün staging girdileri

P32 için [hazır özel depo GitHub destek taslağı](incelemeler/k215/github-destek-talebi.md)
kullanıcı tarafından gönderilir. Gerekli dönüş `talep kimliği + tarih +
GitHub'ın yanıtı + temizlenen/kalan kopyaların kapsamı`dır. Gönderildi bilgisi
tek başına temizlik teyidi değildir; depoyu public açma yetkisi de değildir.

Gerçek ürün kabulü için bir ürün seçildiğinde şu kısa kart doldurulur:

| Alan | Doldurulacak değer |
|---|---|
| Ürün ve exact kaynak | `<P08 anonim kimliği/paket yolu ve commit>` |
| Hedef dilim | `<ör. SMTP, PDF katalog, login/form, göç, TR–EN veya starter>` |
| Sağlayıcı/ortam | `<staging kimliği ve ilgili servis; varsa sentetik hesap/alıcı>` |
| İzinli işlem sınırı | `<hangi test etkileri kabul kapsamında>` |
| Beklenen davranış | `<olumlu/olumsuz sonuç ve ürünün açık sınırı>` |
| Geri dönüş kabulü | `<ürün sahibinin yedek/geri dönüş ve göç kabulü>` |

Canlı parola, API anahtarı veya müşteri verisi bu karta yazılmaz. Mevcut
[sınırlı pilot destek politikası](urun-destek-politikasi.md) ve
[credential başlatma rehberi](credential-baslat.md) korunur. Ürün sahibi canlı
yükseltme/veri göçü/geri dönüş kabulünün sahibidir. Staging erişimi genel
üretim değişikliği veya dışarı mesaj gönderme yetkisi sayılmaz.

## Kullanıcı girdisiyle kapanmayan açık sınırlar

- **P28 takvim:** [syntax freeze kaydı](v1-syntax-freeze-v1.tsv)
  14 Ekim–11 Kasım 2026 penceresini taşır. Günün gelmesi ve gözlem koşulunun
  gerçekleşmesi gerekir; erken tamamlandı yazılamaz.
- **C1/K-288 ve P33/K-293:** araç güvenlik incelemesi retleri nedeniyle
  [açık incelemelerde](urunlesme-acik-incelemeler.md) kalır. Kullanıcıdan ek
  dosya veya tekrar izin istemekle çözülmüş sayılmaz; farklı araç/ajana
  yönlendirilerek ret aşılmaz.
- **P19 teknik kapsam:** BMP dışı sorgu/tanı aralıkları [K-297](lsp-utf16-kabul-20261015.md)
  ile kapandı; [K-298](urunlesme-k298-kabul-20261016.md) a57290f kaynağında native UTF-16 ve
  dar gerçek IDE işlev kabulünü yeniledi. Dört stdio p95'i 200 ms altında,
  en ağır kohort189,885667 ms ölçüldü. Bu shared gözlemsel hedeftir; UI gecikmesi
  veya genel SLA değildir. Aynı kaynağın ikinci RSS ve RC kabulü geçti; K-298 dar kapsamı kapalıdır. Artımlı HIR/tür,
  UI/gerçek ürün ve platform teknik kapsamı açık kalır; yalnız gerçek ürün
  ölçümü P08 girdisine bağlıdır. Bunların tümü kullanıcı girdisi eksikliği sayılmaz.
- **Destek iddiaları:** genel FFI/hostile PDF sandbox, sağlayıcılar için genel
  exactly-once, DB+FS atomikliği, bütün HTTP işlemleri için rollback, genel
  üretim SLA'sı ve zero downtime mevcut yerel kabulün kapsamına girmez.
  Bunlar teslim kartı doldurularak veya test sonucu yeniden adlandırılarak
  vaat edilmez.
- **Ertelenen maddeler:** P17 depolama/IAM, P26 HTML içe alma temizleyicisi,
  P35 yeni ergonomi grameri ve P38 backend, [onaylı planın](urunlesme-plani.md)
  erteleme ve gerçek ihtiyaç koşullarını korur.

Lisans, yayıncı ve public dağıtım kararları yayın aşamasına aittir. Hazır
teknik çalışmaları sürdürmek için kullanıcıdan şimdi yeni bir LTS süresi,
sabit yayın takvimi veya genel SLA seçmesi istenmez.
