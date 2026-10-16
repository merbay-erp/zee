# K-299 — Kabul koşucularının iptal ve kapanış sınırı

16 Ekim 2026. **AÇIK: K-299 kaynak `90fe778e86b9de28046f0f7bd77b47df138a2212` üzerinde
ilk birleşik native kabul başarısızdır. PDF9 ve bağımsız PDF incelemesi ayrı
geçti; service-fault, iç kampanya başlamadan kanonik depo yolu kontrolünde
reddedildi.** Rust test wrapper kökünün kanonikleştirilmesi takip adayındadır;
yeni kaynak kimliği ve bu kaynakta ortak kapılar/native yeniden kabul henüz
yoktur. GB-111 düşük/açık kalır. [Backlog](oncelikli-backlog.md),
[GB-111](guvenlik-bulgulari-v1.tsv), [ADR-141](../adr/141-imzali-pdf-stdio-host.md)
ve [ADR-150](../adr/150-native-worker-fault-ve-kontrollu-resolver.md) birlikte izler.

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

Makbuzlar kaynak SHA/cwd kaydı taşımaz; tek başlarına temiz commit kabulü
değildir. Aşağıdaki dosyalar gözlenen yerel aday ve dış koşu kaydını sabitler;
yeni kaynak kabulünde source/tree/BuildJSON ve binary bağları ayrıca kurulacaktır.

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
`--surekli` güvenlik kontrolü ayrı tarihsel kayıtlardır. Bunlar henüz olmayan
takip kaynağında kapı veya native kabul sayılmaz. GB-111 kapanışı yeni temiz
source/tree, gerekli ortak kapılar ve native9/2 kabul zincirini bekler.

| Dış gözlem kaydı | Bayt | SHA-256 |
| --- | ---: | --- |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/PROOF.json` | 37858 | `3aaacce35041bc77132307459fe7ca01d2c5f3116998ed881888d998a9b2b7ae` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-controller-1.json` | 731 | `b623c6a5269d7b1e04d1ac5b28158e566e4fc1e0699440d71856b41ad93cf717` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/pdf-native/proof.json` | 3504 | `e7adea437a5671027c282fafee20db2151e46a3e6ef5d9cce653bce078660fa8` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/logs/service-fault-testi.stderr.raw` | 545 | `0616b7815322161d9cabd0b873eff1c0cda77827ff209cea6e9a810cfd2cbea7` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/logs/service-fault-testi.stdout.raw` | 152582 | `a54da2189781a5b362ef732ff72664c0bafad11d5b816fe903c8d73cc3729379` |
| `/Users/mustafaerbay/.codex/zee-evidence/2026-10-14-urunlesme/p19-k299-native-1/logs/pdf-independent-inspection.stdout.raw` | 490 | `d9a81a1961bd371451ffa74717982719ac76a23524799cbcb2087f98a9c07863` |

Ham loglar dış kanıtta kalır; bu bölüm anahtar baytı/hash’i veya büyük PDF/ikili kopyalamaz.

## Açık yeniden kabul ve yürütme sınırı

Kaynak ve belge kapıları geçtikten sonra yeni exact temiz SHA/tree ve aynı SHA'dan
derlenen test ikilisiyle mevcut signed PDF9 ve native service2 ayrı yürütülecek.
Bu ikili yayımlanan sürüm artefaktı değildir. Gerçek native
oracles ve vaka sayıları değişmedi. Kaynak/helper/binary başlangıç-son hashleri,
BuildJSON, failed/cancelled/cleanup alanları ve özgün outer-owner kapanışı
birlikte kaydedilmeden K-299/GB-111 kapatılmaz. Tam faz, Cargo wrapper, katalog,
doküman tazeliği, güvenlik ve biçim kapılarının sonucu bu kısa testlerden çıkmaz.

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
