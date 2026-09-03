# ADR-054 — Strict form URL kodlaması

- **Durum:** kabul
- **Tarih:** 3 Eylül 2026
- **İlgili kayıt:** K-176, B-065

## Bağlam

Web adaptörünün sorgu ve form çözücüsü eksik veya hexadecimal olmayan `%xx`
dizisini literal `%` olarak bırakıyor, yüzde çözümünden çıkan geçersiz UTF-8'i
ise replacement karakteriyle sessizce değiştiriyordu. Uygulamanın gördüğü ad
ve değer istemcinin gönderdiği baytlardan farklı olabildiği için doğrulama,
CSRF ve iş kuralı katmanları aynı girdiye ilişkin farklı anlam kurabilirdi.

## Karar

1. Sorgu ve `application/x-www-form-urlencoded` gövde alanlarının adları ile
   değerleri aynı strict çözücüden geçer.
2. `+` U+0020 SPACE olur. Her `%` işaretini tam iki ASCII hexadecimal hane
   izlemek zorundadır; eksik veya kuralsız kaçış HTTP 400'dür.
3. Yüzde çözümünden doğan bayt dizisi exact UTF-8 olmak zorundadır. Kayıplı
   dönüşüm, replacement karakteri ve kısmi kabul yasaktır; geçersiz UTF-8
   HTTP 400'dür.
4. Red, rota gövdesi ve CSRF/alan iş mantığı çalıştırılmadan görünür durum
   yanıtına dönüşür. İstek transaction'ı normal yanıt kapanışında tamamlanır.
5. Davranışın kalıcı kanıtı `web` kipli semantic regression fixture'ıdır;
   `fixed_by=32247c79321b5b350b18b15198a245776b22fc84` exact uygulama
   commit'ini gösterir ve aynı SHA semantic bugfix beyanında bulunur.

## Reddedilen seçenekler

- **Bozuk `%` işaretini literal kabul etmek:** istemci hatasını veri olarak
  saklar ve farklı URL çözücüleri arasında yorum ayrılığı doğurur.
- **`from_utf8_lossy` kullanmak:** farklı ham girdileri aynı görünür metne
  indirger; kimlik ve doğrulama sınırında kabul edilemez.
- **Yalnız gerçek TCP parser'ında doğrulamak:** yüzde kodlama HTTP framing
  değil form semantiğidir; hermetik adaptör de aynı davranışı taşımalıdır.

## Sonuçlar

- Geçerli Türkçe UTF-8 yüzde kodlaması ve `+` davranışı değişmez.
- Bozuk yüzde/UTF-8 girdisi artık sessiz veri bozulması yerine deterministik
  400 üretir; yeni compiler tanı kodu yoktur, hata kataloğu değişmez.
- Duplicate alan politikası bu kararla çözülmez ve ayrı güvenlik dilimidir.
