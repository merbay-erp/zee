# Çok işçi grubu: systemd soketiyle kesintisiz yeniden başlatma

K-343 / ADR-161 eki. `dil çalıştır --web-isci-sayisi N` grubu Unix'te ortak
loopback dinleyiciyi kendisi bağlar; `systemctl restart` sırasında eski
yönetici çıkıp yenisi bağlayana kadar gelen bağlantılar reddedilir.
`--web-systemd-soketi` (ya da `zee.ayar` içinde `guvenlik.systemd-soketi: true`)
verildiğinde yönetici bağlamaz: dinleyiciyi systemd soket etkinleştirmesiyle
devralır. Soketi systemd tuttuğu için yeniden başlatma aralığında gelen
bağlantılar soket kuyruğunda bekler ve yeni grubun işçileri tarafından
yanıtlanır.

## Sözleşme

- Yalnız açık opt-in. `LISTEN_FDS` kendiliğinden algılanmaz.
- `LISTEN_PID` yönetici süreci, `LISTEN_FDS=1`, soket fd 3 olmalıdır. Soket
  `127.0.0.1:<worker-port>` üzerinde dinleyen TCP olarak doğrulanır (GB-133
  doğrulayıcısı). `LISTEN_PID`, `LISTEN_FDS`, `LISTEN_FDNAMES` hemen silinir;
  işçilere ve Zee alt süreçlerine geçmez.
- Devir yoksa ya da geçersizse çıkış 2 ve düzeltme önerisi. Kendiliğinden
  bind'e geri dönülmez.
- Yalnız işçi grubu (`--web-isci-sayisi`) ve Unix. Windows'ta grup zaten
  reddedilir (çıkış 2).

## Birim şablonları

Aşağıdaki iki blok depo CI'ında gerçek systemd ile ölçülür
(`scripts/systemd-soket-olcumu.py` blokları bu dosyadan okur). `@KAPI@`,
`@DIL@`, `@PROJE_KOKU@`, `@KULLANICI@` ve `@ORIGIN@` yerine kendi
değerlerinizi yazın; birim adını (`zee-uygulama`) kendi hizmetinize göre
değiştirin, iki dosyada aynı tutun.

`zee-uygulama.socket`:

```ini
[Unit]
Description=Zee uygulaması dinleyici soketi

[Socket]
ListenStream=127.0.0.1:@KAPI@
Accept=no

[Install]
WantedBy=sockets.target
```

`zee-uygulama.service`:

```ini
[Unit]
Description=Zee uygulaması (çok işçi)
Requires=zee-uygulama.socket
After=zee-uygulama.socket network.target

[Service]
Type=simple
User=@KULLANICI@
WorkingDirectory=@PROJE_KOKU@
ExecStart=@DIL@ çalıştır --web-proxy @ORIGIN@ --web-worker-port @KAPI@ --web-isci-sayisi 4 --web-systemd-soketi @PROJE_KOKU@
KillMode=mixed
TimeoutStopSec=50

[Install]
WantedBy=multi-user.target
```

- `Accept=no`: tek dinleyici soket devredilir (systemd her bağlantı için
  süreç açmaz). `BindIPv6Only` kullanılmaz; adres zaten IPv4 loopback'tir.
- `KillMode=mixed`: SIGTERM yalnız yöneticiye gider; yönetici işçilere
  iletir, kabul edilen istekler tamamlanır (K-336). Süre dolarsa systemd
  grubun kalanına SIGKILL gönderir.
- `TimeoutStopSec=50`: yöneticinin 40 sn boşaltma süresinden uzun olmalıdır.
- Port ve diğer ayarlar `zee.ayar`'daysa `ExecStart` yalnız
  `@DIL@ çalıştır @PROJE_KOKU@` olabilir; `guvenlik` dalında
  `"isci-sayisi": 4, "systemd-soketi": true` ve aynı `worker-port` bulunur.
  `.socket` dosyasındaki port ile `worker-port` aynı olmalıdır; değilse
  yönetici çıkış 2 ile başlamaz.

Etkinleştirme: `systemctl enable --now zee-uygulama.socket`, ardından
`systemctl start zee-uygulama.service`. Yeniden başlatma
`systemctl restart zee-uygulama.service`; soket birimi yeniden başlatılmaz.

## Ölçüm

CI (ubuntu-24.04, gerçek systemd) yukarıdaki şablonlarla birimleri kurar,
dört paralel istemci sürekli istek gönderirken hizmeti beş kez yeniden
başlatır. Rapor başarısız istek sayısını (kapı: 0) ve her yeniden
başlatmadaki en uzun istek süresini (kuyrukta bekleme) yazar. Yerel
taşınabilir eşi `web_cok_surec_testi::isci_grubu_systemd_soketiyle_yeniden_baslatmada_istek_kaybetmez`
testidir (soketi test tutar).

Her istemci 2000 istemci kimliği arasında döner. Tek kimlikle hızlı bir
makine varsayılan uç nokta oran sınırını (kimlik başına 60 sn'de 100,
spec/12) aşar ve 429 yanıtları istek kaybı gibi görünür; ilk CI koşusu
bunu gösterdi (2882 istekte tam 400 = 4 × 100 başarılı). Rapor
başarısızlıkları nedene göre (HTTP durumu ya da bağlantı hatası) ayırır.
Gerçek trafikte oran sınırı ayrı bir sözleşmedir; ölçüm yalnız yeniden
başlatma kaybını sayar.

Ölçüm projesi gerçek ürün gibi `proje.dil` taşır: şablondaki `ExecStart`
proje kökünü alır; bildirim yoksa `dil` P006 ile çıkar ve hizmet hiç
başlamaz (ilk gerçek CI koşusu bunu gösterdi). Başarısızlıkta betik
`systemctl status` ve `journalctl` çıktısını hem adım günlüğüne hem JSON
raporuna (`tani`) yazar; hizmet çökerse 60 sn beklemeden çıkar.

systemd varsayılanı `StartLimitBurst=5`, `StartLimitIntervalSec=10`'dur:
10 saniyede altıncı başlatma hizmeti `start-limit-hit` ile düşürür ve
`.socket` birimi de `service-start-limit-hit` ile kapanır; bu andan sonra
gelen bağlantılar reddedilir. Dağıtım betikleri art arda yeniden
başlatmaları aralıklı yapmalıdır; ölçüm başlatmalar arasında en az 2.5 sn
bırakır. Şablon bu korumayı bilinçli olarak kapatmaz (çökme döngüsünde
soket etkinleştirmesi sınırsız yeniden başlatma üretmesin).

## Açık

Tek süreçli worker için soket etkinleştirmesi, yöneticiden yöneticiye canlı
devir (re-exec), uygulama/kaynak sürümleri arası atomik dağıtım ve Windows.
