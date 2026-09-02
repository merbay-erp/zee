# 09 — Son tarih ve işbirlikli iptal

Normatif kaynak: RFC-0011 §2–§4. Durum: **TANIMLI** (K-085/K-133/K-134).

## `içinde / yetişmezse`

`<Süre> içinde` bloğu, giriş anı + süre olarak tekdüze saat üzerinde mutlak
bir son tarih kurar.

- Son tarih dolmadan biten gövdenin etkileri korunur; `yetişmezse` çalışmaz.
- Son tarih dolduğunda gövdenin kalanı çalışmaz, blokta doğan adlar düşer ve
  varsa yalnız o son tarihe ait `yetişmezse` kolu çalışır.
- `yetişmezse` bittikten sonra dış program olağan akışına devam eder.
- Kol yoksa iptal yönetilmiş sayılır ve dış akış devam eder.
- Saat son tarihe **eşitse** süre dolmuştur.

## İptal noktaları

zee iptali işbirliklidir. Runtime:

1. her blok ve cümle sınırında, dolayısıyla her döngü adımında;
2. kullanıcı işlemlerine yayılan iç bloklarda;
3. `bekle` ve ağ isteği gibi bekleme noktalarında

son tarihi denetler. `bekle`, kalan süreden uzun uyumaz; kalan kadar bekleyip
gövdeyi iptal eder. HTTP bağlantı, yazma ve okuma zaman aşımları kalan tek
bütçeden beslenir. Her socket okuması kalan mutlak süreyi yeniden hesaplar;
aralıklı veri gelişi deadline'ı ileri taşımaz. Kaynakta `içinde` yoksa native
HTTP istemcisi 30 saniyelik mutlak varsayılan son tarih ve başlıklar dahil
8 MiB wire yanıt sınırı kullanır. Süre dolmuş veya sınırı aşmış ağ yanıtı
değere ya da çıktıya dönüşemez.

K-133 ile çıktı/girdi, dosya, sunucu, web yanıtı/yönlendirmesi, çerez/oturum,
eyleyici, CSRF, parola doğrulama, rastgelelik ve `eylem` transaction başlangıcı
gibi dış etki veya pahalı işlem sınırları çağrının hemen önünde son tarihi
yeniden denetler. Görev HTTP isteğinden önce scheduler'a sıra verdiyse kalan
süre bu sıradan **sonra** yeniden hesaplanır; dolmuş görev adaptöre eski bir
süre taşıyamaz ve istek başlatamaz.

K-134'te bir web rota gövdesinin ilk yanıtı, oturum ve çerez mutation'larıyla
aynı istek transaction'ında bekler. Gövde deadline/runtime hatasıyla biterse
başarı yanıtı yayımlanmaz; session/cookie durumu istek başındaki sürüme döner
ve timeout ayrı 504 yanıtı olur. Başarı, yanıtın socket'e eksiksiz yazılmasıyla
commit edilir; yazma hatası da oturum değişikliğini kalıcılaştırmaz.

İptal gözlendikten sonra yeni çıktı, dosya yazma, çerez/yanıt veya eyleyici
etkisi başlatılamaz. Son tarih dolmadan tamamlanmış dış etkiler geri alınmaz;
`içinde` bir transaction değildir.

## İç içe son tarihler

Etkin son tarih en erken mutlak andır. Her son tarih benzersiz sahibini taşır:

- iç tarih önce dolarsa iç `yetişmezse` çalışır; dış süresi kalmışsa dış gövde
  devam eder;
- dış tarih önce dolarsa iptal iç koldan geçip dış sahibine ulaşır; iç
  `yetişmezse` yanlışlıkla çalışmaz.

## Sınır

İptal önleyici değildir. Tek bir kesintisiz yerel ifade ya da işletim
sisteminin iptal edemediği çağrı, bir sonraki işbirlikli noktaya kadar son
tarihi aşabilir; ardından gövde kesinlikle devam etmez. DNS çözümleme ve bazı
dosya sistemi çağrıları bu platform sınırındadır. K-090 scheduler'ındaki çocuk
görevler dış son tarihi miras alır; `bekle` noktası görev ağacını birlikte
iptal eder (spec/14). Çok çekirdekli önleyici paralellik verilmiş söz değildir.
