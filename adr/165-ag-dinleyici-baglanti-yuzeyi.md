# ADR-165 — Adlı dinleyici bağlantı yüzeyi ve bağlantı scheduler'ı

- **Durum:** kabul
- **Tarih:** 23 Kasım 2026
- **İlgili kayıt:** K-381, K-376 (A1), K-355 (`urun-h`), RFC-0039 Aşama A2,
  ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-162 (adlı devir),
  ADR-011 (intrinsic sınırı), spec/12 §9, spec/23, spec/09, spec/14,
  K-372 (yazım bütçesi), K-375 (son tarih), GB-157

## Bağlam

Etkin dogfood ürünü `urun-h` (e-posta sunucusu) protokol motorunu Zee'de
yazmak istiyor. K-376 (A1) on adlı soketi systemd'den adla devralıp
doğruluyor ve tutuyordu; Zee kodu bağlantı kabul edemiyordu. Ürünün saf SMTP
durum makinesi stdio deneme aracıyla sınırlıydı (`"" diye sor` her okumada
istem basar, F010) ve dinleyici kalıbı S036 veriyordu (F009). 23 Kasım 2026
merkez kararı ("Sırayla aç") RFC-0039 A2'yi ZeeMail'in ikinci adımı yaptı.

Yorumlayıcı zaten future tabanlıdır: `eşzamanlı olarak` görevleri tek iş
parçacıklı, deterministik bir scheduler'da, `bekle` noktasında askıya
alınarak ilerler (spec/14). Engellemeyen ağ beklemesi aynı mekanizmanın
dar bir genişlemesiyle kurulabiliyordu.

## Karar

1. **Yüzey mevcut aileler içinde.** `"<ad>" dinleyicisinde bağlantı
   geldiğinde` rota kalıbının bağlantı karşılığıdır (üst düzey olay kaydı,
   taze ortam, örtük `bağlantı` sözlüğü). Okuma ve yazma `dene` Sonuç
   ailesindedir (`bağlantıdan satır okumayı dene`, `bağlantıya X satırını
   yazmayı dene`); ergonomi için `bağlantıya X satırını yaz` cümlesi ve
   `bağlantıyı kapat` eklendi. Yeni anahtar kelime, değer türü ya da koşul
   grameri yoktur; beş yeni kalıp kelimesi K-167 ve syntax freeze kaydına
   girdi. Bağlantı değer olarak taşınmaz: kalıplar o anki görevin
   bağlantısına uygulanır, böylece işlemlere bölünmüş protokol kodu çalışır.
2. **Hata sınıflandırılmış C030 Sonuç'tur.** `tür` Hata verisi `kapandı`,
   `zaman_aşımı`, `satır_uzun`, `satır_sonu`, `kodlama`, `kapanıyor`,
   `yazılamadı` değerlerinden biridir; protokolün doğru yanıtı (500/421)
   Zee'nin kararıdır. Son tarih Ç001'dir ve `yetişmezse` kolu çalışır
   (K-375 modeli).
3. **Satır kuralı çalışma zamanı değişmezidir.** Tek satır sonu CRLF; çıplak
   CR/LF ve karışık diziler normalleştirilmeden reddedilir (SMTP smuggling),
   uzun satır kırpılmaz, geçersiz UTF-8 Zee'ye sızmaz. Kural tek bir saf
   modülde (`dil::ag_satiri`) durur; gerçek reaktör ve hermetik adaptör
   aynı okuyucuyu kullanır.
4. **Zaman mutlaktır.** Okuma ve tıkanan yazma çağrının başından
   `bağlantı_boşta_süresi` içinde tamamlanmalıdır; damla damla gelen bayt
   süreyi uzatmaz (slowloris; K-372 ilkesi). Görevin tamamı
   `bağlantı_ömrü` son tarihine bağlıdır.
5. **Bağlantı scheduler'ı.** Her bağlantı bir görevdir; host okuma/yazma
   ilerleyemeyince `Bekle` döner, görev askıya alınır, scheduler hostun
   `poll(2)` tabanlı olay beklemesine bekleyen görevleri ve en yakın
   uyanma anını verir. Görev başına son tarih yığını ve çalışma bütçesi
   ayrı tutulur; her başarılı satır bütçeyi yeniler. Görev hatası yalnız
   kendi bağlantısını kapatır (**yalıtılmış bağlantı kapsamı**, spec/14 §3'ten
   bilinçli ayrılış). İç içe `eşzamanlı` görevler bağlantı scheduler
   noktası değildir; ağ beklemesini engelleyerek ve son tarihe bağlı yapar.
6. **Reaktör `libc` `poll(2)`.** Yeni bağımlılık yok (mio reddedildi:
   `bağlantı_sınırı` ≤ 4096 için poll yeterli, tedarik yüzeyi büyümez).
   Sınırlar reaktörde zorlanır: süreç geneli ve uzak IP başına bağlantı
   sayısı (aşımda bildirilen ret satırı tek engellemeyen yazımla, Zee görevi
   kurulmadan), satır uzunluğu, bağlantı başına 64 KiB giden tampon (geri
   basınç). Ret günlüğü oran sınırlıdır.
7. **Yetkinlik `ağ-dinleyici`.** Statik denetim (T054) ve çalışma zamanı
   politikası; çocuk modu kapalı. `ağ-sunucusu` anlamı değişmez. Bağlantı
   işlemleri geri alınamaz etkidir: eylem içinde T048.
8. **Hizmet yaşam döngüsü.** Scheduler başlayınca `READY=1`; SIGTERM'de
   yeni kabul durur, `STOPPING=1`, okumalar `kapanıyor` döner, en çok 10
   saniyelik boşaltmadan sonra kalan görevler iptal edilir ve süreç 0 ile
   çıkar.
9. **Kanıt altyapısı.** Bağlantı olay beklemesi ve her işlem IO izine girer
   ve yeniden oynatılır. Hermetik adaptör betiklenmiş istemcileri parça
   parça teslim eder; eşzamanlı oturum sırası deterministik sınanır.
10. **Birleşmeyen kipler.** Aynı programda web sunucusu ve bağlantı bloğu
    (C030), çok işçi grubu (K-376 sınırı), `--güvenli` ve Unix dışı platform.
    Bildirimsiz projede ve bağlantı bloğu olmayan programda hiçbir davranış
    değişmez; mevcut web sunucusu yolu dokunulmadan kalır.

## Reddedilen seçenekler

- **Bağlantı başına süreç (inetd modeli):** yalıtım güçlü ama üst düzey kod
  her bağlantıda yeniden koşar, başlangıç maliyeti ve bellek binlerce
  oturumda kabul edilemez.
- **Bağlantı başına OS iş parçacığı + ayrı yorumlayıcı:** süreç içi dosya
  kilitleri (fcntl) iş parçacıkları arasında yalıtım vermez; `zee-esz-1`
  determinizmi kaybolur (RFC-0039 alternatifleri).
- **Yalnız `dene` biçimli yazma:** her yanıt satırı için Sonuç denetimi
  protokol kodunu şişiriyordu; cümle biçimi hata durumunda yalnız o
  bağlantıyı kapatır.
- **Satır sonunu normalleştirme (LF'yi kabul):** SMTP smuggling'i açar.

## Kanıt

- `compiler/tests/ag_baglantisi_testi.rs` (hermetik): satır akışı ve QUIT,
  çıplak LF/CR, `<CR>.<CR>`, uzun satır ve kodlama sınıfları, iki oturumun
  parça parça iç içe ilerlemesi, bağlantı/uzak adres sınırı ve kibar ret,
  susan istemci zaman aşımı, `içinde`/`yetişmezse` okuma noktasında, görev
  hatasının yalıtımı, yazma enjeksiyonu Sonuç'u, kayıt hataları, çocuk modu,
  `bekle` ile kardeşin ilerlemesi ve IO izi kaydı/yeniden oynatması.
- `compiler/tests/ag_baglantisi_surec_testi.rs` (gerçek süreç + TCP,
  systemd soket devri taklidi): uçtan uca SMTP benzeri oturum ve pipelining,
  eşzamanlı oturumların birbirini beklememesi ve dördüncü bağlantının
  `421` ile kapanması, damla damla bayt gönderen istemcinin mutlak süreyle
  kesilmesi, SIGTERM boşaltması (`421 kapanıyor`, READY/STOPPING), 3000
  satırlık yanıtta geri basınç ve kayıpsız teslim, hiç okumayan istemcinin
  yazma süresinde kesilmesi, yetkinliksiz T054 ve devirsiz çıkış 2.
- `dil::ag_satiri` birim testleri, `proje_testi` P020, io izi kodlama
  birim testi, gerçek systemd ölçümü `scripts/systemd-adli-soket-olcumu.py`
  (bağlantı bloğu ve 465/993 dışı kapılar dahil).

## Sonuçlar

- `urun-h` SMTP alıcısının bağlantı tarafı Zee'de yazılabilir; F009 ve
  F010 sürtünmeleri kapanır.
- Bağlantı içindeki PostgreSQL/dosya/HTTP çağrıları senkron dilimdir;
  uzun DB çağrısı kardeş bağlantıları bekletir (Katman C açık).

## Açık

- Sunucu TLS'i ve STARTTLS (A2b), blok okuma ve dosyadan akıtma (A2c).
- Çok işçi grubuna adla aktarım; binlerce boşta oturumun RSS ölçümü (H4).
- `dil dene` içinden betiklenmiş bağlantı cümlesi.
