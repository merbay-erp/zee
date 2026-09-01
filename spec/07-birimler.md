# 07 — Birimler ve projeler

Normatif kaynak: RFC-0009 §2–3 (geçici kabul). Paket katmanı (§4) spec dışı —
Faz 3/5.

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

Birim çözümü de IO soyutlamasının arkasındadır (BirimYukleyici): testlerde
birimler sahte dünyadan gelir, determinizm sözü birim yüklemede de geçerlidir.
Ayrıştırma "ön tarama + tohumlu ayrıştırma" ile yapılır: kullanan dosya,
birimin işlem adlarını çağrı çözümünde görür.

## Proje bildirimi (TANIMLI — K-076)

- Proje kökünde `proje.dil` bulunur; bildirim de geçerli zee kaynağıdır.
- Tam olarak üç Metin alanı tanımlar: `proje`, `sürüm`, `giriş`.
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
