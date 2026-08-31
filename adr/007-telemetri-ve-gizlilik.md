# ADR-007 — Telemetri ve gizlilik: araçlar veri toplamaz

- **Durum:** kabul (31 Ağustos 2026)
- **Bağlam:** master plan bölüm 28; RFC-0001 (determinizm, güven);
  hedef kitle çocukları içerir (bölüm 3) — gizlilik çıtası en yüksek yerde.

## Karar

1. **Derleyici ve araçlar (`dil`, `dillsp`, playground) hiçbir veri
   toplamaz ve hiçbir yere göndermez.** Ağ erişimi yalnız kullanıcı
   programının kendisi istediğinde (`adresten getir`, `sunucu başlat`)
   ve yalnız programın hedefine olur. Bugünkü gerçekleme zaten böyledir;
   bu ADR onu söze bağlar.
2. **Playground'un sözü sayfada yazar:** "hiçbir şey internete gitmez."
   Kaynak kod, girdiler ve çıktılar tarayıcıdan dışarı çıkmaz; playground
   dosyası internetsiz çalışır (K-039).
3. **Çökme raporları** kendiliğinden gönderilmez. Bir iç hata (örn. T016)
   kullanıcıya "bildir" der; bildirim daima kullanıcının elidir.
4. **Gelecekte telemetri eklenecekse** şartları şimdiden sabittir:
   - yalnız **opt-in** (varsayılan kapalı, sessiz açılış yok);
   - şeması **herkese açık belgeli** ve RFC'den geçmiş;
   - kaynak kodu ve kişisel veri **asla** kapsama giremez;
   - çocuk/okul profillerinde tamamen devre dışı.
5. Paket kayıt sistemi (Faz 5) indirme istatistiği tutacaksa en az veriyle
   tutar; ayrıntısı ADR-006 (registry trust) ile birlikte karara bağlanır.

## Gerekçe

- Güven bu projenin ürünüdür: ebeveynin çocuğuna kurduğu araç, eve veri
  sızdırmaz. "Opt-out telemetri" seçeneği tartışmaya bile alınmadı çünkü
  varsayılanı açık olan hiçbir toplama, çocuk kullanıcıyla savunulamaz.
- Determinizm ilkesiyle de tutarlı: aracın davranışı, ağın veya bir uzak
  servisin durumuna bağlanamaz.

## Sonuçlar

- CI'a "derleyici ikilisi ağ sembolü içermez" türü bir denetim eklemek
  ileride düşünülebilir (şimdilik kod incelemesi + IO soyutlaması yeterli:
  ağ yalnız `GercekIo`'da ve yalnız dil kalıplarından tetiklenir).
- Eğitim/UX araştırması (usability oturumları) telemetri DEĞİLDİR: yüz yüze,
  açık rızalı ve kayıtları `docs/usability-sonuclari/` altında tutulur.
