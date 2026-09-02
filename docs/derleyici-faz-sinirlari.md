# Derleyici faz ve handler sınırları

Bu rehber ADR-012'nin uygulama karşılığıdır. Amaç dosyaları küçük göstermek
değil; bir değişikliğin parser, checker ve runtime'da hangi sorumluluğa ait
olduğunu açık tutmaktır.

## Dizin yapısı

```text
compiler/src/
├── faz.rs                    source→token→parsed→bound program tipleri
├── kimlik.rs                 YapiId/IslemId/SymbolId newtype'ları
├── ayristirici.rs            token/blok/tanım orkestrasyonu
│   └── ayristirici/
│       ├── cumle.rs          cümle son-yüklem dağıtımı
│       ├── ifade.rs          RFC-0021 ifade katmanları ve çağrı lowering'i
│       ├── kaynak.rs         token bölgesi→kesin AST kaynak zarfı
│       └── kurtarma.rs       cümle/girinti senkronizasyonu ve tanı bütçesi
├── cozumleyici.rs            checker geçiş orkestrasyonu ve public API
│   └── cozumleyici/
│       ├── akis.rs           daraltma, gezme ve görev akışı
│       ├── baglam.rs         denetim durumu ve semantic ID dizinleri
│       ├── cagri.rs          işlem imzası ve çağrı uzlaştırması
│       ├── cumle.rs          cümle denetimi
│       ├── donus.rs          kesin sonlanma ve dönüş birleşimi
│       ├── etki.rs           web/uygulama etkisi ve eylem değişmezleri
│       ├── ifade.rs          ifade tür denetimi
│       ├── kaynak.rs         ifade tanısını kesin AST/HIR aralığına bağlama
│       ├── sembol.rs         ad, alan ve sözcüksel kapsam çözümü
│       ├── sozlesme.rs       public parametre/dönüş sözleşmesi
│       ├── turler.rs         tür modeli, tür yazımı ve uzlaşma
│       └── yetkinlik.rs      compile dış dünya/politika kapısı
├── kalici_dosya.rs           kilit/temp/sync/replace orkestrasyonu
│   └── kalici_dosya/
│       └── metadata.rs       platform owner/ACL/xattr/security aktarımı
├── kaynak_sinirlari.rs       ortak değişmez kaynak/CPU/bellek/çıktı profili
├── paket.rs                  yerel+uzak tek proje grafiği ve kilit orkestrasyonu
│   ├── paket/uzak.rs         exact registry çözümü ve kilit kimliği
│   │   └── uzak/politika.rs  yanked/kritik gerekçe olay kimliği
│   └── paket/uzak_wasm.rs    ağsız WASM fail-closed adaptörü
├── tedarik.rs                yayın üretimi/doğrulama orkestrasyonu
│   └── tedarik/kurulum.rs    doğrulanmış `.zep` atomik kaynak kurulumu
├── cli/registry.rs           açık ağ kullanan exact paket komutları
├── registry.rs               root ve metadata güven doğrulayıcısı
│   └── registry/istemci.rs   taşıma/cache/kalıcı durum orkestrasyonu
│       ├── depo.rs           içerik-adresli yol ve sınırlı disk okuma
│       └── tasima.rs         HTTPS/statik ayna adaptörü
└── yorumlayici.rs            değer/IO/scheduler ve yürütme orkestrasyonu
    └── yorumlayici/
        ├── cumle.rs          cümle yürütme
        ├── ifade.rs          ifade değerlendirme
        ├── hir_gecisi.rs     bağlı typed-HIR / raw uyumluluk geçişi
        ├── io_izi.rs         sürümlü bütün-IO kayıt ve dış etkisiz replay
        ├── io_profili.rs     `zee-io-1` tohum ve aralık algoritması
        └── yetkinlik.rs      genel runtime IO policy sarmalayıcısı
```

Alt modüller yalnız üst fazına `pub(super)` görünür. Dil kütüphanesinin public
API'si bu iç ayrımla büyümez.

## Değişiklik yönlendirmesi

| Değişiklik | İlk sahibi | Birlikte kontrol edilecek sınır |
|---|---|---|
| yeni cümle son-yüklemi | parser `cumle` | checker/runtime `cumle`, RFC/spec |
| yeni ifade/postfix | parser `ifade` | RFC-0021 çakışma matrisi, checker/runtime `ifade` |
| parser hata kurtarma | parser `kurtarma` | RFC-0010, kısmi AST invariantı ve LSP tanı sırası |
| AST ifade kaynak aralığı | parser `kaynak` | ADR-030, HIR aralık eşliği, tanı ve LSP tüketimi |
| işlem çağrı uzlaştırması | checker `cagri` | RFC-0006/spec-02/10 ve usability kararı |
| public işlem sözleşmesi | checker `sozlesme` | paket/birim API ve semver sınırı |
| tür yazımı/uzlaşması | checker `turler` | ifade handler'ı ve olumsuz test |
| ifade tanı konumu | checker `kaynak` | AST/HIR kesin aralığı ve tanı regresyonu |
| sembol/alan çözümü | checker `sembol` | `SymbolId`, morfoloji ve kapsam testleri |
| yapı/işlem semantic bağı | `kimlik` + checker `baglam` | ADR-014, AST bağ alanları ve indeks-gerileme testi |
| akış/daraltma | checker `akis` | cümle handler'ı ve flow testleri |
| dönüş/control-flow | checker `donus` | Seçenek/Sonuç ve tüm-yollar kanıtı |
| web/uygulama etkisi | checker `etki` | ADR-011 ve eylem/web kuralları |
| dış dünya yetkinliği | checker `yetkinlik` | RFC-0024/spec-23, proje policy ve T054 |
| yürütme semantiği | runtime `cumle` veya `ifade` | ADR-003, değerlendirme sırası testi |
| IO kayıt/replay protokolü | runtime `io_izi` | RFC-0022, ADR-026, spec/21 ve şema snapshot'ı |
| tohum/rastgele profil semantiği | runtime `io_profili` | RFC-0023, ADR-027, spec/22 ve dizi snapshot'ı |
| runtime dış dünya kapısı | runtime `yetkinlik` + kök `yetkinlik.rs` | RFC-0024, ADR-031, spec/23 |
| atomik dosya metadata'sı | `kalici_dosya/metadata` | RFC-0016, ADR-032, spec/08 ve Tier-1 testleri |
| exact registry proje çözümü | `paket/uzak` | RFC-0020, ADR-006, spec/07/19 ve kilit v3 |
| doğrulanmış `.zep` kurulumu | `tedarik/kurulum` | arşiv exact ağaç doğrulaması, atomik rename ve P016 |
| registry CLI/ağ açma sınırı | `cli/registry` | `ekle/kilitle/paketler`, çevrimdışı varsayımlar ve P017 |
| morfoloji profil uyumluluğu | `morfoloji/uyumluluk` | RFC-0018, spec/13, immutable SHA-256 fixture ve Git-tarih koruğu |
| alan adaptörü | intrinsic kaydı | ADR-011 rehberi, yetkinlik/etki/runtime |

## Büyüme bütçesi

`compiler/tests/mimari_sinir_testi.rs`, köklerin büyük handler'ları geri
yutmadığını ve her faz dosyasının ilan edilmiş satır bütçesini aşmadığını
denetler. Bütçe dolduğunda sayı yükseltilmez; ortak davranış çıkarılır veya
yeni, adı sorumluluğunu anlatan handler modülü açılır. Bütçe değişikliği ancak
ADR-012 gerekçesi ve bu rehber aynı committe güncellenirse kabul edilir.

`katalog_testi.rs` sabit bir kök dosya listesi kullanmaz; `compiler/src`
altındaki bütün Rust dosyalarını özyinelemeli ve sıralı tarar. Yeni handler'da
üretilen bir tanı kodu katalog denetiminden kaçamaz.

Bu test semantik kaliteyi tek başına kanıtlamaz. Tam test paketi, Clippy, WASM
derlemesi ve ilgili RFC/spec conformance'ı her değişiklikte yine zorunludur.

Checker içindeki semantik bağımlılık yönü ve tek sahiplik kuralları ayrıca
ADR-013 ile [checker katman rehberinde](checker-katmanlari.md) bağlayıcıdır.
K-101 ile `baglam`, `sembol` ve `cagri` bütçeleri semantic ID kayıtlarını
taşıyacak kadar gerekçeli biçimde genişletildi; çıplak indeks geri dönüşü ayrı
mimari testle engellenir. Kimlik kuralları ADR-014 ve
[semantic kimlik rehberinde](semantic-kimlik-modeli.md) bağlayıcıdır.
K-102/ADR-015 veri fazlarını [ayrı tiplerde](derleyici-faz-modeli.md) bağladı;
`faz.rs` bütçesi ve standart-hat testi parsed AST'nin yürütülebilir program
gibi kullanılmasını engeller.
K-113/ADR-024 cümle ve dengeli girinti senkronizasyonunu ayrı `kurtarma`
modülüne taşıdı; 160 satır bütçesi parser kökünün recovery ayrıntılarını geri
yutmasını engeller.
K-103/ADR-016 checker'ın tür ve semantic bağ çıktısını
[typed HIR çekirdeğine](typed-hir-modeli.md) taşıdı; `hir.rs` ayrı 180 satır
bütçesine sahiptir ve bağlı programın HIR'sız kurulması mimari testte durur.
K-104 HIR/raw uyumluluk ayrımını `yorumlayici/hir_gecisi.rs` sahibine taşıdı;
140 satır bütçesi ve standard-hat testi runtime kökünün yeniden şişmesini ya
da bağlı yürütmenin kaynak adına geri düşmesini engeller.
K-115/RFC-0022 bütün `GirdiCikti` iz şemasını ve iki sarmalayıcıyı
`yorumlayici/io_izi.rs` sahibine ayırdı. 1.250 satır bütçesi runtime kökünün
protokol ayrıntılarını geri yutmasını engeller; biçim/işlem şeması değişikliği
ADR-026 ve spec/21 ile aynı atomik değişiklikte ele alınır.
K-116/RFC-0023 CLI ve playground rastgeleliğini `yorumlayici/io_profili.rs`
sahibinde birleştirdi. 80 satır bütçesi `zee-io-1` algoritmasını runtime
kökünden ayırır; dizi snapshot'ını kıran değişiklik ADR-027/spec-22 ve yeni
profil kimliği olmadan yapılamaz.
K-122/RFC-0018 `zee-tr-1` semantic parmak izi üretimini
`morfoloji/uyumluluk.rs` sahibine ayırdı. 140 satır bütçesi kanonik akışı
profil kökünden uzak tutar; immutable fixture ile Git-tarih koruğu aynı profil
kimliği altında davranış değişikliğini kapatır. K-123 bu koruğu kök
`conformance/morfoloji/zee-tr-N.json` ve `sema-vN.schema.json` artefaktlarına
genişletti; Rust testi yalnız yayımlanmış, derleyiciden bağımsız sözleşmenin
çalışan motorla uyuşmasını denetler.
K-126/ADR-030 ifade kaynaklandırmasını `ayristirici/kaynak.rs`, checker tanı
yükseltmesini `cozumleyici/kaynak.rs` sahibine ayırdı. 80 ve 40 satırlık
bütçeler kesin kaynak politikasının büyük ifade handler'larına geri
dağılmasını engeller.
K-127/ADR-031 compile yetkinlik taramasını `cozumleyici/yetkinlik.rs`, runtime
IO kapısını `yorumlayici/yetkinlik.rs`, ortak policy/origin/IP modelini kök
`yetkinlik.rs` ve native HTTPS'yi `ag_istemcisi.rs` sahibine ayırdı. İlanlı
380/250/520/180 satır bütçeleri etki, runtime ve CLI köklerinin bu güvenlik
sorumluluğunu geri yutmasını engeller.
K-128/ADR-032 atomik replace'in platform metadata aktarımını
`kalici_dosya/metadata.rs` sahibine ayırdı. `kalici_dosya.rs` K-135'in atomik
karşılaştır-ve-yaz ilkeliyle 850,
metadata adaptörü 260 satır bütçesindedir; owner/group, ACL/xattr ve Windows
security merge ayrıntıları genel kalıcılık akışına geri yayılamaz.
K-129/ADR-033 kaynak bütçesi değerlerini `kaynak_sinirlari.rs` içinde tek
sahipli yaptı. K-131 bounded reader'ı `kaynak_sinirlari/okuma.rs`, domain
görünümlerini `kaynak_sinirlari/profiller.rs` sahibine ayırdı. Kök/profil/okuma
sırasıyla 260/300/120 satır bütçesindedir; lexer/runtime/LSP/CLI yalnız bu
değeri tüketir, yeni dağınık limit sabiti ekleyemez.
K-132 LSP JSON üretimini `lsp/cikti.rs` sahibine ayırdı. Bu modül 120 satır,
LSP kökü 1.500 satır bütçesindedir; yanıt kaçışı ve boyut muhasebesi yeniden
semantic handler'lara dağılamaz.
K-134 rota seçimi, güvenlik önsözü, taze ortam ve 30 saniyelik request yürütme
akışını `yorumlayici/web_istek.rs` sahibine ayırdı. 180 satır bütçesi web
yaşam döngüsünün runtime köküne geri gömülmesini engeller; transaction
commit/rollback'i `GirdiCikti` adaptör sınırında kalır.
K-135 registry istemcisini, K-136'nın exact çıktı kimliği ekleriyle 600 satırlık
`registry/istemci.rs` sahibinde;
içerik-adresli disk ilkellerini 100 satırlık `depo.rs`, HTTPS statik taşıyıcıyı
120 satırlık `tasima.rs` sınırında tutar. Metadata doğrulama `registry.rs`te,
DNS/IP ve redirect korkulukları ortak `ag_istemcisi.rs`te kalır.
K-136 exact proje/cache çözümünü 480 satırlık `paket/uzak.rs`, güvenlik
baypasını güncel yanked/kritik olay kimliğine bağlayan kilit okuyucusunu 180
satırlık `paket/uzak/politika.rs`, doğrulanmış
arşiv açmayı 220 satırlık `tedarik/kurulum.rs` ve açık ağ kullanan komut
yüzeyini 500 satırlık `cli/registry.rs` sahibine ayırır. Normal graph/derleme
bu CLI modülünü çağırmaz; sessiz ağ açmama değişmezi fiziksel sınırdır.
WASM'de native registry/tedarik modülü derlenmez; 140 satır bütçeli
`paket/uzak_wasm.rs` yerel grafiği korur ve uzak bildirimde P016 ile fail-closed
kalır.
K-130 değer grafiği hesabını `yorumlayici/kaynak.rs`, bütçeli değer/JSON/CSV
yazımını `yorumlayici/metin.rs`, süreç-geneli izin sayacını
`kaynak_sinirlari/baglanti.rs` sahibine ayırdı. Sırasıyla 280/240/60 satır;
runtime cümle/ifade handler'ları yalnız 620/760 satır bütçesi taşır.
