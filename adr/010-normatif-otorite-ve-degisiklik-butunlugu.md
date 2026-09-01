# ADR-010 — Normatif otorite ve değişiklik bütünlüğü

- **Durum:** kabul (1 Eylül 2026; K-118 tazelik otomasyonu revizyonu 2 Eylül)
- **Bağlam:** manifesto 3/12, master plan bölüm 23/25, spec ve RFC'lerde
  biriken davranış farkları, v1.0 sürüm kapıları

## Bağlam

zee hızlı gelişirken bir davranış aynı anda kaynak kodda, testte, spec'te,
RFC'de, karar günlüğünde ve öğretici belgelerde anlatılıyor. Bu belgeler farklı
amaçlara sahiptir. Örneğin RFC-0006 eski ilk-çağrı/özyineleme sınırını
anlatırken spec ve gerçekleme daha yeni davranışı taşıyordu; RFC-0011 gelecekteki
iptal sözünü anlatırken v0 runtime işi bitirdikten sonra süreyi ölçüyordu.

Bir dilde iki belge çatıştığında derleyici yazarının tahmin yürütmesi,
manifestonun “aynı kaynak = tek anlam” ve “compiler tahmin etmez” ilkelerini
bozar. Bu nedenle otorite sırası kadar, değişikliğin hangi dosyaları birlikte
güncellemesi gerektiği de bağlayıcı olmalıdır.

## Karar

### 1. Belge rolleri

1. **MANIFESTO + RFC-0001 anayasadır.** Değişmez tasarım sınırlarını koyar;
   tek bir sözdizimi ayrıntısının güncel cevabı değildir.
2. **Kabul edilmiş ADR mimari/güvenlik sınırını bağlar.** Bir dil cümlesinin
   kesin anlamını tek başına tarif etmez.
3. **`spec/` geçerli dilin normatif sözleşmesidir.** “Bu kaynak bugün geçerli
   mi, türü ne, ne yapar?” sorusunun tek ayrıntılı cevabı buradadır.
4. **RFC değişikliğin gerekçesi ve yetkisidir.** Kabul edilmiş bir RFC,
   bağlandığı spec bölümü ve conformance testleri aynı değişiklikte
   güncellenmeden yürürlükteki dili değiştirmez. Entegrasyon tamamlanınca
   kesin güncel davranışı yine spec anlatır; RFC nedenini ve geçmişini korur.
5. **Conformance/golden testleri yürütülebilir kanıttır.** Spec'i değiştiremez;
   çelişirse test ya da spec düzeltilmeden sürüm çıkmaz.
6. **Gerçekleme sözleşmeye uyar.** Kaynak kodun fiilen yaptığı şey tek başına
   dil semantiği olamaz; fark bir hata ya da açıkça etiketlenmiş prototip
   sınırıdır.
7. **Karar günlüğü tarihsel kayıt; README/rehberler öğretici görünüm; sürüm
   notu uyumluluk duyurusudur.** Hiçbiri normatif semantiği sessizce değiştirmez.

### 2. Çelişki kuralı

Bir çatışmada “üsttekini seçip devam et” yapılmaz. Çatışma bir **sürüm
engelleyicisidir**:

- mevcut davranış soruluyorsa spec korunur ve gerçekleme/test ona getirilir;
- davranış değişecekse RFC (güvenlik/mimaride ayrıca ADR), spec, test, hata
  kataloğu ve sürüm notu tek değişiklik kümesinde güncellenir;
- güvenli olmayan çalışan prototip, normatif özellik gibi sunulmaz; açık
  deneysel sınır ve opt-in taşır.

### 3. Bir dil değişikliğinin tamamlanma tanımı

Kullanıcıya görünen bir dil/araç davranışı ancak gerekenlerin tamamı varsa
“tamam”dır:

- karar kaydı/RFC;
- normatif spec;
- olumlu ve olumsuz conformance testi;
- tanı kodu ve katalog satırı gerekiyorsa ikisi birlikte;
- kırıcı ya da güvenlik anlamlıysa sürüm notu ve göç yolu;
- README/rehberde özellik seviyesi: kararlı, geçici veya deneysel.

Bu zincirin bir halkası eksikse özellik release-gate'te açık kalır.

### 4. Dokümantasyon tazelik kapısı

Her toplu geliştirme işinin etki incelemesi; spec, RFC/ADR indeksleri, hata
kataloğu, sürüm notu, v1 kapısı, master plan, README ve ilgili rehberleri
kapsar. Gereken belge kodla **aynı commit** içindedir; belgeyi sonraya bırakmak
tamamlanma değildir. Değişmeyen dosya, “etkilenmedi” kararının sonucudur.

Makine en az şu yapısal bayatlıkları CI'da engeller:

- kaynak tanısı ↔ `docs/hata-katalogu.md` birebirliği;
- her RFC/ADR/spec dosyasının kendi indeksinde bulunması;
- her numaralı RFC/ADR/spec'in `docs/kanit-haritasi-v1.tsv` içinde tam bir
  durum ve var olan yürütülebilir test dosyalarıyla eşleşmesi;
- depo içi Markdown bağlantılarının var olan hedefe gitmesi;
- README'deki canlı golden/test/tanı/RFC/ADR/spec sayılarının depo ağacından
  üretilmiş blokla byte-byte eşleşmesi.

K-118'den itibaren hareketli sayılar elle güncellenmez. İç araç
`cargo run --bin depo_sayilari -- --yaz` ile işaretli README bloğunu ortak
atomik yazma çekirdeği üzerinden üretir; normal test paketi `--denetle` kipini
çalıştırır. Tarihsel karar kayıtlarındaki o güne ait sayılar snapshot olarak
kalır ve bu canlı bloğun kapsamına girmez.

Anlamsal bayatlık bütünüyle otomatik bulunamaz. Bu nedenle son kullanıcıya
görünen gerçek, `docs/surumler.md` ve `docs/v1-surum-kapilari.md` içinde açık
özellik/eksik kanıt ayrımıyla elle doğrulanır. Proje kökündeki `AGENTS.md` bu
kapıyı sonraki geliştirme oturumları için kalıcı çalışma kuralı yapar.

## Sonuçlar

- Spec güncel dil için tek okunacak yer olur; RFC geçmişi silmeden gerekçeyi
  korur.
- Hızlı prototip yapmak serbesttir ama prototip etiketi ve üretim korkuluğu
  zorunludur.
- Doküman drift'i kozmetik borç değil, test/derleme hatasıyla aynı ciddiyette
  sürüm engelleyicisidir.
- Yeni RFC/ADR/spec dosyası indekslense bile kanıt haritasına eklenmeden; yeni
  test/tanı/karar sayısı README canlı bloğuna üretilmeden CI geçmez.
- Her özellikte daha çok dosya birlikte değişebilir; bunun bedeli, yıllar sonra
  dili devralan kişinin niyeti tahmin etmek zorunda kalmamasıdır.
