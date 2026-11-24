# ADR-171 — Dosyadan dosyaya akışlı base64 çözme, bayt arama ve 32 MiB hedef tavanı (RFC-0043 Dilim 2, kısmi)

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-395, RFC-0043 (Dilim 2, kısmi), ADR-167 (Dilim 1),
  ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu),
  ADR-170 (paylaşılan veri kökü, K-394), spec/26, spec/08, spec/21, spec/24,
  GB-166

## Bağlam

`urun-h` posta motoru gelen iletinin eklerini ADR-167 (K-389) yüzeyiyle
çıkarıyor. 25 MB'lık iletideki tek 15–18 MB'lık base64 ekte iki yol da
tıkandı (ZM-F039):

- Satır satır okuma ~4,7 MB'ta C024'e çarpıyor: spec/24 çalışma değeri
  zarfı kümülatif tahsisi sayıyor (ayrı iş K-393).
- 1 MiB pencereli Dilim 1 çözümü çalışıyor (20 pencere × 1 048 554 bayt,
  15,3 MB, 0,56 sn), ama ek gövdesinin **bayt** başlangıcı bulunamıyor:
  pencereler `Metin` olduğu için harf konumu bayt konumuna eşit değil
  (8bit/UTF-8 başlık), dilde bayt arama yok. Üstelik `eklemeyi dene` hedef
  tavanı 16 MiB'tır; 18 MB'lık ek sığmaz.

Mustafa 24 Kasım 2026'da RFC-0043 Dilim 2'den yalnız bu üç parçanın
öne alınmasını onayladı. Ürün kaydı etkin, reproducer depo içinde
(`dogfood/urun-h/buyuk-ek/`); yeni tür ya da grammar kuralı yok.

## Karar

1. **Dosyadan dosyaya akışlı çözme** (`dosya-yazma` derlemede; kaynak
   çalışma zamanında ayrıca `dosya-okuma`; `DisYazma`):

   ```zee
   tutamak yol dosyasının konum baytından n baytını base64 çözerek hedef dosyasına eklemeyi dene olsun
   ```

   `konum` ≥ 0, `n` 1…50 331 648 (48 MiB: 32 MiB çıktının 76 karakterlik
   CRLF satırlı base64'ü ~43,8 MiB). Kaynak 64 KiB parçalarla okunur, saf
   `kodlama::Base64AkisCozucu` ile çözülür (ADR-167'nin katı MIME base64
   kurallarının aynısı; Dilim 1 bellek çözücüsü de bu durum makinesini
   kullanır), çıktı `kalici_dosya::akisla_sona_ekle` ile hedefe akar.
   Aralık hiçbir aşamada `Metin` olmaz, bütün içerik bellekte tutulmaz,
   spec/24 zarfına sayılmaz.
   - **Atomiklik:** kilit altında var olan hedef geçici dosyaya akışla
     kopyalanır (sembolik bağ izlenmez), çözülen baytlar arkasına yazılır,
     `fsync` + atomik adlandırma ile yayımlanır. Çözme hatası, kaynak okuma
     hatası ya da tavan aşımı hedefi hiç değiştirmez; geçici dosya silinir.
     ADR-167'nin oku-birleştir-değiştir eklemesiyle aynı gözlenebilir
     sözdür, yalnız bellek yerine geçici dosya kullanır.
   - **Tanı:** bozuk base64 C032, `konum` = **kaynak dosyadaki** mutlak bayt
     (`uzunluk` için aralığın sonu); dosya/aralık/yol/tavan/eylem C013.
     Kaynak ve hedef aynı dosyaysa C013 (kendini okuyan ekleme yok).
   - **Sonuç:** K-389 tutamağı (`yol`, `sha256` hedefin bütünü, `bayt`
     toplam) + `eklenen` (bu çağrıda çözülen bayt). Boş aralık hedefi
     oluşturur, `eklenen: 0`.
   - **Eylem:** K-389 akışlı eklemesi gibi eylem içinde C013.
2. **Bayt arama** (`dosya-okuma`, `DisOkuma`):

   ```zee
   yer yol dosyasında konum baytından "--sınır" aramayı dene olsun
   ```

   İğne 1–1024 UTF-8 bayt `Metin` (ifade olabilir); sonuç ilk eşleşmenin
   mutlak bayt konumu ya da `-1`. Karşılaştırma bayt bayttır. Tarama
   `konum`dan en çok 64 MiB'tır ve KMP ile doğrusaldır (saldırgan girdi
   `O(n·m)` taramaya zorlayamaz; bellek iğne boyuyla sınırlı). Pencere
   dolduğunda dosya sürüyorsa C013 — belirsiz `-1` yoktur. MIME yapısı
   (sınır, başlık katlama, `Content-Transfer-Encoding`) dilde
   yorumlanmaz; ürünün MIME ayrıştırıcısı bu iki ilkelle bayt konumlarını
   kurar.
3. **Hedef tavanı 16 → 32 MiB:** `çözerek … yazmayı|eklemeyi dene` ve yeni
   akışlı ekleme hedefin toplamını `kodlama::AZAMI_HEDEF_BAYTI` (32 MiB)
   ile sınırlar. Yükleme, dosya yanıtı ve dosya özeti tavanları (16 MiB)
   değişmez; GB-166 DoS gerekçesini taşır.
4. **IO izi (içeriksiz, tam replay):** `dosyadan_base64_ekle [kaynak, konum,
   adet, hedef] → ok,<sha256>,<toplam>,<eklenen> | hata,<mesaj> |
   kodlama,<tür>,<konum>,<mesaj>`; `dosyada_bayt_ara [yol, konum, <iğne
   bayt>, <iğne sha256>] → ok,<konum> | hata,<mesaj>`. İki olay da içerik
   ya da iğne taşımaz ve replay'de kayıtlı sonucu döndürür.
5. **Yol ve veri kökü:** iki yol da politika katmanının
   `dosya_yolunu_denetle` ve adaptörün `dosya_yolu` kuralından geçer; K-394
   (ADR-170) `veri:` önekini bu ortak kuralda çözdüğü için ek uyarlama
   gerekmez.

## Değerlendirilen seçenekler

- **Dilim 1 pencerelerini bayt konumuyla döndürmek** (pencere + bayt
  uzunluğu) — reddedildi: sorunu ürüne taşır, her pencere yine `Metin`
  zarfına yazılır; 18 MB'ta zarf yine dolar.
- **Genel `Baytlar` türü** — reddedildi: core freeze altında yeni tür ve
  tüm kalıp ailesi gerektirir; Dilim 2 kalanında tartışılır.
- **MIME parçası çıkarma kalıbı (`ekini çıkar`)** — reddedildi: MIME
  ayrıştırması ürün sorumluluğudur; dile yalnız bayt konumu ve akış ilkeli
  eklendi.
- **Tavanı 64 MiB yapmak** — reddedildi: ürünün ileti sınırı 25 MB, en
  büyük tek ek ~18 MB; 32 MiB yeterli ve iki katlık disk/IO yükü sınırlı.
- **`memmem`/naif arama** — naif arama 64 MiB × 1024 bayt kötü durumda
  karesel; yeni crate tedarik kaydı açar. KMP 40 satırlık saf koddur.
- **Arama penceresi bitince `-1`** — reddedildi: ürün "yok" ile "bakılmadı"
  ayırt edemezdi.

## Sonuçlar

- Ürün 18 MB'lık eki bayt aramayla bulup tek çağrıda çıkarır; C024 ve
  pencere hesabı yoktur (`zm_f039_on_sekiz_mb_ek_bayt_aranip_tek_cagrida_cikarilir`).
- Freeze yüzeyi: kalıp kelimeleri `dosyasında`, `aramayı`; API
  `dosya.dosyadan_base64_cozerek_ekle`, `dosya.bayt_ara`; iz olayları
  `dosyadan_base64_ekle`, `dosyada_bayt_ara` (`dil-yuzeyi-v1.tsv`,
  1.0.0-dev). Yeni tanı kodu yok.
- Fuzz: yeni kalıplar `lexer_parser` hedefinin kapsamındadır (kalıp
  eşleme, yeni token türü yok); akış çözücüsü ve KMP için ayrı `kodlama`
  fuzz hedefi K-389'dan beri açık iştir. RC fuzz kapısı bu değişiklikle
  değişmez.
- Açık: dosyadan dosyaya quoted-printable, UTF-8 bayt sayısı,
  metin↔base64/onaltılık, kod noktası, base64url (RFC-0043 Dilim 2 kalanı);
  C024 kümülatif zarfı (K-393).

## Kanıt

- `compiler/tests/kodlanmis_dosya_testi.rs`:
  `zm_f039_on_sekiz_mb_ek_bayt_aranip_tek_cagrida_cikarilir`,
  `dosyadan_cozme_tavan_sinirda_kacis_bozuk_ve_eylem_hedefe_dokunmaz`,
  `bayt_arama_parca_sinirinda_utf8_ignede_ve_tarama_sinirinda`,
  `hermetik_dosyadan_cozme_arama_konum_ve_aralik_reddi`,
  `k395_iz_icerik_tasimaz_ve_tamamen_oynatilir`,
  `dosyadan_cozme_kaynak_icin_dosya_okuma_ister`,
  `no_clobber_tavan_yol_gecisi_ve_eylem_reddi_hedefi_bozmaz` (32 MiB)
- `compiler/src/kodlama.rs` birim testleri (akış çözücüsü her parça
  sınırında, mutlak konum, KMP parça sınırı) ve
  `compiler/src/kalici_dosya/cozulmus.rs` (akışlı eklemede tavan ve üretici
  hatası hedefe dokunmaz)
- Dogfood: [`dogfood/urun-h/buyuk-ek`](../dogfood/urun-h/buyuk-ek/README.md)
