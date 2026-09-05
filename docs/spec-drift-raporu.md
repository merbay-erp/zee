# Spec maddesi ↔ test kanıtı drift raporu

<!-- `cd compiler && cargo run --locked --bin spec_drift -- --rapor-yaz` üretir. Elle değiştirme. -->

Bu rapor K-171/ADR-065 kapısının güncel görünümüdür. Madde; `spec/` altında ZORUNLU/ZORUNDA/YASAK/TANIMLI/AÇIK işaretçisi taşıyan paragraf, liste öğesi ya da başlıktır. Kimlik dosya adı + sadeleştirilmiş metnin parmak izidir; metin değişince kimlik değişir ve kanıt yeniden incelenir.

| Bölüm | Madde | Kanıtlı | Kısmi | Açık |
|---|---:|---:|---:|---:|
| `spec/01-sozcukleme.md` | 6 | 5 | 1 | 0 |
| `spec/02-dizim.md` | 7 | 5 | 2 | 0 |
| `spec/03-adlar-ve-kapsam.md` | 6 | 6 | 0 | 0 |
| `spec/04-turler.md` | 13 | 12 | 1 | 0 |
| `spec/05-degerlendirme.md` | 9 | 8 | 1 | 0 |
| `spec/06-hata-modeli.md` | 7 | 7 | 0 | 0 |
| `spec/07-birimler.md` | 7 | 7 | 0 | 0 |
| `spec/08-kalici-dosya.md` | 5 | 4 | 1 | 0 |
| `spec/09-son-tarih-ve-iptal.md` | 1 | 1 | 0 | 0 |
| `spec/10-disari-acik-islem-sozlesmesi.md` | 4 | 3 | 1 | 0 |
| `spec/11-uygulama-eylemleri-ve-web-adaptoru.md` | 2 | 2 | 0 | 0 |
| `spec/12-web-guvenlik-profili.md` | 6 | 5 | 1 | 0 |
| `spec/13-surumlu-morfoloji-profili.md` | 8 | 8 | 0 | 0 |
| `spec/14-yapilandirilmis-eszamanlilik.md` | 2 | 2 | 0 | 0 |
| `spec/15-yapilandirilmis-hata-degeri.md` | 3 | 3 | 0 | 0 |
| `spec/16-keyfi-hassasiyetli-ondalik.md` | 2 | 2 | 0 | 0 |
| `spec/17-deger-semantigi-ve-gezme.md` | 5 | 5 | 0 | 0 |
| `spec/18-paket-yayini.md` | 13 | 12 | 1 | 0 |
| `spec/19-registry-metadata-guveni.md` | 10 | 10 | 0 | 0 |
| `spec/20-ifade-grameri.md` | 7 | 7 | 0 | 0 |
| `spec/21-deterministik-io-izi.md` | 5 | 5 | 0 | 0 |
| `spec/22-deterministik-io-profili.md` | 5 | 4 | 1 | 0 |
| `spec/23-yetkinlik-ve-outbound-guvenligi.md` | 10 | 10 | 0 | 0 |
| `spec/24-kaynak-guvenlik-profili.md` | 7 | 6 | 1 | 0 |
| `spec/25-postgresql-veri-erisimi.md` | 7 | 4 | 2 | 1 |
| `spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md` | 2 | 1 | 1 | 0 |
| `spec/27-uyumluluk-ve-surumleme.md` | 10 | 7 | 2 | 1 |
| **Toplam** | **169** | **151** | **16** | **2** |

## Kısmi kanıtlı maddeler

- `spec/01-sozcukleme.md#7da78144c1ad1213` — - Kodlama **ZORUNLU** UTF-8'dir. — UTF-8 olmayan kaynak dosyanın açık reddi ayrı testte değil; lexer yalnız geçerli UTF-8 üzerinde fuzz/golden ile kanıtlı.
- `spec/02-dizim.md#fe0bd61f6615fe9d` — - Blok açan satırdan sonra girintili en az bir satır **ZORUNLU** (S007). — S007 için doğrudan olumsuz vaka yok; girinti kurtarma testleri eksik gövdeyi dolaylı kapsar.
- `spec/02-dizim.md#218b7d3be6574e46` — - Bir satırın girintisi, açık bloklardan birinin hizasına inmiyorsa **YA… — S005 hizasız girinti için doğrudan kodlu olumsuz vaka yok; kurtarma testleri girinti sınırını dolaylı kapsar.
- `spec/04-turler.md#8c66243a23919002` — - `<ad> al` başlangıç biçiminde parametre türleri bütün erişilebilir çağ… — T017 iki sırada kanıtlı; parametre sayısı uyuşmazlığı T015 için doğrudan olumsuz vaka yok.
- `spec/05-degerlendirme.md#a15d6be626477b4e` — Girdi bitişi (TANIMLI) — C005 girdi bitişi için doğrudan olumsuz vaka yok; `programı bitir` erken sonlanması golden 03/28 ile kanıtlı.
- `spec/08-kalici-dosya.md#532a9e61aa1fe70f` — - Dayanıklılık/kilit primitive'leri bulunmayan platform işlemi reddetmek… — Primitive'siz platform reddi yalnız desteklenen üç platformda derlenen koda bağlı; böyle bir platform CI matrisinde yok.
- `spec/10-disari-acik-islem-sozlesmesi.md#f57d072e0522c682` — - Parametresiz public işlem de dönüş satırını yazmak ZORUNDADIR. — Parametresiz public işlemde eksik dönüş satırı için ayrı olumsuz vaka yok; T039 parametreli biçimde kanıtlı.
- `spec/12-web-guvenlik-profili.md#5b46cd1562479ba8` — `__Host-` öneki nedeniyle Domain niteliği YASAK, Path `/` ve Secure ZORU… — Çerez adı ve HTTPS korkulukları kanıtlı; Path=/, Secure, HttpOnly, SameSite=Lax niteliklerinin tek tek reddi/varlığı doğrudan test edilmiyor.
- `spec/18-paket-yayini.md#2af407978947e1d4` — Üretici dört dosyayı yazmadan önce bütün kümeyi tüketici doğrulayıcıyla … — Yazmadan önce tam doğrulama ve yarım küme reddi kanıtlı; süreç çökmesi ortasında yarım küme senaryosu enjekte edilmiyor (söz de verilmiyor).
- `spec/22-deterministik-io-profili.md#d68db0760744fd45` — Profilin bu gözlemlerinden birini kıran değişiklik yeni profil kimliği, … — Snapshot gözlem değişikliğini yakalar; yeni profil kimliği/RFC zorunluluğu süreç kuralıdır, makinece yalnız snapshot farkıyla görünür.
- `spec/24-kaynak-guvenlik-profili.md#6077d317d3fd2556` — Sorgu 64 KiB, parametre listesi 100 öğe, tek parametre 64 KiB, okuma son… — Değerlerin tek profilden okunması mimari testle kanıtlı; sorgu/parametre/sonuç/migration sınırının bir fazlasında red gerçek PostgreSQL gerektirdiği için hermetik CI'da yok.
- `spec/25-postgresql-veri-erisimi.md#61d77038cc9e2321` — `sslmode=require`, `<BAĞLANTI_DEĞİŞKENİ>_TLS_CA_PEM` ortam değişkeninde … — Köksüz require ve prefer reddi hermetik; gerçek hostname/CA reddi ile TLS 1.2 tabanı yalnız ortam değişkenli saha testinde koşar.
- `spec/25-postgresql-veri-erisimi.md#d1bef824169efd6e` — - Okuma sütunları non-null metin olmak ZORUNDADIR. Farklı PostgreSQL tip… — TEXT bind ve metin sütun sözleşmesi hermetik sahte adaptörle kanıtlı; NULL/non-text sütunun gerçek sürücüde reddi CI'da PostgreSQL olmadığı için saha testindedir.
- `spec/26-binary-yukleme-ve-dosya-yasam-dongusu.md#71d603fe37b0b83a` — Unsafe rota CSRF belirtecini tek `X-Zee-CSRF` başlığından alır. Birden ç… — Binary akış, özet ve orphan görünürlüğü hermetik; `X-Zee-CSRF` başlığı ve çoklu Content-Type reddi gerçek TCP adaptöründe doğrudan test edilmiyor.
- `spec/27-uyumluluk-ve-surumleme.md#b31a6f2c9df44412` — - `dil sürüm`, LSP `serverInfo.version` ve yayın SBOM'u aynı kimliği bas… — Üç yüzey aynı CARGO_PKG_VERSION sabitini okur; SBOM/LSP/CLI çıktılarını birbirine eşitleyen ayrı test yok.
- `spec/27-uyumluluk-ve-surumleme.md#15d606bdb76e4dc1` — - Kaldırılan biçim sessizce kabul edilmez: derleyici tanı üretir ve öner… — Henüz kaldırılmış bir kullanıcı yüzeyi yok; tanı+öneri kanıtı ilk DEP kaldırmasında regresyon korpusuna eklenecek.

## Açık maddeler

- `spec/25-postgresql-veri-erisimi.md#f94f82cb0245cf0a` — Managed-provider sertifika rotasyonu, birden çok worker'ın toplam bütçe … — K-169/K-173 release tatbikatı ve çok-worker toplam bütçe provası bekliyor; async driver, PostgreSQL dışı DB ve ORM vaat edilmiyor.
- `spec/27-uyumluluk-ve-surumleme.md#285d87941dc0c3ba` — 1.0 sonrasında dondurulmuş yüzeylerde kırıcı değişiklik yalnız yeni ana … — Edition alanının `proje.dil` sözdizimi ayrı RFC ister; 1.0 öncesi bağlayıcı yalnız RFC-0028 çerçevesidir.

## Sürüklenme

Yok: her normatif madde kayıtlı, her kayıt var olan bir maddeye ve var olan test işlevine bağlı.

Sürüklenme otomatik bir semantik hüküm değildir: kayıtsız madde kanıt ister, bayat kayıt ya silinir ya yeni kimliğe taşınır; seçicinin maddeyi gerçekten kanıtladığı kod incelemesinde değerlendirilir.
