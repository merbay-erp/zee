# Yetkinlik ve ağ güvenliği

Zee projeleri dış dünyaya varsayılan olarak kapalı başlar. İzin, kaynak
dosyasındaki bir satırla gizlice değil, projeyi çalıştıran kişinin görebildiği
`proje.dil` bildirimiyle açılır.

## En küçük örnekler

Yalnız proje içindeki bir dosyayı okumak:

```zee
yetkinlikler "dosya-okuma" listesi olsun
ağ_hedefleri boş liste olsun
```

Bir HTTPS API'sine bağlanmak:

```zee
yetkinlikler "ağ" listesi olsun
ağ_hedefleri "https://api.example.com" listesi olsun
```

Yerel geliştirme sunucusuna düz HTTP ile bağlanmak:

```zee
yetkinlikler "ağ", "yerel-ağ" listesi olsun
ağ_hedefleri "http://127.0.0.1:8080" listesi olsun
```

Web uygulaması çalıştırmak:

```zee
yetkinlikler "ağ-sunucusu", "web-oturumu", "kriptografi" listesi olsun
ağ_hedefleri boş liste olsun
```

`ağ-sunucusu` inbound sunucu, `ağ` outbound istemcidir; biri diğerini açmaz.

Yerel PostgreSQL dogfood profili:

```zee
yetkinlikler "veritabanı" listesi olsun
veritabanı_hedefi "postgresql://127.0.0.1:5432/uygulama" olsun
veritabanı_bağlantı_değişkeni "UYGULAMA_DATABASE_URL" olsun
veritabanı_göçleri "göçler" olsun
```

Bağlantı URL'si kaynakta tutulmaz. İlk profil yalnız loopback ve
`sslmode=disable` kabul eder; production TLS desteği anlamına gelmez.

## Neyi korur?

- T054 programın istemediğin bir dış dünya yetkisini kullanacağını çalışmadan
  önce, gerçek kaynak bölgesinde gösterir.
- P015 bozuk izin listesini veya ana uygulamadan daha güçlü olmak isteyen
  paketi proje yüklenirken durdurur.
- Ağ hedefi tam şema+host+port ile eşleşir. Redirect kapalıdır; DNS sonucu
  private/loopback/metadata veya IANA özel-kullanım/geçiş öneklerine kayarsa
  bağlantı kurulmaz.
- Public internet için HTTPS zorunludur. Düz HTTP ve yerel ağ bilinçli ikinci
  onay ister.
- Proje dosya erişimi kök dışına çıkan `..`, mutlak yol ve sembolik bağ
  kaçışını reddeder.

## Profil farkları

`dil çalıştır proje-klasörü` manifesti birebir uygular. `dil çalıştır
--güvenli dosya.dil` çocuk profili olarak yalnız yerel dosyaları açar.
Bildirimsiz tek dosya, eski geliştirici akışını kırmamak için daha geniştir;
yine de public olmayan ağ ve düz HTTP açılmaz.

`AgHedefi` ve `yetkinlik/origin.rs`, şema+DNS/IPv4/IPv6+port kimliğinin tek
sahibidir. K-139'dan itibaren `--web-proxy` production origin'i ile inbound
`Host`, `Forwarded host` ve unsafe `Origin` doğrulaması da bu parser'ı yeniden
kullanır. Böylece outbound allowlist ile web proxy aynı origin yazımına farklı
anlam veremez. Web profili yalnız HTTPS ve loopback proxy bağlantısını kabul
eder; ayrıntı [web production profili](web-production-profili.md) içindedir.

Bu katman, aynı makinedeki kötü niyetli başka bir sürece karşı container
değildir ve paket başına ayrı tenant sağlamaz. Güvenilmeyen kodu güçlü bir OS
sandbox/container içinde çalıştırmak hâlâ host uygulamanın sorumluluğudur.
