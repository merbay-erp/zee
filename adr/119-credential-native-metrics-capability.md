# ADR-119 — Credential launcher için tek native metrik capability

- **Durum:** kabul, tek POSIX metadata capability
- **Tarih:** 15 Ekim 2026
- **İlgili kayıt:** K-260, GB-081

Final exact kaynak ve K-257 soak kabulü ayrı kayıttadır.

K257 native-owned request/DB pipe metrikleri, credential launcher default close_fds sınırıyla birlikte uygulanamadığından typed reject kısmi sınırdı. Gerçek zee-cekirdek B5/P13 ve D3/P25 mevcut ürün ihtiyaçlarının kompozisyonu için tek designated write-only metadata channel yeterlidir. Yeni grammar/compiler API/credential/environment yetkisi açılmaz.

Karar: launcher --native-metrics-fd N explicit capability'yi credential/source okumadan FIFO/O_WRONLY/O_NONBLOCK/uid/SIGPIPE/PIPE_BUF/stdioalias sınırlarıyla doğrular; native --web-metrik-fd kaynak öncesi exacteşliğini ister. Native constructor bağımsız yeniden denetler. Supervisor yalnız kendi pipe'ını native subprocess'e exact pass_fds ile taşır; launcher kopyasını spawn sonrası kapatır. Varsayılan noinherit ve source/exit metadata korunur. Invalid capability yalnız fixed reason native_metrics_capability üretir; arbitrary descriptors, private source diagnostic veya credential content taşınmaz.

Risk: sameUID caller descriptor ownership ve tekthread beforecreation sözleşmesini ihlal edip FD'yi değiştirebilir; ambient descriptors hiçbir durumda transfer edilmez. Invalid/incompletecommand failclosed, validateddescriptor credentialfail'de close, successfulchildgroupcleanup supervisor sahibi. SIGPIPE disposition kurulmaz veya restore edilmez; mevcut ignored durum zorunlu. Default noFD eski launcher14 ve nativeworker25 sözleşmesine bağlıdır. Yeni dependency yok; Python fcntl POSIX rol sorgusu.

Kanıt: scripts/tests/credential-metrics-test.py7 actual subprocess/model vaka; scripts/tests/credential-metrics-native.py iki actual native credential-decrypt worker, HUP freshchildren/rollup/oldgroupcleanup ve TERMcleanup. Metadata plaintext/ciphertext canary içermez. K257 compiler finalexactSHA/soak kapılarına bağımlı; root docs/index/faz/GB bağları ve insan B5/D3 ürün eşliği review'u ile atomik kapanır. Tam P13/P14 veya livecustomer acceptance iddiası değildir.

K-267 dar opt-in genişlemesi: [ADR-125](125-credential-native-listener-kompozisyonu.md) typed listener/ready/activate ve ayrı host-only startup capability bileşimini ekler. Yukarıdaki doğrudan host/tek metrik sınırı eski standalone profilin tarihsel kapsamıdır; varsayılan noinherit korunur. Yeni profil yalnız owned Unix host içindir; Windows/systemd/file-native/live adoption açıktır.
