# ADR-039 — Hasım hosta karşı sürümlü WASM C ABI

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-142, B-055, V1-P1-10

## Bağlam

Eski playground ABI'si hosttan gelen pointer ve uzunluğu doğrudan
`slice::from_raw_parts` ile Rust dilimine çeviriyordu. Null/kayıt dışı/iç
pointer, yanlış uzunluk ve taşma Rust'ın güvenlik önkoşullarını ihlal
edebiliyordu. Geçersiz UTF-8 sessizce boş metne dönüyor; `Vec::from_raw_parts`
ile bırakma yanlış uzunlukta veya iki kez çağrıldığında allocator durumunu
bozabiliyordu. Sonuç başlığını okumadan önce hostun tampon boyunu doğrulayacağı
bir yüzey yoktu.

## Karar

1. Dışa aktarılan sözleşme `dil_abi_surumu() == 2` ile exact sürümlüdür. Host
   farklı sürümde çalıştırmaya başlamaz.
2. `dil_bellek_ayir(n)` sıfırla başlatılmış kutulu tamponu modül içi kayda
   alır. Sıfır/aşırı boyut, tahsis veya canlı kota hatası null döndürür. Boş
   metin yalnız `(null, 0)` çiftidir.
3. `dil_calistir` yalnız kaydın başlangıç pointer'ı, exact uzunluğu ve girdi
   türü eşleşince byte'ları sahipli kopyaya alır. Null+pozitif, iç/kayıt dışı
   pointer, sonuç tamponunun girdi olması ve her uzunluk uyuşmazlığı çekirdeğe
   ulaşmadan reddedilir. Pointer aritmetiğiyle host belleği okunmaz.
4. Kaynak ve soru girdisi ayrı ayrı strict UTF-8 olmak zorundadır. Hata boş
   programa çevrilmez; sahipli, uzunluk-önekli `WASM ABI HATASI` sonucu olur.
5. Sonuç kaydı girdi kaydından türle ayrıdır. Host önce
   `dil_sonuc_tamponu_uzunlugu(ptr)` ile kayıtlı toplamı alır, dört baytlık
   little-endian gövde uzunluğunun `toplam - 4` olduğunu doğrular ve strict
   UTF-8 çözer.
6. `dil_bellek_birak(ptr, n)` yalnız canlı başlangıç pointer'ı ve exact boy
   eşleşince sahipliği düşürüp `1` döndürür. Yanlış, kayıt dışı ve çift bırakma
   durumu değiştirmeden `0` döndürür. `(null, 0)` başarılı no-op'tur.
7. Tek tampon en çok merkezî metin bütçesi + dört bayt, aynı anda en çok sekiz
   tampon ve 64 MiB kayıtlı byte olabilir. Bu genel ABI tahsis zarfıdır;
   kaynak/girdi türüne özgü daha erken bütçe B-056'nın ayrı kararıdır.
8. Çağrı senkrondur. Paylaşımlı WASM belleği açılmaz; host çağrı sürerken
   tamponu değiştirmez. JavaScript'in bütün linear memory'yi yazabilmesi
   nedeniyle ABI aynı sayfadaki kötü niyetli JavaScript'e karşı bir gizlilik
   veya bütünlük sandbox'ı iddia etmez; amaç kayıt dışı verinin Rust
   güvenlik önkoşuluna dönüşmemesidir.

## Reddedilen seçenekler

- **Host sözleşmeye uyar varsayımı:** tarayıcı adaptöründeki tek hata ya da
  sürüm drift'i Rust tarafında tanımsız davranışa dönüşebilir.
- **Yalnız linear-memory sınır kontrolü:** adres aralıkta olsa bile tahsis
  başlangıcı, sahiplik, tür ve exact boy kanıtlanmaz; iç/stale pointer ayrılmaz.
- **UTF-8'i kayıplı veya boş kabul etmek:** saldırı ile gerçek boş programı
  gözlenebilir biçimde aynı yapar.
- **Sonuç boyunu yalnız sonuç içinden okumak:** host, başlığı okumadan önce dört
  baytın dahi canlı tahsise ait olduğunu doğrulayamaz.
- **Her çağrıda önceki sonucu örtük bırakmak:** sahipliği gizler, eşzamanlı
  tüketiciyi ve hata ayıklamayı kırar. Her tampon açıkça ve exact bırakılır.

## Sonuçlar

- ABI v1 hostu bilinçli olarak uyumsuzdur; depo playground'u v2 sürümünü
  açılışta doğrular.
- Kayıtlı tamponlar çekirdeğe geçmeden kopyalanır; geçersiz çağrıdan sonra aynı
  instance çalışmaya devam eder.
- Native regresyon matrisi ve ayrı `wasm_abi` libFuzzer hedefi pointer,
  uzunluk, UTF-8, sonuç başlığı ve yaşam döngüsünü sürekli zorlar.
- Pointer başarılı bırakmadan sonra geçersizdir. Genel allocator'larda olduğu
  gibi, daha sonra aynı sayısal adres yeni tahsise verilirse eski sayının
  nesne kimliği olduğunu iddia etmeyiz; host yaşam süresi sözleşmesine uyar.
