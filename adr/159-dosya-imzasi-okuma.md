# ADR-159 — Yüklenen dosyanın ilk baytlarını sınırlı okuma (içerik imzası)

- **Durum:** kabul
- **Tarih:** 12 Kasım 2026
- **İlgili kayıt:** K-333, RFC-0036, ADR-059 (core freeze), ADR-061 (binary
  yükleme sahipliği), ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu),
  spec/26, spec/21, GB-130

## Bağlam

zee-cekirdek 1.18.0 (exact ürün commit'i
`e8beb39e4fe9486331a9605307ff56468df473bc`) oturumsuz ziyaretçiden CV alan
`POST /basvuru-yukle` rotasını ekledi. Rota uzantı, boyut ve ad denetler;
dosyanın gerçekten bildirilen türde olduğunu (sihirli bayt) **kabul anında**
denetleyemez, çünkü spec/26 yüklenen dosya için yalnız taşı/sil/listele/sha256
verir ve `dosyasını okumayı dene` UTF-8 metin bekler. Ürün bugün imzayı yalnız
indirmede, ayrı bir Node varlık sunucusunda denetliyor; imzası tutmayan dosya
diske ve veritabanına girip yöneticinin başvuru listesine düşüyor.

Core freeze (ADR-059) yeni kalıbı varsayılan olarak reddeder. İhtiyaç gerçek
ürün dosyasında (`secimli/basvuru-formu/kaynak/rota_basvuru.dil`) duruyor ve
mevcut yüzeyle karşılanamıyor: SHA-256 özeti imza vermez, metin okuma ikili
dosyada başarısızdır, proje dışı süreç çağırma yetkinliği yoktur.

## Karar

1. Tek salt-okur ifade eklenir:

   ```zee
   imza yol dosyasının ilk 8 baytını onaltılık okumayı dene olsun
   ```

   `yol` Metin, bayt sayısı TamSayı (değişken ya da sabit); sonuç `Metin
   sonucu`: dosyanın ilk N baytı **küçük harf onaltılık**, dosya daha kısaysa
   olan kadarı (boş dosya `""`).
2. N **1 ile 64** arasında olmak zorundadır; dışındaki değer IO'ya
   ulaşmadan C013 başarısız Sonuç verir. 64 sabit üst sınırdır (yaygın
   imzalar ≤ 16 bayttır; üst sınır içeriği dilde genel ikili okuma aracına
   dönüştürmez).
3. `dosya-okuma` yetkinliği ister (yoksa derlemede T054); yol kuralları
   `sha256 özetini almayı dene` ile aynıdır: proje sınırında mutlak/üst dizine
   çıkan yol ve kök dışına çıkan sembolik bağ reddedilir. Normal dosya
   olmayan hedef (dizin) reddedilir. Bütün beklenen arızalar C013 `Hata`dır,
   worker'ı düşürmez.
4. Statik etki `DisOkuma`; eylem savepoint'ini ve PostgreSQL/dosya yazım
   ayrımını etkilemez. IO trace/replay şema-1 `dosya_ilk_baytlari` olayı exact
   yol + bayt sayısı argümanı ve sonuçla kaydedilir; replay dosyaya dokunmaz.

## Değerlendirilen seçenekler

- **İçerik türü (`dosyasının içerik türünü almayı dene` → MIME)** —
  reddedildi. Tanınan tür listesi ve tespit kuralları dilin sözü olurdu;
  spec/26 içerik türü doğrulamasını açıkça kapsam dışı bırakır, her yeni tür
  derleyici değişikliği isterdi ve "türü tanındı" sonucu güvenlik denetimi
  sanılabilirdi. İlk-bayt okuması ürüne karşılaştırmayı bırakır, dil yalnız
  sınırlı okuma sözü verir.
- **Genel ikili okuma / Bayt türü** — reddedildi; yeni değer türü ve
  büyük yüzey ister, ürün ihtiyacı 4–8 bayttır.
- **Sabit 8 bayt (sayı argümansız)** — reddedildi; PDF 5, OLE 8, ZIP 4
  bayt ister ve tek sabit bir ürünün imza tablosunu dile gömerdi.
  Sayı argümanı ürünün tam imza uzunluğunu okumasını sağlar, böylece önek
  karşılaştırması için yeni metin işlemi gerekmez.
- **Büyük harf / ayraçlı onaltılık** — reddedildi; `sha256 özetini almayı
  dene` ve `yukleme_sha256` küçük harf ayraçsızdır.

## Sonuçlar

- Ürün kabul anında imzası tutmayan CV'ye 415 döner ve geçici dosyayı siler;
  indirmedeki denetim savunma derinliği olarak kalır.
- İçerik imzası eşleşmesi dosyanın zararsız olduğunu kanıtlamaz
  (polyglot dosya); antivirüs/içerik doğrulaması hâlâ spec/26 dışıdır (GB-130).
- `dil dene` ve playground'un `ToplayanIo` adaptörü bellek içi metni bayt
  olarak verir; gerçek ikili kanıt `proje_testi` CLI vakasındadır.
- Freeze yüzeyi: yeni kalıp kelimesi `onaltılık`, API `dosya.ilk_baytlar`,
  iz olayı `dosya_ilk_baytlari` (`dil-yuzeyi-v1.tsv`, 1.0.0-dev).

## Kanıt

- `compiler/tests/proje_testi.rs::dosya_imzasi_gercek_ikili_dosyada_sinirli_ve_proje_kokunde_okunur`
- `compiler/tests/dosya_yasam_dongusu_testi.rs::ilk_baytlar_kucuk_harf_onaltilik_ve_sinir_disi_c013`
- `compiler/tests/dosya_yasam_dongusu_testi.rs::ilk_baytlar_tur_hatasi_derlemede_reddedilir`
- `compiler/tests/io_izi_testi.rs::dosya_yasam_dongusu_izi_fiziksel_etki_yapmadan_oynatilir`
- Dogfood: [`dogfood/zee-cekirdek/dosya-imzasi`](../dogfood/zee-cekirdek/dosya-imzasi/README.md)
