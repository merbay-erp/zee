# ADR-166 — Liste sıra erişimi: dar sıra eki tokeni ve ödünç alan O(1) okuma

- **Durum:** kabul
- **Tarih:** 24 Kasım 2026
- **İlgili kayıt:** K-386, K-180, K-164/F010, K-179, K-355/F002, RFC-0029,
  RFC-0008 (Sonuç), RFC-0019 (değer semantiği), RFC-0021 (ifade katmanları),
  ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-074 (ürün sürtünmesi),
  spec/01, spec/17, spec/20

## Bağlam

Listede N'inci öğeye tek yol sayaçlı gezmeydi. Dört bağımsız kaynak (kanıt
özeti, kelime avı, ürün H, bağımsız karşılaştırma raporundaki ikili arama)
aynı sayaç döngüsünü yazdı; ikili arama O(log n) yerine O(n log n) çıktı.
K-180 syntax freeze sonrasına bırakılmıştı. 24 Kasım 2026 merkez kararı
(Mustafa) bunu `dogfood-change` istisnasıyla şimdi açtı. Core freeze altında
değişiklik en küçük güvenli kapsamda kalmalı ve önceki hiçbir programın
anlamını değiştirmemelidir.

## Karar

1. **Yüzey:** RFC-0029 aday C — `<liste>nin <sıra>. öğesi` ve
   `<liste>nin <sıra>. öğesini almayı dene` (Sonuç). Liste tamlayan ekli tek
   ad, sıra tek ad ya da TamSayı sabiti. 1 tabanlı.
2. **Sözcükleme sınırı:** Noktayı genel noktalama olarak açmak yerine tek dar
   `SiraEki` tokeni eklenir. Nokta yalnız önceki TamSayı/Kelime/Metin tokenine
   bitişik, aynı satırda ve ardından boşluk + `öğe…` kelimesi gelirse bu
   tokendir; aksi her nokta S001'dir. Bu karar iki riski kapatır: (a) eski
   programlar — o nokta önceden her yerde S001 olduğundan token dizileri
   aynıdır; (b) `3.14` ondalık karışıklığı — RFC-0013 önerisi korunur.
   Metin öncesine izin, `"1". öğesi` gibi hatayı S001 yerine öğretici T058'e
   taşımak içindir.
3. **AST:** iki ifade düğümü (`ListeOgesi`, `ListeOgesiDene`); erişim/postfix
   katmanında tam bölge kalıbı. Kalıba oturmayan sıra eki S049 ile durur;
   sessiz yeniden yorum yoktur (spec/20 "TANIMLI" kuralı).
4. **Tür:** sıra TamSayı, nesne öğe türü belli Liste (T058); sabit sıra < 1
   derleme hatası (T059). Sonuç türü öğe türü ya da `Sonuç<öğe>`.
5. **Çalışma zamanı ve O(1):** liste ifadesi ad ise değer ortamdan **ödünç**
   alınır, yalnız seçilen öğe `clone` edilir (spec/17 değer kopyası). Ad
   olmayan liste ifadesi genel yolla değerlendirilir (bugünkü parser yalnız
   ad üretir). Aralık dışı: `dene`siz C031 (sıra + adet + `almayı dene`
   önerisi, sıranın kesin konumu), `dene` ile `SIRA_DISI` kodlu Hata.
6. **Yazma açılmaz.** Ürün kanıtı yalnız okuma istiyor; yazma T053 kaynak
   sabitliğine yeni yol ve `olsun` ad konumuna grammar genişlemesi getirir.
   Girişim S049 ile gezme yazmasını önerir.
7. **Kapsam dışı:** hesaplanan sıra ifadesi ve iyelik zinciri (ara ad
   zorunlu), dilimleme, sondan sayma, sözlük sırası, kopya-üzerine-yazma.

## Değerlendirilen seçenekler

- **Genel `.` noktalaması / köşeli parantez:** spec/02 sembol yasağıyla
  çelişir, `3.14` belirsizliğini geri getirir. Reddedildi.
- **Yazıyla sıra (`üçüncü öğesi`):** `-inci` eki `zee-tr-1`'de yok; değişken
  sıra veremez. Reddedildi (yeni morfoloji eki gerekmedi).
- **Yalnız `almayı dene` (Sonuç zorunlu):** döngü sayacı gibi sınırı belli
  kullanımda gürültü; `ilki`/C007 emsaliyle çelişir. Reddedildi; iki biçim
  birlikte açıldı.
- **0 tabanlı:** `ilki` ve Türkçe "3. öğe" okunuşuyla çelişir; ürün
  sayaçlarındaki 0/1 karışıklığını sürdürür. Reddedildi.

## Sonuçlar

- Kanıt özeti ve kelime avı sayaç döngüleri kalktı (iki yardımcı işlem
  silindi); ürün çıktıları bayt bayt aynı; K-165 Rust/Go eşdeğerleri
  değişmedi.
- İkili arama ölçümü: n 400 kat büyürken arama süresi 1,7 kat (RFC-0029
  ölçüm tablosu).
- **Bilinen sınır:** listeyi işlem argümanı olarak geçmek spec/17 gereği O(n)
  kopyadır; sıra erişimi bunu değiştirmez. Ayrı performans işi.
- Yeni katman kenarı, modül ya da bağımlılık yoktur (katman fixture'ı
  değişmez); yeni güvenlik sınırı yoktur — sıra `usize::try_from` +
  `checked_sub` ile taşmasız, öğe kopyası mevcut değer bütçesine tabidir.

## Kanıt

- `compiler/tests/liste_sira_erisimi_testi.rs` — olumlu/olumsuz conformance.
- `compiler/tests/liste_sira_bellek_testi.rs` — sayan global ayırıcıyla O(1)
  kopya kanıtı (ödünç yolu kapatılınca erişim başına ~960 KB'ye çıkar ve test
  kırılır; doğrulandı).
- `golden/34-liste-sira-erisimi.dil`, `dogfood/kanit-ozeti/korpus/f010-sira-erisimi-cozumu.dil`.
- `scripts/liste-sira-olcumu.py` — gözlemsel ölçüm.
