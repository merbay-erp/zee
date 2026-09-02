# ADR-019 — LSP çerçeve ve JSON girdi sınırları

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-107, B-047, V1-P0-17

## Bağlam

`dillsp` stdio çerçevesi başlık sonunu sınırsız arıyor ve bildirilen
`Content-Length` kadar belleği doğrulamadan ayırıyordu. Elle yazılmış mini JSON
ayrıştırıcısı iç içelik/düğüm bütçesi taşımıyordu. Ayrıca yüksek Unicode vekili
sonrasında gelen kodun düşük vekil olduğunu doğrulamadan çıkarma yapıyor ve
JSON metni içindeki kaçışsız kontrol karakterlerini kabul ediyordu.

Editör aracı ağ sunucusu değildir; yine de proje, dosya veya istemci girdisi
güvenilir kabul edilemez. Tek bozuk mesaj process belleğini ya da yığını
sınırsız büyütmemelidir.

## Karar

`dillsp` her gelen çerçevede:

- başlığı CRLF ayıracı dahil en çok 8 KiB ile sınırlar;
- büyük/küçük harften bağımsız tam bir ve yalnız bir `Content-Length` ister;
- gövdeyi ayırmadan önce 8 MiB sınırını uygular;
- başlık ve gövdeyi sıkı UTF-8 olarak doğrular;
- çerçeve hatasında akışla yeniden eşleşme uydurmak yerine Türkçe hata yazıp
  fail-closed kapanır.

Mini JSON ayrıştırıcısı ayrıca:

- en çok 128 iç içelik seviyesi ve 100.000 değer düğümü kabul eder;
- yüksek vekili yalnız `U+DC00..U+DFFF` düşük vekiliyle birleştirir;
- tek düşük vekili, eksik/yanlış çifti ve kaçışsız `U+0000..U+001F`
  karakterlerini reddeder;
- doğrudan Rust API çağrısında da 8 MiB metin sınırını uygular.

## Sonuçlar

- `Content-Length` artık tahsis buyruğu değil, sınır içinde doğrulanan protokol
  bilgisidir.
- Derin veya çok düğümlü JSON doğal yığın/bellek tükenmesine ulaşmadan durur.
- Geçersiz surrogate girdisi debug/release farkı olmadan `None` sonucudur;
  panic veya taşma yolu yoktur.
- K-129 açık belge sayısını 256, toplam belge belleğini 128 MiB ve LSP dışa
  giden yanıtını 8 MiB ile ortak `KaynakSinirlari` altında kapattı.
- K-131 çerçeve, JSON, açık belge ve outbound limitlerinin sayısal sahipliğini
  tek LSP görünümünde topladı. Yanıtın tahsis sırasında bütçeli üretilmesi
  K-132/B-025'te; duvar-saati/cancellation davranışı B-026'da açık tutulur.
- Üç çerçeve ve dört JSON testi tahsis öncesi boyutu, tekrar/eksik uzunluğu,
  derinlik/düğüm bütçesini ve Unicode olumsuzlarını korur.
