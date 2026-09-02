# ADR-050 — Gerçek LSP process cold-start ölçümü

- **Durum:** kabul; exact baseline kaydı bekleniyor
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-153, B-061

## Bağlam

K-148'in `lsp_soguk` metriği yalnız aynı süreçte `Sunucu::yeni()` ve
`initialize` çağrısını ölçüyordu. 541 ns'lik sonuç editörün gördüğü başlangıç
maliyeti değildi: işletim sistemi process spawn'ı, `dillsp` ikilisinin
yüklenmesi, stdin/stdout boruları, LSP `Content-Length` framing'i ve
capabilities yanıtının istemciye ulaşması ölçüm dışında kalıyordu. Adı gerçek
cold-start izlenimi vererek doğru veriye yanlış anlam yüklüyordu.

## Karar

1. Eski metrik sayıları değiştirilmeden `lsp_engine_initialize` adını alır;
   yalnız in-process engine kurulumunu ve initialize işlemeyi ifade eder.
2. Yeni `lsp_process_cold_start`, monoton saati process spawn çağrısından önce
   başlatır; gerçek `dillsp` ikilisini piped stdin/stdout ile açar, çerçeveli
   initialize isteğini yazar ve tam çerçeveli `id=1` capabilities yanıtı
   okunduğunda durdurur. Süreç temizliği ölçüm aralığının dışındadır.
3. Yanıt başlığı 8 KiB, gövdesi 8 MiB ile sınırlıdır. Eksik framing, UTF-8,
   kimlik veya capabilities kanıtı ölçümü başarısız yapar; yalnız process'in
   başlamış olması başarı değildir.
4. Koşucu `--dillsp YOL` ile exact ikiliyi alır; varsayılanı kendi yanındaki
   platforma uygun `dillsp` ikilisidir. CI önce release `dillsp` üretir ve bu
   yolu açıkça verir.
5. Mevcut LSP'de initialize sırasında workspace tarama/yükleme davranışı
   yoktur. Bu yüzden olmayan bir `workspace_load` metriği uydurulmaz; böyle
   bir lifecycle eklendiğinde ayrı kimlik ve iş yükü zorunludur.

## Reddedilen seçenekler

- **Eski 541 ns ölçümü cold-start saymak:** kullanıcı maliyetinin büyük
  bölümünü dışarıda bırakır.
- **`Command::output()` ile yalnız process çıkışını beklemek:** dillsp normalde
  uzun yaşayan sunucudur ve initialize-ready sınırını ölçmez.
- **Sahte stdio/aynı süreç adaptörü:** işletim sistemi yükleme ve framing
  maliyetini yeniden gizler.
- **Workspace yüklemesi varmış gibi sayı yayımlamak:** mevcut ürün davranışına
  dayanmayan ölçüm üretir.

## Sonuçlar

- Engine iç maliyeti ile editörün ilk capabilities yanıtına kadar gördüğü
  cold-start maliyeti ayrı trendlerdir.
- Gerçek iki ikili entegrasyon testi olcum→dillsp→stdio→capabilities yolunu her
  Tier-1 platformun faz matrisinde çalıştırır.
- Shared CI gözlemsel kalır. Exact temiz commit'te alınan ilk 25 örneklik
  taban ayrı provenance commit'iyle bu kararın baseline bekleyen kısmını
  kapatacaktır.
- Grammar, runtime semantiği, tanılar, RFC ve normatif spec değişmedi.
