# 03 — Adlar ve kapsam

Normatif kaynak: RFC-0004/RFC-0018 (geçici kabul), günlük
K-011/K-013/K-034/K-041/K-089. Sürümlü tablonun ayrıntısı spec/13'tedir.
Tanı kodları: A bölümü.

## Morfolojik ad çözümü (TANIMLI)

Bir kelime, kapsamdaki adlarla şu adaylar üzerinden eşleştirilir:

1. **Kelimenin kendisi** (ek yok).
2. **Ek ayıklama:** desteklenen hâl/iyelik/araç ekleri sondan atılır.
   Desteklenen ek listesi `zee-tr-1` profilinin parçasıdır (K-089);
   kaynak-of-truth `compiler/src/morfoloji.rs: EK_TABLOSU`. Kapsam:
   zamir n'li biçimler (K-041: `zarı+ndan`), İKİ KATMANLI zincir
   (K-061: `fiyat+ı+yla`) ve ünsüz ikizleşmesi geri çevrimi
   (K-049: `üssü→üs`, sertleşmeyle `reddi→ret`).
3. **Ünsüz yumuşaması geri çevrimi:** `sayacı → sayaç`, `kitabı → kitap`,
   `yurdu → yurt`, `çocuğu → çocuk`.
4. **Ünlü düşmesi geri çevrimi:** `şekle → şekil`, `burnu → burun`.

Kurallar (TANIMLI):

- Adaylardan **tam bir tanesi** kapsamda bir ada denk gelirse çözüm odur.
- Hiçbiri denk gelmezse **A001** (tanı, kapsamdaki adları listeler).
- Birden fazlası denk gelirse **A002** — belirsizlik dilde hatadır; sessiz
  öncelik YOKTUR.
- Sözlük YOKTUR: çözüm yalnız kapsam adlarına bakar; bir kelimenin
  "Türkçede kelime olması" gerekmez.

## Parametre bağlama adı (TANIMLI — K-398)

`<ad> al` ve `<ad> <Tür> olarak al` satırında parametre, belirtme ekinin
yapısal olarak atıldığı gövdeye (`sayıyı → sayı`, `üssü → üs`) bağlanır.
Gövde ünsüz yumuşaması ya da ünlü düşmesiyle kalmış bir biçimse ve sözlük
kökü o yazımın `zee-tr-1` aday kümesindeyse parametre **sözlük köküne**
bağlanır (`kalıbı → kalıp`, `belirteci → belirteç`, `eşiği → eşik`,
`adedi → adet`, `içeriği → içerik`, `ucu → uç`, `metni → metin`,
`izni → izin`). Seçim profil çıktısını değiştirmez: `zee-tr-1` aday
kümesi, kanonik üretim ve uyumluluk kaydı aynıdır.

Geri çevrim kuralları (TANIMLI) alt çizgili adın son parçasına bakar ve
sözlüğe dayanmaz:

- sonu `b`/`c` ünlü ya da `l`/`m`/`n`/`r` sonrasıysa `p`/`ç` olur
  (`kalıb`, `uc`, `kalb`); `kayb` gibi iki adımlı biçim olduğu gibi kalır;
- en az iki heceli parçada ünlü sonrası `d` `t`, son `ğ` `k` olur
  (`aded`, `eşiğ`); `ng` `nk` olur (`reng`); tek heceli `ad`, `kod`, `ağ`,
  `dağ`, ünsüz sonrası `d` (`bildirild`) ve ünlü sonrası `g` (`katalog`)
  değişmez;
- son iki ünsüzün ikincisi `n`/`r` ve ilki `l`/`r` değilse ya da küme `ğz`
  ise önceki ünlünün dar eşi araya girer (`metn → metin`, `fikr → fikir`,
  `ağz → ağız`); `l`/`m` sonlu kümeler (`şekl`, `ism`, `film`, `izinl`) ve
  `modern` değişmez.

Uyumluluk (TANIMLI): K-398 öncesi program parametreyi yumuşamış gövdeyle
(`kalıb`, `metn`) yazmış olabilir. Doğrudan eşleşme yoksa bu yazım ya da onu
veren ekli biçim aynı parametreye çözülür; `kalıb … olsun` yeni ad doğurmaz,
parametreye atamadır. Yalnız işlem parametreleri için geçerlidir; daha önce
çözülen hiçbir başvurunun anlamı değişmez.

- Gövdede sözlük köküne (`başlık … olsun`) farklı türde değer verilmesi
  ZORUNLU olarak T002'dir ve öneri adın parametre olduğunu söyler (K-398
  öncesi bu ayrı bir yereldi).
- Parametre gövdede sözlük köküyle yeniden atandıktan sonra eski yazımla
  (`başlığ`) başvurmak ZORUNLU olarak A002'dir: iki K-398 öncesi değerden
  hangisinin kastedildiği sessizce seçilmez.

AÇIK (K-398): iki adımlı geri çevrim (`kaybı → kayıp`, `kaydı → kayıt`),
`l`/`m` sonlu ünlü düşmesi (`şekli`, `ismi`, `aklı`), `kt`/`fs`/`hs`
kümeleri (`vakti`, `nefsi`, `şahsı`) ve iyelik+belirtme çakışması
(`ip_adresini → ip_adresin`) eski bağlamayı korur; sözlük gerektirir ya da
`-lı` sıfatlarıyla (`izinli`, `resmi`) ayırt edilemez.

## İleri morfoloji (TANIMLI — K-072/K-089)

Çözümlemenin simetriği `ek_uydur`/`ek_zinciri_uydur` üreteçleridir; soyut eki
yeni köke Türkçe uyumla kanonik giydirir. Tek ve iki katmanlı üretim aynı
profil tablosundan gelir; `üret → çöz` kök+ek zincirini korur. Editörün
yeniden adlandırması bunun üstüne kuruludur. K-120'den itibaren editör önce
checker'ın `SymbolId` bağıyla tek kapsamı seçer; morfoloji yalnız o semantic
kimliğin yazımlarını yeniden giydirir. A002 veya başka derleme hatasında
metin benzerliğiyle hedef seçmek YASAKTIR.

## Örtük çoğul (TANIMLI — K-013)

`her sayı için` gezerken koleksiyon, tekil adın çoğulundan bulunur:
`sayı → sayılar/sayiler` uyumuyla. Bulunamazsa A003.

## Blok kapsamı (TANIMLI — K-034)

- Gövdede (`ise`, döngü, `işlem`, kol...) doğan ad **gövdeyle ölür**;
  döngü değişkeni dahil.
- Gövde içinde **dıştaki** ada `olsun` ile atama, dıştaki adı günceller ve
  kalıcıdır — gölgeleme yapısal olarak yoktur; tür değişmezliği (T002)
  aynen uygulanır.
- Çözümleyici ve yorumlayıcı birebir aynı kuralı uygular.

## Benzersizlik

İşlem adları benzersizdir (A005); yapı adları benzersizdir (A006);
birimlerden gelen adlar sessiz gölgelenmez (A008 — bkz. 07).
