# K-299 — Kabul koşucularının iptal ve kapanış sınırı

16 Ekim 2026. **K-299 dar durum: dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli; GB-111:
kapalı; toplu commit öncesi son belge kapısı gerekli.** Kaynak `8178920e03fed782a817d1864b8280fcb80812a3`, tree `1eccf96d1a407880bf161c55bdd55c4d9a144866`.
temiz C2 native2 iki Cargo sahibiyle geçti: signed PDF9 ve service-fault2; alt vakalar Cargo owner sayısına eklenmez; C2 öncesi gate4: 13 adım geçti; tam faz 998/1023 geçti, 0 başarısız, 25 atlanan
Birleşik bağımsız inceleme C2 kaynak/gate4/security2/commit/native2 zincirini doğruladı; küçük public arşiv hashleri doğrulandı; toplu commit öncesi son belge kapısı ayrıca gerekli Kaynak öncesi stdlib ve kapı kanıtları ile
temiz C2 üzerinde çalışan native kabul aşağıda ayrı kayıtlıdır.
[Backlog](oncelikli-backlog.md), [GB-111](guvenlik-bulgulari-v1.tsv),
[ADR-141](../adr/141-imzali-pdf-stdio-host.md) ve
[ADR-150](../adr/150-native-worker-fault-ve-kontrollu-resolver.md) birlikte izler.

## Bulgu ve değişiklik

PDF native kabul betiğinde erken SIGTERM/INT, kendi ayrı oturumdaki çocuklarını
ve sentetik anahtarlarını finally temizliğine ulaştıramayabilirdi. Test betiği
artık ilk owned komuttan önce yalnız iptal bayrağı yazan handler kurar; temporary
ve owner kurulumu ile her iki keygen aynı try/finally kapsamındadır. Tamamlanmış
ve yarıda kesilmiş keygen dosyaları fresh0700 dizinin açık dirfd'sindeki iki exact
isimle temizlenir. Tamamlanmış dosyanın inode kimliği de korunur; key baytları
okunmaz, arşivlenmez veya hashlenmez. Başarısız temizlik passed makbuz üretemez.

Üretim PDF host zaten CANCELLED, handler ve finally kullanır. İç sahip kapanırken
dış test wrapper'ının bir saniyelik grace'i yetersiz kalabildiğinden yalnız
başarısız/iptal edilmiş host komutunun kapanış payı on saniyeye çıkarılır. Normal
komut ve üretim Supervisor davranışı korunur. Testin hata-yolu finally'si de
gerçek grace10 assertinden sonra temizlik için en az10 saniye ayırır; bu test
koruması yanlış üretim değerini yeşile çeviremez.

Service-fault kurucusu child başlatmaz. Yeni prepare aşaması ilk H.run/temiz
kaynak sorgusundan önce handler kurar; dış finally erken hatada da çalışır.
Borrowed fixture kurucusunun yarım temporary'si açıkça temizlenir. Cleanup
sırasında iptal bayrağı sıfırlanmaz veya SIGTERM yok sayılmaz; iptal edilmiş
koşu failed kalır. Sentinel/worker kapanışı ilk Popen handle'larıyla yapılır;
PID keşfi veya başka bir sürecin sahipliğini devralma yoktur.

Yerel test koşucusu ve sentetik materyal sınırı nedeniyle GB-111 düşük önemlidir.
İnceleme gerçek bir anahtar sızıntısı veya üretim istismarı gözlendiğini söylemez.
SIGKILL, powerloss veya arızalı dosya sisteminde finally garantisi yoktur.

## Gerçek kısa test kanıtı

İki doğrudan Python koşusu özgün dış sahibin exit0 ve closed/reaped makbuzuyla
geçti. PDF'nin ikinci koşusu test-finally temizleme payı düzeltmesini içerir;
ilk koşu dış kanıtta korunur. Bu alt vakalar gerçek compiler/PDF kampanyası
değildir: inert key dosyaları ve sonlu owned Python çocukları kullanılır.

| Hazırlık | Alt vaka | Sonuç | Özgün dış sahip |
| --- | ---: | --- | --- |
| PDF stdlib, ikinci koşu | 9 | 0 başarısız, OK | exit0; closed/reaped |
| Service-fault stdlib, ilk koşu | 10 | 0 başarısız, OK | exit0; closed/reaped |

Mevcut7+7 modele beş Python iç vaka eklendi; Cargo owner sayısı değişmedi.
PDF grubunun sahibi IO, service-fault grubunun sahibi web/security'dir. İki
normal Cargo wrapper ve iki explicit ignored native wrapper korunur; iç vaka
sayıları Cargo toplamına eklenmez. PDF wrapper'ın exact iç sayısı7→9 olur.
Mevcut dedicated native ek_kapsam ilişkileri değişmez.

Bu stdlib makbuzları kaynak SHA/cwd kaydı taşımaz; tek başlarına temiz commit
kabulü değildir. Aşağıdaki dosyalar gözlenen kaynak öncesi aday ve dış koşu
kaydını sabitler. Temiz C2 native kabulünün source/tree/BuildJSON/binary bağı
ayrı makbuzlarla kurulur: [C2 native9/2 makbuzu](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/native2-proof.json); [C2 birleşik bağımsız inceleme](incelemeler/urunlesme-20261014/k298-a57290f/reviews/k299/c2/combined-acceptance-review.json).

| Dış kayıt | Bayt | SHA-256 |
| --- | ---: | --- |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-pdf-unit-2.json` | 383 | `682124b43ef9b2b662ba1f1588ffda927d30354192d93636a3be3e0ada6354c5` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-pdf-unit-2.log` | 1369 | `1cbb2556822ca577742307cba55305f4a0d977514c7d86bdb5648485dd141867` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-service-unit-1.json` | 385 | `57ad0423a568eb07f18fba57317273ab90769e8aeb0f60f29294fb68af5ab420` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-service-unit-1.log` | 109 | `a26ea7f02bc540f9360da588a6b480824fc2bc6f9e04cf21b591c2c293ad6d74` |

Dış log/receipt arşivi bu sayfanın içinde değildir; özgün baytlar verilen
yollardadır. Aşağıdaki aday dosya hashleri belge hazırlığında salt okunur alınır.

| Aday dosya | SHA-256 |
| --- | --- |
| `scripts/tests/pdf-plugin-native.py` | `72449d567cae60541f01b3bef362ca8150d56ef7825308e9ae6b83bc4cad78ee` |
| `scripts/tests/service-fault-native.py` | `bb3b5a73abb8161eb17e860487bd763cecc3394136bf68edd5dcc27472dfd0fe` |
| `scripts/tests/pdf-plugin-test.py` | `3e0bb17f50b7ea979f5d78353c5c0169cb554f71e2f7ebfeca3f442aa16452a0` |
| `scripts/tests/service-fault-test.py` | `75b333fa4c6f6af3c42d234965a6db585d7239bba6d7e7318606f38d5b480fbe` |
| `compiler/tests/pdf_plugin_testi.rs` | `796c12e0908e678e4abae813081eeb70eedd1d0da14e61826d0a2ce09d64c18f` |

## İlk C native kabulünde kanonik depo yolu reddi

`90fe778e86b9de28046f0f7bd77b47df138a2212` kaynağında native1 kampanyasının
PDF Cargo seçicisi ve dokuz iç vakası geçti; bağımsız PDF incelemesi de üç
sayfa, gömülü font ve bir görseli doğruladı. Bu dar PDF sonucu birleşik
kampanyanın geçtiği anlamına gelmez. Service-fault Cargo seçicisi exit101 ile
kaldı; libtest bölümünün süresi0,04 saniye, Cargo komutunun toplam süresi
0,629197 saniyedir. Service iç proof dosyası oluşmadı; iki worker/fault vakası
çalıştırılmış veya başarısız worker vakaları olarak sayılmaz.

Rust wrapper'ın `CARGO_MANIFEST_DIR.join("..")` sonucu `compiler/..` bileşenini
taşıyordu. Python Campaign ilk kurulumda `p.resolve() == p` koşulunu zorunlu
tutar; wrapper'ın bu kanonik olmayan `--repo` değeri iç hazırlıktan önce
reddedilir. Bu bulgu üç saniyelik worker hazırlık sınırı veya timeout değildir.
Dar takip adayı `compiler/tests/service_fault_testi.rs` içindeki `root()`
yolunu kanonikleştirir; Python kabul sınırı gevşetilmez. `compiler/src`,
worker/host davranışı, Cargo owner sayısı ve native iki-vaka oracle'ı değişmez.

Top proof `failed`, özgün dış controller observed_exit=exit=2 ve reaped=true
olarak korunur. `source_end_clean=false`, başarı yolundaki son kaynak kontrolüne
ulaşılmadığını gösterir; tek başına depo kirlenmesi gözlemi değildir. PDF
başarısı C kimliğinde kalır; takip kaynağına devralınmaz. İlk996/2/25 ve
ikinci997/1/25 matris başarısızlıkları, sonra C adayında geçen998/0/25 ve
`--surekli` güvenlik kontrolü ayrı tarihsel kayıtlardır. Bu tarihsel sonuçlar takip kaynağına kendiliğinden aktarılmaz. Bu tarihsel başarısızlıkların ardından takip kaynağı C2
`8178920e03fed782a817d1864b8280fcb80812a3` için ayrı durum: dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli;
temiz C2 native2 iki Cargo sahibiyle geçti: signed PDF9 ve service-fault2; alt vakalar Cargo owner sayısına eklenmez; C2 öncesi gate4: 13 adım geçti; tam faz 998/1023 geçti, 0 başarısız, 25 atlanan.
GB-111: kapalı; toplu commit öncesi son belge kapısı gerekli; bağımsız inceleme Birleşik bağımsız inceleme C2 kaynak/gate4/security2/commit/native2 zincirini doğruladı; küçük public arşiv hashleri doğrulandı; toplu commit öncesi son belge kapısı ayrıca gerekli.

| Dış gözlem kaydı | Bayt | SHA-256 |
| --- | ---: | --- |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/PROOF.json` | 37858 | `3aaacce35041bc77132307459fe7ca01d2c5f3116998ed881888d998a9b2b7ae` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-controller-1.json` | 731 | `b623c6a5269d7b1e04d1ac5b28158e566e4fc1e0699440d71856b41ad93cf717` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/pdf-native/proof.json` | 3504 | `e7adea437a5671027c282fafee20db2151e46a3e6ef5d9cce653bce078660fa8` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/logs/service-fault-testi.stderr.raw` | 545 | `0616b7815322161d9cabd0b873eff1c0cda77827ff209cea6e9a810cfd2cbea7` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/logs/service-fault-testi.stdout.raw` | 152582 | `a54da2189781a5b362ef732ff72664c0bafad11d5b816fe903c8d73cc3729379` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/logs/pdf-independent-inspection.stdout.raw` | 490 | `d9a81a1961bd371451ffa74717982719ac76a23524799cbcb2087f98a9c07863` |

Ham loglar dış kanıtta kalır; bu bölüm anahtar baytı/hash’i veya büyük PDF/ikili kopyalamaz.

## Exact C2 kabul kaydı ve yürütme sınırı — dar kaynak/native/arşiv kapsamı kapalı; toplu commit öncesi son belge kapısı gerekli

Temiz kaynak C2 `8178920e03fed782a817d1864b8280fcb80812a3`, tree `1eccf96d1a407880bf161c55bdd55c4d9a144866` ve aynı SHA’dan
derlenen test ikilisi kullanılır; ikili yayımlanan sürüm artefaktı değildir.
Native signed PDF sonucu: temiz C2 üzerinde 9/9 iç vaka ve bağımsız PDF içerik/sayfa/font/görsel incelemesi geçti; native service-fault
sonucu: kanonik test-wrapper köküyle temiz C2 üzerinde 2/2 iç vaka geçti. Bu iki Cargo seçicinin iç9/2 vakası,
stdlib9/10 veya önceki K-298 extra2 ile toplanmaz.
Temiz C2 başlangıç/bitiş source/tree, CLI ve iç BuildJSON kimlikleri, 20 gerçek komut, proof dahil 78 public dosya/611082 bayt ve normal exit0/reap bağımsız olarak doğrulandı. İç anahtar temizliği ve worker/sentinel kapanışı hash ile bağlı koşucu kanıtı ve Cargo başarısına dayanır; gizli dizin/anahtar incelenmedi, iç stderr/zaman çizelgelerinin tamamı public kanıtta değildir. Dış 330 saniye iptal payı controller şemasından bağımsız kanıtlanmış sayılmaz Birleşik bağımsız inceleme C2 kaynak/gate4/security2/commit/native2 zincirini doğruladı; küçük public arşiv hashleri doğrulandı; toplu commit öncesi son belge kapısı ayrıca gerekli

Kaynak öncesi takip kapıları: Gate4 üzerinde fmt, depo sayıları, kanıt özeti, tam faz matrisi (998/1023 geçti, 0 başarısız, 25 atlanan), clippy, işlev eğilimi, spec-drift, katalog/tazelik, conformance/semantic-regresyon/core-freeze korukları, diff ve sır taraması olmak üzere 13 gerçek adım geçti; özgün controller normal exit0/reaped kapandı. Bu koşunun makbuzda kayıtlı kaynak tabanı üzerindeki
end_overlay/source pinleri C2 commit ağacıyla kaynak commit makbuzundaki exact 10 dosya blobu ve bağımsız inceleme ile eşlendi; gate4 ve security2 ilk C artı aynı aday farkında koştu.
Takip kapıları temiz C2 üzerinde yeniden çalıştırılmış diye yazılmaz. Güvenlik adına
gerçekte çalıştırılan kontroller: bash scripts/guvenlik-kapisi.sh --surekli gerçek koşusu geçti; compiler ve fuzz için 2 cargo-deny kontrolü, stable fuzz derlemesi ve 5+11=16 libtest başarılı, 0 başarısız/0 atlanan; özgün controller normal exit0/reaped. Bu kaynak öncesi kanıt aynı aday farkının C2 commit bağıyla kayıtlıdır.
Bundan `guvenlik-kapisi.sh --surum-adayi` veya C2 kaynaklı RC/soak/IDE sonucu
çıkarılmaz.

İlk son belge kapısı dokuz başarılı kontrolden sonra sır taramasında exit1 ile
başarısız oldu; `--surekli` adımı çalıştırılmadı. Özgün controller normal
exit1/reaped ile kapandı. İlk proof ve controller dış `p19-k298-k299-final-docs-1`
ve `p19-k298-k299-final-docs-controller-1.json` kayıtlarında korunur; bu sonuç
C2 native9/2 veya S kaynağının RC/RSS kabulünü yeniden sınıflandırmaz.
Bildirilen 25 eşleşmenin tamamı, 13 public JSON dosyasında üç belge sürümü, iki eski tarama günlüğü ve güvenlik kapısı betiğinin gerçek baytlarından yeniden hesaplanan altı SHA-256 özetine bağlandı. Bu eşleşmeler gizli kimlik bilgisi değildir; inceleme yalnız bildirilen eşleşmeleri sınıflandırır, tam bir bağımsız sır taraması yapmaz ve başarısız kapıyı geçmiş saymaz. Sınıflandırmanın dış kaydı `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-archive-false-positive-classification.json` yolundadır.

Son belge ve sürekli güvenlik kapısının ikinci gerçek koşusu, uygulanan
belge/şeffaf dış referans görünümünü ve kaynak pinlerini aynı 11 kontrolle
denetleyerek dış
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-final-docs-2/PROOF.json`
makbuzunu üretecek; bu kayıt ile
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-final-docs-controller-2.json`
üzerindeki özgün controller kapanışı doğrulanmadan toplu commit yapılmaz.
Son belge kapısının makbuzu arşiv üretiminden sonra dış kanıt köküne yazılır.
Bu arşiv kendi sonraki kapı makbuzunu veya onun hashini içermez; kapı makbuzu
hazırlanmış arşiv ve belge pinlerini denetler. Bu kapsam açıklaması kapının
sonucunu önceden başarılı ilan etmez.

İlk başarısız kapı1 **996/1023 geçti, 2 başarısız, 25 atlanan**, kapı2
**997/1023 geçti, 1 başarısız, 25 atlanan** olarak korunur. İlkinde iki
NotFound, ikincisinde nested Rust dogfood executable NotFound vardır.
K-299’a ait cache relocation üretim kaynak değişikliği değildir. v4 sürücüsü
miras alınan CARGO_TARGET_DIR değerini kaldırır; özgün v4 hazırlık kaydının
`prepared-not-executed` değeri geçmişte korunur. Yeni başarı önceki iki
failure kaydını değiştirmez; tekrarlar case sayılarına eklenmez.

| Yeni C2 kabul kaydı | Exact kaynak/bağ | SHA-256 |
| --- | --- | --- |
| Kaynak commit makbuzu | [C2 kaynak commit makbuzu](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/source-commit2.json); C2/tree | `cea3e9ebcddb5d75a9dfedaadb81e1dc5c02a2d24754f1d9cd0e6bd25bd8dc3a` |
| Kaynak öncesi takip kapıları | [Gate4 makbuzu](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/gates4-proof.json); overlay→C2 | `0fdf7dc22b5eec715b83104f7d82bdcfcd19881d615f543d8088191e562bc964` |
| Takip kapıları özgün controller | [Gate4 özgün controller](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/gates4-controller.json); exit0/reaped | `be2eacc748cd9da7cdb41440003a447e3c753980dd15e97fa009c3217a44b72e` |
| C2 native9/2 proof | [C2 native9/2 makbuzu](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/native2-proof.json); C2/tree/BuildJSON | `bfca13b42cabd6ff44e4b18b1888b96234644a6001b8bb871116943c1eb148e5` |
| C2 native özgün controller | [Native2 özgün controller](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/native2-controller.json); exit0/reaped | `8bc6ed00260829577ba6a414adb0039f88c258855d8229b8d2c311255caff17c` |
| Bağımsız inceleme | [C2 birleşik bağımsız inceleme](incelemeler/urunlesme-20261014/k298-a57290f/reviews/k299/c2/combined-acceptance-review.json) | `63a12aa14770c89352ac7afc11aae2a6f3662cc6fe70c3157ba037afd69f070b` |
| Güvenlik kontrolleri | [Security2 makbuzu](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/security2-proof.json) | `6e15fcab9834c13015ea306478699982c548da753cf540e7694112ca701abd56` |
| Güvenlik özgün controller | [Security2 özgün controller](incelemeler/urunlesme-20261014/k298-a57290f/receipts/k299/c2/security2-controller.json); exit0/reaped | `ab89441f37a5ba88f98051116ef17566f8b8b6a987f941369e3845c5c4080fef` |
| Önceki iki failed kapı ve relocation | [Başarısız denemelere bağlı görünüm manifesti](incelemeler/urunlesme-20261014/k298-a57290f/CURRENT-ARCHIVE-MANIFEST.json) | `aafb1ac5dfbe2c552f4beab0a5c7dd6245d4b21e3e4403229a1adae0f26edd2f` |

Depodaki küçük kanıt görünümü kopyalanan public kayıtlarla açık dış referans
kayıtlarını ayırır. Bu görünümde 120 dosya özgün baytlarıyla, 13 dosya okunabilir dış referans olarak bulunur; README, RAW-SCOPE, SELECTION ve manifest yeniden üretilmiştir. Her dış referans, korunmuş özgün dosyanın açık yolunu, SHA-256 değerini ve bayt sayısını verir. Özgün 133 kayıt ve 270 hash bağının tümü bu görünümün kendi içinde bulunmaz; bu bağlar dış özgün seçki üzerinden doğrulanır. Tarayıcı kuralı veya ignore istisnası eklenmez, özgün içerik kodlanarak ya da gizlenerek temsil edilmez.
Tablodaki makbuz SHA-256 değerleri özgün dosya baytlarına aittir; dış referans
kaydı kendi manifestindeki ayrı hash'iyle izlenir. Özgün küçük seçki
`/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k298-k299-original-small-archive` altında korunur; depodaki görünümün
manifesti bu seçkinin `6e13b044becd1aee8c4d021bc47e3141c132b295c9e3aec8f6a2679016a1c265`
hash'iyle karıştırılmaz. Büyük binary/PDF/Cargo logları dış kapsam kaydıyla
kalır; ephemeral anahtar dosyası, anahtar baytı veya anahtar hash’i okunmaz ya
da arşivlenmez. [Küçük görünüm ve dışta kalan ham kanıt kapsamı](incelemeler/urunlesme-20261014/k298-a57290f/RAW-SCOPE.json)

Nested çocukları kapatan kabul betiğinin dış sahibi yeterli cooperative kapanış
payı tanımalıdır. İncelenen bütçe en az45 saniye dış TERM grace ayırır; mevcut
doğrudan watchdog330 saniye pay taşır. İç PDF başarısız host için10 saniye;
service normal çalışma için120 saniye cooperative bütçe kullanır. Çalışma
timeout'u ile kapanış payı ayrı tutulur; araya kısa grace'li bir H.run wrapper'ı
eklenmez. Bunlar dosya sistemi/OS gecikmesine karşı gerçek zaman SLA'sı değildir.

Üretim compiler/grammar/API, pdf-plugin-host, worker-supervisor, lifecycle_core,
imza/trust/protokol, native DNS ve kalıcı cache değişmedi. Yeni compiler
regression/provenance satırı veya uyumluluk yüzeyi eklenmez. Spec/RFC normu,
tanı kataloğu, Rust bağımlılık grafiği, LSP iş yükü ve performans geçmişi bilinçli
değişmez. Bu iş yalnız mevcut kabul araçlarının lifecycle correctness dilimidir.

K-298 kaynak `a57290f7a1c1fad07cda3f266b2a221f09aa6404` üzerindeki
RC/soak/IDE/stdio kanıtları kendi kaynak kimliğinde kalır ve K-299 HEAD üzerinde
çalıştırılmış sayılmaz. S3 native9/2 tarihsel kaynağında korunur. Genel FFI/PDF
sandbox/native DNS/fault matrisi, ürün/insan/fiziksel platform/yayın kapıları ve
C1/K-288 ile P33/K-293 açık sınırları bu kayıtla kapanmaz.
