# K-334 — Açık depo iz temizliği: tarihçe yeniden yazımı

17 Kasım 2026 dış taraması (GB-131) iki tarihçe izi bıraktı: K-214 açık
depo politikasının tuzsuz SHA-256 terim listesi ve iç altyapı/envanter
metinleri; ayrıca üç commit kurumsal, 36 commit makine adı/yerel IP taşıyan
yazar adresi kullanıyordu. Aşama 1 (PR #14) güncel ağacı anahtarlı HMAC
politikasına geçirdi. Kullanıcı 17 Kasım'da tarihçenin de temizlenmesini
onayladı.

## Yöntem

K-326 ile aynı tek geçişli `git fast-export | fast-import` akışı (yerel,
depo dışı betik):

1. `docs/acik-depo-politikasi-v1.json` bütün tarihçeden çıkarıldı; eski
   tuzsuz hash listesi hiçbir erişilebilir commit'te yoktur.
2. Blob metinlerinde iç koşucu adı, sentetik host olarak kullanılan kuruluş
   adı, önizleme adı ve canlı ürün sayım/envanter cümleleri Aşama 1'deki
   genel ifadelerle değiştirildi; bir commit mesajındaki koşucu adı da.
   Kesin metin listesi açık depoya alınmaz.
3. Kurumsal ve makine adresli yazar/kaydeden kimlikleri GitHub noreply
   adresine çevrildi. Kişisel adresler ve GitHub web birleştirmelerinin
   kaydeden kimliği değişmedi.
4. `regression/v2.tsv`, iki beyan TSV'si, `docs/sir-yanlis-pozitif-v1.tsv`,
   onu doğrulayan `guvenlik_kapisi_testi.rs` satırı ve K-214/K-215/K-326
   eşlemelerinin yeni sütunu o ana kadar yazılmış yeni ata SHA'sına bağlandı
   (tek adımlı çözüm korunur). Commit mesajlarındaki tam/kısa SHA atıfları
   eşlendi. Merge topolojisi, tarihler ve K/vaka kimlikleri korunur.

Sonuç: 577 commit'in 275'i yeni SHA aldı (ilk değişen 18 Eylül). Yeniden
yazılmış ana dal ucu `16cdd1c95e03ac71dfb129a263e4c5e05d6dd5f8`; eski uç
`5d2fafd73371e38c4f10a1d28e26e34175b96cc6` ile ağaç farkı yalnız yukarıdaki
kayıt satırları ve ADR-061'deki bir sayımdır. Bu kayıt onun üstündeki takip
commit'idir.

## Eşleme ve tarihi kanıt

[SHA eşlemesi](sha-esleme-v1.tsv) eski→yeni tam SHA'dır; [ref eşlemesi](ref-esleme-v1.tsv)
dalları, etiketleri ve Enterprise ayna ucunu kaydeder. K-215 kuralı aynen
geçerlidir: ölçüm, soak, fuzz-RC, platform, ikili, SBOM/provenance, yama
kaynağı ve inceleme makbuzları gerçek ölçüldükleri özgün SHA'yı korur; soy bu
eşleme ve HEAD atası şartıyla doğrulanır (`dogfood_karsilastirma_testi`).
`v1.0.0-rc.1` etiket mesajı özgün RC kaynağını korur. İki
`dependency-retention/postgres-protocol-*` etiketi, adlarındaki korunan
kaynak SHA'sının yeni karşılığıyla yeniden adlandırıldı.

CI push tabanı `ci-taban-kimligi.py` ile yalnız iki kayıtlı eski uç için
(GitHub `5d2fafd7…`, Enterprise `d3ca6ea7…`) exact yeni ataya çevrilir.

Canlı ürünlerde ve tüketici depolarda eski SHA adlı derleyici ikilileri
(`dil-<7hex>`) içerik olarak aynı kaynaktan derlenmiştir; yeniden derleme
gerekmez, karşılık bu eşlemededir. Tüketici `dil-kur@<sha>` pinleri yeni
SHA'ya taşınır.

## Dış sınır

GitHub'ın yönetilen `refs/pull/*` ref'leri ve önbellekleri kullanıcı push'u
ile silinmez; destek talebi teyit edilene kadar GB-131 açık kalır. Git2 aynası
yeniden yazım sırasında HTTP 502 verdi; eşitlemesi ayrıca kaydedilir.
