# ADR-171 — Deneysel web yalnız localhost; üretim yalnız proxy/worker yolu

- **Durum:** kabul
- **Tarih:** 25 Kasım 2026
- **İlgili kayıt:** K-400, spec/12 §5a, ADR-034 (HTTPS reverse-proxy sınırı),
  ADR-059 (core freeze), ADR-066 (güvenlik bulguları), ADR-162 (adlı
  dinleyici devri), ADR-164 (CSP), ADR-168 (istek boyut sınırları), GB-170

## Bağlam

Bağımsız bir inceleme ve merkez backlog'u "deneysel web yalnız localhost"
sözünün bir üretim sözleşmesine dönüşmesini istedi. Filo (site worker'ları ve
resmî site) üretimde `--web-proxy … --web-worker-port …` ile, resmî site
ayrıca `--web-isci-sayisi N --web-systemd-soketi` ile koşuyor; 25.11.2026
salt okuma süreç denetimi hiçbir üretim sürecinde `--deneysel-web`
bulmadı.

Kod incelemesi (K-400 öncesi):

- Deneysel kip dinleyiciyi sabit `WEB_BIND_IP = 127.0.0.1` üzerinde açar;
  adresi değiştiren bayrak, ortam değişkeni ya da proje alanı yoktur.
  Native listener, işçi grubu ve systemd devri yalnız `GuvenliProxy`
  kolunda kullanılır. Yani bind düzeyinde public dinleme yolu yoktu.
- Ama deneysel kip `Host` başlığını hiç denetlemiyordu. Loopback'e bağlı bir
  HTTP sunucusu bu hâliyle DNS rebinding'e açıktır: kullanıcının ziyaret
  ettiği saldırgan sayfa kendi alan adını kısa TTL ile 127.0.0.1'e çözdürür,
  aynı köken sayıldığı için yerel prototipin yanıtlarını okur ve form
  gönderir. Public Host'u koruyan bir tünel ya da reverse proxy (yaygın
  tünel istemcilerinin varsayılanı) da deneysel worker'ı sessizce internete
  açıyordu; deneysel kipte Secure çerez, HSTS ve Forwarded doğrulaması yoktur.
- systemd soket devri altında başlatılan deneysel süreç devri yok sayıp
  127.0.0.1'e bağlanmaya çalışıyordu (çoğu durumda belirsiz `EADDRINUSE`).

## Karar

1. **Bind değişmez:** deneysel kip yalnız `127.0.0.1`; adres seçimi açılmaz.
   Kabul edilen eş loopback değilse 403 (derinlemesine savunma, iki kipte de).
2. **Host kapısı:** deneysel kipte her istek tek `Host` taşır ve adı
   `localhost`, `*.localhost` (RFC 6761), noktalı ondalık `127.0.0.0/8` ya da
   `[::1]` olur; kapı serbesttir (tarayıcı, yerel ön kapı ve geliştirme
   proxy'leri farklı kapı yazar). Aksi 421 `Misdirected Request` — istek
   rota, dosya ve oturum yüzeyine ulaşmaz, gövde üretim bayraklarını önerir.
   İlk retle süreç başına bir kez, Host değeri yazılmadan stderr uyarısı.
3. **Yönlendirme başlıkları reddedilmez:** `Forwarded`/`X-Forwarded-*`
   deneysel kipte zaten kimlik değildir. Resmî sitenin yerel önizleme ön
   kapısı bu başlıkları gönderir ve Host'u `127.0.0.1:<kapı>` yapar; onları
   reddetmek yerel akışı bozar, güvenlik kazancı Host kapısının üstüne bir
   şey eklemez.
4. **systemd devri reddi:** adlı dinleyici bildirimi olmayan projede
   `LISTEN_FDS` ya da `LISTEN_PID` varken deneysel kip kaynak yürütülmeden
   çıkış 2 ile durur ve üretim bayraklarını önerir.
5. **Üretim tek yoldur:** internete açık her süreç `--web-proxy` ailesindedir
   (§5, §7). Bu ADR yeni üretim davranışı eklemez; spec/12 §5a mevcut
   garantileri tek yerde toplar.
6. **Yüzey:** yeni CLI bayrağı, grammar, kalıp, tanı kodu, IO izi şeması ya da
   `dil::api::v1` yok; değişiklik yalnız `dil_cli` web adaptöründedir.

## Değerlendirilen seçenekler

- **Açık opt-in ile public deneysel dinleme** (`--deneysel-web-adresi`):
  gerçek ihtiyaç yok (filo ve yerel akışlar loopback), yeni CLI yüzeyi core
  freeze'e aykırı; reddedildi.
- **Host'u dinlenen kapıyla birebir eşleme:** yerel ön kapılar ve geliştirme
  proxy'leri farklı kapı yazar; rebinding'e karşı ad denetimi yeterli.
  Reddedildi.
- **Yönlendirme başlıklarını reddetme:** madde 3; reddedildi.
- **`INVOCATION_ID`/`NOTIFY_SOCKET` ile systemd altında deneysel kipi
  reddetme:** geliştiricinin kullanıcı birimindeki meşru prototipini bozar;
  yalnız soket devri (üretim dinleyicisinin işareti) reddedilir.

## Sonuçlar

- DNS rebinding ve public Host'lu tünel deneysel kipe ulaşamaz.
- Açık sınır: Host'u loopback olarak yeniden yazan yerel proxy/tünel
  çalışma zamanınca gerçek yerel ön kapıdan ayırt edilemez; operatör kuralı
  (`docs/web-production-profili.md`) ve dağıtım öncesi süreç denetimi kapsar.
- Göç: localhost/127.0.0.1 ile açılan yerel akış değişmez; başka adla açılan
  deneysel sunucu üretim profiline geçer (`docs/surumler.md`).
