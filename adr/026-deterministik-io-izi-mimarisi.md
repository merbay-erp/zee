# ADR-026 — Deterministik IO trace/replay mimarisi

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-115, B-027, V1-P0-25

## Bağlam

Zee determinizmi “aynı program + aynı IO dünyası = aynı çıktı” diye tanımlar.
Testlerde `ToplayanIo` bu dünyayı hermetik kuruyordu; fakat gerçek bir CLI
koşusundaki klavye, rastgelelik, saat, dosya, ağ, sensör ve web olaylarını
sonradan aynı sırayla yeniden üretmenin sürümlü bir artefaktı yoktu.

Yalnız sonuçları kaydetmek yetersizdir: program değişip aynı sayıda IO çağrısı
yapsa yanlış argümanla sessizce ilerleyebilir. Yalnız girdileri kaydetmek de
çıktı ve dış etki sırasındaki sapmayı kaçırır. Formatın ayrıca bozuk/büyük
girdiye, sır sızıntısına ve yarım tüketilen replay'e karşı fail-closed olması
gerekir.

## Karar

`GirdiCikti` sınırının tamamı iki public sarmalayıcıyla izlenir:

- `IzKaydedenIo<T>`, gerçek veya hermetik adaptörü çağırır; işlem adı,
  argümanlar, sonuç ve küresel sırayı kaydeder.
- `IzYenidenOynatici`, başka adaptör kullanmaz; sıradaki kaydın işlem ve
  argümanlarını birebir doğrular, kayıtlı sonucu döndürür ve dış etkiyi
  yeniden uygulamaz.

Replay; dosya/ağ/oturum/transaction gibi etkileri tekrar gerçekleştirmez.
Çağrının aynı yerde, aynı argümanla yapılmasını doğrular. Konsol çıktısı,
istemler ve simülatör ışık izi kullanıcıya yeniden gösterilebilir; CLI bunu
ancak bütün iz hatasız ve eksiksiz tüketildikten sonra basar.

Şema-1 satır biçimi RFC-0022 ve spec/21'de normatiftir. Başlık sürümü, her
olayda kesintisiz sıra, bilinen işlem adı, argüman/sonuç alan sayıları ve küçük
harfli hex UTF-8 alanları vardır. Okuyucu 64 MiB, 100.000 olay ve olay başına
4.096 alan sınırını yürütmeden önce uygular; bütün işlem-özel sonuç şemalarını
da doğrular. Bilinmeyen sürüm/işlem, kanonik olmayan sayı/hex, sıra boşluğu,
argüman farkı ve tüketilmeyen olay hatadır.

`dil iz kaydet <iz> <program> [argümanlar]` gerçek koşuyu kaydeder ve izi
atomik yazar. `dil iz oynat <iz> <program>` dış dünyasız replay yapar. İz
kaynak dosyanın üzerine yazılamaz; `*.zee-io-izi` varsayılan olarak Git dışında
kalır.

İzler girdi, dosya/ağ içeriği, çerez ve belirteç taşıyabileceği için özel
artefakttır ve açık kullanıcı isteği olmadan üretilmez. Parola ve PHC özet
argümanları ham ya da yalnız hexlenmiş biçimde değil, SHA-256 parmak iziyle
kaydedilir. Bu, bütün izin gizli olmadığı anlamına gelmez.

## Değişmezler

1. Her olay kesintisiz ve 1 tabanlı küresel sıra taşır.
2. İşlem, argüman veya sıra farkı kayıtlı sonuçla ilerleyemez.
3. Replay sonunda tek bir olay bile artamaz.
4. Format ve işlem şeması yürütmeden önce bütçeli ve fail-closed doğrulanır.
5. Replay dış dosya, ağ, sensör, oturum veya transaction etkisi uygulamaz.
6. Trace kaydı varsayılan değildir; hassas artefakt olarak atomik yazılır.
7. Ham parola ve PHC özeti iz alanı olamaz.
8. İşlem sözlüğü ya da alan semantiğinin kırıcı değişimi yeni iz sürümü ister.

## Sonuçlar

- B-027 ve V1-P0-25 kapanır.
- Beş çekirdek ve iki CLI entegrasyon testi; byte snapshot'ı, tam program
  round-trip'i, sıra/argüman/tam tüketim, bozuk şema, sır parmak izi, kaynak
  değişimi ve kaynak üzerine yazma reddini korur.
- Runtime kökü yalnız modül/re-export taşır; 27 IO metodunun format ve
  sarmalayıcı sahipliği `yorumlayici/io_izi.rs` içindedir ve 1.250 satır
  mimari bütçeye bağlıdır.
- Saat/rastgele tohum ve sanal zaman ilerleme semantiğinin sürüm sözleşmesi
  ardıl B-028 işidir; şema-1 mevcut gözlenen sonuçları kaydeder.
