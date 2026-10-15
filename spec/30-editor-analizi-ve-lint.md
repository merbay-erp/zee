# Editör analizi ve isteğe bağlı lint

Bu profil K-287 / ADR-145 ve RFC-0010'un editör dilimidir.

## Aralıklı eşitleme (TANIMLI)

`textDocumentSync` değeri 2'dir. Değişiklik aralıkları UTF-16 kod birimleriyle
yorumlanır; CRLF tek satır sonudur. Surrogate çiftinin ortası, ters veya belge
dışı aralık reddedilir. Bir paketteki değişiklikler sırayla özel aday üzerinde
uygulanır; herhangi biri geçersizse belgenin önceki metni korunmak ZORUNDADIR.

## Biçimlendirme (TANIMLI)

`textDocument/formatting` açık ve geçerli belge için kanonik biçimleyicinin
sonucunu döndürür. Aynı metinde boş düzenleme listesi, değişen metinde tek tam
belge düzenlemesi, kapalı veya bozuk kaynakta null üretir. Sunucunun bu işlemle
dosyaya yazması YASAKTIR.

## Analiz oturumu (TANIMLI)

Token önbelleği exact köken+metin anahtarıyla 64 girdi ve hesaplanan 8 MiB
tahsis bütçesiyle sınırlıdır. Bağımlılıksız yaprak AST önbelleği 64 girdi,
toplam 1 MiB kaynak/köken ve 128.000 kaynak tokenını aşamaz. AST/HIR bağları
ve tür denetimi sonuçları saklanmaz; paket grafiği, kilit ve gerçek yükleyici
her analizde yeniden denetlenmek ZORUNDADIR. Belge kapanınca önbellekler temizlenir.

## Editör metni ve paket sınırı (TANIMLI)

Kaydedilmemiş metin yalnız doğrulanmış ana projenin kendi yerel kaynağını
analizde değiştirebilir. Kökün altında bulunan ayrı paket de bağımlılıktır;
onun içeriğinin veya kilit hatasının editör metniyle örtülmesi YASAKTIR.
Çelişen URI alias'larında disk kaynağı korunur. Başarılı açma/değiştirme ve
kapatma sonrasında açık tüketici belgeler ayrıca bir düzenleme beklemeden
yeniden tanılanmak ZORUNDADIR. Açık envanter mevcut256belge/128MiB metin
sınırında kalır; toplu tanı yanıtı8MiB aşarsa -32001 ret üretir.

## Lint sonuçları (TANIMLI)

Lint CLI'da `denetle --lint`, LSP'de boolean `initializationOptions.lint` ile
açılır. Başarılı derlemenin uyarıları CLI JSON'da ayrı `uyarilar` dizisinde ve
başarı çıkış koduyla, LSP'de severity 2 ile gösterilmek ZORUNDADIR. Derleme
hatası varken lint tahmini yapılmaz. A012 hiç okunmayan semantic adı, S047
kanıtlanmış erişilemeyen cümleyi, S048 yapısal olarak eksik rota yanıt yolunu
bildirir. Yanıt göndermek akışı sonlandırmaz. İfade HIR bağlarında kullanıcı
işlem çağrısı varsa yanıt etkisi tahmin edilmez ve S048 program boyunca
bastırılır. Gölgeleme spec/03 gereği yoktur.

## Kapsam ve performans (AÇIK)

Oturumun yeniden kullanımı token ve bağımlılıksız ayrıştırma ile sınırlıdır;
artımlı HIR/tür denetimi yoktur. Açık kökler taze analizle yeniden tanılanır;
çok açık belge maliyeti tek belge ölçümüyle eşdeğer değildir. 20.081 satırlık üretilen proje ölçümü gerçek ürün korpusu yerine geçmez.
200 ms hedefi, yeni temiz exact SHA ölçümü ve uzun süreli RSS kanıtı birlikte
incelenmeden genel kabul olarak yazılamaz.
