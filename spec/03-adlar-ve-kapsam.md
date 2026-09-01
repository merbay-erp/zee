# 03 — Adlar ve kapsam

Normatif kaynak: RFC-0004 (geçici kabul), günlük K-011/K-013/K-034/K-041.
Tanı kodları: A bölümü.

## Morfolojik ad çözümü (TANIMLI)

Bir kelime, kapsamdaki adlarla şu adaylar üzerinden eşleştirilir:

1. **Kelimenin kendisi** (ek yok).
2. **Ek ayıklama:** desteklenen hâl/iyelik/araç ekleri sondan atılır.
   Desteklenen ek listesi sürümlemeli grammar'ın parçasıdır (K-011);
   kaynak-of-truth `compiler/src/cozumleyici.rs: kok_adaylari`. Kapsam:
   zamir n'li biçimler (K-041: `zarı+ndan`), İKİ KATMANLI zincir
   (K-061: `fiyat+ı+yla`) ve ünsüz ikizleşmesi geri çevrimi
   (K-049: `üssü→üs`, sertleşmeyle `reddi→ret`).
3. **Ünsüz yumuşaması geri çevrimi:** `sayacı → sayaç`, `kitabı → kitap`,
   `yurdu → yurt`, `çocuğu → çocuk`.
4. **Ünlü düşmesi geri çevrimi:** `şekle → şekil`, `burnu → burun`.

Kurallar (TANIMLI):

- Adaylardan **tam bir tanesi** kapsamda bir ada denk gelirse çözüm odur.
- Hiçbiri denk gelmezse **A001** (tanı, kapsamdaki adları listeler).
- Birden fazlası denk gelirse **A002** — belirsizlik dilde hatadır; sessiz
  öncelik YOKTUR.
- Sözlük YOKTUR: çözüm yalnız kapsam adlarına bakar; bir kelimenin
  "Türkçede kelime olması" gerekmez.

## İleri morfoloji (TANIMLI — K-072)

Çözümlemenin simetriği üreteçtir: `ek_uydur`, soyut eki yeni köke Türkçe
uyum kurallarıyla giydirir (dörtlü/ikili ünlü uyumu, y/n tamponları,
ta/te benzeşmesi, çok-hecede p→b/ç→c/t→d/k→ğ, nk→ng her hecede).
Editörün yeniden adlandırması bunun üstüne kuruludur.

## Örtük çoğul (TANIMLI — K-013)

`her sayı için` gezerken koleksiyon, tekil adın çoğulundan bulunur:
`sayı → sayılar/sayiler` uyumuyla. Bulunamazsa A003.

## Blok kapsamı (TANIMLI — K-034)

- Gövdede (`ise`, döngü, `işlem`, kol...) doğan ad **gövdeyle ölür**;
  döngü değişkeni dahil.
- Gövde içinde **dıştaki** ada `olsun` ile atama, dıştaki adı günceller ve
  kalıcıdır — gölgeleme yapısal olarak yoktur; tür değişmezliği (T002)
  aynen uygulanır.
- Çözümleyici ve yorumlayıcı birebir aynı kuralı uygular.

## Benzersizlik

İşlem adları benzersizdir (A005); yapı adları benzersizdir (A006);
birimlerden gelen adlar sessiz gölgelenmez (A008 — bkz. 07).
