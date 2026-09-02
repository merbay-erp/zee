# 24 — Kaynak güvenlik profili

Normatif kaynak: RFC-0025, ADR-033. Durum: **TANIMLI — K-129/K-130/K-131/K-132**.

## Profil sahipliği

Resmî derleyici, runtime, CLI ve LSP aynı değişmez `KaynakSinirlari` profilini
kullanmak ZORUNDADIR. Program, paket veya ortam bu sınırları sessizce
yükseltemez. Sınırsız fallback ve başarılı görünen truncate YASAKTIR.
HTTP/ağ, web oturumu, IO izi, LSP, paket/registry, tanı ve kalıcı dosya
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

## LSP

LSP sunucusu en çok 256 açık belge ve toplam 128 MiB belge metni saklar. Tek
outbound JSON mesajı 8 MiB'ı aşamaz. Fazla belge sunucu durumuna eklenmez;
önceki belge sürümü korunur. Aşım S045 bildirimi veya kimlikli istekte JSON-RPC
`-32001` kaynak hatası olarak görünür. Yanıt zarfı, kimlik, kaçışlı metin,
diagnostics ve rename düzenlemeleri aynı bütçeli yazıcıya parça parça
yazılmalıdır; önce sınırsız JSON kurup sonra boyut ölçmek YASAKTIR. Kısmi gövde
yayımlanamaz.

## Ayrı kapsam

B-025/K-132 kaynak bütçesi kapsamı tamamdır. Duvar-saati/cancellation
sözleşmesi B-026'nın ayrı kapsamıdır.
