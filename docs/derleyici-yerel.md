# K-270 — yerel compiler import ve proje launcher

POSIX pilotu mevcut K-233 exact compiler seçicisine yalnız explicit yerel klasör import’u ve proje-local launcher ekler. İndirme, otomatik update, global PATH/dil shim, production key veya release üretimi yoktur.

## İki farklı kilit

`derleyici.kilit.json` mevcut beş alanlı `zee-derleyici-pin-1` compiler kimliğidir. `proje.kilit` v3 ise paket/morfoloji/kaynak ve registry güven zincirini sabitler; compiler pini değildir. Launcher current compiler pinini her çağrıda mevcut seçiciye okutur. Aynı compiler proje komutunda kendi dependency lock’unu doğrular; araç kilidi yazmaz veya dependency solver çalıştırmaz. Bağımlılıksız eski projede lock yokluğu kabul edilebilir; bağımlı projede eksik/stale lock P008 olarak kalır.

## Kullanım

Operatör mevcut exact proje pinini, trusted katalog + imzasını, verifier SHA’sını ve açılmış imzalı bundle klasörünü sağlar. `--depo` önceden oluşturulmuş operatöre ait, group/world writable olmayan dizindir. Bundle metadata trust/verifier/executable yolunu seçemez.

```sh
python3 -I scripts/derleyici-yerel.py import \
  --proje /exact/project --kaynak /local/opened-bundle --depo /owned/compiler-store \
  --guven /trusted/catalog.json --guven-anahtari sha256:<trusted-key-id> \
  --dogrulayici /trusted/surum_artefakti --dogrulayici-sha256 <exact-sha256>
python3 -I scripts/derleyici-yerel.py launcher \
  --proje /exact/project --depo /owned/compiler-store \
  --guven /trusted/catalog.json --guven-anahtari sha256:<trusted-key-id> \
  --dogrulayici /trusted/surum_artefakti --dogrulayici-sha256 <exact-sha256>
/exact/project/.zee-araclar/zee çalıştır denetle
/exact/project/.zee-araclar/zee çalıştır çalıştır --ordinary-application-argument
/exact/project/.zee-araclar/zee lsp
```

Development/dirty/unknown ikili explicit `--pilot` ister; stable dirty/unknown bu yolla kabul edilemez. Bu flag launcher kurulurken bilinçli sabitlenir; pinin kendisi her çağrıda yeniden okunur. Pin değişikliği/rollback mevcut `derleyici-secici.py pinle --degistir` komutudur. Launcher’da `seç`, `çalıştır`, `lsp` vardır; çalıştır tail’i mevcut scoped compiler komutlarına gider. Başka proje veya selector trust/store override tail’i kabul edilmez. LSP stdin/stdout korunur; normal child exit code’u döner.

## Güvenlik ve sahiplik

Import mevcut signedcatalog/verifier/privatebyte snapshot zincirini kullanır; ayrı crypto uygulaması yoktur. Exact yedi dosyanın her dirent gözlemi sayılan bounded inventory’si ve role limitleri doğrulanır; duplicate gözlem hemen reddedilir. Asıl bundle sonradan değişse de private snapshot byte’ları yayımlanır. Authenticated executable operatör yetkileriyle çalışır; sandbox değildir.

Store içinde exact SHA/platform hedefi same-filesystem private staging’den rename edilir. O_EXCL peridentity lock ve owned namespace target recheck eşzamanlı bu-protokol yazarlarını serialize eder; POSIX rename tek başına no-clobber değildir. Same byte existing hedef idempotent; farklı hedef overwrite edilmez. Acquired lock açık fd/inode kimliği korunur, cleanup yalnız path halen aynı dev/ino regular file ise unlink eder. Değiştirilmiş foreign lock silinmez. Stale lock otomatik PID/yaş ile sahiplenilmez.

Yayın sonrası parent fsync hatası exit2 ve `zee-local-publication-1`, `error=publication_failed`, `published=true` verir: hedef zaten görünürdür ve silinmez; durability doğrulanmamıştır. Yayın öncesi IO hatasında published=false olabilir. Retrying önce durumu inceleyerek aynı byte’lar için idempotent import yapabilir. File/fsync/rename fault cleanup yalnız edinilmiş stage/lock’a uygulanır. SIGKILL/güç kaybında mutlak cleanup/durability sözü yoktur.

Launcher `.zee-araclar` dizinini no-clobber yayımlar: executable `zee`, private `selector.py`, `receipt.json`. Existing/foreign hedef üzerine kurulmaz; değiştirme/kaldırma komutu bu dilimde yoktur. Receipt `zee-project-launcher-1` şeması, `zee-local-compiler-tools-posix-1` profili, uid ve managed file hashlerini taşır. Kurulum config’i trusted selector/project/store/catalog/key/verifier/Python bootstrap yollarını launcher içine sabitler; repo’dan alınmış script/policy kendiliğinden güvenilir olmaz. Operatör explicit kurulumu ve interpreter/runtime güveni sınırdır. Current kullanıcı kendi executable/policy’sini değiştirebildiği için bu mekanizma same-UID tamperproof root-of-trust değildir.

Her çağrıda selector bounded nofollow inode/stat eşlikli okunur, beklenen hash ile karşılaştırılır ve özel temporary kopyası izolasyonlu Python bootstrap’ta çalıştırılır. Selector’ın mevcut importları stdlib’dir; sibling proje modülü yüklenmez. Mevcut verifier/build/config probe ve safe_environment davranışı korunur. Tool/parse/IO retleri raw argv, kullanım metni veya özel dosya yolu basmaz; kullanıcı uygulamasının normal stdout/stderr’si aktarılır. Proje komutunun mevcut tanıları ayrı sözleşmedir.

Owned store dizinleri current UID ve group/world writable olmama koşuluyla kullanılır. Bu protokol kötü niyetli aynı UID namespace/stat restoration saldırısına karşı atomik filesystem snapshot veya OS sandbox sağlamaz. Dynamic dependency/PATH/runtime closure, Windows reparse ve native minOS kabulü ayrıca açıktır. Launcher genel process supervisor veya daemon değildir; existing seçicinin synchronous child/LSP ömrünü korur.

## Test ve açık kabul

```sh
python3 -I scripts/tests/derleyici-secici-test.py --compiler-dir compiler/target/debug
python3 -I scripts/tests/derleyici-yerel-test.py --compiler-dir compiler/target/debug
```

Existing25 crypto/privatebyte/pin/config davranışı değiştirilmeden korunur. Yeni14 vaka actual process publish yarışı, foreign lock replacement, pre/post publication IO arızası, source swap, exact inventory/owned path, launcher current pin/project scope, missing/corrupt bootstrap helper quiet exit2, duplicate-iterator bounded ret ve masked input retlerini kapsar. Gerçek dil+dillsp byte çiftine ephemeral test anahtarıyla signed fixture hazırlanır; LSP initialize/shutdown/exit yanıtları ve aynı actual BuildJSON doğrulanır. Gerçek proje komutunda bağımlılıksız eksik lock kabulü, bağımlı eksik lock P008, geçerli lock kabulü ve dependency kaynak değişince P008 ile pin/lock/app-source byte eşliği sınanır. Test anahtarı/sentetik sidecar gerçek publisher veya two-clean-clone release doğruluğunu kanıtlamaz.

Aday dış stage test kabulü root ortak Cargo/fullphase kabulü yerine geçmez. Native Windows/minOS, gerçek ürün adoption/önceki minor rollback, production keys, download/update ve global install/PATH açık kalır; P07 bütünü kapanmış sayılmaz.
