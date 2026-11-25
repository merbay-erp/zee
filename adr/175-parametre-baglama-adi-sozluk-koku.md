# ADR-175 — Parametre bağlama adı sözlük köküne bağlanır (profil değişmeden)

- **Durum:** kabul
- **Tarih:** 25 Kasım 2026
- **İlgili kayıt:** K-398, `urun-h` ZM-F030, `kanit-ozeti` K-164/F006,
  ADR-059 (core freeze), ADR-046 (katman yönü), ADR-067 (dogfood korpusu),
  RFC-0018 (sürümlü morfoloji), K-367 (şapkalı ünlü ölçümü), spec/03, spec/13

## Bağlam

Parametre satırı (`kalıbı al`, `metni Metin olarak al`) adı ayrıştırıcıda
yapısal olarak bağlar: belirtme eki atılır, ünsüz yumuşaması ve ünlü düşmesi
geri çevrilmez (`kalıb`, `belirtec`, `eşiğ`, `aded`, `metn`, `içeriğ`,
`uc`). Gövdede ekli başvurular (`kalıbın`) `zee-tr-1` aday kümesinden yine
çözülür, ama sözlük kökü (`kalıp`) A001 verir ve tanılar/editör yumuşamış
gövdeyi gösterir. `urun-h` bunu iki kez bildirdi (ZM-F030, 23 ve 24 Kasım
2026; ZeeMail a180e0c, `imap/kaynak/ana.dil` `kalıbı` yerine `deseni`);
`kanit-ozeti` aynı sürtünmeyi K-164/F006 olarak korpusa almıştı.

`zee-tr-1` profilinin değişmesi yasaktır (spec/13, RFC-0018; K-367 ölçümü
bir profil düzeltmesinin 53 248 kanonik üretimin 10 306'sını değiştirdiğini
gösterdi). Kullanıcı kararı (25 Kasım 2026): düşük riskli yol — profil
çıktısına dokunmadan bağlama adı seçimini değiştirmek.

## Karar

1. **Seçim aday kümesinden yapılır.** `morfoloji::baglama_koku` yapısal
   gövdeden (`yalin_ad`) sözlük kökünü kurallı olarak türetir; ayrıştırıcı
   (`parametre_adi`) bu kökü yalnız ekli yazımın `kok_adaylari` kümesinde
   ise seçer, değilse gövdeyi korur. `cozumleri_bul`, `kok_adaylari`,
   `ek_uydur`, profil dökümü ve uyumluluk kaydı (`e6034e73…`) değişmez.
2. **Kurallar sözlüksüzdür ve son ad parçasına bakar** (spec/03):
   `b`/`c` sonu ünlü ya da sonorant sonrası `p`/`ç`; iki+ heceli parçada
   ünlü sonrası `d`→`t`, `ğ`→`k`; `ng`→`nk`; ikincisi `n`/`r` olan yükselen
   küme ya da `ğz` için dar ünlü. Tek heceli `ad`/`kod`/`ağ`/`dağ`, ünlü
   sonrası `g` (`katalog`), `l`/`m` sonlu kümeler (`-lı` sıfatı, `resmi`,
   `film`) bilinçli olarak dışarıda bırakılır.
3. **Eski yazım uyumluluğu yalnız parametrelere uygulanır.** Doğrudan ve
   morfolojik eşleşme yoksa yumuşamış gövde (`kalıb`) ya da onu veren ekli
   biçim aynı parametreye çözülür; `kalıb … olsun` atamadır. Bu yol yalnız
   önceden A001 olan yazımı kabul eder; daha önce çözülen hiçbir başvurunun
   anlamı değişmez. Public `ad_cozumle` (HashMap yüzeyi) değişmez.
4. **Sessiz anlam değişimi yerine tanı.** K-398 öncesi `başlık … olsun`
   ayrı bir yereldi, artık parametreye atamadır: farklı türde T002 olur ve
   öneri adın parametre olduğunu söyler; parametre sözlük köküyle yeniden
   atandıktan sonra eski yazımla başvuru A002'dir.
5. **Katman:** `ayristirici → morfoloji` kenarı eklenir (ikisi de
   `sozdizimi`; morfoloji yalnız `guvenlik`e bağlıdır, çevrim yok).

## Ölçüm (uygulamadan önce ve sonra)

- Aday kümeleri: ZM-F030'un yedi adı ve 50 benzer ad için kök adayı kümesi
  sözlük kökünü içerir; `kaybı`, `kaydı` gibi iki adımlı biçimler içermez.
- Korpus: depo içi 353 ve yerel 21 Zee projesinin (ZeeMail dahil) 1 420
  `.dil` dosyasındaki 19 450 parametre satırının 752'si (53 farklı ad) yeni
  köke bağlanır (`başlık`, `kimlik`, `sonuç`, `metin`, `belirteç`, `izin`…).
  Kuralın ilk biçimi `-lı` sıfatlarını (`izinli` → `izinil`), geçmiş zaman
  bayraklarını (`bildirildi` → `bildirilt`) ve `kayb` → `kayp` biçimini
  bozuyordu; bu ölçümle daraltıldı.
- `dil denetle` eski/yeni karşılaştırması (103 proje + 1 670 dosya): tek fark
  `kanit-ozeti` F006 vakasının A001'den başarıya geçmesidir (depo içi ve
  ürünleşme kopyası). Yumuşamış gövdeyi çıplak yazan yüzlerce başvuru
  (`belirtec`, `yoğunluğ`) uyumluluk yoluyla aynı kalır.

## Sonuçlar

- `kalıbı al` gövdede `kalıp` yazılır; tanı, hover ve rename sözlük kökünü
  gösterir. Mevcut programlar değişmeden denetimden geçer.
- Nominatif yazılmış `-cı` adlar (`kullanıcı al` → `kullanıç`) ve ASCII adlar
  (`kullanici` → `kullaniç`) önceki gibi yanlış ama farklı bir gövdeye
  bağlanır; ekli/çıplak başvurular her iki yazımla çözülür. Belirtme yazımı
  (`kullanıcıyı al`) önerilen biçimdir.
- AÇIK: iki adımlı geri çevrim, `l`/`m` sonlu ünlü düşmesi, `kt`/`fs`/`hs`
  kümeleri ve iyelik+belirtme çakışması (`ip_adresini`) sözlük ya da profil
  değişikliği ister; `zee-tr-2` kararına kadar eski bağlama korunur.
