# ADR-125 — Typed credential/native listener bileşimi (K-267/P25)

- **Durum:** kabul; temiz S typedcredential/listener model5+native5 dar Unix kabulü; fullP25/ürün/platform açık
- **İlgili güvenlik sınırı:** GB-083

Kayıtlı zee-cekirdek D3/P25 credential ihtiyacı ve B6 native listener, B5
metadata ihtiyacının bileşimidir. Kaynak ürün commit cd5d12f0008620819e6167af5146e83503fd2b38;
D3 talebi README'sindeki exact request SHA694c0b357c4ff22c16987a65dbcd8d68344553d345b8693a00ed1deba74eaed7,
B5 normalleştirilmiş talep bölümü SHA0466b376c023f719b7788fae3655226577b490de1289c1010337f10a67422971
ve B6 etkilenen genel ana.dil SHAf0a65a0b1272a717728db4050fd450fd85900434ed6d5309bdcb9bbf9f138619
önceki ADR-118/119 ve anonim reproducer kayıtlarıyla korunur. Özel ürün kodu,
sunucu veya gerçek credential kopyalanmadı. Yeni grammar/public Rust facade
ve file-native sır gizliliği sözü yoktur.

CredentialRoute yalnız credential root+existing env bindings taşır. Host her
nesilde fresh nonce/receipt ve kendi loopback TCP LISTEN socketini oluşturur.
Yeni explicit host seçenekleri listener/ready/activate/startup FD, generation,
worker-port ve binary-SHA ile all-or-none olur; optional eski designated metric
FD tek ek native metadata rolüdür. Arbitrary pass_fds listesi sunulmaz.
Kaynak öncesi exact native argv eşliği, FIFO yönleri, STDIO/rol/pipe-peer alias,
127.0.0.1 exact port, TCP LISTEN/nonblocking ve generation formatı secret
okunmadan borrowed validator ile; secret okunduktan sonra Popen öncesi tekrar
sınanır. Defaultnoinherit korunur. Bilinmeyen pre-source seçenek typed route'ta
reddedilir; source sonrası argv capability vermez.

Startup FIFO native uygulama stdout'u değildir: yalnız launcher writer'ı alır,
actual kendi Popen child PID+generation+binarySHA kaydını <=512 tek nonblocking
write ile yayınlayıp kapatır; startup FD native pass_fds'e dahil değildir.
Native stdout/stderr eski credential sözleşmesi gereği DEVNULL'dur. Host trusted
startup PID'yi native ready PID/build/port/nonce ve fresh app receipt ile eşler;
native PID'yi edinmez, sorgulamaz veya tekil öldürmez. Yetki owned launcher Popen
ve WNOWAIT held group'tadır. Startup bozuk/eksik/aşan/yanlış bağ ise failclosed;
ACTIVATE+EOF öncesi admission açılmaz. Parent FD kopyaları spawn sonrası kapanır.
Yeni typed rolün STDIO alias reddinde sabit hata JSON'u bile kontrol FIFO'suna
bırakılmaz; diğer retler yalnız sabit metadata reason taşır.

Python host/launcher aynı borrowed role modülünü kullanır; native Rust doğrulaması
ayrıca korunur. Darwin own-PID proc_pidfdinfo PIPEINFO=6 exact184bytes,
handle160/peer168 ile opposite pipe-end alias'ını reddeder. Yerel Apple SDK C
probe bunu ve TCP state offset0'ı doğruladı; yabancı PID sorgusu, handle logu,
kurulu kernel kaynak eşliği/minimumOS/Windows kabulü ileri sürülmez.

Kanıt: capability model/subprocess5 (16 invalid subprocess alt-senaryo dahil),
default credential14 ve metrics7; actual dirty d53 ile crypto bootstrap,
startup/readiness/admission, fresh overlap cleanup, exact metrics records,
wrong generation no-admission ve preclosed metric reader best-effort5. Bu sayılar
underlying alt-senaryolar veya wrapper toplamlarıyla eşsiz sayıya çevrilmez.
Final temiz kaynak, normal Cargo/faz/doc/index entegrasyonu ve canlı adoption
ayrı açıktır. Windows DACL/handle, systemd deployment, file-native env inheritance,
key rotation without restart, farklı app revision/live adoption bu değişiklikle
kapanmış sayılmaz. P25 bütünü kısmi kalır.

İlk commitlenmemiş Python adayının incelemesinde malformed `--native-ready-fd=N`
ve yalnız metrik rolü, eksik typed profil nedeniyle quiet taramadan kaçıp aliased
STDIO kontrol pipe'ına sabit metadata yazabiliyordu. İlk özellik kaynak commit'i
öncesi düzeltme her named FD rolünü bağımsız tarar; spaced ve `key=value` yalnız
quiet alias algılamasında kabul edilir, typed yetki olarak kabul edilmez. Sayısal
sınır/overflow/bozuk önceki değer sonraki gerçek alias taramasını durdurmaz.
Mevcut model5 içindeki gerçek subprocess alt-senaryoları tüm5rol/stdout/stderr,
equals, metrik-only ve bozuk/taşan/tekrarlı önceki değerler için byte0/exit2
ister; subprocess audit hook secret/source open ve native Popen'u reddeder.
Bu Python aday bulgusu GB-083 bilinçli sınır kaydındadır; tarihsel compiler
bug SHA veya yeni regression.dil icat edilmez.

## 15 Ekim 2026 — temiz S yerel kanıt güncellemesi

S common dedicated credential-listener model5 ve actualnative5 aynı cleanS binary/source pinleriyle geçti; credential14 ve metric composition kendi ayrı kohortlarıdır, unique toplam yapılmaz. Actual owned overlap/startup/admission/generation ve metricbest-effort sınırları korundu. Genel secretstore/rotation/provider/liveadoption/Windows/fullP25 açık.

[Güncel dar S raporu](../docs/urunlesme-final-kabul-20261015.md).
