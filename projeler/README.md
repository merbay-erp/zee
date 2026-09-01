# Proje kitaplığı — oyna, boz, yeniden yap

Golden korpus dilin *belgesi*dir; burası dilin *oyun bahçesi*. Her proje
tek dosyadır, kolaydan zora sıralıdır ve bir çocuğun tek oturuşta bitirip
"şimdi şunu da eklesem?" diyeceği kadar küçüktür.

En kolay yol: `playground/zee-playground.html`'i çift tıkla aç, projeyi
yapıştır, Çalıştır'a bas. (Girdi isteyen projelerde cevaplarını Girdiler
kutusuna satır satır yaz.) Ya da: `dil çalıştır projeler/<dosya>.dil`.
Üç web demosu localhost'ta
`dil çalıştır --deneysel-web projeler/<dosya>.dil` ile açıkça çalıştırılır.
`girisli-panel` ayrıca HTTPS reverse proxy arkasında
`dil çalıştır --web-proxy https://panel.example ...` profilini gösterir.

| Proje | Öğrettiği | Fikir: şunu da dene |
|---|---|---|
| [carpim-tablosu.dil](carpim-tablosu.dil) | iç içe döngü, aralık | Yalnız tek sayıların tablosunu yazdır |
| [hikaye.dil](hikaye.dil) | sorma, metin birleştirme | Kendi masal kalıbını uydur |
| [quiz.dil](quiz.dil) | karşılaştırma, puan sayma | Soruları kendi derslerinden seç |
| [zar-oyunu.dil](zar-oyunu.dil) | rastgelelik, koşul zinciri | 3 el yerine "5'e ilk ulaşan" yap |
| [market-listesi.dil](market-listesi.dil) | ondalık para, işlem + test | İndirim işlemi ekle, testini yaz |
| [asal-sayilar.dil](asal-sayilar.dil) | iç içe döngü, bölümden kalan | 100'e kadar çıkar; ikiz asalları bul |
| [kumbara.dil](kumbara.dil) | "olana kadar", büyüyen harçlık | Haftalık zam yerine faiz dene |
| [gizli-dil.dil](gizli-dil.dil) | sözlük = şifre defteri | Çözücüyü de yaz (tersine defter) |
| [kelime-sayaci.dil](kelime-sayaci.dil) | sayaç sözlüğü, "yoksa sıfırla" | En çok geçen kelimeyi de bul |
| [gun-sayar.dil](gun-sayar.dil) | Tarih türü, gün aritmetiği | Doğum gününe kaç gün kaldığını hesapla |
| [mini-site.dil](mini-site.dil) | deneysel localhost sitesi: yöntemli rotalar + HTML | Yeni bir GET sayfa rotası ekle |
| [panel-not-defteri.dil](panel-not-defteri.dil) | CSRF korumalı form + POST adaptörü + yeniden kullanılabilir eylem | Aynı kaydetme eylemini CLI'dan çağır |
| [girisli-panel.dil](girisli-panel.dil) | Argon2id + sunucu oturumu/rol + CSRF + transaction'lı eylem | Demo parolasını kendi PHC özetinle değiştir |
| [envanter.dil](envanter.dil) | stok defteri: yapı listesi + JSON yedek | CSV yedeği de al; en pahalıyı bul |
| [roket.dil](roket.dil) | geri sayan aralık, bekleme | 10'dan başlat; kaçışta iptal ekle |

Hepsi regression testindedir (`compiler/tests/projeler_testi.rs`) — dil
değişirse bu projeler kırılamaz; kırılırsa CI söyler.

Yeni proje eklemek istersen: dosyanın başına `# Ad — tek cümle` ve
`# Öğretilen: ...` yorumlarını koy, testini `projeler_testi.rs`'e ekle.
