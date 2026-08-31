# ADR-009 — Dilin adı: **zee**

- **Durum:** kabul (kurucu, kararı 31 Ağustos 2026'da proje yürütücüsüne
  devretti; bu ADR o yetkiyle yazıldı)
- **Tarih:** 31 Ağustos 2026

## Bağlam

Master plan "çalışma adı belirlenecek" diyordu ve risk kaydı isim seçiminden
önce çakışma taraması şart koşuyordu (bölüm 35: "isim seçmeden
domain/trademark/repo taraması"). Proje klasörünün adı ilk günden beri
"zee" idi ve kurucu anlamını açıkladı: **Zeynep Eliz Erbay** — dilin kendisine
miras bırakılacağı çocuk.

## Karar

1. Dilin resmî adı **zee**'dir; yazılışı hep küçük harfle, okunuşu Türkçe
   "ze". Uzun/ayırt edici biçim: **zee dili**.
2. Dosya uzantısı **`.dil`** kalır — `merhaba.dil` okunuşu ("merhaba dil")
   dilin ruhuna uygundur ve uzantı, adın gölgesinde değil anlamın hizmetindedir.
3. CLI adı **`dil`** kalır: `dil çalıştır oyun.dil` bir Türkçe cümledir;
   `zee çalıştır` bunu bozar. LSP ikilisi `dillsp` kalır.
4. Depo/paket adı: `zee`.

## Çakışma taraması (31 Ağu 2026)

- "zee programming language" araması: iki küçük, hareketsiz hobi deneyi
  (GitHub: ZeeCompiler, zee-language) — yerleşik ürün, topluluk ya da marka
  iddiası yok. "Z#" (okunuşu Zee-sharp) ve "Z" dilleri ayrı adlardır.
- Medya markası Zee (TV) tamamen farklı sektördedir; karışma olasılığı düşük.
- Sonuç: yazılım dili alanında ayırt edicilik yeterli; "zee dili" birleşimi
  aramada temiz sonuç verir.
- **Koşul:** Türkiye'de marka tescili (TÜRKPATENT, 9/42. sınıflar) ve alan adı
  kararı, master plan bölüm 26 gereği hukuk incelemesiyle yapılacak — bu ADR
  onu bekletmez ama yerini tutmaz. Marka kullanımıyla kod lisansı ayrıdır.

## Gerekçe

İsim; kısa, telaffuzu Türkçe, klavyede kolay ve **projenin varoluş nedeninin
kendisi**. Bu dil bir çocuğun adını taşısın diye yazılıyor; adın başka bir şey
olması, projenin manifestosuna aykırı bir yapaylık olurdu. Teknik ölçütler
(çakışma, kısalık, ayırt edicilik) da aynı sonucu veriyor — kalp ile aklın
aynı kapıya çıktığı ender karar.

## Sonuçlar

- README ve belgeler "çalışma adı" ibaresini bırakır; ad resmîdir.
- Kaynak yüzeyinde hiçbir şey değişmez (uzantı ve CLI korunur) — sıfır göç.
- Gelecekte uluslararası tanıtımda "zee — a Turkish-first programming
  language" biçimi kullanılır; dil yüzeyi Türkçe kalır (bölüm 30).
