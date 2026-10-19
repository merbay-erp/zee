# ADR-154 — Kod lisansı: MIT

- **Durum:** kabul edildi
- **Tarih:** 19 Ekim 2026
- **İlgili:** master plan bölüm 26, K-224, P32, P39

## Bağlam

Master plan, çekirdek dilin çocukların ve eğitim kurumlarının önüne ücret
duvarı koymamasını ve compiler, stdlib, CLI ile temel araçların açık kaynak
olmasını hedefler. K-224'te lisans kararı bekletilmiş, sınırlı pilot destek
seçilmişti. Kaynak depo 19 Ekim 2026'da temiz geçmişle yeniden herkese açıldı.

## Karar

Depodaki zee kaynağı (derleyici, dil sunucusu, standart kitaplık, araçlar,
belgeler, örnekler ve resmî site kaynağı) **MIT lisansıyla** yayımlanır.
Lisans metni kök `LICENSE` dosyasındadır; Cargo paketleri `license = "MIT"`
taşır. Üçüncü taraf bileşenler kendi lisanslarını korur
(`compiler/third_party/`, yazı tipleri ve `golden/` altındaki lisans dosyaları).

## Gerekçe

- Eğitim ve ticari kullanımda en az sürtünme; okul, öğrenci ve şirket ayrı
  lisans incelemesi gerektirmeden kullanabilir.
- Tüm Rust bağımlılıkları zaten izin verici lisanslıdır (`compiler/deny.toml`
  izin listesi); MIT ile bileşik dağıtımda çatışma yoktur.
- Copyleft, eğitim kurumlarının ve ürün ekiplerinin benimsemesini zorlaştırır;
  Apache-2.0'ın patent maddesi değerli olsa da tek kişilik projede MIT'in
  sadeliği seçildi.

## Kapsam dışı

- **Marka:** "zee" adı ve logosu kod lisansından ayrıdır (master plan §26);
  marka politikası ayrı yazılacaktır.
- Destek, LTS ve yayın takvimi bu kararla söz verilmiş olmaz
  ([destek politikası](../docs/urun-destek-politikasi.md)).
- İmzalı artefakt yayını ve yayıncı anahtarı ayrı V1 kapılarıdır.

## Sonuçlar

P39 (lisans/yayın kararı) lisans yönünden kapanır; marka politikası ve yayın
kanalı açık kalır. P32 (açık depo) depo açılışıyla kapanır.
