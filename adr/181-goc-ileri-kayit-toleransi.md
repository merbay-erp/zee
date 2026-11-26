# ADR-181 — `dil göçür` ileri kayıt toleransı (A/B kök geri dönüşü)

- **Durum:** kabul
- **Tarih:** 26 Kasım 2026
- **İlgili kayıt:** K-407, `urun-i` (ZeeOS), ADR-095 (göç durum/kuru),
  ADR-059 (core freeze), ADR-064 (uyumluluk yüzeyi), spec/25

## Bağlam

`dil göçür` (ADR-095) `_zee_gocleri` tablosunda proje ağacında olmayan bir
kayıt görünce C026 ile durur: "Veritabanında kayıtlı bir migration dosyası
proje ağacında yok". Bu, kaybolan ya da yeniden adlandırılan göçü yakalayan
doğru fail-closed kuraldır.

ZeeOS A/B kök güncellemesi aynı veritabanını iki kök dilimiyle paylaşır.
Yeni dilim N+1 açılışta daha yeni bir göçü uygular; sağlık denetimi düşer ve
önyükleyici eski dilim N'ye geri döner. Dilim N'nin göç birimi `dil göçür`ü
koşar, N+1'in kaydını ağaçta bulamaz ve düşer: geri dönüş, geri dönmenin
tam amacı olan çalışan eski sürümü açamaz. Elle tablo düzeltmek veritabanını
N+1'e yeniden geçişte tutarsız bırakır. Kullanıcı kararı (26 Kasım 2026):
"ileri şema toleransı", açık bayrakla. Ürün kanıtı ve yeniden üretim
adımları: [`dogfood/urun-i/KOKEN.md`](../dogfood/urun-i/KOKEN.md) (ZeeOS
`6e8af89a`, defter uygulaması kümeleri N ve N+1).

## Karar

1. **Bayrak:** `dil göçür --ileri-kayitlara-izin`. Kip değil toleranstır;
   bayrak tekildir, `--durum` ve `--kuru` ile birlikte verilebilir.
   Bayraksız davranış bayt bayt aynıdır.
2. **Tolere edilen tek şekil:** ağaçtaki her göç aynı sürüm, ad ve SHA-256
   ile uygulanmış ve ağaçta olmayan her kaydın sürümü ağaçtaki en yüksek
   sürümden büyük. Bu kayıtlar "ileri kayıt" olarak raporlanır; migration
   SQL'i yürütülmez, kayıtlar silinmez ya da değiştirilmez. Uygulama yolu
   bugünkü metadata tablosu hazırlığını ve advisory lock'u alır, ardından
   boş transaction'ı commit eder.
3. **Fail-closed korunur:** bayrak varken de C026'dır —
   değişmiş ad ya da özet; ağaçtaki en yüksek sürümden küçük ya da ona eşit
   kayıp dosya; ileri kayıt varken ağaçta uygulanmamış (araya giren) göç
   ("İleri kayıtlar varken bekleyen migration uygulanamaz", `surum` verisi).
   Son kural, eski ağacın yeni şemaya yarım bir göçü uygulamasını engeller.
4. **Çıktı:** apply ve `--durum`/`--kuru` özet satırından sonra
   `İleri kayıt: N kayıt …` yazar; apply ve `--durum` her ileri kayıt için
   `surum\tileri\tad\tsha256` satırı ekler. İleri kayıt yoksa çıktı değişmez.
5. **API:** iç `dil::postgresql` modülünde `gocleri_planla_secenekli` ve
   `gocleri_uygula_secenekli` eklenir; eski işlevler `false` sarmalayıcısıdır.
   `GocPlani`/`GocRaporu` `ileri` alanını taşır. `dil::api::v1` değişmez.

## Neden daha geniş değil

İleri şemanın eski kodla uyumu (eklenen sütun, değişen kısıt) derleyicinin
bilemeyeceği bir ürün sözleşmesidir; bayrak yalnız "bu kayıtları tanımıyorum
ama dokunmuyorum" der. Ürün, ileri göçlerin geriye uyumlu (genişleten)
olmasından sorumludur. Kayıtları geri alma, aşağı göç (down migration) ya da
kayıt temizliği AÇIKTIR ve ayrı ürün kanıtı ister.

## Güvenlik

Yeni ağ, dosya ya da kimlik yüzeyi yoktur; bayrak yalnız daha az SQL yürütür
ve hiçbir kaydı değiştirmez. Kaybolan/değişen göç kontrolü bayrakla da
geçerlidir. Bu yüzden ayrı GB satırı açılmaz.

## Kanıt

- Saf plan: `compiler/src/postgresql.rs`
  `goc_plani_ileri_kayitlari_yalniz_bayrakla_ve_ust_sinirda_tolere_eder`,
  `goc_plani_ileri_kayit_bayraksiz_ve_ara_kayipta_reddedilir`,
  `goc_plani_ileri_kayit_degismis_ad_ozet_ve_bekleyen_gocu_gizlemez`.
- Gerçek PostgreSQL 16 (dedicated, `scripts/postgresql-goc-ci.py`):
  `izole_goc_durum_kuru_checksum_rol_ve_atomik_uygulama_sinirini_korur`
  ileri kayıtta bayraksız reddi, bayrakla owner/runtime planını, SQL'siz
  apply'ı, araya giren göç ve değişmiş özet reddini, metadata'nın
  değişmediğini sınar; ardından `dogfood/urun-i/defter-geri-donus/`
  aynasında KOKEN a–d adımlarını (N, N+1, bayraksız N reddi, bayrakla N ve
  N+1'e yeniden geçiş) koşar.
- CLI: `compiler/tests/cli_goc_plani_testi.rs`
  `goc_ileri_kayit_toleransi_kiplerle_birlikte_kabul_edilir_ve_tekrari_reddedilir`.

Yeni tanı kodu yoktur; C026 korunur. Katman kenarı, bağımlılık, önbellek ya
da kalıcı durum eklenmez; `cli-goc-ileri-kayit` profil satırı
`dil-yuzeyi-v1.tsv`'ye girer.
