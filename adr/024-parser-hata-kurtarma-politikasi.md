# ADR-024 — Parser hata kurtarma ve LSP tanı bütçesi

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-113, B-021, V1-P0-23

## Bağlam

RFC-0010 çoklu tanıyı editör deneyimi için gerekli görüyordu. Parser kökünde
üst düzey bir hata yakalandığında satır ve olası gövde atlanabiliyordu; fakat
girintili bir blok içindeki hata `blok_ayristir` üzerinden ebeveyne kaçıyordu.
Bu durumda geçerli kardeş cümleler bloktan dışarı sızabiliyor, ebeveyn düğüm
kayboluyor ve parser'ın fiziksel `derinlik` sayacı yüksek kalabiliyordu.
Sonraki geçerli üst düzey işlem/yapı/test tanımı bu sızıntı yüzünden yanlış
S021 alabiliyordu.

Yapı alanı, `göre` kolu ve eşzamanlı görev gibi özel blok okuyucuları da ilk
bozuk satırda bütün ebeveyni düşürüyordu. LSP böylece kullanıcının tek satırlık
geçici yazım hatasından sonra sağlam kodu yanlış kapsamda görüp ikincil tanı
üretebiliyordu.

## Karar

Kurtarmalı parser iki güvenilir senkronizasyon noktası kullanır:

1. `SatirSonu`, hatalı cümlenin yatay sınırıdır.
2. Dengeli `Girinti`…`Cikinti`, yalnız hatalı başlığa ait alt gövdenin
   dikey sınırıdır.

Bir cümle hata verdiğinde parser kalan aynı satırı tüketir. İmleç hemen bir
`Girinti` üzerindeyse o dengeli alt ağaç atlanır; sonraki aynı-girintili kardeş
korunur. Blok ayrıştırma hatayı ebeveyne kaçırmaz, yerel tanı havuzuna yazar ve
aynı blokta sürer. Fiziksel `derinlik` her girilen blok için mutlaka geri
alınır.

Yapı alanları, `göre` kolları, eşzamanlı görevler, `değilse` ve `yetişmezse`
devamları aynı politikayı uygular. Kısmi AST yalnız normal parser'ın mümkün
saydığı düğümleri taşır: örneğin bütün görevleri bozuk eşzamanlı blok veya hiç
geçerli değer kolu kalmayan `göre`, boş sentetik düğüm üretmek yerine ana
tanısını döndürür.

Parser kurtarma sahipliği `ayristirici/kurtarma.rs` modülündedir. Çoklu tanı
hattı parser/checker/birim tanılarını `(satır, sütun, kod, mesaj)` anahtarıyla
kararlı kaynak sırasına koyar ve belge başına en çok 20 tanı yayımlar. Bu
bütçe CLI `denetle` ve LSP için ortaktır. Normal derleme/çalıştırma ilk tanıda
durma davranışını korur; kısmi AST yürütülebilir program değildir.

## Değişmezler

1. Hatalı iç cümle, sağlam kardeş cümleyi ebeveyn bloktan dışarı çıkaramaz.
2. Hata kurtarma parser `derinlik` durumunu sonraki üst düzey tanıma sızdıramaz.
3. Hatalı başlığın dengeli gövdesi atlanır; sonraki aynı-girintili cümle
   atlanmaz.
4. Özel blok okuyucusu tek bozuk satır yüzünden sonraki geçerli satırı yutmaz.
5. Kısmi AST, ADR-023 parser invariantlarından geçer ve yürütülebilir diye
   etiketlenmez.
6. Tanılar kaynak konumunda deterministik sıradadır ve toplam 20'yi aşmaz.
7. Lexer'ın token üretmeden durduğu lexical hata bu ADR kapsamında yeniden
   yazılmaz; o durumda tek güvenilir tanı döner.
8. Çoklu tanı hattında aynı eksik tanımın tekrarı gürültüdür (K-166/ADR-070):
   A001/A003/A007 aynı kök için (ekli biçimler dahil) yalnız ilk konumda
   yayımlanır; parser'ın düşürdüğü satırın baş sözü ile denetimi başarısız
   `olsun` tanımının adı kök nedeni raporlanmış sayılır; başlığı bozuk işlemin
   çağrıları T016 iç tutarlılık tanısı üretmez. Farklı kökler ayrı raporlanır.
9. Girintisiz kalan işlem/yapı/`göre`/döngü başlığının S007'si girinti
   önerisi taşır; gövde dışına düşen `sayıyı al` parametre satırı oturum
   kalıbı S043 yerine yol gösteren S004 üretir.

## Sonuçlar

- B-021 ve V1-P0-23 kapanır.
- Altı parser recovery ve bir gerçek LSP yayım regresyonu; iç kardeş, yapı
  alanı, kapsam derinliği, `göre`, eşzamanlı görev, tanı bütçesi ve kaynak
  sırasını korur.
- Parser kökü 1.200 satır bütçesinde kalır; recovery modülü ayrı 160 satır
  mimari bütçesine bağlanır.
- Tanı kodlarının sürümler arası kimliği ardıl K-114/ADR-025 fixture kapısıyla
  tamamlanmıştır.
