# K-252 native SMTP süre arızası

Owned loopback `127.0.0.1` fixture, LF/EOF göndermeden her40ms bir bayt
verir. Kaynak before965c3a9 ve çalışma ağacı after aynı native `gonder`
API'sine 200ms verir. Before3011ms/68byte; after201ms/8byte. İkisi de
tek bağlantı ve sabit dış hata maskesidir. After çalışma ağacı kanıtıdır;
exact kaynak commit'inde test/kampanya kabulü [021bff4 temiz ortak
kontrolünde](../common-021bff4/README.md) ayrıca bağlandı. Kaynak düzeltmesi
`c0f79aded235266f75cda1bda9244d766d8a32cc`, regression fixed_by ve beyanları
takip 021bff4 commit'indedir; çalışma ağacı ölçümü temiz kaynak ölçümü olarak
yeniden etiketlenmez.

Reproducer `eposta::tests::smtp_yavas_yanit_mutlak_butceyi_yenileyemez`
ve [minimal semantic fixture](../../../../regression/security/smtp-yavas-yanit.dil)
ile depoda tekrar edilir. Ağ fixture'ı gerçek native sender kullanır;
müşteri/provider bağlantısı, teslim başarısı veya DNS iptali kanıtı değildir.

İlk external probe kurulumunda postgres-protocol patch/path ve lock hataları,
sonraki wrapper taşımasında eksik test trait importları yerel ham loglarda
korunur; final başarılı kanıt eski başarısız logun üstüne yazılmaz.
