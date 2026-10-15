# Zee 0.8 cookbook — owned site dilimleri

- İlk genel proje: `dil yeni site`, sonra `kilitle`, `denetle`, `dene`.
- [İçerik/admin/medya starter](icerik-paneli.md): ayrı creator yeni hedefi
  oluşturur; explicit DB göçü, private PHC ve local TLS girdilerinden sonra
  owned host başlatılır. Yeni compiler komutu/installer yoktur.
- [Starter API](starter-api.md): raw upload + gerçek PNG teslimi; HTML escaping,
  native session/CSRF ve kalıcı unknown latch sınırları.
- [Generic SQL modül yaşam döngüsü](../../module-sql-lifecycle.md): ayrı schema
  descriptor ile CRUD ve veri koruyan SQL down/up. Bu starter'ın ORM/API'si
  veya otomatik migration zinciri olduğu iddia edilmez.
- [Signed PDF host](../../pdf-plugin-host.md): farklı purpose-bound signed
  stdio protokolü; mevcut backend/runtime ve publisher trust açık seçilir.
  Starter varsayılanı PDF plugin veya production key yetkisi açmaz.

Bu tarifler çalışma örnekleridir. Exact kaynak/binary kabulü, gerçek müşteri
migration'ı, novice30min deneyi ve canlı deploy ayrı kanıttır; birbirinin
yerine yazılmaz. Sabitlenen compiler seçimi için mevcut yerel seçici/launcher,
proje kaynak/bağımlılık kilidi için `proje.kilit` kullanılır; bu ikisi farklıdır.
