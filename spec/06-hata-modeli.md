# 06 — Hata modeli

Normatif kaynak: RFC-0010 (kabul), RFC-0008 (geçici kabul).
Katalog: [hata-katalogu.md](../docs/hata-katalogu.md) — kaynakla iki yönlü
tutarlılığı testle zorlanır (katalog_testi).

## Tanı sözleşmesi (TANIMLI — RFC-0010)

Her tanı ŞU dördü taşır: **kod** (S/A/T/C/D + sayı), **Türkçe mesaj**,
**kaynak konumu** (satır + işaret), **öneri**. Rapor `Ayrıntı için:
dil hata KOD` satırıyla biter ve o komut katalogdan açıklama basar.
İngilizce sızıntı **YASAK**tır; iç hata bile Türkçe raporlanır.

Kod aileleri: **S** sözcükleme/dizim · **A** ad çözümü · **T** tür ·
**C** çalışma zamanı · **D** test doğrulaması · **Ç** iç akış (kullanıcıya
görünmez; Ç000 `programı bitir` nöbetçisidir).

## Hata sınıflandırması (TANIMLI — RFC-0008)

1. **Programcı hatası** → derleme tanısı (S/A/T). Örn. korumasız `değeri`
   erişimi T036'dır ve programa hiç giremez.
2. **Beklenen dünya hatası** → değere dönüştürülür: `... dene` Sonuç üretir
   (`dosyasını okumayı dene`, `sayısını almayı dene`); işlemden hata
   `"..." hatasını döndür` ile çıkar (mesaj Metin — T032).
3. **Beklenmeyen çalışma hatası** → C tanısıyla durdurma (taşma C002,
   sıfıra bölme C003...). Sessiz devam **YASAK**tır.

`boş`/`yok` ayrımı bilinçlidir: koleksiyonun boşluğu `boşsa`, değerin
yokluğu Seçenek'tir; `null` kavramı dilde yoktur.

## Test anlamı (TANIMLI)

`test "<açıklama>"` blokları `dil dene` ile koşar. Her test **taze ortamda**
ve **hermetik IO** ile çalışır: saat sabit, rastgelelik tohumlu, dosya/ağ
sahte. Başarısız doğrulama D001'dir ve beklenen/bulunan değerleri gösterir.
Testler kaynak dosyanın yanında yaşar ve playground dahil her yürütücüde
aynı sonucu verir.
