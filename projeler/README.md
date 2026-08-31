# Proje kitaplığı — oyna, boz, yeniden yap

Golden korpus dilin *belgesi*dir; burası dilin *oyun bahçesi*. Her proje
tek dosyadır, kolaydan zora sıralıdır ve bir çocuğun tek oturuşta bitirip
"şimdi şunu da eklesem?" diyeceği kadar küçüktür.

En kolay yol: `playground/zee-playground.html`'i çift tıkla aç, projeyi
yapıştır, Çalıştır'a bas. (Girdi isteyen projelerde cevaplarını Girdiler
kutusuna satır satır yaz.) Ya da: `dil çalıştır projeler/<dosya>.dil`

| Proje | Öğrettiği | Fikir: şunu da dene |
|---|---|---|
| [carpim-tablosu.dil](carpim-tablosu.dil) | iç içe döngü, aralık | Yalnız tek sayıların tablosunu yazdır |
| [hikaye.dil](hikaye.dil) | sorma, metin birleştirme | Kendi masal kalıbını uydur |
| [quiz.dil](quiz.dil) | karşılaştırma, puan sayma | Soruları kendi derslerinden seç |
| [zar-oyunu.dil](zar-oyunu.dil) | rastgelelik, koşul zinciri | 3 el yerine "5'e ilk ulaşan" yap |
| [market-listesi.dil](market-listesi.dil) | ondalık para, işlem + test | İndirim işlemi ekle, testini yaz |

Hepsi regression testindedir (`compiler/tests/projeler_testi.rs`) — dil
değişirse bu projeler kırılamaz; kırılırsa CI söyler.

Yeni proje eklemek istersen: dosyanın başına `# Ad — tek cümle` ve
`# Öğretilen: ...` yorumlarını koy, testini `projeler_testi.rs`'e ekle.
