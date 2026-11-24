# ADR-169 — DNS stub çözücü, loopback AD biti ve kendi tel biçimi sınırı

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-390, K-383/RFC-0044 Aşama B1 (B1a), K-355 (`urun-h`),
  ZM-R006, ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-011 (intrinsic
  sınırı), ADR-038 (tedarik zinciri), ADR-022/ADR-056 (fuzz ve RC kapısı),
  K-362/GB-109 (bağımsız kâhin emsali), K-375/RFC-0042 (son tarih modeli),
  K-173/ADR-069 (soak), spec/31, spec/23, spec/21, GB-164

## Bağlam

Etkin dogfood ürünü `urun-h` (ZeeMail) internete açık posta sunucusu olmak
için DNS'e muhtaç: uzak teslimde MX seçimi ve null MX, gelen postada SPF,
DKIM anahtarı, DMARC kaydı, DANE için TLSA. Bugün Zee kodu yalnız sistem
çözücüsüyle dolaylı A/AAAA çözümü görüyor; ürün motoru DNS yanıtlarını
dışarıdan verilen bir "defter" sözlüğüyle alıyor ve canlıda defter boş.
Ürün reproducer'ı ZM-R006 (`"…" için MX kayıtlarını sorgulamayı dene`)
bugün S015 veriyor. 24 Kasım 2026 merkez kararı ("Sırayla aç") RFC-0044 B1
ve B2'yi ZeeMail'in sıradaki adımları yaptı; B1 bu ADR'dir.

RFC-0044 B1 çözücü modeli olarak "tel biçimi + dar stub istemci →
bildirilmiş loopback doğrulayıcı çözücü"yü seçmiş, tel biçimi için yeni bir
crate önermişti (`hickory-proto` ya da `domain`). Uygulama günü ön
değerlendirmesi (cargo-deny, RustSec, ağaç):

| Aday | Yeni paket | Lisans | `unsafe` | RustSec |
|---|---|---|---|---|
| `hickory-proto 0.26.3` (`std`) | +46 (idna + icu4x zinciri, url, tracing, thiserror, jni…) | MIT/Apache | yok | 2026'da 13 kayıt; çoğu DNSSEC doğrulamada, tel biçiminde O(n²) sıkıştırma ve başlık sayacı bellek amplifikasyonu (≥ 0.26.2 yamalı) |
| `domain 0.12.3` (`std`) | +10 (octseq, hashbrown, jiff…) | BSD-3 | 50 dosyada | RUSTSEC-2026-0310: toplu panik/soundness/kaynak tükenmesi (≥ 0.12.3 yamalı) |

## Karar

1. **Üretimde yeni bağımlılık yok.** DNS tel biçimi `dil::dns_teli`
   saf modülünde (`#![forbid(unsafe_code)]`, wasm32 dahil derlenir) yalnız
   ihtiyaç kadar yazılır: tek soruluk sorgu (RD=1, AD=1 — RFC 6840 §5.7,
   EDNS0 1232 bayt, DO=0) ve yanıt ayrıştırma. İki crate de üretim için
   reddedildi: `hickory-proto` +46 paket ve derleyici ikilisine Unicode
   normalizasyon zinciri getirir (yolumuz U-label'ı zaten reddeder);
   `domain` `unsafe` yoğun ve Eylül 2026 toplu soundness kaydı taşır.
   Kriptografi yazılmaz ilkesi (RFC-0039 ilke 2) değişmez: DNS tel biçimi
   kriptografi değildir, sınırlı bir ikili ayrıştırıcıdır; K-293/K-362 HTTP
   ayrıştırıcısının emsalidir.
2. **Dar kabul kümesi.** QR=1, OPCODE=0, tek soru; cevap ≤ 64, dört bölüm
   toplamı ≤ 128 kayıt; dört bölümün hepsi yapısal doğrulanır, sondaki fazla
   bayt reddedilir. Tanınan türler A, NS, CNAME, SOA, MX, TXT, AAAA, DNAME,
   OPT (yalnız ek bölümde, kök adlı, tek; NSID/ECS seçeneği reddedilir),
   TLSA; başka tür fail-closed `bozuk_yanıt`tır. Sıkıştırma işaretçisi
   yalnız bu adın başladığı segmentten geriye ve ad başına ≤ 16 kez izlenir
   (döngü yapısal olarak imkânsız); tel adı ≤ 255 bayt. rdata uzunluğu
   içerikle exact eşleşmeli; TXT ≤ 4 KiB birleştirilmiş, TLSA verisi ≤ 4 KiB.
   CNAME zinciri ≤ 8 (döngü dahil aşım `boyut_aşımı`).
3. **Bağımsız kâhin yalnız testte.** `hickory-proto =0.26.3` dev-dependency
   ve fuzz bağımlılığıdır. Sözleşme tek yönlüdür: Zee'nin kabul ettiği her
   paketi hickory de kabul eder ve cevap bölümünü (ad, tür, sınıf, TTL,
   rdata) aynı okur (Zee ⊆ hickory). Bilinçli katı farklar: bilinmeyen tür,
   sondaki bayt, işaretçi kuralı, NSID/ECS, boş TXT. `compiler/tests/ortak/
   dns_kahini.rs` test ve fuzz'da ortaktır; ana test 40.000 deterministik
   bozulmuş paket ve kalıcı korpusu oynatır. Yeni `dns_yaniti` fuzz hedefi
   günlük smoke'a ve RC kapısına (`fuzz-rc.yml`, `rc-kampanyasi.py`,
   `rc-kaniti.py`, `guvenlik-kapisi.sh`) girer; sonraki RC adayı beş hedef
   ister.
4. **Çözücü modeli: yalnız loopback, önbelleksiz stub.** `proje.dil`
   `yetkinlikler "dns"` + `dns_çözücü "127.0.0.1:53"` (ya da `[::1]`) +
   `dns_dnssec "zorunlu"|"bilgi"` üçü birlikte bildirilir (P015; varsayılan
   yok). Loopback dışı adres reddedilir: uzak çözücünün AD biti güvenilmez
   (RFC 4035 §4.9.3, RFC 6698 §4.1 — Postfix `dnssec` ve Exim DANE modeli).
   Her sorgu `getrandom` 16 bit kimlik ve işletim sisteminin rastgele geçici
   kapısıyla, çözücüye `connect` edilmiş soketten gider. Önce başlık+soru
   eşleşmesi (kimlik, QR, ad, tür, sınıf) aranır; eşleşmeyen her paket
   atılır ve bekleme son tarihe kadar sürer, eşleşen ama gövdesi bozuk ya
   da sınır dışı paket hatadır. `TC=1` aynı sorguyu TCP ile (≤ 64 KiB)
   yineler; TCP yanıtı da kesikse hata. Süreç içi önbellek yoktur (RFC-0044
   ilke 7): önbellek ve zehirleme direnci yerel çözücünündür; K-173 soak
   koşusu gerekmez.
5. **AD biti yalnız `zorunlu` kipte anlamlı.** `zorunlu`: açılışta kök
   `SOA` sorgusu `AD=1` dönmezse süreç kaynak yürütülmeden çıkış 2 ile
   durur (`dns_dnssec zorunlu:` iletisi). Satırdaki `doğrulanmış` = AD ∧
   `zorunlu`; `bilgi` kipinde hep `yanlış`. Bu B4 DANE'nin önkoşuludur;
   B1'de hiçbir güvenlik kararı DNSSEC'e bağlanmaz. Çözücü önerisi ürün
   görüşüyle unbound (systemd-resolved `allow-downgrade` DANE'ye uygun
   değil); ZeeOS paketlemesi ZeeOS kararıdır.
6. **Zee yüzeyi ürün reproducer'ının biçimidir.** `AD için MX|TXT|TLSA|adres
   kayıtlarını sorgulamayı dene` → `Sonuç<Metin sözlük listesi>`; kayıt
   başına bir satır: ortak `ad`, `ttl`, `doğrulanmış` ("doğru"/"yanlış");
   MX `öncelik`, `hedef` (öncelik + ad sırası; null MX `.`); TXT `metin`
   (parçalar birleşik, UTF-8 olmayan bayt U+FFFD); `adres` A sonra AAAA,
   `aile` (`ipv4`/`ipv6`) + kanonik `adres`; TLSA `kullanım`, `seçici`,
   `eşleme`, `veri` (küçük onaltılık). Kayıtlar deterministik sıralı ve
   tekilleştirilmiştir. RFC taslağındaki `alanın … kayıtlarını` fikri ürünün
   gerçek biçimiyle değiştirildi; protokol adları çevrilmez ve büyük
   harftir. Yeni tür açılmaz (kapalı intrinsic tür kümesi).
7. **Kalıcı yanıt başarı, geçici hata Sonuç.** NXDOMAIN ve NODATA boş
   listedir (RFC 7208/7489 için ikisi de "kayıt yok"). Ayrımları ve olumsuz
   yanıtın doğrulanmışlığı host iç API'sinde (`Cozum::durum`) ve IO izinde
   taşınır; Zee yüzeyinde B4 (MX örtük A, DANE) ihtiyacıyla açılır.
   Başarısız Sonuç yalnız geçici hatadır: yeni çalışma zamanı kodu **C033**,
   Hata verisi `tür` ∈ `zaman_aşımı`, `sunucu_hatası` (SERVFAIL — DNSSEC
   `bogus` dahil —, FORMERR, NOTIMP, diğer), `ret` (REFUSED), `ağ`,
   `boyut_aşımı`, `bütçe` (B2 değerlendirici bütçeleri için ayrılmış),
   `geçersiz_ad` (ASCII LDH + `_`, etiket 1–63, ≤ 253 oktet; tel biçimine
   hiç girmez), `bozuk_yanıt`. SERVFAIL hiçbir yolda boş listeye dönüşmez.
8. **Son tarih (K-375 modeli).** Sorgu başına 5 sn; etkin son tarihin
   kalanı daha kısaysa o. Çağrıdan önce ve sonra son tarih denetlenir;
   dolmuşsa `yetişmezse` kolu çalışır (Ç001), adaptöre eski süre gitmez.
   Görev içinde sorgudan önce scheduler'a sıra verilir. IO izi süreyi
   kaydetmez.
9. **Yetkinlik.** Yeni `dns` yetkinliği (`ağ`dan ayrı: yalnız bildirilmiş
   loopback çözücüye 53'e gider, outbound origin listesi açmaz). Bildirimsiz
   geliştirici akışında ve `--güvenli` çocuk modunda kapalıdır. Etki
   `DisOkuma`dır (eylem içinde serbest).
10. **Deterministik IO.** Yeni IO izi işlemi `dns_sorgula`: argüman
    `[ad, tür]`, sonuç `ok|hata` şemalı; replay ağa gitmez. Hermetik
    adaptörde (`ToplayanIo::dns_bolgesi`) kayıtsız ad `alan_yok` döner;
    `dil dene` sahte çözücüsü boş ve yetkili bölgedir. Statik bölge dosyası
    açık kalır.

## Sonuçlar

- ZM-R006 derlenir; ürün MX/TXT/TLSA'yı DNSSEC durumuyla sorgulayabilir.
  B2 (SPF/DKIM/DMARC, A-R temizliği) bu çekirdeği host değerlendiricilerinde
  kullanır.
- Derleyici ikilisinin üretim bağımlılık ağacı değişmez; yalnız dev ve fuzz
  kilitleri `hickory-proto` zincirini taşır (cargo-deny bans/lisans/kaynak
  ve RustSec temiz, 24 Kasım 2026).
- Kendi ayrıştırıcımızın bakım yükü kâhin + fuzz + RC kapısıyla karşılanır;
  yeni kayıt türü ihtiyacı (PTR, SRV, SVCB) bu ADR'ye ek ister.

## Açık

- PSL veri dosyası ve kurumsal alan (B1b, ZM-F025) B2 dilimiyle gelir.
- Zee yüzeyinde NXDOMAIN/NODATA ayrımı, olumsuz yanıtın doğrulanmışlığı,
  CNAME kanonik adının ayrı alanı (B4 ihtiyacı).
- `dil dene` için statik bölge dosyası; gerçek unbound ile imzalı/imzasız/
  bogus entegrasyon koşusu (RFC-0044 B1 kabul 5, CI'da isteğe bağlı) yapılmadı.
- SMTP oturumu ve süreç başına DNS bütçesi B2 değerlendiricileriyle gelir;
  B1 yorumlayıcısı tek iş parçacıklıdır, eşzamanlı sorgu 1'dir.
- U-label/IDN alanları (SMTPUTF8 ile birlikte).
