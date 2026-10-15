# ADR-046 — Production katman sahipliği ve bağımlılık yönü

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-149, B-057

## Bağlam

K-099/ADR-012 büyük parser/checker/runtime handler'larını fiziksel modüllere,
K-144/ADR-041 kritik işlevleri boyut/karmaşıklık eğilimine bağladı. Bunlar bir
dosyanın büyümesini gösterir; fakat küçük bir parser dosyasının checker'ı,
runtime'ın parser ayrıntısını veya LSP'nin compiler geçişini doğrudan çağırması
aynı eşikleri aşmadan katman sınırını bozabilir. Büyüyen compiler'da esas risk
yalnız satır sayısı değil sorumluluk ve bağımlılık yönüdür.

İlk graph incelemesi iki gerçek ters kenar buldu: morfoloji parmak izi SHA-256
için paket modülüne, tedarik RFC 3339 zamanı için yorumlayıcıya bağımlıydı.
İki yardımcı da üst sahibin davranışı değil katmanlardan bağımsız ilkeldi.

## Karar

1. Bütün production Rust ağacı 35 üst sahibin birine atanır. Her sahip;
   `temel`, `model`, `altyapi`, `sozdizimi`, `semantik`, `proje`, `web`,
   `runtime`, `adapter` veya `muhendislik` katmanındadır. Katmanların izinli
   hedef yönü kapı kodunda kapalı küme olarak tanımlıdır.
2. `katman-mimarisi-v1.tsv`, her sahibin sıralı ve exact doğrudan iç
   bağımlılıklarını ve sorumluluğunu kaydeder. Yeni/kayıp sahip kadar eklenen
   veya artık kullanılmayan kenar da bilinçli taban incelemesi olmadan
   fail-closed'dur. Exact tabana eklemek ters katman kenarını meşrulaştırmaz.
3. Kaynak tarayıcı yorum/metin/karakter ile test-only kodu ayırır; `crate::`,
   `dil::`, toplu `use` ve açık modül yollarını toplar. Kökü takma adla
   gizlemek reddedilir. Hedefe özgü production kenarları platform birleşimi
   olarak korunur; böylece Linux'ta görünmeyen native/WASM drift'i saklanmaz.
4. Parser semantic/runtime'a, checker runtime/adaptöre, runtime parser/checker/
   LSP/WASM'a ve LSP parser/checker/runtime'a doğrudan bağımlanamaz. Adaptör ve
   mühendislik araçları iç katmanları tüketebilir; ürün katmanları mühendislik
   araçlarını tüketemez.
5. SHA-256 sahibi `paket`ten temel `guvenlik` modülüne taşınır. Gregoryen
   dönüşüm yeni temel `zaman` modülüne taşınır; runtime public yolu uyumluluk
   için yeniden dışa aktarılır. Böylece morfoloji→paket ve tedarik→runtime
   ters kenarları tabana alınmadan kaldırılır.

## Reddedilen seçenekler

- **Yalnız satır/fonksiyon bütçesine güvenmek:** küçük ama ters yönlü importu
  göremez.
- **Sadece birkaç yasak metin aramak:** yeni modül ve daha önce düşünülmemiş
  yeni kenar fail-open kalır.
- **Bulunan bütün kenarları tek katmanda serbest bırakmak:** güncel graph'ı
  listeler fakat mimari yönü korumaz.
- **Test bağımlılıklarını production graph'a katmak:** HIR testinin runtime'ı
  doğrulaması gibi meşru aşağı-akış kanıtını ürün bağımlılığı sanır.
- **Yeni parser bağımlılığı gelince fixture'ı otomatik yenilemek:** gözden
  geçirme kapısını sıradan snapshot güncellemesine indirger.

## Sonuçlar

- Her production dosyası bir sahibin altında; her açık iç kenar exact ve
  katman yönüyle denetlenir. Yeni sahip/kenar sessizce CI'dan geçemez.
- Ortak SHA-256 ve takvim ilkelleri üst katmanlardan aşağı taşındı; kullanıcı
  davranışı ve public runtime tarih yolu korunur.
- Faz matrisi katman testini `Engineering gates` altında çalıştırır. Dil
  sözdizimi, runtime semantiği, tanılar, RFC ve normatif spec değişmez.
- K-149 sonrası makine backlog'unda bağımlılık-hazır açık compiler maddesi
  kalmaz. B-001 ve B-002'nin kapanışı önceden ilan edilmiş gerçek çocuk ve
  profesyonel usability verisini beklemeye devam eder.

## K-150 tamamlayıcı karar

ADR-047 aynı exact graph'ın SCC'lerini de fail-closed yaptı. Sahip sayısı
`semantic_model` ve `tani_politikasi` ile 37'ye çıktı; iki çevrim kırıldı ve
paket/registry/tedarik çevrimi süreli izinle görünür borç olarak sınırlandı.
K-160/ADR-059 bu son borcu fiziksel sahiplik ayrımıyla kapattı; güncel graph'ta
SCC ve izin sayısı sıfırdır. Güncel gerçek için katman rehberi bağlayıcıdır.

K-215 SBOM path ağacı doğrulaması aynı mühendislik sahibinde kalır; katman/çevrim kapıları yeni kenar olmadığını doğrular.

K-240 GET handler ve IO replay trait uygulamasını mevcut yorumlayıcı
sahibinin alt dosyalarına ayırır. Yeni üst sahip veya temel→adaptör geçişi
yoktur; exact owner kenarları/SCC fixture’ı yalnız farkı susturmak için
güncellenmez, kapıda aynı graf olarak doğrulanır.

K-240 SMTP testleri ayrı native test dosyasına, DB/posta profil erişicileri
mevcut domain profil dosyalarına taşınır. Test isimleri/faz sahipliği, profil
değerleri, public yüzey ve üretim gövdeleri korunur; spec kanıtı yeni exact
test dosyasına taşınmıştır.

K-253: yeni private CLI `web_kapanis` adapter sahibi processwide SIGTERM
disposition ve listener poll yaşam döngüsünü taşır. `dil_cli -> web_kapanis`
exact kenarı bilinçli eklenir; helper yalnız std/libc kullanır, temel veya
semantic modelden adapter'a ters geçiş ve yeni SCC yoktur. Domain handler,
transaction ve desteklenen v1 facade sahipliği değişmez (ADR-113).

## K-257–K-260 production sahipliği incelemesi

Yeni private CLI `web_metrikleri` ve `web_listener` adaptör sahipleri eklendi.
Exact ileri kenarlar `dil_cli -> web_metrikleri`, `dil_cli -> web_listener` ve
`web_listener -> surum` olarak fixture'da bilinçli tanımlıdır. Metrik sahibi
std/libc ve sınırlı numeric state taşır; PostgreSQL havuzunun callback/observer
arayüzü sahiplik sınırıdır, `postgresql` temel/altyapıdan CLI adaptöre import
etmez. Snapshot/state() callback dışında havuz sahibinde örneklenir; CLI sınırlı
veri snapshotını tüketir. IO
havuz callback'ine girmez. Yeni `postgresql/havuz/metrik.rs` mevcut PostgreSQL
sahibinin alt dosyasıdır. `web_kapanis` K253 sahibinde kalır.

K258/K260 Python host/credential kompozisyonu Rust graph'a hayali kenar eklemez;
K246 exact Popen/group/WNOWAIT ve tek designated descriptor ownership korunur.
Desteklenen `dil::api::v1` export/grammar aynı kalır. Katman/SCC fixture'ı yalnız
snapshot susturmak için yenilenmez; kaynak graph, ters kenar ve public allowlist
aynı root dalgasında kapılardan geçer. Clean final gate olmadan yeni source
SCC/API kabulü tamamlanmış sayılmaz. K262 aynı base'deki correctness dokümanı
bu append ile birleştirilir; eski full-file içerikle overwrite yapılmaz.
