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

Bu katman, aynı makinedeki kötü niyetli başka bir sürece karşı container
değildir ve paket başına ayrı tenant sağlamaz. Güvenilmeyen kodu güçlü bir OS
sandbox/container içinde çalıştırmak hâlâ host uygulamanın sorumluluğudur.
