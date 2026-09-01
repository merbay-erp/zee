# ADR-012 — Parser, checker ve runtime için fiziksel faz modülleri

- **Durum:** kabul
- **Tarih:** 1 Eylül 2026
- **İlgili kayıt:** K-099, B-005, V1-P0-10

## Bağlam

Bootstrap derleyicinin parser, tür denetleyici ve yorumlayıcısı çalışır ve
geniş bir regresyon korpusuna sahiptir. Ancak üç kök dosya 2709, 3067 ve 3181
satıra; cümle/ifade handler'ları da tek fiziksel bağlama büyümüştü. Yeni bir
özellik çoğu zaman aynı dosyanın birkaç yüz satır daha uzamasına yol açıyor,
inceleme sınırını ve ilerideki backend/self-hosting geçişini zorlaştırıyordu.

Yalnız dosya adını değiştirmek yeterli değildir. Faz sahipliği görünür olmalı
ve kök dosyaların handler'ları yeniden içine alması otomatik testte durmalıdır.

## Karar

Derleyici şu fiziksel sınırlara ayrılır:

| Katman | Kök sorumluluğu | Handler modülleri |
|---|---|---|
| parser | token akışı, blok ve tanım orkestrasyonu | `ayristirici/cumle.rs`, `ayristirici/ifade.rs` |
| checker | tür modeli, bağlam ve denetim orkestrasyonu | `cozumleyici/cumle.rs`, `cozumleyici/ifade.rs`, `cozumleyici/cagri.rs` |
| runtime | IO, scheduler, değer modeli ve orkestrasyon | `yorumlayici/cumle.rs`, `yorumlayici/ifade.rs` |

Modüller `pub(super)` ile yalnız kendi üst fazına açılır; crate'in public
API'sini büyütmez. Bu taşıma davranış değişikliği değildir.

Kaynak-mimari testi iki kuralı otomatik korur:

1. büyük handler adları kök dosyalara geri dönemez;
2. kök ve handler dosyaları ilan edilmiş satır bütçesini aşamaz. Bütçeye
   sığmayan yeni alan kendi handler/modül sınırını açmak zorundadır.

Hata kataloğu bekçisi de `compiler/src` altındaki bütün Rust modüllerini
özyinelemeli ve sıralı tarar; tanılar alt modüle taşınınca görünmez olamaz.

## Değişmezler

1. Parser'ın RFC-0021 katman ve tam tüketim davranışı değişmez.
2. Checker'ın tanı kodu, kaynak konumu, çıkarım ve akış sonucu değişmez.
3. Runtime'ın değerlendirme sırası, kısa devre, IO ve scheduler izi değişmez.
4. Modül taşıma public Rust API ya da zee kullanıcı yüzeyi eklemez.
5. Satır bütçesi kalite ölçütünün tamamı değildir; yalnız yeniden birleşmeyi
   engelleyen mekanik alt sınırdır. Checker'ın semantik fazları B-006/K-100 ve
   ADR-013 ile ayrıca ayrılmıştır.

## Sonuçlar

- Kök parser 1160, checker 963, runtime 1965 satıra indi.
- Cümle, ifade ve çağrı değişikliklerinin sahipliği incelemede görünür oldu.
- Üç mimari sınır testi gelecekteki büyümeyi ayrı modüle zorlar.
- Hata kodu↔katalog birebirlik testi yeni alt dizinleri otomatik kapsar.
- Mevcut 393 davranış testi değişmeden, toplam 396 testle taşıma doğrulandı.
- Yeni normatif dil spec'i gerekmez; kullanıcı semantiği değişmemiştir.
- Checker'ın bu fiziksel sınır üstündeki semantik sahipliği ADR-013'e
  devredildi; checker kökü 143 satırlık orkestrasyona indi.
