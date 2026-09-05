//! K-172/ADR-067 dogfood korpusu: yalnız gerçek ürün sürtünmesinden doğan
//! başarı ve başarısızlık vakaları; her `dogfood/**/*.dil` kaynağı manifestte
//! tam bir kez bulunur ve ürünün yetkinlik politikasıyla koşar.

use dil::yetkinlik::YetkinlikPolitikasi;
use dil::yorumlayici::ToplayanIo;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const MANIFEST: &str = include_str!("../../dogfood/korpus-v1.tsv");
const SEMA: &str = "# zee-dogfood-korpusu-1";
const ASGARI_NOT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Vaka {
    vaka: String,
    urun: String,
    kaynak_is: String,
    kip: String,
    beklenti: String,
    tani: String,
    cikti: Vec<String>,
    dosya: String,
    not: String,
}

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn oku(goreli: &str) -> String {
    std::fs::read_to_string(depo().join(goreli))
        .unwrap_or_else(|hata| panic!("{goreli} okunmalı: {hata}"))
}

fn kaynak_isi_gecerli(kayit: &str) -> Option<&str> {
    let (k, f) = match kayit.split_once('/') {
        Some((k, f)) => (k, Some(f)),
        None => (kayit, None),
    };
    let sayi = k.strip_prefix("K-")?;
    let rakamlar = sayi.bytes().take_while(u8::is_ascii_digit).count();
    if rakamlar < 3 || !sayi[rakamlar..].chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    if let Some(f) = f {
        let dilim_gecerli = |parca: &str| {
            parca
                .strip_prefix('F')
                .is_some_and(|sayi| sayi.len() == 3 && sayi.bytes().all(|x| x.is_ascii_digit()))
        };
        if !f.split('-').all(dilim_gecerli) {
            return None;
        }
    }
    Some(k)
}

fn vakalari_oku() -> Vec<Vaka> {
    let mut satirlar = MANIFEST.lines();
    assert_eq!(satirlar.next(), Some(SEMA), "dogfood korpus şeması");
    let mut vakalar: Vec<Vaka> = Vec::new();
    let mut gorulen = BTreeSet::new();
    for satir in satirlar.filter(|s| !s.is_empty() && !s.starts_with('#')) {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [vaka, urun, kaynak_is, kip, beklenti, tani, cikti, dosya, not] = alanlar.as_slice()
        else {
            panic!("dogfood korpus satırı dokuz alan taşımalı: {satir:?}");
        };
        assert!(
            !vaka.is_empty()
                && vaka
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{vaka}: vaka kimliği küçük ASCII kebab-case olmalı"
        );
        assert!(
            gorulen.insert((*vaka).to_string()),
            "{vaka}: yinelenen vaka"
        );
        assert!(
            kaynak_isi_gecerli(kaynak_is).is_some(),
            "{vaka}: kaynak işi K-NNN ya da K-NNN/FNNN olmalı: {kaynak_is}"
        );
        assert!(
            matches!(*kip, "denetle" | "calistir"),
            "{vaka}: kip denetle|calistir olmalı"
        );
        assert!(
            matches!(*beklenti, "basarili" | "basarisiz"),
            "{vaka}: beklenti basarili|basarisiz olmalı"
        );
        if *beklenti == "basarili" {
            assert_eq!(*tani, "-", "{vaka}: başarılı vaka tanı taşımaz");
        } else {
            assert!(
                tani.len() == 4
                    && matches!(tani.chars().next(), Some('S' | 'A' | 'T' | 'C' | 'D' | 'P')),
                "{vaka}: başarısız vaka dört karakterli tanı kodu ister"
            );
            assert_eq!(*cikti, "-", "{vaka}: başarısız vaka çıktı taşımaz");
        }
        if *kip == "denetle" {
            assert_eq!(*cikti, "-", "{vaka}: denetle kipinde çıktı yoktur");
        }
        assert!(
            dosya.starts_with("dogfood/") && dosya.ends_with(".dil"),
            "{vaka}: dosya dogfood/ altında .dil olmalı"
        );
        assert!(
            not.chars().count() >= ASGARI_NOT,
            "{vaka}: not en az {ASGARI_NOT} karakter olmalı"
        );
        let cikti = if *cikti == "-" {
            Vec::new()
        } else {
            cikti.split('|').map(str::to_string).collect()
        };
        vakalar.push(Vaka {
            vaka: (*vaka).to_string(),
            urun: (*urun).to_string(),
            kaynak_is: (*kaynak_is).to_string(),
            kip: (*kip).to_string(),
            beklenti: (*beklenti).to_string(),
            tani: (*tani).to_string(),
            cikti,
            dosya: (*dosya).to_string(),
            not: (*not).to_string(),
        });
    }
    assert!(vakalar.len() >= 10, "ilk taban en az on vaka taşır");
    vakalar
}

fn aktif_urunler() -> BTreeMap<String, String> {
    let kayit = oku("docs/dogfood-projeleri-v1.tsv");
    assert_eq!(kayit.lines().next(), Some("# zee-dogfood-projeleri-1"));
    kayit
        .lines()
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .filter_map(|s| {
            let alanlar = s.split('\t').collect::<Vec<_>>();
            (alanlar.len() == 4 && alanlar[3] == "active")
                .then(|| (alanlar[0].to_string(), alanlar[1].to_string()))
        })
        .collect()
}

fn dil_dosyalarini_topla(klasor: &Path, sonuc: &mut BTreeSet<String>) {
    let mut girdiler = std::fs::read_dir(klasor)
        .expect("dogfood klasörü okunmalı")
        .map(|g| g.expect("girdi").path())
        .collect::<Vec<_>>();
    girdiler.sort();
    for yol in girdiler {
        if yol.is_dir() {
            dil_dosyalarini_topla(&yol, sonuc);
        } else if yol.extension().and_then(|u| u.to_str()) == Some("dil")
            && yol.file_name().and_then(|a| a.to_str()) != Some("proje.dil")
        {
            let goreli = yol.strip_prefix(depo()).expect("depo içi yol");
            sonuc.insert(goreli.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn urun_politikasi(kok: &str) -> YetkinlikPolitikasi {
    let bildirim = dil::proje::bildirimi_oku(&oku(&format!("{kok}/proje.dil")))
        .unwrap_or_else(|hata| panic!("{kok}/proje.dil geçerli olmalı: {hata:?}"));
    bildirim.yetkinlik_politikasi()
}

#[test]
fn dogfood_korpusu_manifesti_tam_tekil_ve_urune_baglidir() {
    let vakalar = vakalari_oku();
    let urunler = aktif_urunler();
    let gunluk = oku("kararlar/gunluk.md");
    let mut agac = BTreeSet::new();
    dil_dosyalarini_topla(&depo().join("dogfood"), &mut agac);
    let manifestte = vakalar
        .iter()
        .map(|v| v.dosya.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        agac, manifestte,
        "dogfood altındaki her .dil kaynağı manifestte tam bir kez bulunmalı (proje.dil hariç)"
    );
    for vaka in &vakalar {
        let kok = urunler
            .get(&vaka.urun)
            .unwrap_or_else(|| panic!("{}: ürün {} etkin kayıtta değil", vaka.vaka, vaka.urun));
        assert!(
            vaka.dosya.starts_with(&format!("{kok}/")),
            "{}: dosya etkin ürün kökü {kok} altında değil",
            vaka.vaka
        );
        let k = kaynak_isi_gecerli(&vaka.kaynak_is).expect("biçim doğrulandı");
        assert!(
            gunluk.contains(k),
            "{}: kaynak işi {k} günlükte yok",
            vaka.vaka
        );
        if let Some((_, f)) = vaka.kaynak_is.split_once('/') {
            let ilk = f.split('-').next().expect("F parçası");
            assert!(
                gunluk.contains(ilk),
                "{}: dogfood dilimi {ilk} günlükte yok",
                vaka.vaka
            );
        }
        let kaynak = oku(&vaka.dosya);
        assert_eq!(
            dil::api::v1::bicimle(&kaynak).expect("biçimlenmeli"),
            kaynak,
            "{}: dogfood kaynağı kanonik biçimde değil",
            vaka.vaka
        );
        assert!(
            kaynak.starts_with("# "),
            "{}: kaynak sürtünmeyi anlatan başlık yorumu taşımalı",
            vaka.vaka
        );
    }
}

#[test]
fn dogfood_korpusu_basari_ve_basarisizlik_beklentilerini_korur() {
    let vakalar = vakalari_oku();
    let urunler = aktif_urunler();
    for vaka in &vakalar {
        let kaynak = oku(&vaka.dosya);
        let politika = urun_politikasi(&urunler[&vaka.urun]);
        let derleme = dil::kaynagi_derle(&kaynak).and_then(|program| {
            dil::cozumleyici::yetkinlikleri_denetle(&program, &politika).map(|()| program)
        });
        match (vaka.kip.as_str(), vaka.beklenti.as_str()) {
            ("denetle", "basarili") => {
                assert!(
                    derleme.is_ok(),
                    "{}: derlenmeli: {:?}",
                    vaka.vaka,
                    derleme.err()
                );
            }
            ("denetle", "basarisiz") => {
                let hata = derleme
                    .err()
                    .unwrap_or_else(|| panic!("{}: {} beklenirdi", vaka.vaka, vaka.tani));
                assert_eq!(hata.kod, vaka.tani, "{}: {}", vaka.vaka, hata.mesaj);
            }
            ("calistir", beklenti) => {
                let program = derleme.unwrap_or_else(|hata| {
                    panic!("{}: çalıştırma vakası derlenmeli: {hata:?}", vaka.vaka)
                });
                let mut io = ToplayanIo::yeni(Vec::new());
                let sonuc = dil::yorumlayici::calistir_io(&program, &mut io);
                if beklenti == "basarili" {
                    assert!(sonuc.is_ok(), "{}: çalışmalı: {:?}", vaka.vaka, sonuc.err());
                    assert_eq!(io.cikti, vaka.cikti, "{}: çıktı", vaka.vaka);
                } else {
                    let hata = sonuc.expect_err("başarısız vaka tanı üretmeli");
                    assert_eq!(hata.kod, vaka.tani, "{}: {}", vaka.vaka, hata.mesaj);
                }
            }
            _ => unreachable!("kip/beklenti doğrulandı"),
        }
    }
}

#[test]
fn kaynak_isi_bicimi_fail_closed_dogrulanir() {
    assert_eq!(kaynak_isi_gecerli("K-163/F030"), Some("K-163"));
    assert_eq!(kaynak_isi_gecerli("K-163/F014-F032"), Some("K-163"));
    assert_eq!(kaynak_isi_gecerli("K-160A"), Some("K-160A"));
    for bozuk in ["K163", "K-16", "K-163/F30", "K-163/G030", "F030", "K-163/"] {
        assert!(kaynak_isi_gecerli(bozuk).is_none(), "{bozuk} reddedilmeli");
    }
}
