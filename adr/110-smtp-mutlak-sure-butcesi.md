# ADR-110 — Düz metin SMTP mutlak süre bütçesi

- **Durum:** kabul
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-252, GB-072

## Bağlam ve karar

Gerçek loopback native reproducer'da 200 ms bütçe, her 40 ms bir bayt veren
LF-siz yanıtta 3011 ms sürdü. Tek bağlantı ve C029 maskelemesi korundu;
işlem başına idle timeout toplam oturum bütçesini korumuyordu.

MIME yolundaki özel `SonTarihliTcp` parent eposta modülüne taşınır ve eski
plain yol da kullanır. Tek checked Instant son tarihi DNS çözümlemesinden
önce kurulur; bağlantı ve her TCP read/write kalan bütçeyle çalışır. TLS
ve STARTTLS aynı sarmalayıcıyı devralır. Timeout setter hataları yutulmaz.
Sıfır/overflow bütçe ağdan önce reddedilir. Public API, grammar, ileti
kodlaması, C029 maskesi ve otomatik tekrar yasağı değişmez.

## Kanıt ve sınır

`eposta::tests::smtp_yavas_yanit_mutlak_butceyi_yenileyemez` 200 ms çağrının
bir saniye tolerans içinde sabit maskeyle dönmesini; geçersiz bütçe testi
DNS/ağ öncesi reddi kanıtlar. [Minimal semantic fixture](../regression/security/smtp-yavas-yanit.dil)
aynı arızayı gerçek native sender ve tek bağlantıyla yürütür. Exact fixed_by
ve compiler/core-freeze beyanları kaynak commit'ini izleyen committe bağlanır.

Sistem resolver iptal edilemez; bu karar DNS dahil sert toplam wall-time
sözü vermez. TLS iç hesaplama, OS scheduler ve tamponlar için gerçek zaman
sözü yoktur. Deadline dolunca yeni read/write başlamaz; teslim belirsizliği
C029 olarak kalır. Cache/kalıcı tampon, yeni sahip veya bağımlılık eklenmez.
ADR-046/049 katman/ölçüm sözleşmeleri değişmez; benchmark tarihçesi üretilmez.

## Belge etki incelemesi

Spec23, güvenlik kaydı, backlog/günlük, sürümler, master plan ve sürüm kapısı
aynı dalgada güncellenir. RFC kararı/indeksi, hata kataloğu, API ve uyumluluk
yüzeyi, öğretici komutlar değişmez; yeni tanı ya da özellik yoktur. Tanı
kalitesi ve performans tarihçesi yeni sonuç gerektirmez; parser kurtarması
ve sabit ölçüm iş yükü değişmedi. README sayıları araçla yenilenir.

Ölçüm notu (K-267 test hazırlığı): macOS nativeTLS bağlam hazırlığı ile native
hata metninin oluşturulması eşzamanlı platform servislerini çağırabilir. Bunlar
Read/Write öncesinde veya ağ hatasından sonra süre tüketebilir; TCP
okuma/yazmaları tek kalan monoton son tarihi korur. IO iptal testi bu platform
hazırlığını ölçülen sessiz peer sınamasından önce tamamlar. Bu gözlem toplam
çağrı wall-time sınırı veya benchmark tarihçesi değildir.
