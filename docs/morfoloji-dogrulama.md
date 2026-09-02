# Morfoloji doğrulama rehberi

Bu belge K-111/B-016 ve K-122/B-008'in `zee-tr-1` için kalıcı kanıt haritasıdır. Morfoloji
kurallarını değiştirmez; RFC-0018 ve spec/13'teki üret→çöz, belirsizlik ve
Unicode sınırlarını daha geniş girdilerde sürekli sınar.

## Korunan değişmezler

1. Geçerli her tek ek ve `iyelik + dış ek` zinciri için kanonik üretimin
   çözümleri arasında özgün `(kök, ek-zinciri)` bulunur.
2. Bir yüzeyin kapsamdaki kök adayı yoksa A001, tam bir adayı varsa o ad,
   birden çok adayı varsa A002 üretilir. Sıra, sözlük veya olasılık seçimi yoktur.
3. NFC birleşik Türkçe harfler lexer'dan geçer. Aynı görünen ayrıştırılmış
   NFD yazım birleştirici im taşıdığı için S029 ile reddedilir; sessiz
   normalizasyon veya iki farklı kaynak yazımını tek ada eşleme yapılmaz.
4. Profil tablosu, yüzey sırası ve azami iki katman snapshot ile sabittir.
5. Tablo, 53.248 kanonik üretim+çözüm vektörü ve 11 ham sınır yüzeyi
   `morfoloji-zee-tr-1.sha256` kaydıyla sabittir; kayıt Git geçmişine
   girdikten sonra güncellenemez.

## Stable regresyon katmanı

`compiler/tests/morfoloji_testi.rs` her ana test koşusunda şunları yürütür:

- elle seçilmiş ses sınıfları ve ters ses değişimi korpusu;
- profil tablosunun bütün tek ve geçerli iki katmanlı ekleri;
- 4.096 deterministik üretilmiş, 2–32 kod noktalı geçerli Zee kökü;
- üretilmiş 2.048 yüzeyin bütün kök adaylarıyla ad çözümü;
- `ğ/ö/ş/â/İ` için NFC olumlu ve NFD→S029 olumsuzları;
- yapısal sonek çakışmaları ve bilinen `payı/sayacı/fiyatıyla/zarından`
  belirsizlikleri.
- çalışan profil kaydının immutable `zee-tr-1` SHA-256 fixture'ıyla byte
  eşitliği ve `dil morfoloji --uyumluluk` görünürlüğü.

Yayımlanmış fixture'ın tarihsel değişmezlik kapısı ve yeni `zee-tr-N` açma
protokolü [profil uyumluluk rehberindedir](morfoloji-profil-uyumlulugu.md).

## Mutation katmanı

`compiler/fuzz/fuzz_targets/morfoloji.rs`, her byte girdisini geçerli bir Zee
köküne dönüştürür. Girdinin seçtiği tek veya iki katmanlı zinciri üretir,
özgün soyut çözümün korunduğunu ve elde edilen bütün adayların A001/A002
politikasından kaçamadığını doğrular. Dört başlangıç tohumu gecelik CI cache'i
ile büyür; crash girdileri artifact olarak saklanır. Ortak komutlar ve küçültme
akışı [compiler fuzz rehberindedir](fuzzing.md).

İlk yerel smoke kampanyası sabit `nightly-2026-08-31` ve `cargo-fuzz 0.13.2`
ile 31 saniyede 527.966 girdi yürüttü; crash, panic veya property ihlali bulmadı.
Bu sürekli mutation kanıtıdır, bütün olası dizilerin biçimsel ispatı değildir.
