# 10 — Dışa açık işlem sözleşmesi

Normatif kaynak: RFC-0006 §1/§5, RFC-0007 §2 ve RFC-0009 §2–§4.1.
Durum: **TANIMLI** (K-086).

## Progressive disclosure sınırı

Ana programın kendi yerel işlemi başlangıç biçimini kullanabilir:

```text
işlem iki katını bul
    sayıyı al
    sonucu sayı ile 2 nin çarpımı olsun
    sonucu döndür
```

Bu biçimde parametre ve dönüş türü gövdeden/ilk çağrıdan çıkarılır. Aynı işlem
birim ya da paket üzerinden başka kaynağa açıldığında ise tam sözleşme
**ZORUNLU**dur (T039):

```text
işlem iki katını bul
    sayıyı TamSayı olarak al
    TamSayı döndürür
    sonucu sayı ile 2 nin çarpımı olsun
    sonucu döndür
```

- Bütün parametreler `<ad> <Tür> olarak al` biçimindedir; açık ve çıkarımlı
  parametre karıştırılamaz (T037).
- Parametrelerden hemen sonra tam bir `<Tür> döndürür` satırı gelir. Değer
  üretmeyen işlem `değer döndürmez` yazar.
- Parametresiz public işlem de dönüş satırını yazmak ZORUNDADIR.
- Tür yazımı basit/yapı türü ya da `T listesi`, `T sözlüğü`, `T seçeneği`,
  `T sonucu` biçimidir. Bilinmeyen dönüş T040'tır.

## Gövde kanıtı

Açık işlem çağrılmasa bile gövdesi denetlenir. Bildirilen dönüş ile bütün
`döndür` dallarının birleşimi aynı olmak ZORUNDADIR (T041). Değer bildiren
işlemde olağan akışın gövde sonuna düşemediği muhafazakâr biçimde kanıtlanır;
eksik `değilse`/eşleştirme yolu T042'dir. `değer döndürmez` yazan işlem değer
ya da hata döndüremez.

## v1 tür ve uyumluluk modeli

v1 public işlemleri bilinçli olarak **monomorfiktir**. Her parametre ve dönüş
tek somut tür taşır; kullanıcı tanımlı generic/trait ABI'si v1 sözü değildir.
TamSayı'nın açık Ondalık (ve kapsayıcı eşleri) parametresine kayıpsız
genişlemesi bir çağrı uyarlamasıdır; imzayı değiştirmez.

v1'in kararlı sınırı kaynak ABI'sidir; ikili ABI/FFI kararlılığı verilmez.
Bir paket sürümünde şu değişiklikler kırıcıdır ve ana sürümü artırır:

- işlem adını, parametre sayısını/sırasını ya da parametre türünü değiştirmek;
- bildirilen dönüş türünü değiştirmek;
- public yapı alanını kaldırmak veya türünü değiştirmek.

Aynı imzayı koruyan gövde düzeltmesi yama; geriye uyumlu yeni işlem/yapı
eklemek küçük sürümdür. `proje.kilit` tam kaynak içeriğini ayrıca SHA-256 ile
sabitler; bu tekrar üretilebilirlik güvencesi, sürüm uyumluluğunun yerine
geçmez.

## Görünürlük

v1'de birim/paket girişindeki bütün işlem ve yapılar dışa açıktır; ayrı bir
`dışa aç`/`özel` işaretleyicisi yoktur. Bu nedenle tam imza zorunluluğu bütün
bu işlemlere uygulanır. Bir kaynağın `kullan` ile aldığı işlem kendi public
yüzeyine örtük olarak yeniden açılmaz; üst kaynak yalnız doğrudan tanımları
görür. İleride görünürlük veya açık re-export eklenmesi ayrı RFC ve kırıcı
grammar kararı ister.
