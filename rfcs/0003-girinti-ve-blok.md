# RFC-0003 — Girinti ve Blok Modeli

- **Durum:** taslak (gerçeklenmiş davranışı belgeler; bir açık soru)
- **Tarih:** 31 Ağustos 2026
- **İlgili günlük kayıtları:** K-006 (döngüler), K-005 (koşullar); anti-örnek A06
- **İlgili golden programlar:** 05, 06, 12, 23, 25, 30 (iç içe bloklar)
- **Gerçekleme:** `compiler/src/sozcukleyici.rs` (girinti yığını),
  `compiler/src/bicimleyici.rs` (kanonik basım)

## Özet

Blok yapısını YALNIZ girinti belirler. Süslü parantez ya da `bitir/son` gibi
kapatıcı kelime yoktur ve olmayacaktır (manifesto 5; anti-örnek A06).

## Kurallar

1. **Girinti yalnız boşluk karakteriyle yapılır.** Sekme (tab) → **S003**
   ("Sekme yerine 4 boşluk kullan").
2. **Blok açılışı:** bir satırın girintisi, çevreleyen bloğunkinden büyükse
   yeni blok açılır (Girinti tokenı). Hangi cümlelerin blok beklediği ilgili
   cümle RFC'lerinde tanımlıdır; blok bekleyen cümleden sonra girinti
   gelmezse **S007**.
3. **Blok kapanışı:** girinti azaldığında, yeni girinti düzeyi daha önce
   açılmış düzeylerden BİRİYLE tam eşleşmelidir; eşleşmiyorsa **S005**
   ("girinti hizası önceki bloklardan hiçbiriyle uyuşmuyor"). Eşleşen düzeye
   kadar Cikinti tokenları üretilir.
4. **Boş satırlar ve yalnız-yorum satırları blok yapısını etkilemez.**
   Girintileri serbesttir; biçimleyici yorum satırını bir sonraki kod
   satırının hizasına çeker.
5. **Dosya sonu** açık tüm blokları kapatır.
6. **Kanonik girinti adımı 4 boşluktur.** Biçimleyici (`dil biçimle`) her
   düzeyi tam 4 boşlukla basar; korpusun tamamı kanoniktir.

## Belirsizlik analizi

Girinti yığını kuralı (3), Python'un bilinen "tutarsız dedent" sınıf
hatalarını derleme hatasına çevirir: hiçbir kaynak iki farklı blok ağacına
ayrıştırılamaz. Sekme yasağı (1), "görünüşte aynı hizada, bayt olarak farklı"
durumunu kökten kaldırır — determinizm ilkesinin (RFC-0001 §3) gereği.

## Açık soru

Sözcükleyici bugün blok açılışında 4'ten farklı ama TUTARLI artışları da kabul
eder (örn. hep 2 boşluk); biçimleyici bunları 4'e çevirir. Soru: kaynak
düzeyinde de 4 boşluk zorunlu olsun mu?

- **Zorunlu olsun:** tek doğru biçim, araçsız ortamda bile tutarlılık.
- **Serbest kalsın (mevcut):** çocuğun 2-3 boşlukla yazdığı program da çalışır;
  `dil biçimle` nazikçe düzeltir. Öğretici hata yerine öğretici araç.

Eğilim: mevcut davranış (serbest + biçimleyici) — usability oturumlarında
doğrulanacak, v1.0 spesifikasyon dondurmasından önce karara bağlanacak.

## Dört soru süzgeci

1. Doğal mı? — Evet: "içeride olan, içindedir" görsel sezgisi.
2. Deterministik mi? — Evet: S003/S005/S007 ile her sapma hatadır.
3. Öğrenilebilir mi? — Evet: tek kural, kapatıcı simge ezberi yok.
4. Savunulabilir mi? — Evet: formatter idempotentliği + C011 token güvencesiyle
   büyük kod tabanlarında da güvenli.

## Korpus etkisi

Yok — korpus kanonik 4-boşluk biçimindedir; `golden_korpus_idempotent` testi
30 dosyayı her derlemede doğrular.
