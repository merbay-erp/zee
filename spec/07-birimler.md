# 07 — Birimler

Normatif kaynak: RFC-0009 §2 (geçici kabul). Paket katmanı (§3) spec dışı —
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
