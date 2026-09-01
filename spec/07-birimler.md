# 07 — Birimler ve projeler

Normatif kaynak: RFC-0009 §2–4.1 (geçici kabul). Uzak registry/yayın (§4.2)
spec dışıdır — Faz 5.

## Model (TANIMLI)

- **Dosya = birim.** `hesap_araclari birimini kullan`, aynı klasördeki
  `hesap_araclari.dil` dosyasını bağlar; bulunamazsa A010.
- Birim adı tanımlayıcı kurallarına uyar (tire kullanılamaz — RFC-0009
  dosya adı notu).
- Kullanan dosya, birimin **tanımlarını** (işlem, yapı) görür; birimin
  **üst düzey cümleleri kapsüllüdür** — kullananın çıktısına karışmaz.
- Birimin testleri `dil dene` kapsamına `birim: <ad>` önekiyle katılır.

## Çakışma ve döngü (TANIMLI)

- Aynı ad iki kaynaktan gelirse **A008** — sessiz gölgeleme yoktur;
  çözüm kullanıcıya bırakılır (adlardan birini değiştir / tek kaynağa topla).
- Birimlerin döngüsel kullanımı **YASAK** (A009); ortak tanımlar üçüncü
  birime taşınır.

## Yükleme anlamı (TANIMLI)

Birim çözümü de IO soyutlamasının arkasındadır. Eski `BirimYukleyici` API'si
korunur; proje araçları isteyen dosyanın ve yüklenen kaynağın kimliğini taşıyan
kökenli yükleyiciyi kullanır. Testlerde birimler sahte dünyadan gelir,
determinizm sözü birim yüklemede de geçerlidir.
Ayrıştırma "ön tarama + tohumlu ayrıştırma" ile yapılır: kullanan dosya,
birimin işlem adlarını çağrı çözümünde görür.

## Proje bildirimi (TANIMLI — K-076)

- Proje kökünde `proje.dil` bulunur; bildirim de geçerli zee kaynağıdır.
- Üç zorunlu Metin alanı tanımlar: `proje`, `sürüm`, `giriş`. İsteğe bağlı
  `yerel_bağımlılıklar`, göreli klasör yollarından oluşan Metin listesidir.
- Sürüm `X.Y.Z`; giriş proje içindeki göreli bir `.dil` yoludur. Mutlak yol,
  `..`, ters bölü ve platform sürücü öneki yasaktır (P003/P004).
- `dil çalıştır`, `dil denetle` ve `dil dene` bir klasör aldığında giriş
  kaynağını bu bildirimden bulur; doğrudan dosya kullanımı geriye uyumludur.
- `dil biçimle <klasör>` gizli/hedef klasörleri ve sembolik bağları izlemeden
  bütün `.dil` kaynaklarını sıralı toplar. Önce hepsi bellekte doğrulanır;
  herhangi biri hatalıysa hiçbir kaynak yazılmaz (K-077).
- Çalışan programın göreli dosya IO yolları giriş dosyasının klasöründen
  çözülür; çağıran kabuğun o anki klasörü programın anlamını değiştirmez.
- Bildirimde yan etki, işlem, yapı, test, bilinmeyen veya tekrarlı alan yoktur
  (P001/P002). Böylece proje keşfi kaynak çalıştırmadan deterministiktir.

## Yerel paket ve kilit (TANIMLI — K-078)

- `X paketini kullan`, kullanan kaynağın sahibi olan projenin **doğrudan**
  `yerel_bağımlılıklar` üyesi X'in giriş kaynağını bağlar; yoksa A011.
- Paket adı bağımlı bildirimin `proje` alanıdır ve küçük harfli tek
  Türkçe/Latin tanımlayıcı olmalıdır. Grafikte adlar benzersizdir (P007).
- Paket de birim gibi yalnız işlem/yapı/test tanımlarını dışarı verir; üst
  düzey cümleleri çalışmaz. Paket içi birim, onu kullanan gerçek dosyanın
  klasöründen çözülür.
- Geçişli bağımlılık çözülür ve kilitlenir ama doğrudan bildirilmedikçe
  kaynakta kullanılamaz. Bildirim döngüsü ve aynı adlı ayrı paket hatadır.
- `dil kilitle <proje>` bütün grafiği ada göre sıralı `proje.kilit` dosyasına
  yazar: sürüm, ana projeye göre göreli yol, bağımlılık kenarı ve bütün `.dil`
  kaynaklarının SHA-256 özeti. Mutlak yol yazılmaz.
- Var olan kilit güncel grafikle byte-byte aynı değilse proje komutları P008
  verir. Kilit doğrulandıktan sonra aynı komut kaynakları yeniden okumaz.
- Kaynak toplama gizli/hedef klasörleri ve sembolik bağları izlemez; paket
  girişi ve birimleri kendi proje köklerinin dışına çıkamaz (P009).
