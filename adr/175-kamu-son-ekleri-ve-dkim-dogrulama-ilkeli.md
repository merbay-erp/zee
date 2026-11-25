# ADR-175 — Kamu son ek listesi (PSL) ve DKIM imza doğrulama ilkeli

- **Durum:** kabul
- **Tarih:** 25 Kasım 2026
- **İlgili kayıt:** K-390 ikinci dilimi, RFC-0044 Aşama B1b ve B2b, K-355
  (`urun-h`), ZM-F024, ZM-F025, ADR-059 (core freeze), ADR-064 (dil yüzeyi),
  ADR-011 (intrinsic sınırı), ADR-038 (tedarik zinciri), ADR-167 (K-389
  SHA-256 yüzeyi), ADR-169 (B1a DNS), K-397/B3 (DKIM imza tarafı), spec/31,
  spec/21, spec/23, GB-168

## Bağlam

ZeeMail (`urun-h`) gelen postada DMARC değerlendirmesi için iki yapı taşına
muhtaçtır. Birincisi `From:` alanının **kurumsal alanı**dır (RFC 7489 §3.2,
ZM-F025). İkincisi DKIM imzasının doğrulanmasıdır (RFC 6376, RFC 8463,
RFC 8301; ZM-F024 gövde özetinin devamı). B1a (ADR-169) DNS TXT sorgusunu
getirdi; DKIM anahtarı (`<s>._domainkey.<d>`) ve DMARC kaydı (`_dmarc.<d>`)
artık sorulabiliyor, ancak doğrulama ve kurumsal alan hesaplanamıyor.

24–25 Kasım 2026 merkez kararı (b) dogfood-minimal kapsamı belirledi:
- PSL veri dosyası ve `kurumsal alanı` ifadesi.
- Ring ile RSA-SHA256 ve Ed25519 doğrulama ilkeli.
- `rsa-sha1` tanıyla reddedilir; 1024 bitten kısa ve 4096 bitten uzun RSA
  anahtarı reddedilir.
- Kanonikleştirme Zee'dedir; SHA-256 için K-389 yüzeyi kullanılır, ikinci
  bir özet API'si açılmaz.
- `Authentication-Results` host'ta işlenmez.

Uygulama günü iki sürtünme ortaya çıktı:
1. **CRLF.** DKIM'in kanonik imza girdisi ve gövdesi CRLF satır sonu ister
   (RFC 6376 §3.4). Zee metin sabitinde `\r` kaçışı yoktur (spec/01:
   yalnız `\"`, `\\`, `\n`). Bağlantı satır okuyucuları da CRLF'yi atar.
   Zee kodu CRLF'li baytı Metin olarak kuramaz; K-389 `metnin sha256
   özeti` bu yüzden CRLF'li kanonik gövdeyi özetleyemez.
2. **Biçim.** `bh=` base64'tür, K-389 `metnin sha256 özeti` ise
   onaltılıktır.

## Karar

1. **PSL derleyiciye gömülü değildir.** Proje dosyayı, proje köküne göreli
   yol ve exact SHA-256 ile bildirir:
   ```text
   kamu_son_ekleri "veri/public_suffix_list.dat" olsun
   kamu_son_ekleri_özeti "<64 küçük harf onaltılık>" olsun
   ```
   - İki alan birlikte ve yalnız `dns` bildirimiyle geçerlidir. Mutlak yol,
     `..` ya da ters bölü içeren yol P015'tir; yeni proje tanı kodu
     açılmaz.
   - Host açılışta dosyayı bir kez okur, özeti doğrular ve ayrıştırır;
     hata varsa süreç kaynak yürütülmeden çıkış 2 verir. Kural listesi
     süreç ömrünce değişmez.
   - Dosya en çok 4 MiB ve 100 000 kural olabilir.
   - ICANN ve PRIVATE bölümlerinin ikisi de kullanılır.
   - ASCII dışı (U-label) kural satırı ilk sürümde yok sayılır; sorgu adı da
     yalnız ASCII LDH ve alt çizgi etiketleri kabul eder.
2. **`<ad> alanının kurumsal alanı`** bir Metin verir. Algoritma
   publicsuffix.org "en uzun eşleşen kural"dır: istisna kazanır, joker bir
   etiket tüketir, varsayılan kural `*`'dır. Sonuç kamu son eki artı bir
   etikettir, küçük harfli ve sonda noktasızdır.
   - Geçersiz ad, kamu son ekinin kendisi ya da daha kısa bir ad boş Metin
     verir; DMARC kodu bunu hizalanmamış sayar. Saldırganın denetlediği
     `From:` alanı yürütmeyi durduramaz.
   - Yetkinlik `dns`'tir.
   - Liste bildirilmemişse yürütme C034 ile durur. Bu, bir yapılandırma
     hatasıdır.
   - IO izine `kurumsal_alan` olayı (`[ad]` → metin) yazılır; replay
     listesiz yeniden oynatır.
3. **DKIM doğrulama ilkeli:**
   ```text
   <b> dkim imzasını <veri> verisi için <p> anahtarıyla <a> olarak doğrulamayı dene
   ```
   - Dört işlenenin her biri tekil token'dır. Sonuç Metin sözlüğüdür:
     `doğrulandı` (`"doğru"`/`"yanlış"`), `algoritma` ve `anahtar_biti`.
   - Yetkinlik `kriptografi`'dir. Etki saftır ve IO izine olay yazılmaz;
     aynı girdi aynı sonucu verir.
   - `rsa-sha256` RSASSA-PKCS1-v1_5 + SHA-256'dır. `p=` SPKI ya da çıplak
     PKCS#1 `RSAPublicKey` olabilir; birçok yayıncı ikincisini kullanır.
   - `ed25519-sha256` RFC 8463'e uyar: imza SHA-256(veri) üzerinde
     PureEdDSA'dır ve `p=` ham 32 bayttır.
   - Kriptografi rustls'in kilitli aynı `ring 0.17.14`'üdür; doğrudan
     bağımlılık olarak açıldı, ağaca yeni crate girmedi.
     `RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY` doğrulayıcısı
     kullanılır, 4096 bit üst sınırı host'ta uygulanır.
   - **İmzanın tutmaması ret değildir:** başarılı Sonuç'ta
     `doğrulandı=yanlış` olarak döner (DKIM `fail`). Ayrıştırılamayan ya da
     politika dışı girdi ise C034 `tür` verili rettir (DKIM `permerror`).
     Ret türleri: `algoritma_reddedildi` (`rsa-sha1`),
     `algoritma_bilinmiyor`, `anahtar_iptal` (boş `p=`), `anahtar_bozuk`,
     `anahtar_kısa`, `anahtar_uzun`, `imza_bozuk` (base64 değil ya da boyu
     anahtara uymuyor) ve `veri_bozuk`.
   - Algoritma, anahtar ve imza kriptografiden önce elenir; böylece 4096
     bit üstü anahtar ve boyu uymayan imza CPU harcatamaz.
4. **LF → CRLF (TANIMLI).** `veri` içindeki her LF imza girdisinde CRLF
   olur; veride CR varsa `veri_bozuk` döner.
   - Kanonikleştirme yine Zee'dedir: `h=` seçimi, relaxed/simple başlık
     kuralları ve `b=` değerinin boşaltılması. Zee satır sonunu `\n` ile
     yazar.
   - SMTP alıcısı çıplak CR/LF'yi zaten reddettiği için (`ag_satiri`)
     dönüşüm kayıpsızdır.
   - Çift dönüşüm ve çıplak CR belirsizliği yapısal olarak kapalıdır.
   - Aynı kural `dkim_kripto::kanonik_baytlar` ile K-397 imza tarafına
     açıktır.
5. **`<x> gövdesinin dkim gövde özeti`** (`kriptografi`, 25 Kasım 2026
   merkez kararı) Zee'nin kanonikleştirdiği gövdeyi aynı LF→CRLF kuralıyla
   baytlara çevirir ve SHA-256'sını base64 Metin olarak verir (`bh=`
   biçimi). Boş simple gövde Zee'de `"\n"`, boş relaxed gövde `""`dir.
   Gövde CR içerirse yürütme C034 ile durur; SMTP gövdesi çıplak CR
   taşıyamadığı için bu bir ürün kanonikleştirme hatasıdır, saldırgan girdisi
   değil.
   - Bu genel bir özet API'si değildir: yalnız DKIM'e özgü CRLF kuralını
     taşır. Simple/relaxed gövde kanonikleştirmesi ve `l=` Zee'dedir.
   - Önce düşünülen `base64 metninin onaltılık karşılığı` dönüşümü
     minimalite gereği çıkarıldı; özet doğrudan `bh=` biçiminde gelir.
6. **Tek kripto yüzeyi.** `dil::dkim_kripto` algoritma adlarını, bit
   sınırlarını, imza girdisi kuralını (`kanonik_baytlar`) ve gövde özetini
   (`govde_ozeti`) tek yerde tutar. K-397 (B3) imza
   tarafı aynı modüle eklenir; ikinci bir algoritma tablosu açılmaz.
7. **Host'ta olmayanlar:** `Authentication-Results` temizliği/yazımı,
   başlık ayrıştırıcısı, SPF, DMARC politikası ve ARC. Bunlar RFC-0044
   B2a/B2c/B2d/B2e'de ayrı K-işleriyle gelir.

## Gerekçe

- **Kanonikleştirme neden Zee'de:** DKIM'in kırılgan yeri politika ve
  ayrıştırma tutarlılığıdır ("Composition Kills"). Bu kararlar ürünün
  başlık ayrıştırıcısıyla birlikte değişir. Host'un vermesi gereken tek
  şey, Zee'de yazılması güvensiz ya da imkânsız olan kriptografik
  çekirdektir.
- **Neden ring:** rustls'in kilitli aynı sürümü olduğundan yeni bir
  tedarik zinciri yüzeyi açmaz. ed25519-dalek de ağaçtadır ve testte
  bağımsız diferansiyel imzalayıcı olarak kullanılır.
- **Neden PSL dosyası:** derleyiciye gömülü bir liste, ürünün DMARC
  davranışını derleyici yükseltmesine gizlerdi. Exact SHA-256 ise aynı
  liste ile aynı kurumsal alan sonucunun çıkmasını yeniden üretilebilir
  kılar.

## Sonuçlar

- Olumlu: ZeeMail TXT ile anahtarı alıp imzayı doğrulayabilir; DMARC
  hizalaması için kurumsal alanı hesaplayabilir.
- **Açık:**
  - `dil dene` için PSL yüklemesi; şu an yalnız gerçek süreç ve hermetik
    `ToplayanIo` yüklüyor.
  - `psl` ve `dkim_anahtari` fuzz hedefleri. Şimdilik 20 000'er
    deterministik bozulmuş girdi birim testi var.
  - IDN (U-label) PSL kuralları.
  - DMARCbis ağaç yürüyüşü.
  - PSL güncellik uyarısı (CLI/lint).
- Risk: ayrıştırıcı tutarlılığı Zee kodunun sorumluluğundadır. Host tek
  başlık ayrıştırıcısını B2a'da verecektir; GB-168 bu riskin açık
  kaydıdır.
