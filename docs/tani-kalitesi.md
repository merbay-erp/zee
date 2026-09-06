# Tanı kalitesi raporu

<!-- `cd compiler && cargo run --locked --bin tani_kalitesi -- --rapor-yaz` üretir. Elle değiştirme. -->

K-166/ADR-070 kapısı: golden korpusuna deterministik acemi hatası mutasyonları uygulanır (244 vaka, 14 hata sınıfı). Sınıf = (operatör, ilk tanı kodu); en az 3 vakalı sınıflar frekans sırasıyla en çok 50 taneye kadar kapıya girer. Ölçütler: öneri %100, işaret hatalı satırda ≥ %90, gürültü medyan ≤ 2 ve tepe ≤ 4. “En sık” sıralaması gerçek kullanıcı verisi (K-161/K-162) gelene kadar bu korpusun frekansıdır.

| Sıra | Operatör | Kod | Vaka | İşaret | Öneri | Gürültü medyan | Gürültü tepe | Kapı |
|---:|---|---|---:|---:|---:|---:|---:|---|
| 1 | `tirnak-kapatma` | S002 | 30 | %100 | %100 | 1 | 1 | evet |
| 2 | `buyuk-harf` | S004 | 29 | %100 | %100 | 1 | 1 | evet |
| 3 | `ingilizce-print` | S004 | 29 | %100 | %100 | 1 | 1 | evet |
| 4 | `bos-blok` | S007 | 25 | %100 | %100 | 1 | 4 | evet |
| 5 | `sekme-girinti` | S003 | 25 | %100 | %100 | 1 | 1 | evet |
| 6 | `esittir` | S001 | 24 | %100 | %100 | 1 | 1 | evet |
| 7 | `girinti-sil` | S007 | 24 | %100 | %100 | 2 | 4 | evet |
| 8 | `olsun-eksik` | S004 | 23 | %100 | %100 | 1 | 2 | evet |
| 9 | `ad-yazim` | A001 | 19 | %100 | %100 | 1 | 2 | evet |
| 10 | `parantez` | S001 | 10 | %100 | %100 | 1 | 1 | evet |
| 11 | `nokta-ondalik` | S001 | 3 | %100 | %100 | 1 | 1 | evet |
| 12 | `ad-yazim` | A003 | 1 | %0 | %100 | 1 | 1 | seyrek |
| 13 | `girinti-sil` | S005 | 1 | %0 | %100 | 1 | 1 | seyrek |
| 14 | `olsun-eksik` | S015 | 1 | %100 | %100 | 1 | 1 | seyrek |

## İstisnalar

Yok.

## İhlaller

Yok.

## Vakalar

| Golden | Operatör | Beklenen satır | Kod | Satır | Öneri | Gürültü |
|---|---|---:|---|---:|---|---:|
| 01-merhaba-dunya.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 01-merhaba-dunya.dil | `ingilizce-print` | 5 | S004 | 5 | var | 1 |
| 01-merhaba-dunya.dil | `buyuk-harf` | 5 | S004 | 5 | var | 1 |
| 02-degiskenler.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 02-degiskenler.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 02-degiskenler.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 02-degiskenler.dil | `ad-yazim` | 9 | A001 | 9 | var | 1 |
| 02-degiskenler.dil | `ingilizce-print` | 9 | S004 | 9 | var | 1 |
| 02-degiskenler.dil | `buyuk-harf` | 9 | S004 | 9 | var | 1 |
| 03-girdi-alma.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 03-girdi-alma.dil | `esittir` | 6 | S001 | 6 | var | 1 |
| 03-girdi-alma.dil | `olsun-eksik` | 6 | S004 | 6 | var | 1 |
| 03-girdi-alma.dil | `ad-yazim` | 8 | A001 | 8 | var | 1 |
| 03-girdi-alma.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 03-girdi-alma.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 04-hesap-makinesi.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 04-hesap-makinesi.dil | `esittir` | 6 | S001 | 6 | var | 1 |
| 04-hesap-makinesi.dil | `olsun-eksik` | 6 | S004 | 6 | var | 1 |
| 04-hesap-makinesi.dil | `ad-yazim` | 11 | A001 | 11 | var | 1 |
| 04-hesap-makinesi.dil | `ingilizce-print` | 16 | S004 | 16 | var | 1 |
| 04-hesap-makinesi.dil | `buyuk-harf` | 16 | S004 | 16 | var | 1 |
| 05-kosullar.dil | `girinti-sil` | 8 | S007 | 7 | var | 4 |
| 05-kosullar.dil | `sekme-girinti` | 8 | S003 | 8 | var | 1 |
| 05-kosullar.dil | `tirnak-kapatma` | 8 | S002 | 8 | var | 1 |
| 05-kosullar.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 05-kosullar.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 05-kosullar.dil | `ad-yazim` | 7 | A001 | 7 | var | 1 |
| 05-kosullar.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 05-kosullar.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 05-kosullar.dil | `parantez` | 7 | S001 | 7 | var | 1 |
| 05-kosullar.dil | `bos-blok` | 7 | S007 | 7 | var | 4 |
| 06-donguler.dil | `girinti-sil` | 6 | S007 | 5 | var | 1 |
| 06-donguler.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 06-donguler.dil | `tirnak-kapatma` | 6 | S002 | 6 | var | 1 |
| 06-donguler.dil | `esittir` | 12 | S001 | 12 | var | 1 |
| 06-donguler.dil | `olsun-eksik` | 12 | S004 | 12 | var | 1 |
| 06-donguler.dil | `ad-yazim` | 13 | A001 | 13 | var | 1 |
| 06-donguler.dil | `ingilizce-print` | 6 | S004 | 6 | var | 1 |
| 06-donguler.dil | `buyuk-harf` | 6 | S004 | 6 | var | 1 |
| 06-donguler.dil | `bos-blok` | 5 | S007 | 5 | var | 1 |
| 07-sayi-tahmini.dil | `girinti-sil` | 9 | S007 | 8 | var | 1 |
| 07-sayi-tahmini.dil | `sekme-girinti` | 9 | S003 | 9 | var | 1 |
| 07-sayi-tahmini.dil | `tirnak-kapatma` | 9 | S002 | 9 | var | 1 |
| 07-sayi-tahmini.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 07-sayi-tahmini.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 07-sayi-tahmini.dil | `ad-yazim` | 12 | A001 | 12 | var | 1 |
| 07-sayi-tahmini.dil | `ingilizce-print` | 13 | S004 | 13 | var | 1 |
| 07-sayi-tahmini.dil | `buyuk-harf` | 13 | S004 | 13 | var | 1 |
| 07-sayi-tahmini.dil | `bos-blok` | 8 | S007 | 8 | var | 1 |
| 08-listeler.dil | `girinti-sil` | 14 | S007 | 13 | var | 2 |
| 08-listeler.dil | `sekme-girinti` | 14 | S003 | 14 | var | 1 |
| 08-listeler.dil | `tirnak-kapatma` | 9 | S002 | 9 | var | 1 |
| 08-listeler.dil | `esittir` | 6 | S001 | 6 | var | 1 |
| 08-listeler.dil | `olsun-eksik` | 6 | S004 | 6 | var | 1 |
| 08-listeler.dil | `ad-yazim` | 7 | A001 | 7 | var | 1 |
| 08-listeler.dil | `ingilizce-print` | 9 | S004 | 9 | var | 1 |
| 08-listeler.dil | `buyuk-harf` | 9 | S004 | 9 | var | 1 |
| 08-listeler.dil | `bos-blok` | 13 | S007 | 13 | var | 1 |
| 09-liste-isleme.dil | `girinti-sil` | 12 | S007 | 11 | var | 2 |
| 09-liste-isleme.dil | `sekme-girinti` | 12 | S003 | 12 | var | 1 |
| 09-liste-isleme.dil | `tirnak-kapatma` | 16 | S002 | 16 | var | 1 |
| 09-liste-isleme.dil | `esittir` | 6 | S001 | 6 | var | 1 |
| 09-liste-isleme.dil | `olsun-eksik` | 6 | S004 | 6 | var | 1 |
| 09-liste-isleme.dil | `ingilizce-print` | 16 | S004 | 16 | var | 1 |
| 09-liste-isleme.dil | `buyuk-harf` | 16 | S004 | 16 | var | 1 |
| 09-liste-isleme.dil | `bos-blok` | 11 | S007 | 11 | var | 1 |
| 10-sozlukler.dil | `girinti-sil` | 11 | S007 | 10 | var | 1 |
| 10-sozlukler.dil | `sekme-girinti` | 11 | S003 | 11 | var | 1 |
| 10-sozlukler.dil | `tirnak-kapatma` | 7 | S002 | 7 | var | 1 |
| 10-sozlukler.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 10-sozlukler.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 10-sozlukler.dil | `ad-yazim` | 7 | A001 | 7 | var | 2 |
| 10-sozlukler.dil | `ingilizce-print` | 11 | S004 | 11 | var | 1 |
| 10-sozlukler.dil | `buyuk-harf` | 11 | S004 | 11 | var | 1 |
| 10-sozlukler.dil | `parantez` | 10 | S001 | 10 | var | 1 |
| 10-sozlukler.dil | `bos-blok` | 10 | S007 | 10 | var | 1 |
| 11-metin-islemleri.dil | `girinti-sil` | 12 | S007 | 11 | var | 1 |
| 11-metin-islemleri.dil | `sekme-girinti` | 12 | S003 | 12 | var | 1 |
| 11-metin-islemleri.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 11-metin-islemleri.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 11-metin-islemleri.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 11-metin-islemleri.dil | `ad-yazim` | 7 | A001 | 7 | var | 1 |
| 11-metin-islemleri.dil | `ingilizce-print` | 7 | S004 | 7 | var | 1 |
| 11-metin-islemleri.dil | `buyuk-harf` | 7 | S004 | 7 | var | 1 |
| 11-metin-islemleri.dil | `parantez` | 11 | S001 | 11 | var | 1 |
| 11-metin-islemleri.dil | `bos-blok` | 11 | S007 | 11 | var | 1 |
| 12-islem-tanimi.dil | `girinti-sil` | 6 | S007 | 5 | var | 2 |
| 12-islem-tanimi.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 12-islem-tanimi.dil | `esittir` | 15 | S001 | 15 | var | 1 |
| 12-islem-tanimi.dil | `olsun-eksik` | 15 | S004 | 15 | var | 1 |
| 12-islem-tanimi.dil | `ad-yazim` | 18 | A001 | 18 | var | 1 |
| 12-islem-tanimi.dil | `ingilizce-print` | 19 | S004 | 19 | var | 1 |
| 12-islem-tanimi.dil | `buyuk-harf` | 19 | S004 | 19 | var | 1 |
| 12-islem-tanimi.dil | `bos-blok` | 5 | S007 | 5 | var | 1 |
| 13-islem-parametreleri.dil | `girinti-sil` | 6 | S007 | 5 | var | 2 |
| 13-islem-parametreleri.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 13-islem-parametreleri.dil | `tirnak-kapatma` | 10 | S002 | 10 | var | 1 |
| 13-islem-parametreleri.dil | `ingilizce-print` | 10 | S004 | 10 | var | 1 |
| 13-islem-parametreleri.dil | `buyuk-harf` | 10 | S004 | 10 | var | 1 |
| 13-islem-parametreleri.dil | `bos-blok` | 5 | S007 | 5 | var | 1 |
| 14-not-ortalamasi.dil | `girinti-sil` | 5 | S007 | 4 | var | 2 |
| 14-not-ortalamasi.dil | `sekme-girinti` | 5 | S003 | 5 | var | 1 |
| 14-not-ortalamasi.dil | `tirnak-kapatma` | 17 | S002 | 17 | var | 1 |
| 14-not-ortalamasi.dil | `esittir` | 14 | S001 | 14 | var | 1 |
| 14-not-ortalamasi.dil | `olsun-eksik` | 14 | S004 | 14 | var | 1 |
| 14-not-ortalamasi.dil | `ad-yazim` | 15 | A001 | 15 | var | 1 |
| 14-not-ortalamasi.dil | `ingilizce-print` | 17 | S004 | 17 | var | 1 |
| 14-not-ortalamasi.dil | `buyuk-harf` | 17 | S004 | 17 | var | 1 |
| 14-not-ortalamasi.dil | `parantez` | 18 | S001 | 18 | var | 1 |
| 14-not-ortalamasi.dil | `bos-blok` | 4 | S007 | 4 | var | 1 |
| 15-secenek-turu.dil | `girinti-sil` | 6 | S007 | 5 | var | 2 |
| 15-secenek-turu.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 15-secenek-turu.dil | `tirnak-kapatma` | 17 | S002 | 17 | var | 1 |
| 15-secenek-turu.dil | `esittir` | 13 | S001 | 13 | var | 1 |
| 15-secenek-turu.dil | `olsun-eksik` | 13 | S004 | 13 | var | 1 |
| 15-secenek-turu.dil | `ad-yazim` | 14 | A001 | 14 | var | 1 |
| 15-secenek-turu.dil | `ingilizce-print` | 17 | S004 | 17 | var | 1 |
| 15-secenek-turu.dil | `buyuk-harf` | 17 | S004 | 17 | var | 1 |
| 15-secenek-turu.dil | `parantez` | 16 | S001 | 16 | var | 1 |
| 15-secenek-turu.dil | `bos-blok` | 5 | S007 | 5 | var | 1 |
| 16-sonuc-ve-hata.dil | `girinti-sil` | 8 | S007 | 7 | var | 3 |
| 16-sonuc-ve-hata.dil | `sekme-girinti` | 8 | S003 | 8 | var | 1 |
| 16-sonuc-ve-hata.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 16-sonuc-ve-hata.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 16-sonuc-ve-hata.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 16-sonuc-ve-hata.dil | `ad-yazim` | 7 | A001 | 7 | var | 1 |
| 16-sonuc-ve-hata.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 16-sonuc-ve-hata.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 16-sonuc-ve-hata.dil | `parantez` | 7 | S001 | 7 | var | 1 |
| 16-sonuc-ve-hata.dil | `bos-blok` | 7 | S007 | 7 | var | 2 |
| 17-dosya-okuma.dil | `girinti-sil` | 8 | S007 | 7 | var | 2 |
| 17-dosya-okuma.dil | `sekme-girinti` | 8 | S003 | 8 | var | 1 |
| 17-dosya-okuma.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 17-dosya-okuma.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 17-dosya-okuma.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 17-dosya-okuma.dil | `ad-yazim` | 10 | A003 | 7 | var | 1 |
| 17-dosya-okuma.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 17-dosya-okuma.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 17-dosya-okuma.dil | `bos-blok` | 7 | S007 | 7 | var | 1 |
| 18-dosya-yazma.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 18-dosya-yazma.dil | `ingilizce-print` | 5 | S004 | 5 | var | 1 |
| 18-dosya-yazma.dil | `buyuk-harf` | 5 | S004 | 5 | var | 1 |
| 19-csv-analizi.dil | `girinti-sil` | 11 | S007 | 10 | var | 2 |
| 19-csv-analizi.dil | `sekme-girinti` | 11 | S003 | 11 | var | 1 |
| 19-csv-analizi.dil | `tirnak-kapatma` | 7 | S002 | 7 | var | 1 |
| 19-csv-analizi.dil | `esittir` | 7 | S001 | 7 | var | 1 |
| 19-csv-analizi.dil | `olsun-eksik` | 7 | S004 | 7 | var | 2 |
| 19-csv-analizi.dil | `ad-yazim` | 10 | A001 | 10 | var | 2 |
| 19-csv-analizi.dil | `ingilizce-print` | 16 | S004 | 16 | var | 1 |
| 19-csv-analizi.dil | `buyuk-harf` | 16 | S004 | 16 | var | 1 |
| 19-csv-analizi.dil | `bos-blok` | 10 | S007 | 10 | var | 1 |
| 20-json-verisi.dil | `tirnak-kapatma` | 6 | S002 | 6 | var | 1 |
| 20-json-verisi.dil | `esittir` | 6 | S001 | 6 | var | 1 |
| 20-json-verisi.dil | `olsun-eksik` | 6 | S004 | 6 | var | 1 |
| 20-json-verisi.dil | `ad-yazim` | 8 | A001 | 8 | var | 1 |
| 20-json-verisi.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 20-json-verisi.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 21-tarih-ve-saat.dil | `tirnak-kapatma` | 8 | S002 | 8 | var | 1 |
| 21-tarih-ve-saat.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 21-tarih-ve-saat.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 21-tarih-ve-saat.dil | `ad-yazim` | 8 | A001 | 8 | var | 1 |
| 21-tarih-ve-saat.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 21-tarih-ve-saat.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 22-yapilar.dil | `girinti-sil` | 6 | S007 | 5 | var | 3 |
| 22-yapilar.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 22-yapilar.dil | `tirnak-kapatma` | 10 | S002 | 10 | var | 1 |
| 22-yapilar.dil | `esittir` | 9 | S001 | 9 | var | 1 |
| 22-yapilar.dil | `olsun-eksik` | 9 | S004 | 9 | var | 1 |
| 22-yapilar.dil | `ad-yazim` | 10 | A001 | 10 | var | 1 |
| 22-yapilar.dil | `ingilizce-print` | 13 | S004 | 13 | var | 1 |
| 22-yapilar.dil | `buyuk-harf` | 13 | S004 | 13 | var | 1 |
| 22-yapilar.dil | `bos-blok` | 5 | S007 | 5 | var | 2 |
| 23-desen-eslestirme.dil | `girinti-sil` | 8 | S005 | 10 | var | 1 |
| 23-desen-eslestirme.dil | `sekme-girinti` | 8 | S003 | 8 | var | 1 |
| 23-desen-eslestirme.dil | `tirnak-kapatma` | 5 | S002 | 5 | var | 1 |
| 23-desen-eslestirme.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 23-desen-eslestirme.dil | `olsun-eksik` | 5 | S004 | 5 | var | 2 |
| 23-desen-eslestirme.dil | `ingilizce-print` | 9 | S004 | 9 | var | 1 |
| 23-desen-eslestirme.dil | `buyuk-harf` | 9 | S004 | 9 | var | 1 |
| 23-desen-eslestirme.dil | `bos-blok` | 7 | S007 | 7 | var | 1 |
| 24-http-istemcisi.dil | `tirnak-kapatma` | 6 | S002 | 6 | var | 1 |
| 24-http-istemcisi.dil | `esittir` | 6 | S001 | 6 | var | 1 |
| 24-http-istemcisi.dil | `olsun-eksik` | 6 | S004 | 6 | var | 1 |
| 24-http-istemcisi.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 24-http-istemcisi.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 25-web-sunucusu.dil | `girinti-sil` | 9 | S007 | 8 | var | 1 |
| 25-web-sunucusu.dil | `sekme-girinti` | 9 | S003 | 9 | var | 1 |
| 25-web-sunucusu.dil | `tirnak-kapatma` | 8 | S002 | 8 | var | 1 |
| 25-web-sunucusu.dil | `bos-blok` | 8 | S007 | 8 | var | 1 |
| 26-paralel-gorevler.dil | `girinti-sil` | 10 | S007 | 9 | var | 4 |
| 26-paralel-gorevler.dil | `sekme-girinti` | 10 | S003 | 10 | var | 1 |
| 26-paralel-gorevler.dil | `tirnak-kapatma` | 11 | S002 | 11 | var | 1 |
| 26-paralel-gorevler.dil | `ingilizce-print` | 24 | S004 | 24 | var | 1 |
| 26-paralel-gorevler.dil | `buyuk-harf` | 24 | S004 | 24 | var | 1 |
| 26-paralel-gorevler.dil | `bos-blok` | 9 | S007 | 9 | var | 3 |
| 27-zaman-asimi.dil | `girinti-sil` | 7 | S007 | 6 | var | 2 |
| 27-zaman-asimi.dil | `sekme-girinti` | 7 | S003 | 7 | var | 1 |
| 27-zaman-asimi.dil | `tirnak-kapatma` | 7 | S002 | 7 | var | 1 |
| 27-zaman-asimi.dil | `ingilizce-print` | 8 | S004 | 8 | var | 1 |
| 27-zaman-asimi.dil | `buyuk-harf` | 8 | S004 | 8 | var | 1 |
| 27-zaman-asimi.dil | `parantez` | 9 | S001 | 9 | var | 1 |
| 27-zaman-asimi.dil | `bos-blok` | 6 | S007 | 6 | var | 2 |
| 28-cli-araci.dil | `girinti-sil` | 7 | S007 | 6 | var | 1 |
| 28-cli-araci.dil | `sekme-girinti` | 7 | S003 | 7 | var | 1 |
| 28-cli-araci.dil | `tirnak-kapatma` | 7 | S002 | 7 | var | 1 |
| 28-cli-araci.dil | `esittir` | 4 | S001 | 4 | var | 1 |
| 28-cli-araci.dil | `olsun-eksik` | 4 | S004 | 4 | var | 1 |
| 28-cli-araci.dil | `ad-yazim` | 6 | A001 | 6 | var | 1 |
| 28-cli-araci.dil | `ingilizce-print` | 7 | S004 | 7 | var | 1 |
| 28-cli-araci.dil | `buyuk-harf` | 7 | S004 | 7 | var | 1 |
| 28-cli-araci.dil | `parantez` | 6 | S001 | 6 | var | 1 |
| 28-cli-araci.dil | `bos-blok` | 6 | S007 | 6 | var | 1 |
| 29-esp32-led.dil | `girinti-sil` | 6 | S007 | 5 | var | 2 |
| 29-esp32-led.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 29-esp32-led.dil | `parantez` | 5 | S001 | 5 | var | 1 |
| 29-esp32-led.dil | `bos-blok` | 5 | S007 | 5 | var | 2 |
| 30-testler.dil | `girinti-sil` | 7 | S007 | 6 | var | 2 |
| 30-testler.dil | `sekme-girinti` | 7 | S003 | 7 | var | 1 |
| 30-testler.dil | `tirnak-kapatma` | 11 | S002 | 11 | var | 1 |
| 30-testler.dil | `bos-blok` | 6 | S007 | 6 | var | 1 |
| 32-ondalik-market.dil | `girinti-sil` | 14 | S007 | 13 | var | 1 |
| 32-ondalik-market.dil | `sekme-girinti` | 14 | S003 | 14 | var | 1 |
| 32-ondalik-market.dil | `tirnak-kapatma` | 9 | S002 | 9 | var | 1 |
| 32-ondalik-market.dil | `esittir` | 5 | S001 | 5 | var | 1 |
| 32-ondalik-market.dil | `olsun-eksik` | 5 | S004 | 5 | var | 1 |
| 32-ondalik-market.dil | `ad-yazim` | 7 | A001 | 7 | var | 1 |
| 32-ondalik-market.dil | `ingilizce-print` | 9 | S004 | 9 | var | 1 |
| 32-ondalik-market.dil | `buyuk-harf` | 9 | S004 | 9 | var | 1 |
| 32-ondalik-market.dil | `nokta-ondalik` | 5 | S001 | 5 | var | 1 |
| 32-ondalik-market.dil | `parantez` | 13 | S001 | 13 | var | 1 |
| 32-ondalik-market.dil | `bos-blok` | 13 | S007 | 13 | var | 1 |
| 33-acik-islem-imzasi.dil | `girinti-sil` | 6 | S007 | 5 | var | 2 |
| 33-acik-islem-imzasi.dil | `sekme-girinti` | 6 | S003 | 6 | var | 1 |
| 33-acik-islem-imzasi.dil | `esittir` | 11 | S001 | 11 | var | 1 |
| 33-acik-islem-imzasi.dil | `olsun-eksik` | 11 | S015 | 11 | var | 1 |
| 33-acik-islem-imzasi.dil | `ingilizce-print` | 14 | S004 | 14 | var | 1 |
| 33-acik-islem-imzasi.dil | `buyuk-harf` | 14 | S004 | 14 | var | 1 |
| 33-acik-islem-imzasi.dil | `nokta-ondalik` | 12 | S001 | 12 | var | 1 |
| 33-acik-islem-imzasi.dil | `bos-blok` | 5 | S007 | 5 | var | 1 |
| hesap_araclari.dil | `girinti-sil` | 4 | S007 | 3 | var | 2 |
| hesap_araclari.dil | `sekme-girinti` | 4 | S003 | 4 | var | 1 |
| hesap_araclari.dil | `tirnak-kapatma` | 10 | S002 | 10 | var | 1 |
| hesap_araclari.dil | `nokta-ondalik` | 6 | S001 | 6 | var | 1 |
| hesap_araclari.dil | `bos-blok` | 3 | S007 | 3 | var | 1 |
