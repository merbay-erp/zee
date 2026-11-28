# ADR-170 — Paylaşılan veri kökü: `veri_kökü`, `veri:` öneki ve ayrı okuma/yazma yetkinliği

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-394, [RFC-0045](../rfcs/0045-paylasilan-veri-koku.md),
  K-382 (takip önerisi), ADR-031 (merkezî yetkinlik), ADR-059 (core
  freeze), ADR-064 (dil yüzeyi), ADR-067 (dogfood korpusu), ADR-105
  (credential kaynağı), ADR-163 (dosya yanıtı), ADR-167 (çözerek yazma),
  spec/07, spec/08, spec/23, GB-168

## Bağlam

`urun-h` posta ürününün alıcı/gönderim/IMAP hizmetleri ve web paneli ayrı
Zee projeleridir (ayrı PostgreSQL rolleri, en az ayrıcalık). Hizmetler
K-389 ile ekleri bayt-kesin dosyaya çözebiliyor; web aynı eki RFC-0041
dosya yanıtıyla sunmalı (ZM-F038). Proje dosya sınırı (spec/07 K-382,
spec/23) her projeyi kendi `proje.dil` klasörüne kapatır; `..`, mutlak yol
ve kök dışı canonical hedefli bağ reddedilir. Geçici çözümler (hizmetleri
web köküne taşımak, ekleri `bytea` saklamak) en az ayrıcalığı ya da 16 MiB
eki bozar.

Core freeze yeni yüzeyi varsayılan olarak reddeder. Ürün kaydı etkin,
reproducer depo içinde (`dogfood/urun-h/paylasilan-depo/`), yeni tür,
grammar ya da kalıp gerekmiyor.

## Karar

1. **Bildirim.** `proje.dil` isteğe bağlı `veri_kökü "AD" olsun` alanı
   kökün mutlak yolunu taşıyan ortam değişkeninin / systemd credential'ın
   adını (A-Z, 0-9, _; ≤ 128 bayt) bildirir. Değer K-339 kaynak kuralıyla
   (credential dizini, yoksa ortam; ikisi birden ret) her `veri:`
   çağrısında okunur.
2. **Yetkinlikler.** Yeni `veri-okuma` ve `veri-yazma`. `veri_kökü` en az
   biriyle, ana projede onlar da yalnız `veri_kökü` ile bulunur; aksi **P023**.
   Kök yalnız ana projeden gelir: paket `veri-*` ihtiyacını kökü olmadan
   bildirir (P015 alt küme), `veri_kökü` bildiremez (P023); paketin `veri:`
   yolları ana projenin köküne çözülür. Kütüphane paketi tek başına
   denetlenirken P023 beklenir; tanı önerisi kök projeden (ya da test kök
   projesinden) denetlemeyi gösterir.
   `dosya-*` veri kökünü, `veri-*` proje dosyalarını açmaz. Bildirimsiz
   tek dosya profilleri ve `--güvenli` kipi `veri-*` içermez. Statik T054
   dosya gereksinimini aynı yönlü `veri-*` ile de karşılanmış sayar; kesin
   yol sınıfı ve yetkinlik çalışma zamanında her çağrıda denetlenir.
3. **Yol.** `veri:` öneki her kipte ayrılmıştır. Sonrası `/` ayrılmış,
   boş olmayan, `.`/`..`/ters bölü/iki nokta/denetim karakteri taşımayan
   bileşenlerdir (bileşen ≤ 255, toplam ≤ 1024 bayt).
4. **Kök doğrulaması.** Mutlak, `/` değil, `.`/`..` yok; `/`'den her bileşen
   `openat(O_DIRECTORY|O_NOFOLLOW)` ile açılır (yolda bağ yok); kök ve üst
   klasörlerin sahibi root ya da etkin kullanıcı; üst klasörler grup/diğer
   yazamaz (yapışkan bit hariç). Linux'ta `fs.protected_hardlinks` açık
   olmalı (K-422 eki: değer okunamıyorsa koruma bilinmiyor kipi).
5. **Yürüyüş ve ata değişmezi.** Kök altındaki klasörler `O_NOFOLLOW` ile
   açılır; yaprağın doğrudan üst klasörü dışındaki her klasör (kök dahil)
   root/etkin kullanıcıya ait ve grup/diğer yazamaz. Paylaşılan yazma
   yalnız son klasörde olur; böylece hiçbir başka kullanıcı ara bileşeni
   doğrulama ile kullanım arasında değiştiremez.
6. **Yaprak.** Bağ reddedilir; var olan yaprak normal dosyadır. Okuma
   `O_NOFOLLOW|O_NONBLOCK` ile açılan tanıtıcıdan yapılır; tanıtıcı
   `S_ISREG` ve `st_nlink == 1` olmalıdır. Bu, okuma, aralıklı okuma,
   SHA-256, ilk bayt, dosya yanıtı, e-posta eki ve bağlantıya aktarımda
   aynıdır.
7. **Kalıcı yazım katmanı (spec/08).** Kilit dosyası ve kilit altındaki iç
   okumalar (ekleme, çözerek ekleme — K-395 32 MiB tavanlı okuma dahil —,
   eylem yedeği, taşıma/silme izi, geri alma) unix'te
   `O_NOFOLLOW|O_NONBLOCK` ile açılır ve normal dosya ister. Bu, bir eş
   yazarın yaprağı iki kez değiştirerek (bağ → normal dosya) kök dışı
   içeriği paylaşılan hedefe kopyalatmasını kapatır. Yayın zaten
   `rename`dir (bağı izlemez).
8. **Kapsam.** Desteklenen: okuma ailesi, dosya yanıtı, e-posta eki,
   bağlantıya aktarma, yazma/ekleme, çözerek yazma/ekleme, taşıma (veri↔veri,
   proje↔veri; EXDEV'de K-332 kopyası), silme, eylem savepoint'i.
   Desteklenmeyen (açık ret): dizin listeleme, görsel küçültme, bağlantı
   blok dosyası ve `.zee/yuklemeler` geçici dizini. Windows/wasm: yok.
9. **Kipler.** Tek dosya, `--deneysel-web dosya.dil` ve `--güvenli`:
   `veri:` reddedilir, yerel dosya oluşturulmaz. `dil dene`: hermetik
   tablo, yol metni anahtar. IO izi yol metnini taşır, kök yolunu taşımaz.
10. **İletiler.** Kökün mutlak yolunu taşımaz; ortam adı ve `veri:` yolu
    yeterlidir.

## Sonuçlar

- Ayrı kullanıcılarla koşan hizmetler aynı veri alt ağacını paylaşır;
  okuyan hizmet yazarlara karşı korunur (RFC-0045 §Tehdit T1–T10).
- Aynı köke yazan hizmetler o alt ağaç için tek bütünlük alanıdır;
  birbirinin dosyasını ezebilir/silebilir. İçerik adresli ad ve no-clobber
  yazım önerilir.
- Ata değişmezi işletmeden yerleşim ister (kök root:root 0755, paylaşılan
  son klasör grup yazılabilir). Yanlış izin fail-closed ret verir.
- macOS'ta çekirdek hard-link koruması yoktur; okuyucu korunur, yazarın iç
  okuması için kalan risk RFC-0045'te yazılıdır (geliştirme platformu).
  K-422 ekiyle bu risk tek-bağ kipiyle kapandı (aşağıda).
- `veri:` ile başlayan proje dosya adları artık proje klasöründe aranmaz
  (envanterde kullanım yok). Eylem içinde bağ olan yaprağı taşıma/silme
  izi C013 verir (önceden bağı izliyordu).

## Kanıt

`compiler/tests/veri_koku_testi.rs`, `compiler/src/guvenlik/veri_koku/testler.rs`,
`dogfood/urun-h/paylasilan-depo/`, spec/07 §Paylaşılan veri kökü, spec/08,
spec/23, hata kataloğu P023, GB-168.

## Ek — K-422: koruma bilinmiyor kipi ve tek-bağ yazımı (28 Kasım 2026)

**Bağlam.** Ürün H (ZM-F051): Ubuntu 22.04'te
`/proc/sys/fs/protected_hardlinks` `0600 root`'tur. Ayrıcalıksız hizmet
kullanıcısı değeri okuyamadı; madde 4 bunu "okunamadı (/proc bağlı değil)"
diye C013 ile reddetti, ekli gelen ileti saklanamadı (452). Ubuntu 24.04'te
dosya `0644`'tür. İleti gerçek nedeni (izin yok) de söylemiyordu.

Madde 4'ün çekirdek korumasına dayandırdığı tehdit: paylaşılan son klasöre
yazabilen eş hizmet (A2), yazarın okuyabildiği ama kendisinin okuyamadığı
bir dosyayı o klasöre sert bağlar. Okuyucu madde 6 ile korunuyordu; yazarın
kalıcı katman iç okumaları (ekleme, çözerek/base64 ekleme, eylem yedeği,
taşıma/silme izi) ise bağlı içeriği okuyup tek bağlı **yeni** dosyaya
yayımlıyordu (aklama; RFC-0045'te macOS kalan riski olarak yazılıydı).
`protected_hardlinks=1` iken eş hizmet yalnız kendi okuyup yazabildiği
dosyayı bağlayabildiğinden bu kopya ona yeni bir şey vermiyordu.

**Karar.**

1. `fs.protected_hardlinks` okumasının sonucu ayrılır: değer `0` →
   ret ("çekirdek koruması kapalı"); dosya yok (ENOENT/ENOTDIR) → ret
   ("/proc bağlı değil"); başka okuma hatası → gerçek nedenle ret; izin yok
   (EACCES/EPERM) → **koruma bilinmiyor**, kök kabul edilir.
2. Veri kökü yazımları değerden bağımsız **tek-bağ kipinde** koşar
   (`kalici_dosya::tek_bag_kipinde`, `veri:` yolu taşıyan her yazım):
   yaprağın yazma kilidi (ayrıca `O_NONBLOCK|O_NOCTTY`, normal dosya),
   var olan hedefin içerik okuması, metadata kaynağı, eylem/taşıma/silme
   izi ve EXDEV kopya okuyucusu `O_NOFOLLOW` tanıtıcıda `st_nlink == 1`
   ister; aksi C013, hiçbir bayt yayımlanmaz. Kip iş parçacığına özel ve
   kapsamlıdır; proje dosyası yazımı değişmez.
3. Okuma ailesi madde 6'daki gibi kalır (tanıtıcıda `S_ISREG` ve
   `st_nlink == 1`). Yaprak çözümü (`fstatat`) okumada çok bağlı yaprağı
   ayrıca erken reddeder; izsiz taşıma kaynağı da bu yoldan geçer.
4. Böylece her açılış yolu tanıtıcı düzeyinde korunur; bilinmeyen kipte
   ADR'nin güvenlik sınırı gevşemez. Yazım yayını zaten `rename`dir ve
   geçici dosya `O_EXCL` ile açılır (bağlı hedefin içine yazılmaz).

**Açılış yolları ve koruma (bilinmeyen kipte de aynı).**

| Yol | Koruma |
|---|---|
| Okuma, aralık, SHA-256, ilk bayt, dosya yanıtı, e-posta eki, bağlantıya aktarım | `okuma_icin_ac`: `O_NOFOLLOW|O_NONBLOCK`, `S_ISREG`, `st_nlink == 1` |
| Satır yazma (üzerine) | metadata kaynağı tanıtıcıda tek bağ; yayın `rename` |
| Satır/çözerek/base64 ekleme | mevcut içerik `O_NOFOLLOW` tanıtıcıda tek bağ |
| Yeni dosyaya çözerek yazma | var olan hedef (bağlı dahil) no-clobber ret |
| Eylem içi yazma/silme/taşıma izi | iz okuması tanıtıcıda tek bağ |
| İzsiz taşıma | kaynak `fstatat` tek bağ; yalnız dizin girdisi taşınır, içerik okunmaz; EXDEV kopyası tanıtıcıda tek bağ |
| İzsiz silme | yalnız dizin girdisi kaldırılır; içerik okunmaz, dış dosya değişmez (serbest) |
| Yazma kilidi | `O_NOFOLLOW|O_NONBLOCK|O_NOCTTY`, normal dosya, tek bağ; içine yazılmaz |

**Kalan risk.** Değer gerçekten `0` iken okunamıyorsa, kilit adına bağlanmış
bir aygıt düğümü (hizmetin okuma-yazma izni olan) açılış anında
açılır ve sonra reddedilir; `O_NONBLOCK|O_NOCTTY` bloklamayı ve denetim
terminali almayı önler, içine yazılmaz. Eylem geri alması kipsizdir: hedefi bağ
izlemeden okuyup eylemin bıraktığı içerikle karşılaştırır; araya bağ
konduysa içerik eşleşmez ve geri alma yazmaz, eşleşse bile yalnız eylemin
kendi önceki içeriği yayımlanır (dış içerik kopyalanmaz).

**Kanıt.** `compiler/src/guvenlik/veri_koku/testler.rs`
(`veri_koku_hard_link_korumasi_okuma_hatalarini_ayirir`,
`veri_koku_koruma_okunamazsa_kok_kabul_edilir_kapaliysa_reddedilir`,
`veri_koku_tek_bag_kipinde_sert_bagli_yaprak_yazima_aklanamaz`,
`veri_koku_tek_bag_kipinde_kilit_adina_baglanan_dis_dosya_kilit_sayilmaz`,
`veri_koku_tek_bag_kipi_yalniz_veri_yazimini_kapsar`),
`compiler/tests/veri_koku_testi.rs::sert_bagli_veri_yapragi_yazim_yollarinda_aklanamaz`,
`regression/security/veri-koku-sert-bag-ekleme.dil`, spec/07, GB-189.
