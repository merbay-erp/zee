# K-252 native SMTP süre arızası

Owned loopback `127.0.0.1` fixture, LF/EOF göndermeden her40ms bir bayt
verir. Kaynak before965c3a9 ve çalışma ağacı after aynı native `gonder`
API'sine 200ms verir. Before3011ms/68byte; after201ms/8byte. İkisi de
tek bağlantı ve sabit dış hata maskesidir. After çalışma ağacı kanıtıdır;
exact kaynak commit'inde test/kampanya kabulü ayrıca bağlanır.

Reproducer `eposta::tests::smtp_yavas_yanit_mutlak_butceyi_yenileyemez`
ve [minimal semantic fixture](../../../../regression/security/smtp-yavas-yanit.dil)
ile depoda tekrar edilir. Ağ fixture'ı gerçek native sender kullanır;
müşteri/provider bağlantısı, teslim başarısı veya DNS iptali kanıtı değildir.

İlk external probe kurulumunda postgres-protocol patch/path ve lock hataları,
sonraki wrapper taşımasında eksik test trait importları yerel ham loglarda
korunur; final başarılı kanıt eski başarısız logun üstüne yazılmaz.
