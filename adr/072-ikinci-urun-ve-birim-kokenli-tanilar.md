# ADR-072 — İkinci gerçek ürün farklı iş yükünde; elmas birim içe alımı ve birim kökenli tanılar

- **Durum:** kabul
- **Tarih:** 6 Eylül 2026
- **İlgili kayıt:** K-164, K-163, K-172, ADR-059, ADR-067, RFC-0009, spec/07, V1-P1-29

## Bağlam

Tek dogfood ürünü (Çatlı/ITWISE Admin: web + PostgreSQL) dilin bir iş yüküne
aşırı uyup uymadığını gösteremezdi; üçüncü dış inceleme K-164 ile "farklı
workload'da ikinci gerçek proje" istedi. Depo aynı zamanda dokuz kayıt
defterini (regresyon, güvenlik, spec maddesi, dogfood, deprecation, soak, iki
beyan) elle okuyordu; tek sayfalık makine üretimi özet yoktu. İkinci ürün bu
gerçek ihtiyacı karşılar: toplu metin/veri işleme (TSV → Markdown), web/DB/ağ
yok.

Ürün yazılırken iki compiler kusuru çıktı. (1) Birim `c`, `a` ve `b`
birimlerini, `b` de `a`yı kullanınca `a`nın işlemleri `c`ye iki yoldan
geliyor ve yükleyici bunu A008 çakışması sayıyordu; spec/07 yalnız döngüyü
yasaklar, elmas içe alımı değil. (2) Birim dosyasındaki ayrıştırma, sözleşme,
gövde, çalışma ve test tanıları birimin satır numarasıyla ama ana kaynağa
göre raporlanıyordu: CLI yanlış satırı alıntılıyor ya da alıntıyı düşürüyor,
LSP tanıyı açık belgede var olmayan satıra koyuyor, hangi dosyanın hatalı
olduğu söylenmiyordu.

## Karar

1. **Ürün.** `dogfood/kanit-ozeti` depo içi ikinci üründür: `proje.dil`
   yalnız `dosya-okuma`/`dosya-yazma` ister; 10 birim (TSV/sayım/markdown
   yardımcıları + yedi bölüm üreticisi) ve giriş `kaynak/ana.dil`;
   `docs/kanit-ozeti.md` sayfasını üretir. `docs/dogfood-projeleri-v1.tsv`
   kaydında `koken_commit` depo içi ürün için ürünün üzerine kurulduğu taban
   commit'idir. Depo-genel araç olduğu için üretim komutu dosya kipinde koşar
   (`dil çalıştır dogfood/kanit-ozeti/kaynak/ana.dil`); proje kipinde dosya
   sınırı ürün köküdür ve bu bilinçli olarak gevşetilmez.
2. **Kanıt.** `kanit_ozeti_testi` ürünü gerçek kayıt defteri içerikleriyle
   hermetik koşar; sayfa depodakiyle bayt bayt aynı olmalı (tazelik), iki
   koşu aynı çıkmalı (determinizm), sayılar bağımsız Rust sayımıyla tutmalı
   (doğruluk) ve bütün birim testleri geçmelidir. CI ayrıca gerçek CLI ile
   sayfayı yeniden üretip `git diff --exit-code` ister.
3. **Korpus `proje` kipi.** ADR-067 manifesti `proje` kipini kazanır: depo
   içi ürünün giriş dosyası birimleriyle diskten yüklenir, ürün politikasıyla
   derlenir, bütün `test` blokları koşar; giriş klasörü altındaki her `.dil`
   o vakaya aittir. Sürtünme ret/çözüm çiftleri `dogfood/<ürün>/korpus/`
   altında kalır. Sözcükleme düzeyinde reddedilen sürtünme (örn. S040)
   biçimlenemez; yalnız beklenen tanısı aynı kodsa kabul edilir.
4. **Elmas içe alım.** Yükleyici her işlem, yapı ve testin TANIMLANDIĞI
   kökeni izler. Aynı ad daha önce aynı kökenden geldiyse ikinci yol sessizce
   atlanır; farklı kökenden gelen aynı ad A008 kalır; birim testleri kökeni
   başına bir kez alınır. Görünürlük kuralı değişmez: kullanan yalnız birimin
   doğrudan tanımını görür.
5. **Birim kökenli tanı.** `Tani`, `Islem` ve `Test` isteğe bağlı `koken`
   taşır. Birim çözümünden dönen, birim işleminin sözleşme/gövde denetiminde,
   çalışma zamanında ve birim testinde doğan tanılar en içteki kökenle
   etiketlenir; satır/sütun o kökenin metnine göredir. CLI kökenli tanıda
   alıntıyı birim dosyasından alır ve "Birim: <köken> (satır N)" başlığı
   yazar; okunamayan köken yalnız başlıkta anılır. LSP tanıyı o birimin
   `kullan` satırına taşır, mesajı "<birim> birimi, satır N: …" ile açar.
   `dil denetle --json` kökenli tanıda ek `koken` alanı verir; kökensiz tanının
   metin ve JSON biçimi bayt bayt değişmez. Tanı kimlikleri değişmedi.

## Reddedilen seçenekler

- **Ürünü proje kipinde koşturmak için dosya sınırına okunabilir ek kök
  eklemek:** güvenlik profilini (spec/23–24) gerçek ürün ihtiyacı olmadan
  genişletir; dosya kipi geliştirici politikasıyla zaten çalışır. İhtiyaç
  gerçekleşirse ayrı RFC ister.
- **Elmas için geçişli tanımları yeniden dışa açmak:** RFC-0009 kapsülleme
  kuralını bozar; yalnız çakışma sayımını köken üzerinden düzeltmek yeterlidir.
- **Birim tanısını ana kaynaktaki `kullan` satırına taşımak (CLI):** gerçek
  konumu gizler; köken + birim metninden alıntı doğru bilgidir. LSP'de
  taşıma zorunludur çünkü tanı açık belgeye yayımlanır.
- **Morfoloji profilini (`zee-tr-1`) F006/F007/F008/F013 için değiştirmek:**
  profil sürümlüdür ve donmuştur (spec/13); ad seçimi kılavuzu ve korpus
  çiftleriyle yaşar, `zee-tr-2` adayı olarak günlüğe yazıldı.

## Sonuçlar

- On altı sürtünme (K-164/F001–F016) günlükte; on biri korpusa ret/çözüm
  çifti olarak girdi; F002 ve F012 compiler düzeltmesidir.
- Üç kayıt defteri artık tek sayfada; bayat sayfa CI'ı durdurur.
- Elmas içe alım ve kökenli tanı `birim_testi` (8 yeni test) ve `lsp_testi`
  ile bağlıdır; `kanit_ozeti_testi` ürünün kendisini kapı yapar.
- K-165 aynı aracın Rust/Go eşdeğerleriyle veri temelli karşılaştırmasını
  ADR-073'te taşır.
