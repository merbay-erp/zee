# ADR-057 — Seçici düzeyinde çoklu test kapsamı

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-158, B-068

## Bağlam

Faz matrisi her testi tek bir birincil faza sahipletiyor ve fazların mimari
aşağı akışını gösteriyordu. Ancak LSP, playground, proje, golden ve semantic
regresyon gibi kritik test grupları tek çalıştırmada birden çok compiler
fazını gerçekten yürütür. Yalnız birincil sahiplik ve dosya adına bakmak bu
çapraz kanıtı görünmez bırakıyor; yalnız aşağı akış grafiğine bakmak ise hangi
test grubunun o yüzeyi doğruladığını söylemiyordu.

## Karar

1. `zee-faz-test-matrisi-2`, tekil birincil sahipliği değiştirmeden seçici
   düzeyinde isteğe bağlı `ek_kapsam` alanı taşır. Biçim
   `secici>faz,faz;secici>faz`dır.
2. Ek kapsam yalnız aynı satırda birincil sahipliği bulunan exact seçiciye ve
   bilinen faz kimliklerine bağlanabilir. Birincil fazı tekrar etmek, aynı
   seçiciyi/fazı yinelemek, boş veya bilinmeyen değer fail-closed hatadır.
3. Kanonik belge ve dinamik CI raporu, her değişen faz için birincil test
   gruplarını, onu çapraz kapsayan gerçek seçicileri ve mimari aşağı akışı ayrı
   sütunlarda gösterir. Ek kapsam test toplamını şişirmez ve sahipliği çoğaltmaz.
4. Metadata dosya adından türetilmez. Yalnız test grubunun yürüttüğü doğrulanmış
   yüzeyler açıkça yazılır; yeni kritik çapraz test aynı değişiklikte güncellenir.

## Reddedilen seçenekler

- **Bir testi birden çok birincil faza yazmak:** toplamı şişirir ve hata
  sahipliğini belirsizleştirir.
- **Aşağı akışı test kapsamı saymak:** mimari etkilenme olasılığı ile gerçekten
  yürütülen kanıtı birbirine karıştırır.
- **Dosya adından otomatik tahmin:** `playground_testi` veya `projeler_testi`
  gibi adların gerçek lexer→runtime kapsamını güvenilir biçimde çıkaramaz.

## Sonuçlar

- Değişiklik incelemesi hangi test gruplarının fazı doğrudan yokladığını görür.
- Birincil faz sayıları ve tam sahiplik kapısı aynı kalır.
- K-159 public facade çalışması, kullanıcı sözünü etkileyen test yüzeylerini
  daha doğru blast-radius bilgisiyle değerlendirebilir.
