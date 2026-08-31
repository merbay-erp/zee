# Manifesto ve değişmez ilkeler

Bu proje İngilizce programlama dillerinin anahtar kelimelerini Türkçeye çeviren bir
katman değil; Türkçenin doğal akışına göre tasarlanmış deterministik bir programlama
dili ve onun eksiksiz ekosistemidir.

**Temel cümle:** Çocukların kullanabileceği bir dil yapacağız; çocuk dili yapmayacağız.

## Değişmez ilkeler

1. **Türkçe-first.** İngilizce anahtar kelime zorunluluğu yok.
2. **Çeviri dili değil.** `if→eğer`, `function→fonksiyon` makyajıyla yetinilmeyecek;
   İngilizce fiil-önce kalıpları yerine Türkçenin **nesne→eylem** akışı esas alınır.
3. **Deterministik.** Aynı geçerli kaynak tek AST ve tek semantik anlam üretir.
4. **AI semantiğin parçası değildir.** AI kod üretebilir, açıklayabilir, hata çözümüne
   yardım edebilir; programın anlamını yalnız lexer, parser, type checker ve compiler
   belirler. Compiler hiçbir cümleyi tahmin etmez.
5. **Noktalama minimum, belirsizlik sıfır.** Girinti blok yapısını belirler; süslü
   parantez, noktalı virgül, `&&`, `||`, `!=` ile başlamak zorunlu değildir.
6. **Statik tür güvenliği + yerel tür çıkarımı** birlikte. Yeni başlayan tür yazmak
   zorunda kalmaz; public API'de açık tür kullanılır.
7. **Güvenli varsayımlar.** Null-safety (Seçenek türü), kaynak güvenliği (scope ile
   otomatik kapanış) ve structured concurrency varsayılandır.
8. **Hata mesajları Türkçe, öğretici ve eyleme dönüktür.**
9. **Çocuk dostudur ama oyuncak değildir.** Progressive disclosure: basit başla,
   güç azalmasın.
10. **Bağımsız çalışır.** Cloud, LLM veya tek ticari sağlayıcı zorunlu değildir;
    offline toolchain mümkündür.
11. **Tooling ürünün parçasıdır.** Formatter, LSP, test, paket, docs, debugger.
12. **Çekirdek küçük ve kararlı;** kütüphane katmanı daha hızlı evrilir.
13. **Self-hosting** uzun vadeli bağımsızlık hedefidir.

## Kontrollü Türkçe sınırı

v1 tam serbest doğal dil **olmayacaktır**. Kontrollü Türkçe kullanılır: iyelik ve hal
eklerinin desteklenen biçimleri grammar'da açıkça tanımlanır; morfolojik serbestlik
heuristic ile değil **sürümlemeli grammar** ile genişler. Böylece okunabilirlik
korunurken parser deterministik kalır.

## Dört soru süzgeci

Her yeni özellik şu dört sorudan geçmelidir:

1. Türkçe doğal mı?
2. Deterministik mi?
3. Öğrenilebilir mi?
4. Profesyonel ölçekte savunulabilir mi?

Dördünden biri **hayırsa** özellik yeniden tasarlanır.
