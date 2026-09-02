# 19 — Registry Metadata Güven Zinciri

Normatif kaynak: RFC-0020 §6–9. Mimari sınır: ADR-006. Tanılar: P013, P014.

Bu bölüm çalışan `root → timestamp → snapshot → targets` byte doğrulayıcısını
tanımlar. HTTPS/statik dosya taşıması, kalıcı cache, CLI ve uzak bağımlılığı
`proje.dil`/`proje.kilit` ile birleştirme henüz bu çalışan yüzeyin parçası
değildir.

## Zarf ve imza girdisi (TANIMLI)

Her rol dosyası tam olarak şu kapalı şemalı zarftır:

```json
{
  "imzali": { "...": "role özgü kapalı şema" },
  "imzalar": [
    { "anahtar_kimligi": "sha256:<64 küçük hex>", "ed25519": "<128 küçük hex>" }
  ]
}
```

Dosya `serde_json::to_vec_pretty` alan sırası, iki boşluklu girinti ve tek son
satır sonuyla byte-byte kanoniktir. Bilinmeyen/tekrarlı alan, farklı boşluk,
imzaların anahtar kimliğine göre kesin artmaması ve fazladan byte YASAKTIR.
Haritalar anahtara göre artan sıradadır.

İmza girdisi byte birleştirmesidir:

```text
"zee-registry-v1\0" || UTF8(rol) || "\0" || COMPACT_JSON(imzali)
```

V1 yalnız Ed25519 kabul eder. Anahtar kimliği açık 32 byte'ın SHA-256
özetidir. Aynı anahtarın yinelenen imzası eşiğe birden fazla katkı yapmaz;
yetkisiz veya doğrulanamayan imza sayılmaz.

## Root (ZORUNLU)

`zee-registry-root-v1`; pozitif monoton `surum`, kesin
`YYYY-MM-DDTHH:MM:SSZ` `sona_erme`, `tutarli_anlik=true`, anahtar haritası ve
tam `root/targets/snapshot/timestamp` rol yetkilerini taşır. Her rol 1–256
tekil/sıralı anahtar kimliği ve `1..N` eşik taşır. En çok 256 anahtar ve 256
zarf imzası vardır; root dosyası en çok 1 MiB'dir.

İlk root byte'ı ağ dışı `sha256:` sabitlemesi, root'un kendi eşiği ve süre
sonuyla doğrulanır. Yeni root sürümü tam `eski+1` olmak ZORUNDADIR. Aynı yeni
zarf hem eski root yetkisinin hem yeni root yetkisinin eşiğini sağlamadan
güven durumu değişmez. Başarısız rotasyon eski root'u korur.

## Çevrimiçi roller (ZORUNLU)

- `zee-registry-timestamp-v1`: snapshot sürüm/boyut/SHA-256 bağını taşır;
  dosya en çok 64 KiB'dir.
- `zee-registry-snapshot-v1`: targets sürüm/boyut/SHA-256 bağını taşır;
  dosya en çok 1 MiB'dir.
- `zee-registry-targets-v1`: exact hedefler ile duyuruları taşır; dosya en çok
  8 MiB, hedef/duyuru sayısı ayrı ayrı en çok 100.000'dir.

Her metadata sürümü pozitiftir; süre sonu güncellemenin başında bir kez alınan
UTC andan kesin büyüktür. Kalıcı en yüksek sürümden küçük gelen rol rollback
olarak reddedilir. Aynı sürüm numarası ancak daha önce kabul edilen aynı
SHA-256 byte içeriğiyle yeniden kullanılabilir; aynı sürümlü farklı imzalı
içerik de reddedilir.

Doğrulama sırası timestamp imza/süre/geçmişi; onun bağladığı snapshot
boyut/özet/sürümü ve imzası; snapshot'ın bağladığı targets boyut/özet/sürümü
ve imzasıdır. Zincirin tamamı geçmeden görülen hiçbir çevrimiçi sürüm/özet
kalıcı duruma uygulanmaz. Daha yeni durum/root kabul edildikten sonra önceden
doğrulanmış eski bir sonuç da uygulanamaz. Böylece geçersiz yüksek sürümlü
zincir veya yarışan bayat sonuç kalıcı durumu zehirleyemez.

## Exact hedef ve yayıncı yetkisi (ZORUNLU)

Targets harita anahtarı `<paket>@<X.Y.Z>`dir. Hedef paket, sürüm,
`zee-tr-1` morfolojisi, izin verilen `sha256:` yayıncı anahtar kimliği,
`yanked`, sıralı duyuru kimlikleri ve dört dosyanın kesin ad/boyut/SHA-256
bilgisini taşır. Dosya adları yalnız şunlardır:

```text
<paket>-<sürüm>.zep
<paket>-<sürüm>.spdx.json
<paket>-<sürüm>.intoto.json
<paket>-<sürüm>.zee-yayin.json
```

Targets bağıyla dört byte dizisi doğrulanmadan yayıncı doğrulayıcı çağrılmaz.
Ardından spec/18 zinciri doğrulanır ve yayıncı/paket/sürüm/morfoloji ile ilk üç
dosya descriptor'ı targets yetkisiyle birebir eşleşir. Öz-imzası geçerli ama
targets tarafından yetkilendirilmeyen yayın P014'tür.

Yanked hedef yeni seçimde varsayılan reddedilir. Etkin `kritik` duyuru da
varsayılan reddedilir. Duyuru kimliği, paket, en az bir sıralı exact etkilenen
sürüm, `düşük|orta|yüksek|kritik` önem, isteğe bağlı düzeltilen exact sürüm ve
etkinlik taşır. Baypas yalnız çağıranın açık politikasıyla mümkündür; gelecekte
kilide gerekçesiyle yazılmadan bağımlılık kabulüne dönüşemez.

## Kaynak limitleri (ZORUNLU)

- `.zep`: 64 MiB;
- SPDX SBOM: 8 MiB;
- provenance: 8 MiB;
- yayın bildirimi: 1 MiB.

Bağlayan üst metadata boyutu, ayrıştırmadan/ayırmadan önce denetlenir. Bütün
özetler küçük harfli 32-byte SHA-256 hex'tir.

## Taşıma, kalıcı durum ve cache (TANIMLI — K-135)

Gerçek istemci yalnız yol/sorgu taşımayan bir `https://` origin kabul eder;
redirect ve ortam proxy'si kapalıdır, DNS sonrası public-IP denetimi ortak ağ
profilindedir. Statik yerleşim şöyledir:

```text
metadata/timestamp.json
metadata/<sürüm>.snapshot.json
metadata/<sürüm>.targets.json
metadata/<sürüm>.root.json
hedefler/sha256/<64 küçük hex>
```

Timestamp ve üst rolün bağladığı sürüm okunarak sonraki sürümlü yol seçilir;
yoldan gelen sürüm veya ayna güven kararı değildir. Tek güncellemede en çok 64
ardışık root rotasyonu izlenir. Her GET çağıranın merkezî metadata/hedef byte
sınırını taşır; 200 ve root aramasındaki 404 dışında durum başarı sayılmaz.

Cache `nesneler/sha256/<özet>` altında bütün metadata ve hedefleri aynı içerik
adresiyle, salt-okunur saklar. İndirilen hiçbir hedef tam root→timestamp→
snapshot→targets ve yayıncı zinciri geçmeden bu adrese yazılmaz. Var olan nesne
her okumada boyut+SHA-256 ile yeniden doğrulanır; bozuk nesne ağ varken bile
sessizce iyileştirilmez veya kullanılmaz.

`durum-v1.json`, `zee-registry-cache-v1` kapalı/kanonik şemasında ağ dışı ilk
root özeti, etkin root nesne özeti, son başarılı doğrulama zamanı ve bütün rol
sürüm+özetlerini taşır. Önce değişmez nesneler atomik yazılır, en son durum
dosyası karşılaştır-ve-değiştir ile yayımlanır. Böylece çökme kullanılmayan
nesne bırakabilir ama yarım durumu görünür yapamaz; yarışan eski istemci yeni
durumu ezemez. Başarısız zincir kalıcı durumu değiştirmez.

Çevrimdışı kip duvar saati/ağ kullanmaz; yalnız son çevrimiçi kabul zamanında
geçerli olduğu kaydedilmiş tam metadata zincirini ve exact hedef nesnelerini
yeniden doğrular. Durum, metadata veya dört hedeften biri yok/bozuksa P016
cache miss/bozulma hatasıdır. Bu kip yeni metadata güncelliği iddia etmez.

## Conformance

Uyumlu gerçekleme en az şunları kanıtlar:

- sabit root özeti, root eşiği ve eski+yeni çift eşikli ardışık rotasyon;
- tam timestamp→snapshot→targets olumlu zinciri;
- rollback, expiry/freeze, aynı sürümle farklı içerik ve mix-and-match reddi;
- geçersiz fast-forward zincirinin kalıcı durumu değiştirmemesi;
- sürümsüz/özetsiz bozuk kalıcı metadata durumunun reddi;
- kanonik olmayan/aşırı metadata'nın fail-closed reddi;
- yanked/kritik duyuru varsayılanı ve yanlış yayıncı reddi;
- RFC 3339 UTC takvim sınırları.

K-135 kalıcı durum, uzak HTTPS/statik taşıma, doğrulanmış cache ve offline
hit/miss'i kanıtlar. Exact proje bildirimi/kilit ve CLI entegrasyonu
tamamlanmadan V1-P1-07 **AÇIK** kalır.
