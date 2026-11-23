# Adlı dinleyici devri ve systemd hizmet bildirimi

K-376 / [ADR-162](../adr/162-adli-dinleyici-devri-ve-hizmet-bildirimi.md) /
[RFC-0039](../rfcs/0039-ag-sunucusu-yetkinligi.md) Aşama A1; bağlantı yüzeyi
K-381 / [ADR-165](../adr/165-ag-dinleyici-baglanti-yuzeyi.md) Aşama A2a.
Normatif maddeler [spec/12 §8–§9](../spec/12-web-guvenlik-profili.md).

Bir proje, dış yöneticinin (systemd) kendisi adına açtığı dinleyicileri
`proje.dil` içinde **adla** bildirir. `dil çalıştır` kaynağı derleyip
yürütmeden önce systemd soket etkinleştirmesini (`LISTEN_PID`, `LISTEN_FDS`,
`LISTEN_FDNAMES`) bu bildirime göre doğrular. Süreç kendisi bind etmez, kapı
<1024 için yetkinlik istemez. `NOTIFY_SOCKET` varsa web sunucusu açılışı
bitirince `READY=1`, SIGTERM'de `STOPPING=1` gönderir.

## Bildirim

```text
dinleyiciler "smtp_v4 0.0.0.0:25", "smtp_v6 [::]:25", "gonderim_v4 0.0.0.0:587", "gonderim_v6 [::]:587" listesi olsun
```

- Her öğe `<ad> <adres>:<kapı>`. Ad `^[a-z][a-z0-9_]{0,31}$` ve systemd
  `FileDescriptorName=` değeriyle aynıdır (`:` ayraç olduğu için yasak).
- Çift yığın iki ayrı adla bildirilir (`_v4`, `_v6` soneki önerilir). Ad,
  adres ve kapı tekildir; en çok 64 dinleyici. IPv6 kapsam kimliği ve IPv4
  eşlemeli IPv6 adresi kabul edilmez. Hata `P018`.
- Bildirim yoksa ya da `boş liste` ise hiçbir şey değişmez; `LISTEN_*`
  ortamı bugünkü gibi yok sayılır.

## Devir sözleşmesi

- `LISTEN_PID` sürecin kendi PID'i, `LISTEN_FDS` bildirilen dinleyici
  sayısına eşit, `LISTEN_FDNAMES` her fd için tam bir ad taşır.
- Eşleme **yalnız adla** yapılır. systemd fd'leri `Sockets=` sırasıyla
  vermez (systemd 257'de iki ayrı ölçümde bildirim dışı sıra gözlendi); sıra
  hiçbir zaman karşılaştırılmaz.
- Her fd dinleyen TCP soketi olmalı ve bildirimdeki aile, adres ve kapıya
  tam bağlı olmalıdır. IPv6 soketinde `IPV6_V6ONLY` açık olmalıdır.
- Eksik ya da fazla ad, yinelenen ad, sayı/ad uyuşmazlığı, yanlış PID,
  yanlış adres/kapı ve V6ONLY'siz IPv6 soketi çıkış 2'dir; ileti adı ve
  aşağıdaki düzeltme önerisini taşır. Bind'e geri dönüş yoktur.
- `LISTEN_*` doğrulamadan önce silinir; soketler `FD_CLOEXEC` ve engellemeyen
  kipe alınır, alt süreçlere sızmaz.
- Bu aşamada Zee kodu adlı dinleyicilere erişmez: süreç onları ömrü boyunca
  açık tutar, gelen bağlantılar çekirdek kuyruğunda bekler. Bağlantı kabul
  yüzeyi RFC-0039 A2'nin işidir. Uzun ömürlü süreç bugün web sunucusudur
  (`sunucu başlat`); HTTP dinleyicisi bugünkü gibi `--web-worker-port`
  üzerinden açılır.
- Çok işçi grubu (`--web-isci-sayisi`), `--güvenli` çocuk modu ve Windows
  adlı devirle birleşmez (çıkış 2).

## Hizmet bildirimi

- `NOTIFY_SOCKET` süreç başında okunur ve ortamdan silinir; işçi ve alt
  süreç bildirim gönderemez (`NotifyAccess=main` ile uyumlu). Mutlak yol ve
  Linux soyut adı (`@…`) desteklenir.
- `READY=1`: tek süreçte `sunucu başlat` dinleyiciyi açtıktan sonra; çok
  işçi yöneticisinde birincil işçi açılışı bitirip işçiler başlatıldıktan
  sonra. Bir kez gönderilir.
- `STOPPING=1`: SIGTERM ile yeni kabul durduğunda (yöneticide kapanışın
  başında). Bir kez gönderilir.
- `NOTIFY_SOCKET` yoksa hiçbir şey gönderilmez; `Type=simple`/`Type=exec`
  birimleri (zeedili.com dahil) değişmez.

## Birim şablonları

Aşağıdaki iki blok ve bağlantı bölümündeki hizmet bloğu depo CI'ında gerçek
systemd ile ölçülür (`scripts/systemd-adli-soket-olcumu.py` blokları bu
dosyadan okur). Her
adlı dinleyici için soket şablonundan bir birim üretilir; `@AD@` adı,
`@ADRES@` `ListenStream` değeridir (IPv6 köşeli parantezle). `@SOKETLER@`
bütün soket birimlerinin boşlukla ayrılmış listesidir. `@KAPI@`, `@DIL@`,
`@PROJE_KOKU@`, `@KULLANICI@` ve `@ORIGIN@` yerine kendi değerlerinizi
yazın; birim adı önekini (`zee-uygulama`) iki şablonda aynı tutun.

`zee-uygulama-@AD@.socket`:

```ini
[Unit]
Description=Zee uygulaması adlı dinleyici @AD@

[Socket]
ListenStream=@ADRES@
FileDescriptorName=@AD@
BindIPv6Only=ipv6-only
Accept=no
Service=zee-uygulama.service

[Install]
WantedBy=sockets.target
```

`zee-uygulama.service`:

```ini
[Unit]
Description=Zee uygulaması (adlı dinleyici devri)
Requires=@SOKETLER@
After=@SOKETLER@ network.target

[Service]
Type=notify
NotifyAccess=main
Sockets=@SOKETLER@
User=@KULLANICI@
WorkingDirectory=@PROJE_KOKU@
ExecStart=@DIL@ çalıştır --web-proxy @ORIGIN@ --web-worker-port @KAPI@ @PROJE_KOKU@
NoNewPrivileges=yes
CapabilityBoundingSet=
AmbientCapabilities=
MemoryDenyWriteExecute=yes
LockPersonality=yes
RestrictNamespaces=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources
PrivateDevices=yes
ProtectSystem=strict
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
ReadWritePaths=@PROJE_KOKU@
UMask=0077
TimeoutStopSec=60

[Install]
WantedBy=multi-user.target
```

- `FileDescriptorName=` zorunludur; yoksa systemd adı birim adından üretir
  (`zee-uygulama-smtp_v4.socket`) ve süreç bunu geçersiz ad diye reddeder.
- `BindIPv6Only=ipv6-only` IPv6 soketinde V6ONLY'yi açar; IPv4 soketinde
  etkisizdir, bu yüzden tek şablon iki aileye de uyar. Çift yığını tek
  `[::]` soketiyle vermek reddedilir.
- `Type=notify` + `NotifyAccess=main`: `systemctl start` web sunucusu
  `READY=1` gönderene dek bekler. Web sunucusu olmayan program `READY=1`
  göndermez; böyle bir birimde `Type=exec` kullanın.
- `CapabilityBoundingSet=` ve `AmbientCapabilities=` boş: kapı <1024 yalnız
  devirle açılır. `MemoryDenyWriteExecute=yes`: çalışma zamanı yazılabilir +
  çalıştırılabilir bellek istemez (JIT yok).
- `ReadWritePaths=` proje kökü: web çalışma zamanı `.zee/` durumunu orada
  tutar.

Etkinleştirme: her soket için `systemctl enable --now zee-uygulama-<ad>.socket`,
ardından `systemctl start zee-uygulama.service`.

## Bağlantı yüzeyi (K-381, RFC-0039 A2a)

Bağlantı bloğu olan program web sunucusu açmaz; hizmet `--web-proxy`
bayrakları olmadan çalışır ve `READY=1`'i bağlantı scheduler'ı başlayınca
gönderir. Normatif maddeler [spec/12 §9](../spec/12-web-guvenlik-profili.md),
karar [ADR-165](../adr/165-ag-dinleyici-baglanti-yuzeyi.md).

```text
# proje.dil
yetkinlikler "ağ-dinleyici" listesi olsun
dinleyiciler "smtp_v4 0.0.0.0:25", "smtp_v6 [::]:25" listesi olsun
bağlantı_sınırı 256 olsun
uzak_adres_bağlantı_sınırı 16 olsun
bağlantı_boşta_süresi 300 olsun
bağlantı_dolu_yanıtı "421 4.3.2 Hizmet mesgul" olsun

# ana.dil — her aile ayrı ad taşıdığı için iki blok aynı işlemi çağırır
"smtp_v4" dinleyicisinde bağlantı geldiğinde
    oturumu sür
"smtp_v6" dinleyicisinde bağlantı geldiğinde
    oturumu sür
```

Bağlantı hizmetinin birimi (soket birimi yukarıdaki şablonun aynısıdır):

`zee-uygulama.service` (bağlantı):

```ini
[Unit]
Description=Zee uygulaması (adlı dinleyici bağlantıları)
Requires=@SOKETLER@
After=@SOKETLER@ network.target

[Service]
Type=notify
NotifyAccess=main
Sockets=@SOKETLER@
User=@KULLANICI@
WorkingDirectory=@PROJE_KOKU@
ExecStart=@DIL@ çalıştır @PROJE_KOKU@
NoNewPrivileges=yes
CapabilityBoundingSet=
AmbientCapabilities=
MemoryDenyWriteExecute=yes
LockPersonality=yes
RestrictNamespaces=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@privileged @resources
PrivateDevices=yes
ProtectSystem=strict
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
ReadWritePaths=@PROJE_KOKU@
UMask=0077
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
```

- SIGTERM (`systemctl stop`/`restart`): yeni kabul durur, `STOPPING=1`,
  bekleyen okumalar `kapanıyor` döner ve Zee protokolün `421` satırını
  yazar; en çok 10 saniye sonra kalan bağlantılar kapatılır.
  `TimeoutStopSec` bu süreden uzun olmalıdır. `Accept=no` dinleyici
  yeniden başlatma aralığında yeni bağlantıları çekirdek kuyruğunda tutar.
- Sınır dolunca bağlantı kabul edilir, `bağlantı_dolu_yanıtı` yazılıp
  kapatılır; ret sayısı günlüğe en çok beş saniyede bir düşer. Bağlantı
  görevinin tanıyla bitmesi `Zee bağlantı #N (<ad>, <uzak>) kapandı: …`
  satırıdır; satır içeriği günlüğe yazılmaz.
- Geliştirme makinesinde systemd birimi kurmadan:
  `systemd-socket-activate -l 127.0.0.1:2525 --fdname=smtp_v4 dil çalıştır .`
  (Linux). Bind'e geri dönüş yoktur; devirsiz `dil çalıştır` çıkış 2 verir.

## Ölçüm

`python3 scripts/systemd-adli-soket-olcumu.py --dil <dil> --cikti <klasör>`
(Linux + systemd + parolasız sudo) dört adlı soketi (iki IPv4, iki
V6ONLY IPv6) ve hizmeti yukarıdaki şablonlardan kurar ve şunları denetler:
`systemctl start` `READY=1` ile döner; günlük dört adın tamamını doğru
adresle eşlediğini yazar; dört soket ve HTTP kapısı bağlantı kabul eder;
ana sürecin UID'i 0 değildir ve `CapEff` sıfırdır; `systemctl stop` temiz
biter. Olumsuz iki koşu: bildirimde fazladan ad (devredilmeyen ad, çıkış 2)
ve `BindIPv6Only=both` (V6ONLY kapalı, çıkış 2). K-381 bağlantı koşusu iki
adlı soketle (IPv4 + V6ONLY IPv6) bağlantı hizmetini kurar: `READY=1` ile
başlar, iki aileden SMTP benzeri oturum (karşılama, yankı, çıplak LF reddi,
QUIT) geçer, ayrıcalıksız ve capability'siz çalışır; açık bir bağlantı
varken `systemctl stop` istemciye `421 … kapanıyor` yazdırır ve hizmet 0 ile
temiz biter. Sonuç `<klasör>/systemd-adli-soket-olcumu.json`.

Kapılar çekirdeğin geçici (ephemeral) aralığının dışından seçilir, her
ailede (0.0.0.0 ve çift yığın `[::]`) boş olduğu doğrulanır ve aynı koşuda
yinelenmez; birim adları koşu başına benzersizdir ve başlangıçta önceki
kesilmiş koşudan kalan ölçüm birimleri temizlenir. Bir soket birimi
başlatılamazsa `systemctl status`, birim günlüğü ve `ss -ltnp` (yoksa
`/proc/net/tcp*`) stderr'e ve JSON'a (`kurulum_hatalari`) yazılır ve koşu
yeni kapılarla en çok üç kez denenir; ölçümün kendi başarısızlığı yeniden
denenmez. Gerekçe (K-376 takibi): geçici aralıktan seçip bırakılan kapı,
runner'daki istemci bağlantılarına verilebiliyordu; yoğun loopback trafiği
altında eski seçim 12 koşunun 12'sinde `systemctl start …socket` ya da HTTP
bind hatası verdi, yeni seçim 12/12 geçti.

