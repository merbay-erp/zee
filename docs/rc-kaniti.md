# Exact kaynak için dış RC kanıtı — K-265

[ADR-124](../adr/124-exact-kaynak-dis-rc-kaniti.md) temiz kaynak S için sonradan oluşan RC kanıtını, aynı kaynak SHA’sına bağlı dış immutable pakette doğrular. Default tracked tarihçe seçimi korunur; kendi SHA’sını taşıyan commit üretme döngüsünün o rotada çözüldüğü iddia edilmez. Kanıt commit’i E, S ile eşdeğer değildir.

Kanonik producer `scripts/rc-kampanyasi.py` yalnız macOS arm64 ve fixed nightly2026-08-31/cargo-fuzz0.13.2 profilidir. Gerçek macOS/kernel/CPU/RAM, araç sürümleri ve canonical producer hashini kaydeder. Exact kaynak ve original repo başlangıç/son clean olmalıdır. On hedef (27 Kasım 2026'dan beri; `psl` ve `dkim_anahtari` dahil) 1800sn explicitASAN, iki paralel hedef üstü, süreç başına2400sn dıştimeout; iki seçili Miri filtre nonempty/zeroignoredfailed ister. Windows/minOS veya tümcrate Miri kapsamı değildir.

Bu komutlar kısa test örneği değildir; final temiz S ve mevcut yetkiyle gerçek uzun kampanya dispatch’i içindir:

```sh
python3 -I -B scripts/rc-kampanyasi.py --repo /abs/repo --source-sha S --out /abs/fresh-campaign --execute-approved-stable-sha
python3 -I -B scripts/rc-kaniti.py export --framework-root /abs/repo --campaign /abs/campaign --output /abs/fresh-bundle
bash scripts/guvenlik-kapisi.sh --surum-adayi --rc-kanit /abs/bundle --rc-kanit-sha256 HASH
```

`S`40hex exact kaynak, `HASH` export’un döndürdüğü64hex manifesttir; çıktıları inceleyip gerçek değerleri kullanın. Geçersiz explicit paket trackedfallback yapmaz. Gate tüm eski deny/fuzzbuild/ledger/supply/clippy/test/guard kontrollerini korur; kaynak pinini ilk araçtan önce alır, evidence öncesi ve sonunda source/bundle kimliğini yeniden denetler.

Şema `zee-rc-evidence-1`; yalnız incelenmiş ownedlocal origin. Tam on ASAN hedefi+2Miri, pinnedcommands, actual>=requested1800..3600 ve gerçek rawstats/exit, hashli corpus/result/log ve initialcorpus exactS gitblob lineage gerekir. Aynı lexical campaignroot on hedefi, dictionary ve fuzz-target yollarını bağlar; bu recorded yollara erişilmez. NoFollow/regular files, shared200000entry/depth32,64MiBfile/256MiBtotal sınırı ve bounded byte snapshot kullanılır. Büyük paket budanmaz, reddedilir.

Hedef kuralı sürümlüdür: üretici `campaign-proof.json`'a `schema: zee-local-asan-miri-campaign-2`, `target_policy: zee-rc-fuzz-hedefleri-10` ve sıralı `target_set` yazar; doğrulayıcı üçünü exact ister ve sonucunda `campaign_schema`/`target_policy` alanlarını döndürür. `zee-local-asan-miri-campaign-1` paketi (en çok sekiz hedef) bu kuralla yeniden yargılanmaz; ayrı iletiyle reddedilir ve yalnız kendi exact kaynak commit'indeki `scripts/rc-kaniti.py` ile doğrulanır. Ayrıntı: [RC fuzz rehberi](fuzz-rc.md#tarihsel-kayıtlar-hangi-kurala-göre-doğrulanır).

Hashler bütünlük sağlar; keyfi selfhashli log authenticated execution değildir. Owned producer veya köken incelemesi gereklidir. Şema1 CI receipt kabul etmez. Var olan workflow rawRC artefaktı üretir, bu yerel paketin yerine otomatik konmaz.

Başarılı gerçek gate sonrası etiket ve iki-klon/SBOM/provenance kaynağı aynı S’dir. Append-only tarihçede sonraki E, satırlarda S’yi korur; E’ye S kabulü taşınmaz. Kullanıcı AGENTS aynı toplu teslimde tarihçe ister; talimat değiştirilmez, sourceS/campaignreceipt/archiveE sınırı açıkça kaydedilir.

[Mevcut kısa testler](../scripts/tests/rc-kaniti-test.py)25 sentetik identity/gate/filebound negatifini ve [koşucu testleri](../scripts/tests/rc-kampanyasi-test.py)9 kısa istatistik/ownedprocess vakasını yürütür. Bunlar gerçek1800sn kampanya kanıtı değildir. Yeni finalclean/ASAN/Miri/900sn host/1800sn soak ve ürün adoption kapıları bu kısa testlerden kapanmaz.
