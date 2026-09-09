# ADR-074 — Üçüncü ürün: sürtünme tekrarı dil tasarımının sinyalidir

- **Durum:** kabul
- **Tarih:** 9 Eylül 2026
- **İlgili kayıt:** K-179, K-164, K-180, ADR-067, ADR-072, K-174

## Bağlam

K-164 ikinci ürünü kurup on altı sürtünme kaydetti; tek üründeki rahatsızlık
anekdottur, bağımsız ürünlerde tekrar eden rahatsızlık tasarım problemidir.
Syntax freeze penceresi (K-174, 6 Eylül–4 Ekim) içinde dile özellik eklenmez;
bu yüzden üçüncü ürünün amacı dili büyütmek değil, mevcut yüzeyin baskı
altında hangi sınıflarda tekrar takıldığını ölçmektir. İş yükü bilinçli olarak
öncekilerden farklı seçildi: web+PostgreSQL (K-163) ve toplu veri (K-164)
yerine stdin döngüsü, rastgelelik, harf işleme ve skor dosyasıyla çalışan
etkileşimli bir komut satırı oyunu.

## Karar

1. **Ürün.** `dogfood/kelime-avi` depo içi üçüncü üründür: beş harfli gizli
   kelimeyi altı denemede bulma oyunu. `proje.dil` yalnız
   `dosya-okuma`/`dosya-yazma` ister; üç birim (harf ipucu, sözlük seçimi,
   skor özeti) ve giriş `kaynak/ana.dil`; sözlük ve skor dosyası giriş
   klasöründedir (proje kipinde `../` yasağı, K-164/F009). 198 kaynak
   satırı, 7 birim testi. `dil çalıştır dogfood/kelime-avi [ad]` ile
   oynanır.
2. **Kanıt.** `kelime_avi_testi` ürünü gerçek sözlükle hermetik koşar:
   kazanma (ipucu, skor satırı, en iyiler özeti), kaybetme (kelime söylenir,
   skor yazılmaz), kısa tahminin hak yakmaması, önceki skorların özete
   girmesi, boş sözlükte çıkış kodu 2 ve bütün birim testleri.
   `dogfood_korpusu_testi` `proje` kipiyle ürünü kapsar (ADR-067/072).
3. **Sürtünme sınıflandırması.** Ürün yazılırken karşılaşılan her sürtünme
   önce K-164 kataloğuyla (F001–F016) karşılaştırılır: aynı tanı sınıfıysa
   K-164 kimliğiyle **tekrar** sayılır ve korpusa yeni çift eklenmez; yeni
   sınıfsa `K-179/FNNN` alır ve ret/çözüm çifti korpusa girer. Kılavuzla
   (K-164 ad seçimi kuralları) önceden önlenen sınıflar ayrıca sayılır.
4. **Tasarım sinyali kuralı.** İki bağımsız üründe tekrar eden sınıf backlog'a
   tasarım maddesi olarak açılır; freeze içinde uygulanmaz, freeze sonrası RFC
   adayıdır. Compiler'a bu ürün için değişiklik yapılmaz.

## Reddedilen seçenekler

- **F010 için freeze içinde sıra erişimi yüzeyi açmak:** K-174 penceresi
  yeni kalıp yasaklar; sinyal kaydedilir, karar takvimden sonra verilir.
- **Skor dosyasını TSV yapmak:** `\t` kaçışı olmadığından (K-164/F011) ham
  sekme gömmek yerine `|` ayraç seçildi; ürün için doğru, dil için not.
- **Oyunu insan usability oturumu saymak:** K-161/K-162 gerçek katılımcı
  ister; bu ürün geliştirici dogfood'udur, usability kanıtı değildir.

## Sonuçlar

- **Yeni sürtünme (2):** F001 iyelik ekli döngü adının örtük çoğulu (A003);
  F003 okunmuş metnin `satırları` yok, `"\n" ile parçaları` gerekir (T028).
- **Tekrar (4 sınıf):** K-164/F010 sıra ile öğe erişimi — bu üründe üç işlemde
  sayaç döngüsü (iki üründe toplam beş yer); K-164/F009 proje kipinde `../`
  (C012) — veri dosyaları giriş klasörüne alındı; K-164/F013 iyelikli adın
  `-ndeki` kaynağı (A001, `tahmin_harflerindeki`); K-164/F011 sekme kaçışı
  yokluğu skor ayracını belirledi.
- **Kılavuzla önlenen (5):** F001 test satırında postfix, F006/F007 parametre
  kökü, F008 tekil/çoğul çakışması, F014 postfix zinciri — ad seçimi kılavuzu
  uygulandığı için ret yaşanmadı.
- **K-164 düzeltmelerinin geri dönüşü:** birim dosyasındaki hatalar bu üründe
  ilk denemeden doğru dosya ve satırla göründü (ADR-072 kökenli tanı);
  düzeltme olmasaydı üç birimlik üründe hata avı ana dosyaya yanlış
  satırla düşecekti.
- **Açılan tasarım maddesi:** K-180 — liste sıra erişimi, freeze sonrası RFC
  adayı. F001/F013 morfoloji iyelik zinciri `zee-tr-2` adayına eklendi.
