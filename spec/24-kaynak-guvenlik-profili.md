# 24 — Kaynak güvenlik profili

Normatif kaynak: RFC-0025, ADR-033, ADR-035, ADR-037, ADR-040. Durum:
**TANIMLI — K-129/K-130/K-131/K-132/K-138/K-140/K-143**.

## Profil sahipliği

Resmî derleyici, runtime, CLI ve LSP aynı değişmez `KaynakSinirlari` profilini
kullanmak ZORUNDADIR. Program, paket veya ortam bu sınırları sessizce
yükseltemez. Sınırsız fallback ve başarılı görünen truncate YASAKTIR.
HTTP/ağ, web oturumu, IO izi, LSP, paket/registry, tanı, kalıcı dosya ve PostgreSQL
katmanları sayısal varsayılanlarını bu profilin domain görünümlerinden okumak
ZORUNDADIR; yerel sabit yalnız ortak değere bağlı geriye uyum alias'ı olabilir.

## Derleme

- Tek UTF-8 kaynak 8 MiB'ı ve 1.000.000 tokenı AŞAMAZ.
- Tek proje 4.096 `.dil` dosyası ve toplam 128 MiB kaynak metnini AŞAMAZ.
- Kaynak veya token aşımı S045'tir; parser/AST kurulmadan durmalıdır.

## Çalışma

- Çağrı derinliği 500'ü aşarsa C019 üretilir.
- Tek çalışma veya web isteği 10.000.000 cümle adımını AŞAMAZ.
- Tek liste/sözlük 1.000.000 öğeyi; tek eşzamanlı grup 1.024 görevi AŞAMAZ.
- Tek üretilen metin 16 MiB'ı AŞAMAZ. Metin birleştirme/değiştirme, HTML,
  değer metni, keyfî hassasiyetli sayı/para, JSON ve CSV üretimi sonucu
  kurmadan bütçeli büyümelidir.
- Çalışma veya web isteği, ortam/koleksiyon yazımları ve görev ortamı klonları
  için yaklaşık 64 MiB, iade edilmeyen saklanan değer tahsis zarfı taşır.
  Muhafazakâr fazla sayım TANIMLIDIR; aşım C024'tür.
- `yaz`, soru istemi ve web yanıtı toplam 16 MiB veya 100.000 çıktı olayını
  aşarsa aşan olay IO'ya verilmeden C023 üretilir.
- Dil programının metin dosyası okuması 16 MiB'ı AŞAMAZ. Atomik append ve
  transaction ön-okuması aynı bounded reader'ı kullanmalıdır.

Uzun yaşayan web sunucusunda her istek bağımsız adım/çıktı bütçesi alır.
Süreç genelinde kabul edilmiş inbound ve başlatılmış outbound ağ bağlantılarının
toplamı 64'ü AŞAMAZ. İzin bütün başarı/hata yollarında bırakılmalıdır; aşım
outbound'da C018 nedeni, inbound'da HTTP 503'tür.

## HTTP isteği

Native web başlığı en çok 16 KiB, gövdesi 64 KiB'dır. Başlık sonu ham baytta
ve yalnız CRLF ile bulunur; geçerli `Content-Length` gövde tahsisinden önce
denetlenir. TE, duplicate CL, obs-fold, NUL ve UTF-8 dışı veri için kayıplı
dönüşüm YASAKTIR. Gövde exact ilan edilen uzunluktadır. Ayrıştırıcı 260 satır
bütçeli `http_istegi.rs` sahibidir ve ham byte fuzz hedefi taşır. Ayrıntılı
wire profili spec/11 ve ADR-037'dedir.

## LSP

LSP sunucusu en çok 256 açık belge ve toplam 128 MiB belge metni saklar. Tek
outbound JSON mesajı 8 MiB'ı aşamaz. Fazla belge sunucu durumuna eklenmez;
önceki belge sürümü korunur. Aşım S045 bildirimi veya kimlikli istekte JSON-RPC
`-32001` kaynak hatası olarak görünür. Yanıt zarfı, kimlik, kaçışlı metin,
diagnostics ve rename düzenlemeleri aynı bütçeli yazıcıya parça parça
yazılmalıdır; önce sınırsız JSON kurup sonra boyut ölçmek YASAKTIR. Kısmi gövde
yayımlanamaz.

Inbound JSON, 128 derinlik ve 100.000 düğüm zarfında RFC 8259 sayı durum
makinesi kullanır; sayıyı binary float'a çevirmeden doğrulanmış lexeme olarak
saklar. `NaN`/`Infinity`, kuralsız sayı ve çözülmüş adı yinelenen nesne alanı
reddedilir. Sözdizim/UTF-8 hatası `-32700`, geçerli JSON içindeki bozuk
tek-nesne JSON-RPC zarfı `-32600` üretir. Bilinmeyen kimlikli yöntem `-32601`,
geçersiz yöntem parametresi `-32602` alır; kimliksiz notification'a response
yazılmaz. Sayısal `id` dönüştürülmeden geri yazılır. Bir çerçevede JSON-RPC
batch kabul edilmez. Ayrıntılı bakım sınırı
[LSP JSON-RPC profilindedir](../docs/lsp-json-rpc-profili.md).

## Playground

Playground tek kaynak metnini en çok 8 MiB, soru yanıtı metnini en çok 1 MiB
ve 4.096 satır kabul eder. Byte sayısı UTF-8 kodlaması üzerindendir. Kaynak ve
soru byte sınırı sahipli kopyadan; soru satır sınırı `Vec<String>` kurulmadan
önce denetlenmek ZORUNDADIR. Sınırdaki değer kabul, bir fazlası görünür
`PLAYGROUND SINIR HATASI` sonucudur; sessiz kesme YASAKTIR.

Kanonik tarayıcı hostu limitleri ABI v3 dışa aktarımlarından okumalı, UTF-8
boyunu byte dizisini kurmadan hesaplamalı ve metni exact WASM tamponuna
`encodeInto` eşdeğeri tek geçişle yazmalıdır. Genel ABI tampon kotası bu daha
dar tür bütçesinin yerine GEÇEMEZ.

## PostgreSQL

Sorgu 64 KiB, parametre listesi 100 öğe, tek parametre 64 KiB, okuma sonucu
10.000 satır/100 sütun/16 MiB sınırındadır. Tek migration 1 MiB, migration
toplamı 8 MiB'ı aşamaz. Adaptör bu değerleri `veritabani()` görünümünden okumak
ZORUNDADIR; yerel ve kullanıcı tarafından yükseltilebilir sabit YASAKTIR.

## Ayrı kapsam

B-025/K-132 kaynak bütçesi, B-051/K-138 protokol-kesin JSON-RPC ve
B-056/K-143 playground ön-tahsis kapsamı tamamdır. Duvar-saati/cancellation
sözleşmesi B-026'nın ayrı kapsamıdır.

Form/text HTTP gövdesi 64 KiB kalır. Exact `application/octet-stream` için
ayrı 16 MiB yükleme zarfı vardır; tahsis bu boyutta yapılmaz, 16 KiB parçalar
temp dosya ve SHA-256 durumuna akar. Dosya SHA-256 işlemi de 16 MiB'ta
fail-closed olur. Bu zarf metin heap limitini genişletmez.
