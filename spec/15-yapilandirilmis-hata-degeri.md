# 15 — Yapılandırılmış Hata değeri

Normatif kaynak: RFC-0008 §3 (geçici kabul), K-091. Durum: **TANIMLI**.

## 1. Değer ve tür

`Sonuç<T>` gösterimi kaynakta tek başarı türü parametresi taşır; hata tarafı
her zaman `Hata`dır. `Hata` birinci sınıf, değişmez bir değerdir ve şu dört
alanı ZORUNLU olarak taşır:

- `kodu: Metin` — `[A-Z][A-Z0-9_]*` biçiminde kararlı etiket;
- `mesajı: Metin` — kullanıcıya gösterilebilir Türkçe açıklama;
- `nedeni: Seçenek<Hata>` — yoksa `yok`, varsa bir alt hata;
- `verisi: Sözlük<Metin, Metin>` — ekleme sırası korunan bağlam.

`Hata`, açık işlem parametresi/dönüşü ve `Hata seçeneği` gibi kapsayıcı tür
yazımlarında kullanılabilir. `sonucun hatası` ifadesinin statik türü Hata'dır;
bu erişim yalnız `başarısızsa` daraltması altında güvenlidir (T036).

## 2. Üretim ve yeniden yayma

Normatif dizim:

```text
<mesaj> hatasını döndür
<KOD> kodlu <mesaj> hatasını [<Hata> nedeniyle] [<Metin sözlüğü> verisiyle] döndür
```

- Eski biçim `GENEL` kodlu, nedensiz ve boş verili Hata üretir.
- `<KOD>` bir metin sabitidir; biçimi S044 ile derlemede doğrulanır.
- Mesaj Metin değilse T032; neden Hata ya da veri Metin sözlüğü değilse T052
  üretilir.
- Neden ve veri birlikteyse kaynak sırası `nedeniyle` → `verisiyle`dir.
- Bir Hata, `<hata> hatasını döndür` ile bütün alanları korunarak yeniden
  yayılabilir. Bu biçime neden/veri eklemek T052'dir; zenginleştirme yeni kodlu
  bir Hata'nın eski Hata'yı neden olarak sarmasıyla yapılır.
- Hata değerleri değişmez ve yalnız yeni hata eski nedeni sarabilir; bu yüzden
  neden zinciri döngü içeremez.

## 3. Erişim, eşleme ve gösterim

`hatanın kodu`, `hatanın mesajı`, `hatanın nedeni`, `hatanın verisi` alanların
normatif erişimleridir. Kod sıradan Metin olduğundan `göre / ise` ile eşlenir.
Neden Seçenek olduğundan `varsa` ile daraltılmadan `değeri` alınamaz.

Geriye uyumluluk ZORUNLUDUR:

- `hata yaz`, `"Hata: " ile hata yaz` ve `hatanın metni` yalnız mesajı verir;
- K-091 öncesi `"..." hatasını döndür` kaynakları aynı kullanıcı çıktısını
  üretir;
- yapılandırılmış alanlar ancak açık erişimle görünür.

`hatanın json metni` şu sabit anahtar sırasında serileşir: `kod`, `mesaj`,
`neden`, `veri`. Neden yoksa JSON `null`, veri yoksa `{}` yazılır. İç içe
neden aynı kuralla özyinelemeli serileşir; veri ekleme sırasını korur.

## 4. Yerleşik kodlar

V1 çekirdeğinin beklenen hata kodları:

| Üretici | Kod |
|---|---|
| Eski `<mesaj> hatasını döndür` | `GENEL` |
| `dosyasını okumayı dene` | `DOSYA_OKUMA` |
| `sayısını almayı dene` | `SAYI_BICIMI` |
| `ondalığını almayı dene` | `ONDALIK_BICIMI` |

Bu kodlar programların eşleme sözleşmesidir. Bir kodun anlamını değiştirmek ya
da kaldırmak kırıcı değişikliktir; sürüm notu ve semver incelemesi ister.

## 5. Tanıdan ayrım

`Hata`, programın yönetebildiği beklenen dünya hatasıdır. S/A/T/C/D/Ç kodlu
`Tani` derleyici/runtime raporudur; kaynak konumu ve öneri taşır, `Sonuç` içine
örtük çevrilmez. Aynı `kod + mesaj` fikrini paylaşmaları iki katmanın
karıştırıldığı anlamına gelmez.

## 6. Conformance kanıtı

Bir gerçekleme en az şunları aynı anda kanıtlamalıdır:

- kod/mesaj erişimi ve `göre` eşlemesi;
- neden zincirini Seçenek daraltmasıyla açma;
- Metin sözlüğü verisinin korunması ve deterministik JSON;
- üç yerleşik `dene` kodu ile `GENEL` geriye uyumu;
- Hata yeniden yayılırken bütün yapının korunması;
- geçersiz kod için S044, yanlış neden/veri için T052;
- eski Sonuç programlarının aynı insan çıktısını üretmesi.
