# 23 — Yetkinlik ve outbound ağ güvenliği

Normatif kaynak: RFC-0024, ADR-031. Gerçekleme kaydı: K-127.

## Yetkinlik kümesi (TANIMLI)

Kararlı proje yazımları şunlardır:

| Yazım | Açtığı sınır |
|---|---|
| `dosya-okuma` | Proje kökündeki dosyayı okuma |
| `dosya-yazma` | Proje kökündeki dosyayı atomik yazma/ekleme |
| `ağ` | Bildirilmiş origin'e outbound HTTP(S) |
| `yerel-ağ` | Private/loopback hedef ve explicit düz HTTP |
| `ağ-sunucusu` | Native loopback web sunucusu/adaptörü |
| `donanım` | Sensör ve ışık adaptörü |
| `web-oturumu` | Oturum, rol, çerez ve CSRF adaptörü |
| `kriptografi` | Parola/kriptografi adaptörü |

`proje.dil` içindeki `yetkinlikler` ve `ağ_hedefleri` yoksa boş listedir.
Değerler tekil olmak ZORUNDADIR. `yerel-ağ` → `ağ`, `web-oturumu` →
`ağ-sunucusu` bağı ZORUNLUDUR. `ağ` ancak boş olmayan `ağ_hedefleri` ile ve
tersi birlikte bulunabilir. İhlal P015'tir.

## Origin ve IP kuralları (TANIMLI)

Bildirim hedefi yalnız `http://host[:port]` veya `https://host[:port]`
origin'idir; yol, sorgu ve parça YASAKTIR. İstek URL'sinin yolu/sorgusu olabilir
ama şema+küçük-harfli ASCII DNS/IP+etkin port exact bildirime eşleşmek
ZORUNDADIR. Kullanıcı bilgisi, kontrol karakteri, ters bölü, sıfır port,
ayraçsız IPv6 ve başka şema YASAKTIR.

Public hedef HTTPS olmak ZORUNDADIR. Düz HTTP bildirimi için `yerel-ağ`
ZORUNLUDUR. DNS çözümünden sonra bağlantıda kullanılabilecek bütün IP'ler
denetlenir: public unicast kabul edilir; loopback, RFC1918, CGNAT ve IPv6 ULA
yalnız `yerel-ağ` ile kabul edilir. Link-local/metadata ile IANA public olmayan
özel-kullanım veya IPv4-geçiş önekleri her profilde YASAKTIR. Yasak ve izinli
IP'yi birlikte döndüren DNS cevabı reddedilir.

Redirect izlenmez; ortam proxy'si kullanılmaz. Native istemci global ve DNS
çözümünde varsayılan 30 saniye son tarih, en fazla 64 KiB response header ve
8 MiB toplam zarf içinde body uygular. HTTPS sertifika/TLS doğrulaması rustls
backend'indedir. Bu sınırların ihlali runtime'da C018'e dönüşür.

K-139/ADR-036 ile aynı `AgHedefi` origin ayrıştırıcısı production web
proxy'nin `--web-proxy`, `Host`, `Forwarded host` ve unsafe `Origin`
sınırlarında da ZORUNLUDUR. Outbound ve inbound güven yüzeyleri ikinci bir
DNS/IPv6/port parser'ı taşıyamaz; web'e özgü HTTPS ve loopback değişmezleri
spec/12'de tanımlıdır.

## Compile ve runtime uygulaması (TANIMLI)

Ana program, bütün testler ve çağrılmayan işlemler dahil her gövdenin dış dünya
ihtiyacı yürütmeden önce çıkarılır. Eksik yetkinlik veya sabit outbound URL'nin
allowlist dışı origin'i kesin AST ifade aralığında T054'tür. Dinamik URL runtime
origin denetiminden geçmek ZORUNDADIR.

Proje çalıştırma ve denetleme manifest politikasını kullanır. `--güvenli`
yalnız proje-kökü dosya okuma/yazmayı açar. Bildirimsiz tek dosya geriye uyumlu
geliştirici profilidir; dosya ve adaptörler açık, outbound yalnız public
HTTPS, `yerel-ağ` kapalıdır. Her runtime IO çağrısı policy sarmalayıcısından;
native dosya/ağ/web adaptörü ayrıca aynı kapıdan geçer.

Proje dosya yolu göreli ve kök içinde olmalıdır. `..`, mutlak yol ve
canonical hedef/ebeveynin sembolik bağla kök dışına çıkması reddedilir. Bu
sözleşme eşzamanlı düşman yerel süreçlere karşı OS sandbox'ı değildir.

## Paket yetkisi (TANIMLI)

Her paket kendi `proje.dil` ihtiyacını bildirir. Paket yetkinlik ve origin
kümeleri ana uygulama politikasının alt kümesi olmak ZORUNDADIR; aksi P015'tir.
Uygulama izni süreçteki ortak üst sınırdır. Paket başına ayrı filesystem,
tenant veya süreç izolasyonu bu sürümde AÇIKTIR ve vaat edilmez.
