# Öğretmen rehberi

Bir sınıfta (ya da evde, tek çocukla) zee'yi sıfır kurulum ve sıfır
internet ile kullanmanın yolu. Buradaki her adım bugün çalışan araçlara
dayanır; hayal ürünü özellik yoktur.

## 1. Kurulum seçenekleri

**A — Hiç kurulum yok (önerilen başlangıç):** `playground/zee-playground.html`
dosyasını USB'yle ya da ağ paylaşımıyla dağıt. Çift tıklayınca tarayıcıda
açılır; derleyicinin tamamı dosyanın içindedir, internet GEREKMEZ ve hiçbir
veri dışarı gitmez (ADR-007). Aynı tohum + aynı girdi = her makinede aynı
çıktı — sınıfta "bende farklı çıktı" tartışması yaşanmaz.

**B — Bilgisayara kurulum:** tek dosya `dil` ikilisi kopyalanır (PATH'e
eklenirse her yerden çalışır). Çocuğun çalıştıracağı komut:

```bash
dil çalıştır --güvenli program.dil
```

`--güvenli` çocuk modudur (K-047): ağ ve sunucu kapalı, dosya erişimi
programın klasörüyle sınırlı. Sınıf makinelerinde ölçünüz bu olsun.

**C — Proje iskeleti:** `dil yeni uzay-oyunum` — içinde `proje.dil`, çalışan
bir program ve bir test olan klasör üretir. Ardından `dil çalıştır uzay-oyunum`
ya da klasörün içinde `dil çalıştır .` yeterlidir.

## 2. Önerilen ders sırası (projeler/ kitaplığıyla)

Her proje tek oturumluktur; "şunu da dene" satırı ödevin kendisidir.

| Oturum | Proje | Kavram |
|---|---|---|
| 1 | Merhaba Dünya + [hikaye](../projeler/hikaye.dil) | yaz, sor, metin birleştirme |
| 2 | [quiz](../projeler/quiz.dil) | koşul, puan biriktirme |
| 3 | [carpim-tablosu](../projeler/carpim-tablosu.dil) | döngü, iç içe döngü |
| 4 | [zar-oyunu](../projeler/zar-oyunu.dil) | rastgelelik, koşul zinciri |
| 5 | [kumbara](../projeler/kumbara.dil) | ondalık para, "olana kadar" |
| 6 | [asal-sayilar](../projeler/asal-sayilar.dil) | bölümden kalan, bayrak |
| 7 | [gizli-dil](../projeler/gizli-dil.dil) | sözlük, kelimelere ayırma |
| 8 | [market-listesi](../projeler/market-listesi.dil) | işlem yazmak, TEST yazmak |
| 9 | [kelime-sayaci](../projeler/kelime-sayaci.dil) | sayaç sözlüğü deseni |
| 10 | [gun-sayar](../projeler/gun-sayar.dil) | tarih aritmetiği |
| 11 | [envanter](../projeler/envanter.dil) | kayıt tabloları, para biçimi, JSON yedeği |
| 12 | [mini-site](../projeler/mini-site.dil) | kendi web siteni sun! |

Sonrası: golden korpus (kolaydan zora numaralı 32 program) ve
[dil turu](dil-turu.md).

## 3. Hata kültürü — en önemli ders

zee'de hata mesajı ceza değil, dersin parçasıdır: her tanı Türkçedir,
yerini gösterir ve bir öneri taşır. Sınıf kuralı önerimiz:

1. Hata çıkınca ÖNCE çocuk okur, sesli.
2. Öneriyi uygular; yetmezse `dil hata <kod>` yazar (katalog ikilinin
   içindedir, internetsiz çalışır).
3. Hâlâ çözülmediyse öğretmene sorar.

"Belirsizlik dile giremez" ilkesi sınıfta hissedilir: iki anlamlı yazım
(A002, S030, S033) sessizce bir anlama bağlanmaz, hata verir — bu bir
tartışma fırsatıdır: "makine neden emin olamadı?"

## 4. Test yazmayı erken öğretin

`test "..."` bloğu + `... olmalı` doğrulaması dilin parçasıdır ve
playground'da bile koşar. 8. oturumdan itibaren her ödevin teslim ölçütü
"testi de yazılmış olması" olabilir. Testler hermetiktir: rastgelelik ve
saat sahte dünyadan gelir, aynı test herkeste aynı sonucu verir.

## 5. Gömülü kitaplık

dört gömülü birim her yerde — `matematik` (mutlak, üs, karekök, obeb,
okek), `liste_araclari` (toplam, uçlar, ortalama, medyan),
`metin_araclari` (tersi, ünlü sayımı) ve `sozluk_araclari` (en çok
geçen) — — playground dahil — kurulumsuz çalışır. Obeb işlemi ders kitabındaki
Öklit yönteminin kendisidir; kaynak kodu [kitaplik/](../kitaplik/) altında
zee'yle yazılıdır ve ÇOCUKLA BİRLİKTE OKUNABİLİR.

## 6. Gizlilik

Araçlar hiçbir veri toplamaz, hiçbir yere göndermez (ADR-007 bağlayıcıdır).
Okul ağı, veli izni, hesap açma gibi konular bu araç için GÜNDEME GELMEZ:
hesap yoktur, sunucu yoktur, telemetri yoktur.

## 7. Geri bildirim

Gözlemlerinizi (hangi kalıp zorladı, hangi hata mesajı anlaşılmadı)
[usability-kiti.md](usability-kiti.md) protokolüyle kaydederseniz dilin
kendisini iyileştirir: sözdiziminin onay kapısı gerçek öğrencilerdir.
