# ADR-087: Açık depoda anonim ürün kanıtı (K-214)

- **Durum:** kabul
- **Tarih:** 13 Ekim 2026
- **İlgili kayıt:** K-214

PR #5/#7, gerçek ürün talebinin kopyası ve kabul logları üzerinden müşteri ve
altyapı kimliğini ana kaynak hattına taşımıştı. Kullanıcı bugünkü geçmişin
anonimleştirilmesini, force push ve exact-SHA kayıtlarının yenilenmesini istedi.
Bu işlem dil davranışını genişletmez; kişisel/kurumsal kimliği ürün kanıtından ayırır.

Ürün adları Site A–F/ürün E olarak anılır. Gerçek ürün deposu, talep ve kaynak
özel ürün tarafında kalır; açık kanıt exact harici commit SHA'sı, küçültülmüş
reproducer ve anonim özet taşır. Yerel/sunucu yolu ve müşteri alan adı taşınmaz.
K-208/K-209/K-210–K-213 dogfood kayıtları ve freeze zinciri korunur.

`scripts/acik-depo-denetle.py` izlenen dosya adlarını, içerikleri, yeni dosyaları,
symlink hedeflerini, ilk commit dahil bütün ata ağaçlarını ve mesajlarını denetler.
`docs/acik-depo-politikasi-v2.json` yalnız normalize edilmiş terim/yol değerlerinin
gizli anahtarlı HMAC-SHA256 özetlerini taşır (K-334; aşağıya bakın). Unicode biçim/harf değişimleri ve ayırıcılar sentetik testle kapsanır.
Hata çıktısı değer veya dosya adı yerine dosya SHA-256'sı ve satırı verir.
Eksik politika ve sığ/kayıp Git geçmişi başarısız olur. Sıkıştırılmış dosyanın
içini veya görseldeki yazıyı tarama iddiası yoktur; ürün kanıtı metin olarak tutulur.
K-214'teki tuzsuz SHA-256 listesi sözlük saldırısına açıktı; K-334 bunu
anahtarlı HMAC ile değiştirdi. Özel terim listesi açık depoda yoktur.

K-215 ile kullanıcı önceki geçişlerin temizliğini de onayladı. Eski kayıt
istisnaları kaldırıldı: politika `all-reachable` ister, fingerprint listesi
boş olmak zorundadır. İlk commit ve eski taban öncesindeki silinmiş içerik
dahil dalın tüm ataları denetlenir. K-163 anonim `legacy-panel` kimliğiyle
provenance/reproducer zincirini korur.

`git filter-repo` yeniden yazımında merge ebeveynleri ve vaka kimlikleri korunur;
kaynak beyanları ve regression `fixed_by`/`introduced_by` yeni SHA'ya bağlanır.
Ölçüm/ikili/provenance kayıtları özgün ölçülen kaynak SHA'sını korur; SHA eşlemesi
yeniden koşulmuş bir benchmark veya yeni ikili kabulü değildir. Kaynak yorumları
ve sentetik SMTP vektörleri anonimleştirilir, eski ürün davranışı testle korunur.

GitHub PR önbellek/ref'lerinin sunucu tarafı silinmesi destek talebi ister.
PR metinleri ve normal dal referansları temizlenir; PR refs/önbelleklerinin
temizliği teyit edilmeden tam silinme iddia edilmez. Enterprise yalnız aynı
kaynağın özel aynasıdır ve Actions kapalıdır. Depoyu yeniden açmak kullanıcı
onayı ister. Karar ve destek taslağı [K-214 kaydındadır](../docs/incelemeler/k214/README.md).

Kanıt: `compiler/tests/guvenlik_kapisi_testi.rs::acik_depo_kapisi_anonim_politikayi_ve_ci_kablosunu_korur`
ve `scripts/tests/acik-depo-test.py` olumlu/olumsuz tarihçe testleri.

K-215 compiler/fuzz PostgreSQL yamasını kayıtlı path kaynağına geçirir;
Git/credential yönlendirmesi kaldırılır. SBOM exact build commit'inin ve
derlenen yerel ağacın SHA-256'sını birlikte doğrular (ADR-068).
`ci-taban-kimligi.py` yalnız K-214 üç eski uç ve K-215 eski master için
kaydedilmiş yeni ata SHA'yı doğrular; genel force istisnası yoktur.
`acik-depo` workflow'u push/PR kaynağını güvenilir master denetleyicisiyle
yalnız veri olarak tarar; PR kodu çalıştırılmaz, token salt-okunurdur.

K-215 kapsam/özel ürün dalı sınırı [yeni kayıttadır](../docs/incelemeler/k215/README.md).

K-326 (20 Ekim 2026), 14–17 Ekim belge sürümlerinde kalan sunucu adı ve
sunucu kök dizini örnek yollarını tarihçede de güncel anonim karşılıklarına çevirir;
kapı tam tarihçede istisnasız geçer. `ci-taban-kimligi.py` iki eski ana
dal ucunu (GitHub ve ayna) yalnız [K-326 eşlemesindeki](../docs/incelemeler/k326/README.md)
exact yeni ataya çevirir; genel force istisnası yine yoktur.

K-239/GB-063, ürünleşme incelemesine yanlışlıkla eklenen üç özel CI ayna
URL’sini rol ve SHA-256 kayıtlarına dönüştürür. Yalnız 14 Ekim izole iş
serisinin 15 commit’i yeniden kurulur; ilk iki commit ve be5699c başlangıcı
sabittir. Kaynak/freeze/regression ve sentetik TLS kökenleri yeni exact
SHA’ya taşınır. Orijinal dal/rapor yayın ağacının dışında saklanır.
[SHA eşlemesi](../docs/incelemeler/urunlesme-20261014/sanitized-sha-map.tsv)
özgün derleme ve native checkpoint sonuçlarını yeni SHA’da üretilmiş saymaz.

## K-334 — Anahtarlı politika ve iz temizliği (17 Kasım 2026)

17 Kasım 2026 dış taraması, K-214 politikasındaki tuzsuz SHA-256 değerlerini
olası müşteri/proje/altyapı adlarıyla sözlük karşılaştırması yaparak eşleştirdi
(GB-131). Gerçek müşteri kaydı veya sır yayımlanmamıştı; açık olan, adların
listeden geri kazanılabilmesiydi. Karar:

- Politika `zee-acik-depo-politikasi-2` şemasına ve
  `docs/acik-depo-politikasi-v2.json` dosyasına taşındı. Her kural yalnız
  `kind`, `length` ve `hmac` taşır; `hmac` = HMAC-SHA256(anahtar, normalize
  değer). Kurallar HMAC değerine göre sıralanır, düz metin sırası sızmaz.
- 32 baytlık anahtar depo dışındadır: GitHub Actions sırrı
  `ZEE_ACIK_DEPO_ANAHTARI` ve yerelde `~/.config/zee/acik-depo-anahtari`
  (0600). Düz terim listesi de depo dışındadır
  (`~/.config/zee/acik-depo-terimleri.json`); politika yalnız
  `scripts/acik-depo-denetle.py --politika-uret <özel liste>` ile yenilenir.
  Liste PR metnine, commit mesajına ya da loga yazılmaz.
- Politika `key_id` = HMAC(anahtar, sabit etiket) taşır. Yanlış anahtar hiçbir
  terimle eşleşmeyeceği için kapı bu durumda geçmez, çalışamadı (çıkış 2) der.
- Anahtar yoksa kapı varsayılan olarak çıkış 3 ile durur. `--anahtar-yoksa atla` (CI’da `ZEE_ACIK_DEPO_ANAHTAR_YOKSA=atla`; master denetleyicisi eski bayrak kümesiyle de çalışsın diye)
  yalnız anahtarın bilinçli verilmediği koşullar içindir ve "ATLANDI … geçti
  sayılmaz" yazar, GitHub'da uyarı ve adım özeti bırakır.
- Fork PR'ına anahtar verilmez (`pull_request_target` dahil): anahtarlı kapı
  dışarıdan gönderilen aday adlara evet/hayır diyen bir sözlük kâhini olurdu.
  Fork içeriği birleştikten sonra `push` taramasında anahtarla denetlenir;
  bakımcı fork PR'ını birleştirmeden önce kapıyı yerelde anahtarla çalıştırır.
  Özel Forgejo aynası sır tanımlı değilse açıkça atlar.
- K-214'ün bir yol kuralının düz metni yeniden kurulamadığı için HMAC'e
  taşınmadı; diğer kurallar birebir korundu. Eksik kural özel listeye
  eklendiğinde politika yeniden üretilir.
- HMAC hesaplaması önceden hazırlanmış iç/dış durum ve önbellekle yapılır; tam
  tarihçe taraması eski SHA-256 sürümünden yaklaşık 1,4 kat yavaştır, CI zaman
  sınırı 20 dakikaya çıkarıldı.

Güncel ağaçta iç koşucu adı, ürün önizleme adı, sentetik host olarak kullanılan
kuruluş adı ve canlı ürün sayım/envanter cümleleri genel ifadelerle
değiştirildi. Sunucu etiketleri ve sunucu kök dizini yolları güncel ağaçta yoktu (K-326).
Sitedeki `dil.wasm` eşleşmesi derleyici bağımlılığının `bitwise` sembolüydü;
üretilmiş ikili değişmedi.

**Bilinçli yayın kararı — ayna referansları:** `itwise-bilisim/zee` adı,
K-202'den önceki kanonik kaynak ve bugünkü özel ayna olarak SBOM/provenance
fixture'larında, tarihsel Actions koşu bağlantılarında ve tedarik belgelerinde
kalır. Bu adlar exact provenance URI'sinin parçasıdır; değiştirilmeleri
imzalı/ölçülmüş kanıtı bozar. Kuruluş adı müşteri kimliği değildir; kurucunun
kendi kuruluşudur ve yayımlanması bilinçlidir. Bu nedenle `itwise` tek başına
politika terimi değildir; yalnız iç koşucu ve önizleme gibi altyapı bileşik
adları temizlenir.

**Yazar e-postası:** Bu depoda commit yazarı ve kaydedeni GitHub noreply adresi
(`<id>+<kullanıcı>@users.noreply.github.com`) kullanır; kurumsal veya kişisel
e-posta yazılmaz. Katkıcı depo içinde yerel ayar yapar:
`git config user.email "<id>+<kullanıcı>@users.noreply.github.com"`.
K-334 öncesi kurumsal adres taşıyan üç commit, ikinci aşamadaki tarihçe
yeniden yazımında `.mailmap` eşlemesiyle noreply'a çevrilir.

İkinci aşama (kullanıcı onaylı): Aşama 1 birleştikten sonra eski tuzsuz hash
listesi, altyapı etiketleri ve kurumsal e-posta tarihçeden `git filter-repo` ile
kaldırılır; tam yedek bundle, force push, ayna eşitlemesi ve tüketici
`dil-kur@<sha>` pinlerinin taşınması K-326 sırasını izler. GitHub PR ref ve
önbellek temizliği destek talebi ister; teyit gelene kadar GB-131 açık kalır.

Kanıt: `scripts/tests/acik-depo-test.py` (17 sentetik vaka: yanlış anahtar,
anahtarsız hata/ATLANDI, düz metin ve tuzsuz hash taşımayan üretim, şema/kural
reddi) ve
`compiler/tests/guvenlik_kapisi_testi.rs::acik_depo_kapisi_anonim_politikayi_ve_ci_kablosunu_korur`.
