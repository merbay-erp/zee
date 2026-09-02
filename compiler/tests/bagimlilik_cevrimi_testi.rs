//! K-150 production bağımlılık graph'ında açıklamasız SCC bırakmama kapısı.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const GRAPH_SEMASI: &str = "# zee-katman-mimarisi-1";
const IZIN_SEMASI: &str = "# zee-izinli-katman-cevrimleri-1";

#[derive(Debug)]
struct GeciciIzin {
    uyeler: BTreeSet<String>,
    son_tarih: u32,
    kaldirma_isi: String,
    gerekce: String,
}

fn fixture_yolu(ad: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(ad)
}

fn graph_oku() -> BTreeMap<String, BTreeSet<String>> {
    let metin = fs::read_to_string(fixture_yolu("katman-mimarisi-v1.tsv"))
        .expect("katman graph fixture'ı okunmalı");
    assert_eq!(metin.lines().next(), Some(GRAPH_SEMASI));
    assert!(metin.ends_with('\n') && !metin.contains('\r'));
    let mut graph = BTreeMap::new();
    for satir in metin
        .lines()
        .filter(|satir| !satir.is_empty() && !satir.starts_with('#'))
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [modul, _, bagimliliklar, _] = alanlar.as_slice() else {
            panic!("katman graph satırı dört alan taşımalı: {satir}");
        };
        let hedefler = if *bagimliliklar == "-" {
            BTreeSet::new()
        } else {
            bagimliliklar.split(',').map(str::to_string).collect()
        };
        assert!(graph.insert((*modul).to_string(), hedefler).is_none());
    }
    for (modul, hedefler) in &graph {
        for hedef in hedefler {
            assert!(
                graph.contains_key(hedef),
                "{modul}: bilinmeyen hedef {hedef}"
            );
        }
    }
    graph
}

fn tarih_sayisi(metin: &str) -> u32 {
    let parcalar = metin.split('-').collect::<Vec<_>>();
    let [yil, ay, gun] = parcalar.as_slice() else {
        panic!("tarih YYYY-AA-GG olmalı: {metin}");
    };
    assert_eq!(yil.len(), 4, "geçersiz yıl: {metin}");
    assert_eq!(ay.len(), 2, "geçersiz ay: {metin}");
    assert_eq!(gun.len(), 2, "geçersiz gün: {metin}");
    let yil = yil.parse::<u32>().expect("yıl sayı olmalı");
    let ay = ay.parse::<u32>().expect("ay sayı olmalı");
    let gun = gun.parse::<u32>().expect("gün sayı olmalı");
    assert!((1..=12).contains(&ay), "geçersiz ay: {metin}");
    let artik = yil.is_multiple_of(4) && (!yil.is_multiple_of(100) || yil.is_multiple_of(400));
    let azami_gun = match ay {
        2 if artik => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    assert!((1..=azami_gun).contains(&gun), "geçersiz gün: {metin}");
    yil * 10_000 + ay * 100 + gun
}

fn bugun_utc() -> u32 {
    let saniye = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("sistem saati epoch sonrasında olmalı")
        .as_secs();
    let gun = i64::try_from(saniye / 86_400).expect("gün i64 içine sığmalı");
    let (yil, ay, gun) = dil::zaman::gunlerden_tarih_utc(gun);
    u32::try_from(yil).expect("CI yılı pozitif olmalı") * 10_000 + ay * 100 + gun
}

fn izinleri_oku() -> BTreeMap<String, GeciciIzin> {
    let metin = fs::read_to_string(fixture_yolu("izinli-katman-cevrimleri-v1.tsv"))
        .expect("çevrim izin fixture'ı okunmalı");
    assert_eq!(metin.lines().next(), Some(IZIN_SEMASI));
    assert!(metin.ends_with('\n') && !metin.contains('\r'));
    let mut izinler = BTreeMap::new();
    let mut uye_kumeleri = BTreeSet::new();
    for satir in metin
        .lines()
        .filter(|satir| !satir.is_empty() && !satir.starts_with('#'))
    {
        let alanlar = satir.split('\t').collect::<Vec<_>>();
        let [kimlik, uyeler, son_tarih, kaldirma_isi, gerekce] = alanlar.as_slice() else {
            panic!("çevrim izin satırı beş alan taşımalı: {satir}");
        };
        assert!(
            kimlik.len() > 1
                && kimlik.starts_with('C')
                && kimlik[1..].bytes().all(|bayt| bayt.is_ascii_digit()),
            "geçersiz çevrim kimliği: {kimlik}"
        );
        let uye_listesi = uyeler.split(',').collect::<Vec<_>>();
        let mut sirali = uye_listesi.clone();
        sirali.sort_unstable();
        sirali.dedup();
        assert_eq!(uye_listesi, sirali, "{kimlik}: üyeler kanonik değil");
        assert!(
            uye_listesi.len() > 1,
            "{kimlik}: SCC en az iki üyeli olmalı"
        );
        assert!(
            kaldirma_isi.len() > 2
                && kaldirma_isi.starts_with("K-")
                && kaldirma_isi[2..].bytes().all(|bayt| bayt.is_ascii_digit()),
            "{kimlik}: kaldırma işi K-nnn olmalı"
        );
        assert!(gerekce.chars().count() >= 40, "{kimlik}: gerekçe yetersiz");
        let uye_kumesi = uye_listesi
            .iter()
            .map(|uye| (*uye).to_string())
            .collect::<BTreeSet<_>>();
        assert!(
            uye_kumeleri.insert(uye_kumesi.clone()),
            "{kimlik}: aynı SCC için ikinci izin olamaz"
        );
        assert!(
            izinler
                .insert(
                    (*kimlik).to_string(),
                    GeciciIzin {
                        uyeler: uye_kumesi,
                        son_tarih: tarih_sayisi(son_tarih),
                        kaldirma_isi: (*kaldirma_isi).to_string(),
                        gerekce: (*gerekce).to_string(),
                    },
                )
                .is_none(),
            "yinelenen çevrim kimliği: {kimlik}"
        );
    }
    izinler
}

fn ileri_ziyaret(
    dugum: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    gorulen: &mut BTreeSet<String>,
    sira: &mut Vec<String>,
) {
    if !gorulen.insert(dugum.to_string()) {
        return;
    }
    for hedef in &graph[dugum] {
        ileri_ziyaret(hedef, graph, gorulen, sira);
    }
    sira.push(dugum.to_string());
}

fn geri_ziyaret(
    dugum: &str,
    ters: &BTreeMap<String, BTreeSet<String>>,
    gorulen: &mut BTreeSet<String>,
    bilesen: &mut BTreeSet<String>,
) {
    if !gorulen.insert(dugum.to_string()) {
        return;
    }
    bilesen.insert(dugum.to_string());
    for hedef in &ters[dugum] {
        geri_ziyaret(hedef, ters, gorulen, bilesen);
    }
}

fn cevrimler(graph: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<BTreeSet<String>> {
    let mut sira = Vec::new();
    let mut gorulen = BTreeSet::new();
    for dugum in graph.keys() {
        ileri_ziyaret(dugum, graph, &mut gorulen, &mut sira);
    }

    let mut ters = graph
        .keys()
        .map(|dugum| (dugum.clone(), BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (kaynak, hedefler) in graph {
        for hedef in hedefler {
            ters.get_mut(hedef)
                .expect("hedef graph'ta olmalı")
                .insert(kaynak.clone());
        }
    }

    gorulen.clear();
    let mut sonuc = BTreeSet::new();
    for dugum in sira.into_iter().rev() {
        if gorulen.contains(&dugum) {
            continue;
        }
        let mut bilesen = BTreeSet::new();
        geri_ziyaret(&dugum, &ters, &mut gorulen, &mut bilesen);
        let oz_cevrim = bilesen.len() == 1 && graph[&dugum].contains(&dugum);
        if bilesen.len() > 1 || oz_cevrim {
            sonuc.insert(bilesen);
        }
    }
    sonuc
}

#[test]
fn production_graphinda_aciklamasiz_ve_suresiz_cevrim_yoktur() {
    let bulunan = cevrimler(&graph_oku());
    let izinler = izinleri_oku();
    let beklenen = izinler
        .values()
        .map(|izin| izin.uyeler.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        bulunan, beklenen,
        "yeni çevrim veya artık gereksiz allowlist kaydı mimari inceleme ister"
    );
    let bugun = bugun_utc();
    for (kimlik, izin) in izinler {
        assert!(
            bugun <= izin.son_tarih,
            "{kimlik} çevrim izni {} tarihinde doldu; {} ile kaldır: {}",
            izin.son_tarih,
            izin.kaldirma_isi,
            izin.gerekce
        );
    }
}

#[test]
fn scc_hesabi_yeni_cevrimi_ve_oz_cevrimi_yakalar() {
    let graph = BTreeMap::from([
        ("a".to_string(), BTreeSet::from(["b".to_string()])),
        ("b".to_string(), BTreeSet::from(["a".to_string()])),
        ("c".to_string(), BTreeSet::from(["c".to_string()])),
        ("d".to_string(), BTreeSet::new()),
    ]);
    assert_eq!(
        cevrimler(&graph),
        BTreeSet::from([
            BTreeSet::from(["a".to_string(), "b".to_string()]),
            BTreeSet::from(["c".to_string()]),
        ])
    );
}
