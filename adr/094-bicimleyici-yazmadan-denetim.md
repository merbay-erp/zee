# ADR-094 — Biçimleyicinin yazmadan denetim kipi

- **Durum:** kabul
- **Tarih:** 14 Ekim 2026
- **İlgili kayıt:** K-229, ürünleşme C2

## Karar ve kabul

`dil biçimle --denetle <dosya|proje>` mevcut kanonik biçimleyiciyi çağırır;
uygun kaynakta exit 0, biçim farkında exit 1, eksik/yanlış CLI kullanımında
exit 2 verir. Kaynak içeriği, mtime ve izinler değiştirilmez. Default biçimleme
yazmaya devam eder. Proje kipi önce bütün uygun kaynakları bellekte hazırlar,
mevcut yol sırası/gizli-hedef klasörü/symlink atlama sınırını korur; bir
kaynak hatalıysa yazma yapılmaz. Denetim kipi atomik_yaz'a ulaşmaz.

Sekiz gerçek CLI kabulü dosya ve proje fark/uygunluk, readonly/mtime,
parse hatası, source yolu, default writer ve bilinmeyen JSON seçimini sınar.
Bu mod CI'a uygundur; lint, artımlı LSP, JetBrains veya editör dağıtım kanalı
bu değişiklikle tamamlanmış değildir. Yeni komut/kalıp/koşul yok; K-174
fingerprint değişmez. Yeni seçenek uyumluluk profil kaydı taşır.

## Etki incelemesi

Normatif spec/07, mevcut RFC-0009/RFC-0010 CLI ve proje çerçevesi geçerlidir.
Owner/edge/SCC/API/ABI/dependency değişmez; yeni kalıcı durum veya cache yok.
Kaynak okuma/bellek hazırlığı mevcut biçimleyici iş yükünü korur; denetim
kipi dosya yazma işini çıkarır, performans tarihçesi veya soak tabanı
üretilmez. Tanı kataloğu mevcut kaynak tanılarını kullanır; yeni tanı kodu
ve parser recovery yoktur. Belge/sürüm/backlog/faz/uyumluluk ve exact
compiler/core-freeze beyanı aynı toplu değişikliktedir. A1 dirty flag'i bütün
kaynak checkout'una conservative uygulanır; gömülü/vendored build girdileri
izlenir (ADR-093), clean release betiği ayrıca bütünüyle temiz ağaç ister.
