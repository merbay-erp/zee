// Üretilmiş dosya — scripts/oturum-programlari-uret.py; elle düzenleme.
window.OTURUM_PROGRAMLARI = [
 {
  "no": "01",
  "dosya": "golden/01-merhaba-dunya.dil",
  "kod": "# 01 — Merhaba Dünya\n# İlk milestone programı. Tek satır, tırnak dışında hiç noktalama yok.\n# Sınanan: metin sabiti + \"yaz\" (nesne→eylem akışı, K-002).\n\n\"Dünyaya merhaba\" yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "02",
  "dosya": "golden/02-degiskenler.dil",
  "kod": "# 02 — Değişkenler ve yazma\n# Sınanan: \"olsun\" ile değer bağlama (K-003), \"ile\" ile metin birleştirme (K-004),\n# Türkçe karakterli tanımlayıcılar (ç ğ ı İ ö ş ü).\n\nisim \"Ayşe\" olsun\nyaş 10 olsun\nşehir \"İzmir\" olsun\n\n\"Benim adım \" ile isim yaz\nisim ile \" \" ile yaş ile \" yaşında\" yaz\nisim ile \" \" ile şehir ile \" şehrinde yaşıyor\" yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "05",
  "dosya": "golden/05-kosullar.dil",
  "kod": "# 05 — Koşullar ve zincirleme\n# Sınanan: \"... ise / değilse\" (K-005), zincir: \"değilse <koşul> ise\",\n# karşılaştırma kalıpları: \"veya daha büyükse\" = >= (K-010).\n\npuan 85 olsun\n\npuan 90 veya daha büyükse\n    \"Pekiyi\" yaz\ndeğilse puan 70 veya daha büyükse\n    \"İyi\" yaz\ndeğilse puan 50 veya daha büyükse\n    \"Orta\" yaz\ndeğilse\n    \"Geçmedi\" yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "06",
  "dosya": "golden/06-donguler.dil",
  "kod": "# 06 — Döngüler\n# Sınanan: \"N kez tekrarla\", \"1 den 100 e kadar her sayı için\" (aralık),\n# \"olduğu sürece\" = while (K-006). Sayı sabitlerinde ekler ayrı yazılır: \"1 den\" (K-011).\n\n10 kez tekrarla\n    \"Merhaba\" yaz\n\n1 den 100 e kadar her sayı için\n    sayı çiftse\n        sayıyı yaz\n\nsayaç 3 olsun\nsayaç 0 dan büyük olduğu sürece\n    sayacı yaz\n    sayacı 1 azalt\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "08",
  "dosya": "golden/08-listeler.dil",
  "kod": "# 08 — Listeler\n# Sınanan: liste sabiti \"3, 7, 1, 9 listesi\" (K-012), \"ekle\",\n# \"adedi/ilki/sonu\" erişimleri, çoğuldan tekile gezinme:\n# \"her sayı için\" → gezilen koleksiyon \"sayılar\" (K-013).\n\nsayılar 3, 7, 1, 9 listesi olsun\nsayılara 5 ekle\n\n\"Adet: \" ile sayıların adedi yaz\n\"İlk: \" ile sayıların ilki yaz\n\"Son: \" ile sayıların sonu yaz\n\nher sayı için\n    sayıyı yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "12",
  "dosya": "golden/12-islem-tanimi.dil",
  "kod": "# 12 — İşlem tanımı (master plandaki örnek, bölüm 6)\n# Sınanan: \"işlem <eylem cümlesi adı>\", parametre bildirimi \"sayıları al\",\n# \"sonucu döndür\". İşlem ÇAĞRISI sözdizimi AÇIK KARARDIR (K-016).\n\nişlem ortalamayı hesapla\n    sayıları al\n\n    toplam 0 olsun\n    her sayı için\n        toplamı sayıyla artır\n\n    sonucu toplamı sayıların adedine böl\n    sonucu döndür\n\nnotlar 80, 90, 100 listesi olsun\n\n# AÇIK (K-016): aşağıdaki çağrı biçimi geçici öneridir.\nortalama notlar için ortalamayı hesapla olsun\nortalamayı yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "14",
  "dosya": "golden/14-not-ortalamasi.dil",
  "kod": "# 14 — Not ortalaması (v0.1 kabul programı)\n# Sınanan: işlem + liste + koşul birlikte; uçtan uca küçük program.\n\nişlem ortalamayı hesapla\n    sayıları al\n\n    toplam 0 olsun\n    her sayı için\n        toplamı sayıyla artır\n\n    sonucu toplamı sayıların adedine böl\n    sonucu döndür\n\nnotlar 70, 85, 90, 60 listesi olsun\nortalama notlar için ortalamayı hesapla olsun\n\n\"Ortalama: \" ile ortalama yaz\nortalama 50 veya daha büyükse\n    \"Geçtin\" yaz\ndeğilse\n    \"Kaldın\" yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "23",
  "dosya": "golden/23-desen-eslestirme.dil",
  "kod": "# 23 — Desen eşleştirme\n# Sınanan: \"X e göre / ... ise\" eşleştirme yüzeyi (K-021), varsayılan kol \"değilse\".\n# Exhaustiveness denetimi tür sistemine bağlanacak (master plan bölüm 8).\n\nşekil \"kare\" olsun\n\nşekle göre\n    \"daire\" ise\n        \"Köşesi yok\" yaz\n    \"kare\" ise\n        \"4 köşesi var\" yaz\n    \"üçgen\" ise\n        \"3 köşesi var\" yaz\n    değilse\n        \"Bu şekli tanımıyorum\" yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "32",
  "dosya": "golden/32-ondalik-market.dil",
  "kod": "# 32 — Ondalık market hesabı (korpus genişletmesi, RFC-0013)\n# Sınanan: bitişik virgül (3,14), onluk tam aritmetik (0,1+0,2=0,3 sözü),\n# tam kısmı / yuvarlanmışı, TamSayı→Ondalık örtük genişleme.\n\nkilo 2,5 olsun\nbirim_fiyat 19,99 olsun\ntutar kilo ile birim_fiyatın çarpımı olsun\n\n\"Tutar: \" ile tutar ile \" lira\" yaz\n\"Yuvarlak: \" ile tutarın yuvarlanmışı ile \" lira\" yaz\n\nkontrol 0,1 ile 0,2 nin toplamı olsun\nkontrol 0,3 e eşitse\n    \"Ondalıklar tam: sürpriz yok\" yaz\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "07",
  "dosya": "golden/07-sayi-tahmini.dil",
  "kod": "# 07 — Sayı tahmini oyunu (v0.1 kabul programı)\n# Sınanan: rastgele sayı, \"olana kadar tekrarla\" = until (K-006),\n# koşul zinciri + girdi + dönüşüm birlikte.\n\ngizli 1 ile 100 arasında rastgele sayı olsun\nbildi yanlış olsun\n\nbildi doğru olana kadar tekrarla\n    \"Tahminin?\" diye sor\n    tahmin yanıtın sayısı olsun\n\n    tahmin gizliden küçükse\n        \"Daha büyük söyle\" yaz\n    değilse tahmin gizliden büyükse\n        \"Daha küçük söyle\" yaz\n    değilse\n        \"Bildin!\" yaz\n        bildi doğru olsun\n",
  "yalniz_profesyonel": false
 },
 {
  "no": "26",
  "dosya": "golden/26-paralel-gorevler.dil",
  "kod": "# 26 — Eşzamanlı görevler (RFC-0011/spec-14 gerçeklemesi)\n# Revizyon (31 Ağu): master plandaki örnekte \"sonucu döndür\" tanımsızdı\n# (K-023). RFC-0011 §1 kuralı: eşzamanlı bloktaki her satır bir görev\n# bağlamasıdır; adlar \"hepsini bekle\"den SONRA sıradan değerdir, önce\n# erişim derleme hatasıdır (T033).\n# K-090: `hepsini bekle` görevleri kaynak sıralı tek-thread scheduler'da\n# ilerletir; bu iki saf görev bekleme noktası taşımadan aynı dilimde tamamlanır.\n\nişlem profili getir\n    kimi al\n    birleşik kim ile \" profili\" olsun\n    birleşiği döndür\n\nişlem faturaları getir\n    kimi al\n    birleşik kim ile \" faturaları\" olsun\n    birleşiği döndür\n\neşzamanlı olarak\n    profil \"Ayşe\" için profili getir\n    faturalar \"Ayşe\" için faturaları getir\n\nhepsini bekle\nprofil yaz\nfaturalar yaz\n",
  "yalniz_profesyonel": true
 }
];
