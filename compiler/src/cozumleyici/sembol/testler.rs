use super::*;

#[test]
fn sira_checkpoint_nested_kapsam_ve_atamalari_korur() {
    let mut tablo = SembolTablosu::yeni(11);
    let dis_id = tablo.insert("dış".into(), Tur::TamSayi);
    let dis = kapsam_baslat(&tablo);
    let yerel_id = tablo.insert("yerel".into(), Tur::TamSayi);
    let ic = kapsam_baslat(&tablo);
    tablo.insert("iç".into(), Tur::TamSayi);
    assert_eq!(tablo.insert("dış".into(), Tur::Metin), dis_id);
    assert_eq!(tablo.insert("yerel".into(), Tur::Metin), yerel_id);
    kapsam_bitir(&mut tablo, &ic);
    assert_eq!(tablo.get("iç"), None);
    assert_eq!(tablo.get("yerel"), Some(&Tur::Metin));
    kapsam_bitir(&mut tablo, &dis);
    assert_eq!(tablo.get("yerel"), None);
    assert_eq!(tablo.get("dış"), Some(&Tur::Metin));
    assert_eq!(tablo.kimlik("dış"), Some(dis_id));
    let yeni_id = tablo.insert("yerel".into(), Tur::TamSayi);
    assert_ne!(yeni_id, yerel_id, "kayıp SymbolId yeniden kullanılmaz");
    assert_eq!(tablo.yeni_adlar.len(), tablo.kayitlar.len());
    let mut eski = tablo.kayitlar.clone();
    let mut sira = tablo.siradaki;
    let mut sinirlar = Vec::new();
    let mut seed = 0x207_u64;
    for _ in 0..4096 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        match seed % 4 {
            0 if sinirlar.len() < 8 => {
                sinirlar.push((
                    kapsam_baslat(&tablo),
                    eski.keys()
                        .cloned()
                        .collect::<std::collections::HashSet<_>>(),
                ));
            }
            1 => {
                if let Some((sinir, adlar)) = sinirlar.pop() {
                    kapsam_bitir(&mut tablo, &sinir);
                    eski.retain(|ad, _| adlar.contains(ad));
                }
            }
            _ => {
                let ad = format!("ad{}", (seed >> 8) % 24);
                let tur = if seed & 1 == 0 {
                    Tur::TamSayi
                } else {
                    Tur::Metin
                };
                let kimlik = if let Some(kayit) = eski.get_mut(&ad) {
                    kayit.tur = tur;
                    kayit.kimlik
                } else {
                    let kimlik = SymbolId::yeni(11, sira);
                    sira += 1;
                    eski.insert(ad.clone(), SembolKaydi { kimlik, tur });
                    kimlik
                };
                assert_eq!(tablo.insert(ad, tur), kimlik);
            }
        }
        assert_eq!(tablo.kayitlar.len(), eski.len());
        assert_eq!(tablo.siradaki, sira);
        assert_eq!(tablo.yeni_adlar.len(), tablo.kayitlar.len());
        for (ad, kayit) in &eski {
            assert_eq!(tablo.get(ad), Some(&kayit.tur));
            assert_eq!(tablo.kimlik(ad), Some(kayit.kimlik));
        }
    }
}
