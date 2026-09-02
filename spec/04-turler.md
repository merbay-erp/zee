# 04 — Türler

Normatif kaynak: RFC-0007, RFC-0008, RFC-0013 (üçü de geçici kabul).
Tanı kodları: T bölümü.

## Tür envanteri

TamSayı (i64) · Ondalık (keyfî katsayılı onluk: gövde × 10⁻ᵏ) · Metin · Mantıksal ·
Liste\<T\> · Sözlük\<Metin, T\> (ekleme sırası korunur) · Seçenek\<T\> ·
Sonuç\<T\> (hata tarafı daima Hata) · Hata · Tarih · Saat · Süre · yapı
türleri · görev/ağ yanıtı (yüzeyleri RFC-0011/golden 24-27'de).

Ondalık tek profesyonel onluk sayı türüdür; katsayı ve ölçek sabit bir makine
kelimesine sığmak zorunda değildir. Tam işlemler ve sonsuz bölme bağlamı
spec/16'da tanımlıdır.

## Değişmezlik ve çıkarım (TANIMLI)

- Tür bildirimi ifadelerde yazılmaz; tür ilk değerden çıkarılır. Yapı
  alanları istisnadır: public yüzeyde tür **ZORUNLU** yazılır (RFC-0007).
- Bir adın türü sonradan **değişemez** (T002); yeni anlam yeni ad ister.
- Bir listenin bütün öğeleri aynı türdendir (T011); sözlük değer türü
  sözlüğün türüne uyar (T021).

## Boş koleksiyon çıkarımı (TANIMLI — K-045)

`boş liste` / `boş sözlük` belirsiz öğe/değer türüyle doğar; **ilk**
ekleme/atama türü somutlar ve bağlamı günceller. Yapılar da liste öğesi
olabilir (K-060: kayıt tabloları); CSV satırları Metin değerli sözlüklerdir
(K-062). Belirsizken okuma
(`ilki`, gezme, `değeri`) derleme hatasıdır. Boş sabit, somut eşiyle
yeniden atamada iki yönde uzlaşır — bu T002 sayılmaz. Mantıksal bir ad
tek başına koşuldur: `hazır ise` (K-044); Mantıksal olmayan ad T005.

## Sayısal genişleme (TANIMLI)

TamSayı, Ondalık beklenen yerde kendiliğinden Ondalığa genişler — işlem
çağrısı dahil (Liste<TamSayı> → Liste<Ondalık> parametre, K-067); dar
ve geniş yerel çağrı kısıtları gövde/HIR denetiminden önce sıra-bağımsız
birleşir; skaler ve sayısal kapsayıcılarda nihai imza Ondalık tarafıdır. Bu v0
korkuluğu yalnız yerel başlangıç işlemlerinde geçerlidir. Birim/paket
işlemleri K-086 ile tam ve monomorfik açık imza taşır; çağrı onları terfi
ettiremez. Tersi
örtük DEĞİLDİR: Ondalıktan tam sayıya `tam kısmı` (sıfıra doğru) ya da
`yuvarlanmışı` (yarımlar sıfırdan uzağa) ile bilinçli inilir.

## İşlem imzaları (TANIMLI — K-083/K-086 progressive disclosure)

- `<ad> al` başlangıç biçiminde parametre türleri bütün erişilebilir çağrı
  kısıtlarından önce birleştirilir; gövde ve dönüş türü nihai imzayla
  denetlenir (K-121). Parametre sayısı/türü T015/T017'ye uymak ZORUNDADIR.
- `<ad> <Tür> olarak al` açık biçiminde bütün parametreler tanımın
  sözleşmesidir. İşlem hiç çağrılmadan gövdesi bu türlerle denetlenir; dönüş
  türü gövdeden o anda çıkarılır ve imza çağrıyla terfi etmez.
- Açık ve çıkarımlı parametre aynı işlemde karıştırılamaz (T037); bilinmeyen
  tür T038'dir. Tür yazımı: basit tür/yapı adı veya `T listesi`, `T sözlüğü`,
  `T seçeneği`, `T sonucu` kontrollü Türkçe biçimleridir.
- Açık Ondalık parametreye TamSayı (ve kapsayıcı eşleri) kayıpsız genişler;
  runtime değeri de dönüştürülür, statik/gerçek tür ayrışmaz.
- Birim/paket üzerinden dışa açılan her işlem bütün parametrelerini ve
  `<Tür> döndürür` / `değer döndürmez` satırını yazmak ZORUNDADIR (T039).
  Bildirim gövde dönüşüyle birebir uyuşur (T040–T042); v1 public modeli
  monomorfiktir ve kaynak ABI kuralları spec/10'dadır.

Özyinelemeli çağrının türü “o ana dek görülen dönüşlerden” çıkarılır; bu yüzden
temel durum özyinelemeli çağrıdan önce en az bir dönüş vermiş olmalıdır (T035).

## Dönüş birleşimi (TANIMLI — RFC-0008)

Bir işlemin dönüş kümesi birleşir:

- {T} → T
- {T, `yok`} → Seçenek\<T\> (yalnız `yok` **YASAK**: içi belirlenemez, T018)
- {T, `hatasını döndür`} → Sonuç\<T, Hata\> (kaynak gösterimi `Sonuç<T>`;
  yalnız hata **YASAK**, T018)
- İki farklı değer türü **YASAK** (T018).

Başarı dalları otomatik sarmalanır: `sayıyı döndür`, Sonuç\<TamSayı\>
işlemde başarı olarak sarılır.

Hata `kod/mesaj/neden/veri` alanlı birinci sınıf türdür. `sonucun hatası`
Hata üretir; alanlar ve üretim kuralları spec/15'tedir.

## Akış-duyarlı daraltma (TANIMLI — K-037)

`X varsa` / `X başarılıysa` / `X başarısızsa` kollarında ve bunların
`değilse` tersinmelerinde `X in değeri` / `X in hatası` erişimi güvenlidir.
Bu dalların DIŞINDA korumasız erişim **YASAK**tır (T036 — derleme hatası).
Daraltma bilinçli olarak dardır: tek koşullu kol + `değilse`; tam veri
akışı analizi yoktur (anlaşılabilirlik ilkesi). C008/C009 çalışma zamanı
kodları yalnız iç savunmadır; kullanıcı programı onları göremez.
