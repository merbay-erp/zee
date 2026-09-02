# Eşzamanlılık conformance profili

Bu belge K-124/B-011'in bakım sözleşmesidir. Amaç scheduler'ın iç
gerçeklemesini dondurmak değil; aynı program ve aynı `zee-io-1` girdileri için
kullanıcının görebildiği sonucu dondurmaktır. Gelecekte çok çekirdekli bir
runtime kullanılabilir, fakat `zee-esz-1` gözlemlerinden hiçbirini
değiştiremez.

Derleyiciden bağımsız kaynaklar:

- `conformance/eszamanlilik/sema-v1.schema.json` — JSON Schema 2020-12;
- `conformance/eszamanlilik/zee-esz-1.json` — profil sözü ve çalıştırılabilir
  Zee kaynak vakaları;
- RFC-0011 ve spec/14 — normatif algoritma, sahiplik ve kapsam sınırları.

## Gözlenebilir söz

`zee-esz-1` aşağıdakileri uyumluluk yüzeyi sayar:

1. Görev dış ortamı bildirim anında değer kopyası olarak alır; ilk poll yalnız
   `hepsini bekle`de başlar.
2. Hazır görevler ve aynı anda uyanan görevler kaynak sırasıyla ilerler.
3. Beklemeler toplanmaz; ortak tekdüze saat en yakın uyanışlara ilerler.
4. Sonuçlar tamamlanma sırasıyla değil kaynak sırasındaki görev adlarına
   bağlanır.
5. İç görev ağacının beklemesi dış gruptaki kardeşe yol verir.
6. İlk yönetilmemiş görev hatası kodunu korur; tamamlanmamış kardeşler sonraki
   çıktı, dosya veya başka IO etkisini üretemez.
7. Dış mutlak son tarih bütün görev ağacına yayılır ve yalnız kendi
   `yetişmezse` sahibine ulaşır.
8. `eylem` savepoint açılışından tamamlama/geri almaya kadar atomik scheduler
   dilimidir.
9. Görevde `programı N ile bitir` kodu korur ve kardeşleri iptal eder.

Bu söz; çıktı dizisi, sanal geçen süre, sonlanma türü/kodu ve son dosya
içeriklerini kapsar. Thread sayısı, future türü, poll altyapısı ve gerçek
duvar-saati zamanlaması gözlenebilir değildir.

## Korpus

İlk korpus 10 vaka taşır:

- snapshot ve tembel başlangıç;
- join öncesi ana kapsam, ilk poll ve eşit uyanış sırası;
- farklı uyanışlarla örtüşen süre ve kaynak sıralı sonuç bağlama;
- birden çok eşit bekleme turu;
- ortak dosyaya aynı anda hazır iki etkinin sırası;
- iç görev ağacı ile dış kardeş ilerlemesi;
- hata sonrası kardeş iptali;
- dış son tarihin bütün ağaca yayılması;
- atomik eylem rollback'i;
- program çıkış kodu ile kardeş iptali.

Her vaka doğrudan Zee kaynak metni ve `beklenen` gözlemini taşır. İkinci bir
derleyici Rust AST/HIR/runtime adlarını bilmeden kaynakları derleyip aynı
sonuçları üretmelidir.

```bash
cd compiler
cargo test --locked --test eszamanlilik_conformance_testi
```

## Sürüm ve değişmezlik

`zee-esz-1.json` ve `sema-v1.schema.json` yayımlandıktan sonra genel
`scripts/conformance-korugu.sh` tarafından Git geçmişine karşı korunur; yerinde
değiştirilemez veya silinemez. Gözlenebilir davranış değişikliği yeni
`zee-esz-N` profilini, veri biçimi kırılması yeni `sema-vN.schema.json`
dosyasını ve RFC/spec/sürüm geçişini birlikte ister.

Çok çekirdekli runtime bu profil altında yalnız iç optimizasyondur. Korpusun
tamamı ve spec/14'ün normatif sırası aynı kaldığı sürece kullanılabilir;
platform scheduler'ı ya da yarış sonucu Zee programına sızarsa uyumsuzdur.

Bu sonlu korpus bütün programların biçimsel ispatı değildir. Normatif referans
model spec/14'tür; mevcut ayrıntılı `ag_ve_esz_testi.rs`, T033/T051 sahiplik
olumsuzları ve `zee-io-1` testleri tamamlayıcı kapılar olarak kalır.
