# Usability oturum kiti

Hedef: master planın Hafta 12 çıktısı — **10 öğrenci (8–14 yaş) + 5
profesyonel** ile korpusu sınamak. Bu kit, oturumu yürütecek kişiye (sana)
her şeyi hazır verir. Sonuçların bağlandığı karar: **RFC-0006 onay kapısı**
(K-032) + K-010/K-013 doğallık doğrulamaları.

## Oturum düzeni (kişi başı ~25 dakika)

1. **Isınma (2 dk):** "Bilgisayara Türkçe komut veren bir dil deniyoruz.
   Doğru cevap yok; takıldığın her yer bizim hatamız, senin değil."
2. **Sesli okuma (8 dk):** Aşağıdaki programları KAĞITTAN sesli okut.
   Kural: satırı önce okusun, sonra "sence bu ne yapar?" — cevabı YAZ.
3. **Tahmin görevleri (8 dk):** Program çıktısını tahmin ettir (aşağıda).
4. **Yazma görevi (5 dk):** Küçük bir işi dilde yazmayı DENESİN (kağıtla).
5. **Kapanış (2 dk):** "En garip gelen satır hangisiydi?" — birebir not al.

## Okutulacak programlar (sırayla)

| Sıra | Program | Neyi sınıyor |
|---|---|---|
| 1 | golden/01, 02, 05 | temel akış, ise/değilse (K-005) |
| 2 | golden/06, 08 | döngüler, örtük çoğul `her sayı için` (K-013) |
| 3 | **golden/12 + 14** | **K-016: `notlar için ortalamayı hesapla olsun` — ANA SORU** |
| 4 | golden/23, 32 | göre-eşleştirme, ondalık `3,14` (RFC-0013) |
| 5 | golden/07 | girdi + koşul zinciri (yalnız profesyonellere: 26 da) |

## K-016 özel protokolü (kritik)

Golden 12'deki çağrı satırını okuttuktan sonra iki kartı göster, hangisi
"daha doğal" sor ve NEDENİNİ yazdır:

- **Kart A:** `ortalama notlar için ortalamayı hesapla olsun`
- **Kart B:** `notlar için ortalamayı hesapla, sonucu ortalama olsun`

Sayım kuralı (önceden taahhüt — sonuca göre eğilme): 15 kişiden **10+**
B derse RFC-0006 revize edilir (B, dönüş değerli çağrılar için eklenir);
aksi halde geçici kabul (A) kesinleşir.

## Görev kartları

**Çocuk yazma görevi:** "Yaşını soran, 10'dan büyükse 'abisin/ablasın',
değilse 'kardeşsin' diyen programı yaz." (Beklenen kalıplar: diye sor,
yanıtın sayısı, ise/değilse.)

**Profesyonel yazma görevi:** "Bir liste sayının ortalamasını alan işlemi
tanımla ve çağır; sıfır bölme durumunu hatasını döndür ile ele al."

## Kayıt formu (kişi başı bir kopya)

```
Yaş/rol: ____   Tarih: ____
Sesli okumada takılan satırlar (birebir): ____
Yanlış tahmin edilen çıktılar (program + beklenen/dediği): ____
K-016 kartı: A / B — nedeni: ____
Yazma görevinde icat ettiği sözdizimi (ALTIN DEĞERİNDE — birebir): ____
"En garip satır": ____
Dört soru puanı (1-5): doğal __ / anlaşılır __ / tekrar ister mi __
```

## Sonuçların işlenmesi

1. Formları `docs/usability-sonuclari/` klasörüne tarih adıyla koy
   (`2026-09-XX-oturum-N.md`).
2. Her takılma bir günlük kaydına (K-0xx) dönüşür; kalıp icatları RFC
   alternatifi olarak kaydedilir.
3. K-016 sayımı RFC-0006'nın Durum satırına işlenir.

> İlk pilot için en doğru ilk katılımcı bellidir: dile adını veren kişi.
> "Merhaba! Bu zee projesi." satırını ilk o okusun.
