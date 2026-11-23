# ADR-162 — Adlı dinleyici devri ve systemd hizmet bildirimi

- **Durum:** kabul
- **Tarih:** 23 Kasım 2026
- **İlgili kayıt:** K-376, K-355 (`urun-h`), K-343, RFC-0039 Aşama A1,
  ADR-059 (core freeze), ADR-064 (dil yüzeyi), ADR-087 (açık depo),
  ADR-161 (çok işçi profili ve K-343 eki), spec/12 §8, spec/08, GB-152

## Bağlam

Etkin dogfood ürünü `urun-h` (e-posta sunucusu; harici köken K-355) ürünün
işletim sistemi tarafındaki Faz 2 çalışma zamanı kararına bağlandı (ürün
deposunda kabul edildi, 23 Kasım 2026). O karar uygulamanın gerçek Zee
çalışma zamanıyla, ayrıcalıksız kullanıcıda, capability olmadan,
`Type=notify` ve adla devredilen dinleyicilerle çalışmasını kabul ölçütü
yapıyor; hizmet yüzeyi kararı (aynı ürün) on adlı soketi (beş kapı × IPv4/IPv6)
tek hizmete veriyor. Ürünün canlı ölçümü (Debian 13, systemd 257.13) iki kez
`LISTEN_FDNAMES` sırasının `Sockets=` sırasından farklı geldiğini gösterdi;
bu depodaki ölçüm (systemd 255) aynı sonucu verdi (`web_v6` fd 3'te).

Bugünkü tek devir K-343'tür: yalnız çok işçi yöneticisi, `LISTEN_FDS=1`,
fd 3, `127.0.0.1:<worker-port>`. Çok soket, ad, IPv6 ve hizmet bildirimi
yoktur; `NOTIFY_SOCKET` hiç okunmaz (`Type=notify` birimi zaman aşımına
düşer). Ürün bugün sözleşmeyi Zee yerine bir Python yoklamasıyla ölçmek
zorunda kaldı.

Ayrıca bu ADR'nin gerçek systemd ölçümü, ürünün hizmet profilindeki
`SystemCallFilter=~@privileged` altında web çalışma zamanının ilk istekte
SIGSYS ile öldüğünü buldu: kalıcı dosyanın atomik değiştirilmesi sahip
zaten aynıyken de `fchown` (`@chown` ⊂ `@privileged`) çağırıyordu.

## Karar

1. **Bildirim `proje.dil` içinde, adla.** Yeni alan
   `dinleyiciler "<ad> <adres>:<kapı>", … listesi olsun`. Ad
   `^[a-z][a-z0-9_]{0,31}$` = systemd `FileDescriptorName=`. RFC-0039 açık
   sorusu "çift yığın" burada kapanır: **aile sonekli iki ayrı ad**
   (`smtp_v4`, `smtp_v6`); tek mantıksal ad + aile eşlemesi reddedildi,
   çünkü systemd adı fd başına verir ve IPv6 soketi ayrı birim ister. Ad,
   adres tekil; ≤ 64; IPv6 kapsam kimliği ve IPv4 eşlemeli IPv6 yok. Bozuk
   bildirim yeni tanı **P018**.
2. **Doğrulama yürütmeden önce, yalnız adla.** Bildirimli proje
   `dil çalıştır` ile açılınca derleme/yürütme öncesi `LISTEN_PID` = kendi
   PID, `LISTEN_FDS` = bildirim sayısı, `LISTEN_FDNAMES` = fd başına tam bir
   geçerli, yinelenmeyen ad; ad kümesi bildirim kümesine eşit. Sıra hiçbir
   zaman karşılaştırılmaz. Her fd dinleyen TCP, bildirimdeki aile/adres/kapıya
   tam bağlı; IPv6'da `IPV6_V6ONLY=1`. Her ihlal çıkış 2 + ad + `.socket`
   önerisi; bind'e geri dönüş yok. `LISTEN_*` önce silinir; soketler
   `FD_CLOEXEC` + engellemeyen kipe alınır.
3. **Aşama A1 sınırı: tut, kabul etme.** Zee kodu adlı dinleyicilere bu
   aşamada erişmez; süreç onları ömrü boyunca açık tutar, bağlantılar
   çekirdek kuyruğunda bekler. Bağlantı yüzeyi (satır okuma/yazma, kabul
   döngüsü) RFC-0039 A2'dir. Yetkinlik (`ağ-dinleyici`) A2 ile gelir: A1'de
   Zee etkisi yoktur, bu yüzden yetkinlik istenmez.
4. **Hizmet bildirimi.** `NOTIFY_SOCKET` her `dil` sürecinin başında okunup
   silinir (işçi/alt süreç bildiremez; `NotifyAccess=main`). `READY=1` bir
   kez: tek süreçte `sunucu başlat` dinleyiciyi açınca, çok işçi
   yöneticisinde birincil açılışı bitip işçiler başlayınca. `STOPPING=1` bir
   kez: tek süreçte SIGTERM ile kabul durunca, yöneticide kapanış başında.
   `NOTIFY_SOCKET` yoksa hiçbir şey gönderilmez (bugünkü birimler değişmez).
   Mutlak yol ve Linux soyut ad (`@`) desteklenir; gönderim hatası
   günlüklenir, süreci durdurmaz.
5. **Ayrıcalıksız sandbox uyumu.** Atomik değiştirme sahip/grup zaten
   aynıysa `fchown` çağırmaz (sonuç sahipliği aynıdır; farklıysa bugünkü gibi
   taşınır). Böylece `CapabilityBoundingSet=`, `MemoryDenyWriteExecute=yes`,
   `SystemCallFilter=@system-service ~@privileged @resources` altında web
   çalışma zamanı çalışır. Çalışma zamanı JIT/yazılabilir-çalıştırılabilir
   bellek istemez.
6. **Birleşmeyen kipler.** Çok işçi grubu (işçi devri adları taşımaz),
   `--güvenli` çocuk modu ve Windows adlı devirle çıkış 2. K-343
   (`--web-systemd-soketi`) değişmez ve bildirimsiz projede `LISTEN_*`
   bugünkü gibi yok sayılır.

## Kanıt

- Süreç testleri (`compiler/tests/adli_dinleyici_testi.rs`): dört soket
  bildirim dışı sırayla adla eşlenir, READY/STOPPING tek kez, test kendi
  kopyalarını kapatınca soketler hâlâ bağlantı kabul eder; dokuz olumsuz
  devir (devirsiz, başka PID, adsız, eksik/fazla ad, sayı/ad, yer
  değiştirmiş ad, yanlış kapı, V6ONLY kapalı joker IPv6) çıkış 2 ve
  öneriyle; bildirimsiz projede LISTEN_* yok sayılır; çocuk modu reddi.
  K-343 grubu: `isci_grubu_systemd_bildirimini_yalniz_yoneticiden_gonderir`.
  Birim: `adli_dinleyici::testler` (sıra karışık iki/dört, eksik/fazla/
  yinelenen/bozuk ad, yanlış PID), `proje_testi` P018, `kalici_dosya`
  sahiplik testleri.
- Gerçek systemd (`scripts/systemd-adli-soket-olcumu.py`, şablonlar
  `docs/adli-dinleyici-devri.md`'den; CI ubuntu-24.04 adımı): systemd 255,
  dört adlı soket, `Type=notify` başlangıcı 82 ms, UID 1000, `CapEff` ve
  `CapBnd` sıfır, `MemoryDenyWriteExecute` + `~@privileged @resources`
  altında HTTP 200, dört soket bağlantı kabul eder, temiz durma; fazla ad ve
  `BindIPv6Only=both` koşuları `ExecMainStatus=2`. Düzeltme öncesi aynı
  profilde ilk istek SIGSYS (aarch64 syscall 55 `fchown`).

## Sonuçlar

- Ürün `Type=notify` birimini gerçek Zee ile kurabilir; yoklama betiği
  gerekmez. Web uygulaması (ürünün yönetim paneli) dinleyicileri tutarken
  hizmet verir.
- Linux belirli bir IPv6 adresine bağlanan sokette V6ONLY'yi kendisi açar;
  V6ONLY denetimi pratikte joker `[::]` soketinde çift yığını yakalar.
- `Accept=no` birimde durdurulan hizmet, kuyrukta bekleyen bağlantı varsa
  systemd tarafından yeniden etkinleştirilir (beklenen systemd davranışı).

## Açık

- Bağlantı kabul/okuma/yazma yüzeyi ve `ağ-dinleyici` yetkinliği (A2),
  engellemeyen scheduler (C), sunucu TLS'i (B).
- Çok işçi grubunda adlı dinleyicilerin işçilere aynı adlarla aktarımı.
- Web sunucusu olmayan programın `READY=1` göndermesi (bugün `Type=exec`).
- HTTP dinleyicisinin de adlı devirden alınması (bugün `--web-worker-port`
  bind'i ya da K-343).
- Ürünün kendi işletim sistemi imajında (AppArmor, dm-verity) uçtan uca
  kabul ürün tarafındadır.
