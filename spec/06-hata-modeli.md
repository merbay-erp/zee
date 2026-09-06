# 06 — Hata modeli

Normatif kaynak: RFC-0010 (kabul), RFC-0008 (geçici kabul).
Katalog: [hata-katalogu.md](../docs/hata-katalogu.md) — kaynakla iki yönlü
tutarlılığı ve sürümler arası kod↔anlam kimliği testle zorlanır
(`katalog_testi`, `tani_kimligi_testi`).

## Tanı sözleşmesi (TANIMLI — RFC-0010)

Her tanı ŞU dördü taşır: **kod** (S/A/T/C/D/P/Ç + sayı), **Türkçe mesaj**,
**kaynak konumu** (satır + işaret), **öneri**. Rapor `Ayrıntı için:
dil hata KOD` satırıyla biter ve o komut katalogdan açıklama basar.
İngilizce sızıntı **YASAK**tır; iç hata bile Türkçe raporlanır.

Kod aileleri: **S** sözcükleme/dizim · **A** ad çözümü · **T** tür ·
**C** çalışma zamanı · **D** test doğrulaması · **P** proje · **Ç** iç akış
(kullanıcıya görünmez; Ç000 `programı bitir` nöbetçisidir).

### Sürümler arası kimlik (TANIMLI — RFC-0010 §2.2)

Yayımlanmış tanı kodu aynı semantik olay için kalıcıdır. Aile/olay anlamı
değişecekse yeni kod ayrılır. Kaldırılan kod katalog ve şema-1 fixture'ında
`ayrılmış` mezar taşı olarak kalır ve tekrar etkinleştirilemez. Fixture; kodu,
aile uyumlu semantik anahtarı ve kanonik katalog özetini birebir korur.

Dinamik mesaj, öneri ve kaynak işareti kimliğin parçası değildir. Yönetilebilir
program değeri `Hata.kod` ile derleyici/çalıştırıcı tanısı `Tani.kod` ayrı
katmanlardır.

### Çoklu tanı ve kurtarma (TANIMLI — RFC-0010 §2.1)

Normal derleme ve çalışma ilk tanıda durur. `dil denetle`, JSON çıktısı ve LSP
aynı çoklu-tanı hattını kullanır. Lexer başarılıysa parser hatalı cümlenin
satır sonuna, varsa yalnız ona ait dengeli girinti gövdesinin sonuna
senkronlanır; sonraki aynı-girintili kardeş korunur. Kısmi AST yürütülebilir
değildir.

Parser/birim/checker tanıları kaynak konumunda kararlı sıradadır ve belge
başına en çok 20 kayıt yayımlanır. Aynı eksik tanımdan doğan A001/A003/A007
tekrarları ve başlığı bozuk işlemin çağrı tanıları bastırılır; kök neden bir
kez raporlanır (K-166, ADR-024 §8). Lexer'ın token üretemediği lexical
hata tek tanıdır. Bu bütçe tanı kimliğini veya kodunu değiştirmez.

## Hata sınıflandırması (TANIMLI — RFC-0008)

1. **Programcı hatası** → derleme tanısı (S/A/T). Örn. korumasız `değeri`
   erişimi T036'dır ve programa hiç giremez.
2. **Beklenen dünya hatası** → değere dönüştürülür: `... dene` Sonuç üretir
   (`dosyasını okumayı dene`, `sayısını almayı dene`); işlemden hata
   `"..." hatasını döndür` ile çıkar. Sonuç'un hata tarafı spec/15'teki
   yapılandırılmış `Hata`dır: kod, mesaj, neden ve veri taşır. Eski metin
   biçimi `GENEL` koduyla aynı çıktıyı korur; kodlu üretim
   `"DOSYA_YOK" kodlu "..." hatasını döndür` biçimidir.
3. **Beklenmeyen çalışma hatası** → C tanısıyla durdurma (taşma C002,
   sıfıra bölme C003...). Sessiz devam **YASAK**tır.

`boş`/`yok` ayrımı bilinçlidir: koleksiyonun boşluğu `boşsa`, değerin
yokluğu Seçenek'tir; `null` kavramı dilde yoktur.

Yönetilebilir `Hata` ile S/A/T/C/D/Ç tanısı ayrı katmanlardır. Hata programın
eşleyip sürdürebildiği değerdir; tanı derleme/çalıştırma raporudur ve örtük
olarak Sonuç'a çevrilmez.

## Test anlamı (TANIMLI)

`test "<açıklama>"` blokları `dil dene` ile koşar. Her test **taze ortamda**
ve **hermetik IO** ile çalışır: saat sabit, rastgelelik tohumlu, dosya/ağ
sahte. Bu dünyanın tohum, sanal saat, FIFO/fallback ve görünür adaptör
davranışı spec/22'deki `zee-io-1` profilidir. Başarısız doğrulama D001'dir ve
beklenen/bulunan değerleri gösterir.
Testler kaynak dosyanın yanında yaşar ve playground dahil her yürütücüde
aynı sonucu verir.
