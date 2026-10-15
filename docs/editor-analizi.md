# Editör analizi ve lint

LSP istemcisi UTF-16 aralıklı değişiklik gönderebilir. Biçimlendirme komutu
mevcut `dil biçimle` çıktısını önerir; düzenlemeyi editör uygular. Projede
`dil biçimle --denetle .` CI kontrolü kullanılmaya devam eder.

İsteğe bağlı uyarıları komut satırında açmak için:

```sh
dil denetle --lint uygulama.dil
dil denetle --lint --json uygulama.dil
```

JSON'da `tanilar` hataları, `uyarilar` uyarıları taşır. Yalnız uyarı varsa
çıkış kodu 0'dır. LSP başlatma seçeneklerine `{"lint": true}` eklenirse aynı
uyarılar severity 2 ile görünür. Varsayılan denetim uyarı üretmez.

| Kod | İnceleme |
|---|---|
| A012 | Ad hiç okunmuyor. Atama yeniden kullanma sayılmaz. Değer ifadesinin etkisini incelemeden satırı silmeyin. |
| S047 | Kesin dönüş/sonlandırma sonrası cümle veya literal koşulla erişilemeyen dal. |
| S048 | Rotanın yapısal olarak bazı yollarında yanıt/yönlendirme yok. İfade HIR bağlarında kullanıcı çağrısı varsa program boyunca bastırılır; çağrı etkisi tahmin edilmez. |

Zee'de iç blokta dış ada atama aynı değişkeni değiştirir; gölgeleme yoktur.
Uyarılar dosya ve semantic kapsam kimliğine göre ayrılır. Bağımlılığın aynı
satırdaki adı ana belgenin kullanılmayan adı sayılmaz.

Analiz değişmeyen tokenları ve bağımlılıksız birimlerin ayrıştırmasını yeniden
kullanır. Paket/kilit doğrulaması ve tür denetimi her seferinde çalışır.
Açık yerel birim kaydedilmeden analiz edilebilir; kilitli paketler diskten
doğrulanır. Bir birim açılınca, değişince veya kapanınca açık tüketici
belgeler otomatik yeniden tanılanır. Bu envanter mevcut256belge/128MiB
metin sınırındadır; toplu yanıt8MiB sınırını aşınca açık -32001 hatası verir. Ayrıntı ve ölçüm sınırları [ADR-145](../adr/145-artimli-editor-analizi-ve-lint.md)
ve [spec/30](../spec/30-editor-analizi-ve-lint.md) içindedir.

## Kritik işlev incelemesi

K-284/K-287 için on yeni işlevin dar tabanı bağımsız incelendi.
`lib` içindeki strict/recovery yükleyiciler farklı hata sözleşmelerini korur;
taze grafik/kilit ve birim yüklemesi cache tarafından atlanmaz. Lint ziyaretçisi
semantic kimlik ve köken kapsamını, eşitleme işlevi özel aday üzerinde sıralı
UTF-16 kontrolünü taşır. CLI test/advisory adaptörleri sonuç ve yürütme
sırasını yönetir. Kapsam envanterinin cümle/ifade ziyaretçileri tek enum
ağacını merkezi derinlik, kaynak ve karar bütçeleriyle dolaşır.

Eski64taban ve genel büyüme payları korunur. LSP/CLI yönlendiricilerinin
büyümesi ortak yayın ve biçim/test-köprü yardımcılarına ayrılarak sınırda
tutuldu. Kapsam, lint, LSP, native bridge ve mimari testleri bu sorumluluk
ayrımının olumlu/olumsuz kanıtıdır. İnceleme sonucu test çalıştırma veya
1800sRSS kabulü yerine geçmez.

## S3 yerel kabul kanıtı

[Exact S3 kabulü ve açık sınırlar](urunlesme-s3-kabul-20261015.md) gerçek canonical/native/IDE ve ortak kapıları bağlar; genel ürün/platform/yayın kabulü değildir.

## S4 kaynak kabulü

[S4 ayrı kaynak kaydı](urunlesme-s4-kabul-20261015.md) `4b7dd76ad5e10bdf1b16bded82b3ea20ad55a9e2`
üzerindeki güncel yerel kanıtı ve açık sınırları gösterir. Önceki S3 sonuçları
tarihsel kalır; genel ürün/platform/yayın veya tam P19 kabulü değildir.

## K-298 — açık metin dizini bakımı (kabul bekliyor)

Açık belgelerin gerçek dosya yolları ve tutarlı kaydedilmemiş metinleri bir
bildirim boyunca ortak dizinde paylaşılır. Çelişen URI alias'ı disk metnini
örtmez; proje/paket/kilit doğrulaması ve tanı sırası korunur. Dizin bildirimin
sonunda bırakılır; sonraki bildirim symlink hedeflerini yeniden çözer.

Bu bakım yeni kullanıcı ayarı veya kaynak göçü gerektirmez. Aynı stdio iş
yükünde yeni exact kaynak ölçümü ve kabulü beklenir; S4'ün lint açık/41 belge
p95 223,698 ms değeri tarihsel başlangıç gözlemidir. 200 ms hedefi, gerçek ürün/UI
gecikmesi ve tam P19 açık kalır. [Karar ve etki incelemesi](../adr/145-artimli-editor-analizi-ve-lint.md#k-298--bildirim-ömürlü-açık-metin-dizini).
