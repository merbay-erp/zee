# ADR-178 — Yanıt ve yönlendirme rota gövdesini bitirir

- **Durum:** kabul
- **Tarih:** 25 Kasım 2026
- **İlgili kayıt:** K-403, spec/11 §5, ADR-059 (core freeze), ADR-066
  (güvenlik bulguları), ADR-157 (istek-yerel hata sınırı), GB-175

## Bağlam

Rota gövdesinde `yanıtını gönder` ve `yönlendir` yanıt taslağını belirliyor
ama akışı sürdürüyordu; aynı istekteki sonraki yanıt cümleleri sessizce yok
sayılıyordu ("ilk yanıt kazanır"). Lint bunu açıkça belgeliyordu (S047/S048
önerisi: "yanıt göndermek bloğu sonlandırmaz"). Web geliştiricilerinin
çoğu dilden (erken `return redirect(...)`) tanıdığı koruma kalıbı bu yüzden
fail-open çalışıyordu:

```text
POST "/sil" adresine istek geldiğinde
    oturum gerekli
    yetkili değilse
        "/giris" adresine yönlendir
    kaydı sil        ← yönlendirmeye rağmen koşuyordu
```

İstemci yönlendirmeyi görür, ama korumadan sonraki mutation (eylem,
veritabanı, çerez, oturum), giden istek ve çıktı koşar. Etkin dogfood ürünü
`urun-h` (ZeeMail) güvenlik denetimi bulguyu 23.11'de "Yüksek" olarak
işaretledi (ürün commit'i `8888734`, `docs/guvenlik/denetim-2026-11-23.md`);
bağımsız dış inceleme 25.11'de ürün `89edf15` anlık görüntüsünde yineledi
("Yönlendirme sonrası kod çalışması", Yüksek). Gözlenen: `web/kaynak/
rota_ayarlar.dil` `POST "/ayarlar/temsilci"` rotasında yeniden doğrulama
koruması yönlendirdiği hâlde sonraki temsilci ayarlama eylemi koştu ve kayıt
oluştu. Ürün geçici olarak korunan gövdeyi `değilse` dalına aldı. Erken yönlendirmeyi yetki
koruması olarak kullanan her Zee web uygulaması (zee-cekirdek tabanlı filo
siteleri dahil) aynı sınıfa açıktır.

## Karar

1. **Çalışma zamanı (birincil düzeltme):** rota gövdesi koşarken
   `yanıtını gönder` (sabit durumlu biçimi dahil) ya da `yönlendir` yanıt
   taslağını başarıyla belirlediği anda rota biter. Uygulama `programı bitir`
   (Ç000) ile aynı iç nöbetçi modelini kullanır: `Ç004` tanısı `Err` yolundan
   bloklar, döngüler, `içinde` blokları ve çağrılan işlem çerçeveleri boyunca
   yükselir; yalnız `web_istegini_calistir` yakalar ve olağan bitiş sayar
   (yanıt/çerez/oturum taslağı commit edilir). Nöbetçi rota yürütücüsü
   dışına sızmaz; IO izine ve operatör günlüğüne girmez.
2. **Kapsam:** yalnız rota gövdesi koşarken (thread-yerel bayrak). Üst
   düzey program, `test` blokları ve CLI çıktı kipi değişmez. Reddedilen
   yönlendirme (C022) ve yanıttan önce oluşan tanılar önceki gibi işlenir.
   Dosya yanıtı bir `Sonuç` ifadesidir; çağıranın başarısızlık dalı
   seçebilmesi için rotayı bitirmez.
3. **Statik yardım (ikincil):** `dil denetle --lint` rota gövdesinde kesin
   yanıttan (her dalı yanıt veren koşul dahil) sonraki cümleyi önerili S047
   ile gösterir. Yeni tanı kodu açılmaz; S047 zaten "ulaşılamaz kod"
   sınıfıdır, yalnız mesaj/öneri ayrışır. İşlem gövdesi rota dışından da
   çağrılabildiği için orada tahmin yürütülmez. S048 önerisindeki "bloğu
   sonlandırmaz" ifadesi düzeltildi.
4. **Yüzey:** grammar, kalıp, tanı kodu, CLI, IO izi şeması ve
   `dil::api::v1` değişmez. `Ç004` iç nöbetçidir; hata kataloğunun iç
   nöbetçi satırlarına eklenir.

## Değerlendirilen seçenekler

- **Yalnız derleyici tanısı (b):** "yanıttan sonra ulaşılamaz kod" tanısı
  koruma kalıbını yakalamaz: kalıpta yönlendirme `ise` bloğunun son
  cümlesidir, sonraki kod erişilebilir başka bir yolda durur. Statik bir
  "olası yanıttan sonra kod" hatası ise geçerli programların çoğunu kırar.
  Tek başına yetersiz; çalışma zamanı düzeltmesinin yardımcısı olarak alındı.
- **Yeni `rotayı bitir` cümlesi:** grammar genişletir (freeze), mevcut
  fail-open kodu düzeltmez.
- **`Akis` ile yeni dönüş türü:** işlem çağrıları ifade içinde
  değerlendirildiğinden yanıtı işlem çerçevesi dışına taşıyamaz; her döngü
  noktasında ayrı yayılım ister. Ç000 emsali tek yayılım yolunu zaten
  doğrulamıştır.
- **İlk yanıttan sonra yanıt cümlesini tanıya çevirmek:** sonraki
  mutation'ı engellemez, yalnız ikinci yanıtı görünür yapar.

## Uyumluluk ve ölçüm

Önceki semantikte bir rotada ikinci yanıt zaten etkisizdi; değişen tek şey
ilk yanıttan sonra koşan yan etkilerdir. Yerel ölçüm (25.11.2026): filo
siteleri, zee-cekirdek, ZeeMail ve depo içi dogfood dahil 1 199 tekil `.dil`
dosyasındaki 6 895 yanıt/yönlendirme cümlesinin 37'sinden sonra aynı yolda
kod koşuyordu. Hepsi ya koruma yönlendirmesi sonrası salt okuma/sayfa
üretimi (yanıtı zaten yok sayılan) ya da karşılıklı dışlayan koşullardı;
yanıttan sonra bilinçli yapılan bir mutation bulunmadı. Görünür yanıt hiçbir
vakada değişmez. Aynı yerel ağaçta 1 267 hedefte (`.dil` dosyaları ve
`proje.dil` kökleri) eski/yeni `dil denetle --lint --json` farkı sıfırdır:
yeni tanı da, yeni S047 uyarısı da yok.

Bilinçli sınır: yanıtı belirledikten sonra iş yapmak isteyen kod (ör. günlük
yazımı) işi yanıttan önceye taşımalıdır. Yanıt sonrası arka plan işi bu
ADR'nin kapsamında değildir.

## Kanıt

- `compiler/tests/rota_sonu_testi.rs`: koruma yönlendirmesinden sonra çerez,
  çıktı ve ikinci yanıt yok; işlemdeki ve döngüdeki yanıt rotayı bitirir;
  yanıttan önceki hata 500 kalır ve işçi sonraki isteği alır.
- `compiler/tests/lint_testi.rs`: rota S047'si, her dalı yanıtlayan koşul,
  koruma kalıbında uyarı yok, işlem/üst düzeyde tahmin yok.
- Regression `yonlendirme-rota-sonu`, `islemde-yanit-rota-sonu`
  (`regression/security/`, `web` kipi; önce 500, sonra yönlendirme/403).
