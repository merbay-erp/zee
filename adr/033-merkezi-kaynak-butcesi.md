# ADR-033 — Merkezî ve değişmez kaynak bütçesi

- **Durum:** kabul
- **Tarih:** 2 Eylül 2026
- **İlgili kayıt:** K-129/K-130/K-131, B-025, RFC-0025, spec/24
- **Karar sahipleri:** çekirdek ekip

## Bağlam

Zee'nin ağ, LSP, registry, paket ve IO trace katmanlarında tek tek güvenlik
limitleri vardı; fakat kaynak metni, token üretimi, runtime adımı, normal çıktı,
koleksiyon büyümesi, eşzamanlı görevler ve toplam LSP belge belleği ortak bir
politikanın sahibi değildi. Bir katmanın sınırlı olması, başka bir katmanın host
belleğini veya CPU'yu tüketmesini engellemiyordu.

Çağrı derinliği C019 ile zaten 500'de kontrollü biçimde kesiliyordu. Bu karar
mevcut korumayı yeniden icat etmez; onu dağınık sabit olmaktan çıkarıp diğer
kaynaklarla aynı sözleşmeye bağlar.

## Karar

1. `KaynakSinirlari` resmî güvenli profilin tek değişmez değer nesnesidir.
   Kullanıcı kaynağı veya proje bildirimi bu limitleri yükseltemez.
2. Tek kaynak 8 MiB, token akışı 1 milyon; tek proje 4.096 `.dil` dosyası ve
   toplam 128 MiB kaynakla sınırlıdır. Dosya boyutu mümkünse metadata'dan,
   her durumda `limit+1` bounded reader ile tahsis büyümeden önce doğrulanır.
3. Bir çalışma 10 milyon cümle adımı, 500 çağrı derinliği, koleksiyon başına
   1 milyon öğe, eşzamanlı grup başına 1.024 görev taşır.
4. Normal dil çıktısı çalışma/istek başına 16 MiB ve 100 bin olayla sınırlıdır.
   Aşan olay dış IO'ya gönderilmeden C023 üretir. Uzun yaşayan web sunucusunda
   her istek yeni çalışma/çıktı zarfı alır.
5. Dil programının metin dosyası okuması ve atomik append/rollback ön-okuması
   16 MiB ile sınırlıdır.
6. LSP en çok 256 açık belge, toplam 128 MiB belge metni ve bir mesaj için
   8 MiB outbound JSON taşır. Reddedilen belge eski sunucu durumunu değiştirmez;
   istemci S045 tanısı veya JSON-RPC kaynak hatası görür.
7. Kaynak/token aşımı S045; runtime adım/çıktı/koleksiyon/görev aşımı C023
   kimliğini kullanır. Sessiz truncate, sınırsız fallback ve host panic yasaktır.
8. K-130 tek metni 16 MiB, çalışma/istek başına muhafazakâr saklanan değer
   tahsisini yaklaşık 64 MiB ve süreç-geneli inbound+outbound ağ bağlantısını
   64 ile sınırlar. Metin/değer aşımı ayrı append-only C024 kimliğidir.
9. K-131 davranış değerlerini değiştirmeden HTTP/ağ, web oturumu, IO izi,
   LSP, paket/registry, tanı ve kalıcı dosya limitlerini domain görünümlerine
   ayırır. Eski sabit adları gerekirse API uyumu için yalnız bu görünümlere
   bağlı alias olabilir; ikinci sayısal sahip YASAKTIR.

## Açık sınır

Saklama fişi gerçek allocator/RSS telemetrisi değildir: değerlerin yaklaşık
dinamik grafiğini, ortam yazımlarını ve görev klonlarını güvenli tarafta fazla
sayar; silme veya yeniden bağlamada bütçeyi geri vermez. Böylece canlı grafiğe
üst sınır olur, fakat profiler sözü vermez. Domain sabit göçü tamamdır. LSP
outbound JSON'u bugün 8 MiB üstünde reddedilir, fakat tam yanıt kurulduktan
sonra ölçülür; üretim sırasındaki tahsis de sınırlandırılmadan B-025 kapanmaz.

## Kanıt

- 8 MiB üstü bellek kaynağı S045 ile reddedilir;
- 100 bin üstü yazma olayı C023 ile dışarı taşmadan kesilir;
- 1.024 üstü eşzamanlı görev grubu gelecekleri kurulmadan C023 olur;
- 16 MiB üstü sparse veri dosyası içerik tahsis edilmeden reddedilir;
- 256 üstü LSP belgesi depoya eklenmez ve 8 MiB üstü outbound gövde `-32001`
  JSON-RPC hatasına dönüşür;
- mevcut C019 özyineleme testi aynı merkezî 500 değerini tüketmeye devam eder.
- 16 MiB üstü birleştirme ve keyfî hassasiyetli sayı/para metni tahsisten önce
  C024 olur; saklanan değer ve görev ortamı klonları yaklaşık 64 MiB zarfı
  aşamaz;
- 64 bağlantı izni doluyken yeni izin reddedilir, bırakılan izin yeniden alınır.
- Mimari sahiplik testi on bir tüketici modülün sayısal limiti ortak profilden
  okuduğunu ve domain modüllerinin ilanlı satır bütçesinde kaldığını doğrular.
