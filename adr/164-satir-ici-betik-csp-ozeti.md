# ADR-164 — Satır içi betik CSP özeti yalnız kaynak değişmezinden

- **Durum:** kabul
- **Tarih:** 23 Kasım 2026
- **İlgili kayıt:** K-379, `urun-h` güvenlik denetimi, spec/12 K-379 bölümü,
  ADR-059 (core freeze), ADR-066 (güvenlik bulguları), ADR-081 (resmî site
  CSP sınırı), GB-153

## Bağlam

Gerçek web adaptörü (`--web-proxy` ve `--deneysel-web`) `text/html` 200
yanıtının gövdesini tarıyor, öznitelik taşımayan **her** `<script>` bloğunun
SHA-256 özetini CSP `script-src` yönergesine ekliyordu (Eylül 2026,
`dcd04ad`). Amaç Zee kodunun yazdığı satır içi betiklerin `'unsafe-inline'`
olmadan çalışmasıydı; ama yanıt gövdesi düz bir Metin olduğundan çalışma
zamanı betiğin kökenini bilmiyordu. Uygulamada tek bir kaçışsız yansıma
(ör. `?q=<script>…</script>`) saldırganın betiğini de özetletiyor ve tarayıcı
onu çalıştırıyordu: CSP XSS'e karşı derinlemesine savunma sağlamıyordu.
Etkin dogfood ürünü `urun-h` (e-posta sunucusu) güvenlik denetimi dört
yansıyan XSS'te saldırgan betiğinin özetinin başlığa eklendiğini gördü
(ürün tarafındaki yansımalar ayrıca kapatıldı).

Yeniden üretim (bu depo, yerel Darwin, release ikili): `GET /ara?q=…`
gövdeyi `"<p>" ile q ile "</p><script>document.title='zee'</script>"`
olarak yansıtan program; düzeltme öncesi CSP iki özet taşıdı
(`alert(1)` ve `document.title='zee'`) ve tarayıcıda yansıyan betik çalıştı.

Dil durumu: görünüm bloğu (K-212) metin ve öznitelikleri kaçışlar, `ham`
düğümü ve `html güvenlisi` dışındaki dize birleştirme kaçışsızdır; yanıt
gövdesi her yoldan sıradan `Metin` olur. `Deger::Metin` köken (şablon/değer)
bilgisi taşımaz.

## Karar

1. **Güven kökeni derleme zamanı değişmezidir.** Program derlendikten sonra
   ve ilk istekten önce, ana kaynak ile derlemenin yüklediği her birimin
   (yerel, paket, gömülü) kaynağı derleyicinin kendi lexer'ıyla yeniden
   sözcüklenir; her dize değişmezinin kaçışları çözülmüş değerinde tam duran
   öznitelikiz `<script>` gövdelerinin özetleri güvenilir kümeyi kurar.
   `test` bloklarındaki değişmezler (parser'ın kuralıyla: derinlik 0'da satır
   başı `test` ve girintili gövdesi) dışarıda kalır: XSS regresyon testleri
   tam da saldırganın göndereceği `<script>alert(1)</script>` gibi dizgileri
   taşır (`urun-h` incelemesi, ilk düzeltme `4b7c962` bunları izinli
   sayıyordu).
2. **Yanıtta yalnız güvenilir özet.** Gövde taraması aynı kuralla (ilk
   `</script>` kapanışı) yapılır; kümede olmayan özet eklenmez. İlk
   engellenen betikte süreç başına bir kez içeriksiz stderr uyarısı.
3. **Kapalı varsayılan.** Küme kurulmadıysa hiçbir satır içi betik izinli
   değildir; `'unsafe-inline'` hiçbir koşulda yoktur.
4. **Yüzey değişmez.** Yeni grammar, kalıp, CLI bayrağı, tanı kodu,
   `dil::api::v1` ya da bağımlılık yok; değişiklik yalnız `dil_cli`
   adaptöründedir (`sozcukleyici` okuması).

## Reddedilenler

- **Yanıt başına nonce.** Çalışma zamanı gövdedeki hangi `<script>`
  etiketinin şablondan, hangisinin yansımadan geldiğini bilemez; her etikete
  nonce eklemek bugünkü açığı aynen korur.
- **Değer köken izi (taint).** `Metin` değerlerine köken bayrağı eklemek
  birleştirme, liste, sözlük, dosya, veritabanı ve IO izi yollarının hepsine
  dokunan semantic değişikliktir; core freeze altında güvenlik düzeltmesi
  için orantısız.
- **Değişmez parçalarından kurulabilen betiğe izin.** Saldırgan, kaynaktaki
  kısa değişmezleri (`"x"`, `"/"`) dizerek yeni program kurabilir.
- **Satır içi betiği tamamen kapatmak.** Dize değişmezindeki meşru betikleri
  (yerel ERP birimleri, `sablonlar/icerik-paneli`) gereksizce kırar.

## Kanıt

- Birim: `main.rs::yansiyan_betik_csp_ozeti_alamaz`,
  `kaynak_betik_ozetleri_yalniz_tek_degismezi_sayar`,
  `html_scriptleri_icerige_bagli_csp_ozetiyle_calisir`.
- Regression `regression/security/test-blogu-betik-csp-ozeti`: test
  bloğunda değişmez duran `alert(1)` yansısa da engelli (`4b7c962`'de izinli).
- Regression `regression/security/yansiyan-betik-csp-ozeti` (kip
  `native-web-csp`, gerçek worker): düzeltme öncesi `izinli alert(1)`,
  sonrası `engelli alert(1)` ve `izinli document.title='zee'`.
- Tarayıcı (Chromium, `--deneysel-web`): önce yansıyan betik çalıştı;
  sonra CSP ihlaliyle engellendi, değişmez betik çalıştı.
- Tüketici karşılaştırması (önce/sonra aynı kaynak, gerçek native sunucu):
  resmî site (`dogfood/zeedili-com`) 12 rotada durum ve CSP birebir aynı
  (satır içi betik yok; `src` ve JSON-LD); yerel ERP UI birimlerinden 23
  değişmez betik taşıyan sayfada 22 özet birebir aynı. Envanter (gerçek
  lexer, 1.926 yerel `.dil`): 155 tam değişmez betik; tek değişmezde
  kapanmayan 55 adaydan biri dışında hepsi test dizesidir. O tek kullanım
  (betiği çalışma zamanında yalnız değişmezleri birleştirerek kuran ve
  ürünün yalnız kendi testlerinden çağrılan bir kabuk birimi) artık izin
  almaz; göç yolu aşağıda. Paylaşılan site çekirdeği profilleri yalnız
  `<script src>` ve JSON-LD kullanır.

## Sonuçlar ve göç

- Meşru satır içi betik **tek bir dize değişmezinde** tam durmalıdır.
  Çalışma zamanı verisi (CSRF belirteci, kimlik, ayar) betiğe
  birleştirilmez; kaçışlı `data-*` özniteliğine yazılıp betikten okunur.
- Parçalı kurulan ya da dosyadan/veritabanından okunan betik statik dosyaya
  (`<script src="/assets/…">`) taşınır ya da tek değişmeze toplanır.
- Uyarı satırı operatöre engellenen betiği haber verir; içeriği yazmaz
  (günlük enjeksiyonu yok).

## Açık

- Kaynak değişmezindeki betiğin saldırganca yeniden yansıtılması (özet
  CSP'nin genel sınırı; yalnız geliştirici betikleri). Yalnız rota/görünüm
  çıktısına akan değişmezleri saymak veri akışı analizi ister; freeze
  sonrası adaydır.
- `style-src 'unsafe-inline'` ayrı bir sertleştirme adayıdır.
- Değer köken izi ileride gerçek dogfood ihtiyacıyla ayrı RFC olabilir.
