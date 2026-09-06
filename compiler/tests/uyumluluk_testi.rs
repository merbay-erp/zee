//! K-167/ADR-064 uyumluluk kapısı: dondurulmuş dil/araç yüzeyi envanteri ve
//! deprecation kayıtları kaynakla birebir kalır; kaldırma yalnız kayıtlı ve
//! sürümlü göç yoluyla olur.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const YUZEY_FIXTURE: &str = include_str!("fixtures/dil-yuzeyi-v1.tsv");
const DEPRECATION_KAYITLARI: &str = include_str!("../../docs/deprecation-kayitlari-v1.tsv");
const TANI_KIMLIKLERI: &str = include_str!("fixtures/tani-kimlikleri-v1.tsv");

const YUZEY_SEMASI: &str = "# zee-dil-yuzeyi-1";
const DEPRECATION_SEMASI: &str = "# zee-deprecation-kayitlari-1";
/// Kullanıcıya verilen uyumluluk sözünün yüzeyleri; kaldırma en az bir alt
/// sürüm serisi önce duyurulur.
const DESTEKLENEN_YUZEYLER: [&str; 8] = [
    "abi", "api", "bicim", "kalip", "komut", "kosul", "profil", "tani",
];
/// Kaynak kümesi doğrudan Rust kaynağından türetilen kelime yüzeyleri.
const KAYNAKTAN_TURETILEN: [&str; 3] = ["kalip", "komut", "kosul"];
const IC_YUZEY: &str = "ic";
const ASGARI_GOC_UZUNLUGU: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
struct YuzeyKaydi {
    yuzey: String,
    oge: String,
    durum: String,
    giris: Surum,
    kaynak: String,
    kayit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeprecationKaydi {
    kimlik: String,
    yuzey: String,
    oge: String,
    sinif: String,
    duyuru: Surum,
    kaldirma: Surum,
    goc: String,
    karar: String,
    durum: String,
}

/// `X.Y.Z` veya geliştirme serisi `X.Y.Z-dev`; aynı sayılarda `-dev` önce gelir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Surum {
    ana: u32,
    alt: u32,
    yama: u32,
    yayimlanmis: bool,
}

impl Surum {
    fn seri(self) -> (u32, u32) {
        (self.ana, self.alt)
    }
}

fn surum_coz(metin: &str) -> Result<Surum, String> {
    let (govde, yayimlanmis) = match metin.strip_suffix("-dev") {
        Some(govde) => (govde, false),
        None => (metin, true),
    };
    let parcalar = govde.split('.').collect::<Vec<_>>();
    if parcalar.len() != 3 {
        return Err(format!("sürüm X.Y.Z ya da X.Y.Z-dev olmalı: {metin:?}"));
    }
    let sayi = |parca: &str| {
        if parca.is_empty() || (parca.len() > 1 && parca.starts_with('0')) {
            return Err(format!("sürüm parçası kanonik onluk olmalı: {metin:?}"));
        }
        parca
            .parse::<u32>()
            .map_err(|_| format!("sürüm parçası sayı olmalı: {metin:?}"))
    };
    Ok(Surum {
        ana: sayi(parcalar[0])?,
        alt: sayi(parcalar[1])?,
        yama: sayi(parcalar[2])?,
        yayimlanmis,
    })
}

fn veri_satirlari(metin: &str, sema: &str) -> Result<Vec<Vec<String>>, String> {
    let mut satirlar = metin.lines();
    if satirlar.next() != Some(sema) {
        return Err(format!("şema başlığı {sema:?} olmalı"));
    }
    Ok(satirlar
        .filter(|satir| !satir.is_empty() && !satir.starts_with('#'))
        .map(|satir| satir.split('\t').map(str::to_string).collect())
        .collect())
}

fn yuzey_kayitlarini_coz(metin: &str) -> Result<Vec<YuzeyKaydi>, String> {
    let mut kayitlar = Vec::new();
    let mut gorulen = BTreeSet::new();
    for alanlar in veri_satirlari(metin, YUZEY_SEMASI)? {
        let [yuzey, oge, durum, giris, kaynak, kayit] = alanlar.as_slice() else {
            return Err(format!("yüzey satırı altı alan taşımalı: {alanlar:?}"));
        };
        if !DESTEKLENEN_YUZEYLER.contains(&yuzey.as_str()) {
            return Err(format!("bilinmeyen yüzey: {yuzey}"));
        }
        if !matches!(durum.as_str(), "aktif" | "kaldirildi") {
            return Err(format!("yüzey durumu aktif|kaldirildi olmalı: {oge}"));
        }
        if oge.is_empty() || kaynak.is_empty() || kayit.is_empty() {
            return Err(format!("yüzey satırında boş alan: {oge:?}"));
        }
        let turetilen = KAYNAKTAN_TURETILEN.contains(&yuzey.as_str());
        if turetilen != (kayit == "-") {
            return Err(format!(
                "kelime yüzeyleri kayıt taşımaz, diğerleri exact kayıt ister: {yuzey}/{oge}"
            ));
        }
        let anahtar = (yuzey.clone(), oge.clone());
        if let Some(onceki) = gorulen.iter().next_back() {
            if *onceki >= anahtar {
                return Err(format!(
                    "yüzey satırları (yüzey, öğe) sırasında ve tekil olmalı: {yuzey}/{oge}"
                ));
            }
        }
        gorulen.insert(anahtar);
        kayitlar.push(YuzeyKaydi {
            yuzey: yuzey.clone(),
            oge: oge.clone(),
            durum: durum.clone(),
            giris: surum_coz(giris)?,
            kaynak: kaynak.clone(),
            kayit: kayit.clone(),
        });
    }
    Ok(kayitlar)
}

fn deprecation_kayitlarini_coz(metin: &str) -> Result<Vec<DeprecationKaydi>, String> {
    let mut kayitlar: Vec<DeprecationKaydi> = Vec::new();
    for alanlar in veri_satirlari(metin, DEPRECATION_SEMASI)? {
        let [kimlik, yuzey, oge, sinif, duyuru, kaldirma, goc, karar, durum] = alanlar.as_slice()
        else {
            return Err(format!(
                "deprecation satırı dokuz alan taşımalı: {alanlar:?}"
            ));
        };
        let beklenen_kimlik = format!("DEP-{:03}", kayitlar.len() + 1);
        if *kimlik != beklenen_kimlik {
            return Err(format!(
                "deprecation kimliği ardışık olmalı: {kimlik} yerine {beklenen_kimlik}"
            ));
        }
        if !DESTEKLENEN_YUZEYLER.contains(&yuzey.as_str()) && yuzey != IC_YUZEY {
            return Err(format!("{kimlik}: bilinmeyen yüzey {yuzey}"));
        }
        if !matches!(
            sinif.as_str(),
            "kaldirma" | "davranis" | "yeniden-adlandirma"
        ) {
            return Err(format!(
                "{kimlik}: sınıf kaldirma|davranis|yeniden-adlandirma olmalı"
            ));
        }
        if !matches!(durum.as_str(), "duyuruldu" | "kaldirildi") {
            return Err(format!("{kimlik}: durum duyuruldu|kaldirildi olmalı"));
        }
        if durum == "kaldirildi" && sinif == "davranis" {
            return Err(format!(
                "{kimlik}: davranış değişikliği kaldırıldı durumu taşıyamaz"
            ));
        }
        if goc.chars().count() < ASGARI_GOC_UZUNLUGU {
            return Err(format!(
                "{kimlik}: göç yolu en az {ASGARI_GOC_UZUNLUGU} karakter olmalı"
            ));
        }
        if !(karar.starts_with("adr/") || karar.starts_with("rfcs/") || karar.starts_with("spec/"))
            || !karar.ends_with(".md")
        {
            return Err(format!(
                "{kimlik}: karar adr/, rfcs/ ya da spec/ Markdown yolu olmalı"
            ));
        }
        let duyuru = surum_coz(duyuru).map_err(|hata| format!("{kimlik}: {hata}"))?;
        let kaldirma = surum_coz(kaldirma).map_err(|hata| format!("{kimlik}: {hata}"))?;
        if kaldirma < duyuru {
            return Err(format!("{kimlik}: kaldırma sürümü duyurudan önce olamaz"));
        }
        let sure_sart = DESTEKLENEN_YUZEYLER.contains(&yuzey.as_str()) && yuzey != "tani";
        if sure_sart && kaldirma.seri() <= duyuru.seri() {
            return Err(format!(
                "{kimlik}: desteklenen yüzeyde kaldırma, duyurudan en az bir alt sürüm serisi sonra olmalı"
            ));
        }
        if kayitlar.iter().any(|k| k.yuzey == *yuzey && k.oge == *oge) {
            return Err(format!("{kimlik}: aynı yüzey/öğe için ikinci kayıt"));
        }
        kayitlar.push(DeprecationKaydi {
            kimlik: kimlik.clone(),
            yuzey: yuzey.clone(),
            oge: oge.clone(),
            sinif: sinif.clone(),
            duyuru,
            kaldirma,
            goc: goc.clone(),
            karar: karar.clone(),
            durum: durum.clone(),
        });
    }
    Ok(kayitlar)
}

fn ayrilmis_tani_kodlari(metin: &str) -> BTreeSet<String> {
    metin
        .lines()
        .filter(|satir| satir.starts_with("ayrilmis\t"))
        .filter_map(|satir| satir.split('\t').nth(1))
        .map(str::to_string)
        .collect()
}

/// Fixture ↔ deprecation ↔ tanı mezar taşı çapraz tutarlılığı.
fn capraz_denetle(
    yuzeyler: &[YuzeyKaydi],
    kayitlar: &[DeprecationKaydi],
    ayrilmis_tanilar: &BTreeSet<String>,
) -> Result<(), String> {
    let kaldirilan: BTreeMap<(String, String), &DeprecationKaydi> = kayitlar
        .iter()
        .filter(|kayit| kayit.durum == "kaldirildi")
        .map(|kayit| ((kayit.yuzey.clone(), kayit.oge.clone()), kayit))
        .collect();
    for yuzey in yuzeyler.iter().filter(|kayit| kayit.durum == "kaldirildi") {
        if !kaldirilan.contains_key(&(yuzey.yuzey.clone(), yuzey.oge.clone())) {
            return Err(format!(
                "{}/{} kaldırıldı ama deprecation kaydı yok",
                yuzey.yuzey, yuzey.oge
            ));
        }
    }
    for kayit in kaldirilan.values() {
        if KAYNAKTAN_TURETILEN.contains(&kayit.yuzey.as_str())
            && !yuzeyler
                .iter()
                .any(|y| y.yuzey == kayit.yuzey && y.oge == kayit.oge && y.durum == "kaldirildi")
        {
            return Err(format!(
                "{}: kaldırılan kelime yüzeyi fixture'da mezar taşı olarak kalmalı",
                kayit.kimlik
            ));
        }
        if kayit.yuzey == "tani" && !ayrilmis_tanilar.contains(&kayit.oge) {
            return Err(format!(
                "{}: kaldırılan tanı kodu kimlik fixture'ında ayrılmış değil",
                kayit.kimlik
            ));
        }
    }
    for kod in ayrilmis_tanilar {
        if !kaldirilan.contains_key(&("tani".to_string(), kod.clone())) {
            return Err(format!(
                "ayrılmış tanı kodu {kod} deprecation kaydı taşımıyor"
            ));
        }
    }
    Ok(())
}

fn depo_koku() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("depo kökü")
        .to_path_buf()
}

fn kaynak(goreli: &str) -> String {
    std::fs::read_to_string(depo_koku().join(goreli))
        .unwrap_or_else(|hata| panic!("{goreli} okunmalı: {hata}"))
}

fn tirnakli_metinler(bolge: &str) -> BTreeSet<String> {
    let mut sonuc = BTreeSet::new();
    let mut kalan = bolge;
    while let Some(bas) = kalan.find('"') {
        let govde = &kalan[bas + 1..];
        let Some(son) = govde.find('"') else { break };
        sonuc.insert(govde[..son].to_string());
        kalan = &govde[son + 1..];
    }
    sonuc
}

fn bolge<'a>(metin: &'a str, bas: &str, son: &str) -> &'a str {
    let baslangic = metin
        .find(bas)
        .unwrap_or_else(|| panic!("{bas:?} kaynakta bulunmalı"));
    let govde = &metin[baslangic..];
    let bitis = govde
        .find(son)
        .unwrap_or_else(|| panic!("{son:?} kaynakta bulunmalı"));
    &govde[..bitis]
}

/// LSP tamamlama listesi dilin kalıp kelime envanteridir (kaynağı: ayrıştırıcı).
fn kaynaktaki_kalip_kelimeleri() -> BTreeSet<String> {
    tirnakli_metinler(bolge(
        &kaynak("compiler/src/lsp.rs"),
        "const KALIP_KELIMELERI",
        "\n];",
    ))
}

fn kaynaktaki_kosul_kelimeleri() -> BTreeSet<String> {
    tirnakli_metinler(bolge(
        &kaynak("compiler/src/ayristirici/ifade.rs"),
        "fn kosul_kelimesi",
        "\n}",
    ))
}

fn kaynaktaki_komutlar() -> BTreeSet<String> {
    let ana = kaynak("compiler/src/main.rs");
    let dagitim = bolge(&ana, "\nfn main", "\nfn kullanim");
    let mut sonuc = BTreeSet::new();
    let mut kalan = dagitim;
    while let Some(bas) = kalan.find("Some(") {
        let govde = &kalan[bas + 5..];
        let Some(son) = govde.find(')') else { break };
        sonuc.extend(tirnakli_metinler(&govde[..son]));
        kalan = &govde[son + 1..];
    }
    sonuc
}

fn yuzey_kumesi(yuzeyler: &[YuzeyKaydi], yuzey: &str) -> BTreeSet<String> {
    yuzeyler
        .iter()
        .filter(|kayit| kayit.yuzey == yuzey && kayit.durum == "aktif")
        .map(|kayit| kayit.oge.clone())
        .collect()
}

#[test]
fn dondurulmus_yuzey_envanteri_kaynakla_birebir() {
    let yuzeyler = yuzey_kayitlarini_coz(YUZEY_FIXTURE).expect("yüzey fixture'ı geçerli olmalı");
    for (yuzey, kaynaktaki) in [
        ("kalip", kaynaktaki_kalip_kelimeleri()),
        ("kosul", kaynaktaki_kosul_kelimeleri()),
        ("komut", kaynaktaki_komutlar()),
    ] {
        let fixture = yuzey_kumesi(&yuzeyler, yuzey);
        let eksik = kaynaktaki.difference(&fixture).cloned().collect::<Vec<_>>();
        let fazla = fixture.difference(&kaynaktaki).cloned().collect::<Vec<_>>();
        assert!(
            eksik.is_empty() && fazla.is_empty(),
            "{yuzey} yüzeyi fixture ile ayrışıyor — kaynakta olup kayıtsız: {eksik:?}; \
             kayıtlı olup kaynakta olmayan (kaldırma deprecation kaydı ister): {fazla:?}"
        );
    }
    for kayit in yuzeyler
        .iter()
        .filter(|kayit| !KAYNAKTAN_TURETILEN.contains(&kayit.yuzey.as_str()))
    {
        let metin = kaynak(&kayit.kaynak);
        assert!(
            metin.contains(&kayit.kayit),
            "{}/{} exact kaydı {} içinde yok: {:?}",
            kayit.yuzey,
            kayit.oge,
            kayit.kaynak,
            kayit.kayit
        );
    }
}

#[test]
fn deprecation_kayitlari_gecerli_ve_kaynakla_tutarli() {
    let yuzeyler = yuzey_kayitlarini_coz(YUZEY_FIXTURE).expect("yüzey fixture'ı");
    let kayitlar = deprecation_kayitlarini_coz(DEPRECATION_KAYITLARI)
        .expect("deprecation kayıtları geçerli olmalı");
    capraz_denetle(
        &yuzeyler,
        &kayitlar,
        &ayrilmis_tani_kodlari(TANI_KIMLIKLERI),
    )
    .expect("fixture, deprecation ve tanı mezar taşları çapraz tutarlı olmalı");
    let kaynak_kumeleri: BTreeMap<&str, BTreeSet<String>> = BTreeMap::from([
        ("kalip", kaynaktaki_kalip_kelimeleri()),
        ("kosul", kaynaktaki_kosul_kelimeleri()),
        ("komut", kaynaktaki_komutlar()),
    ]);
    let lib = kaynak("compiler/src/lib.rs");
    for kayit in &kayitlar {
        assert!(
            depo_koku().join(&kayit.karar).is_file(),
            "{}: karar belgesi yok: {}",
            kayit.kimlik,
            kayit.karar
        );
        if kayit.durum != "kaldirildi" {
            continue;
        }
        if let Some(kume) = kaynak_kumeleri.get(kayit.yuzey.as_str()) {
            assert!(
                !kume.contains(&kayit.oge),
                "{}: kaldırıldı denen {} hâlâ kaynakta",
                kayit.kimlik,
                kayit.oge
            );
        }
        if kayit.yuzey == IC_YUZEY {
            let modul = kayit
                .oge
                .strip_prefix("dil::")
                .unwrap_or_else(|| panic!("{}: iç yüzey dil::<modül> olmalı", kayit.kimlik));
            assert!(
                !lib.contains(&format!("pub mod {modul};")),
                "{}: kaldırılan iç modül {} hâlâ lib.rs'te",
                kayit.kimlik,
                modul
            );
            assert!(
                !depo_koku()
                    .join(format!("compiler/src/{modul}.rs"))
                    .exists(),
                "{}: kaldırılan iç modül dosyası duruyor",
                kayit.kimlik
            );
        }
    }
}

#[test]
fn surum_kimligi_gelistirme_serisini_yayimlanmis_sanmaz() {
    let cargo = kaynak("compiler/Cargo.toml");
    let satir = cargo
        .lines()
        .find(|satir| satir.starts_with("version = "))
        .expect("Cargo sürümü");
    let surum = surum_coz(satir.trim_start_matches("version = ").trim_matches('"'))
        .expect("Cargo sürümü kanonik olmalı");
    let son_etiket = std::process::Command::new("git")
        .args([
            "-C",
            depo_koku().to_str().expect("UTF-8 yol"),
            "tag",
            "--list",
            "v*",
        ])
        .output()
        .expect("git tag");
    let en_yuksek = String::from_utf8_lossy(&son_etiket.stdout)
        .lines()
        .filter_map(|etiket| surum_coz(etiket.trim_start_matches('v')).ok())
        .max();
    if let Some(etiket) = en_yuksek {
        assert!(
            surum > etiket,
            "etiketlenmiş {etiket:?} sonrası çalışma ağacı yayımlanmış kimlik taşıyamaz; \
             Cargo sürümü bir sonraki -dev serisine çekilmeli (ADR-064)"
        );
    }
    assert!(
        surum_coz("0.8.0-dev").expect("dev") < surum_coz("0.8.0").expect("release"),
        "geliştirme serisi aynı sayılı yayından önce sıralanmalı"
    );
}

#[test]
fn kapi_bozuk_fixture_ve_sureli_olmayan_kaldirmayi_reddeder() {
    let sirasiz = format!(
        "{YUZEY_SEMASI}\nkalip\tyaz\taktif\t0.7.0\tcompiler/src/lsp.rs\t-\nkalip\tal\taktif\t0.7.0\tcompiler/src/lsp.rs\t-\n"
    );
    assert!(yuzey_kayitlarini_coz(&sirasiz)
        .unwrap_err()
        .contains("sırasında ve tekil"));
    let kayitli_kelime =
        format!("{YUZEY_SEMASI}\nkalip\tyaz\taktif\t0.7.0\tcompiler/src/lsp.rs\tyaz\n");
    assert!(yuzey_kayitlarini_coz(&kayitli_kelime)
        .unwrap_err()
        .contains("kelime yüzeyleri kayıt taşımaz"));

    let satir = |kimlik: &str, yuzey: &str, duyuru: &str, kaldirma: &str, durum: &str| {
        format!(
            "{kimlik}\t{yuzey}\tornek\tkaldirma\t{duyuru}\t{kaldirma}\tGöç yolu yeterince uzun bir açıklamadır.\tadr/064-uyumluluk-kapisi.md\t{durum}"
        )
    };
    let geriye = format!(
        "{DEPRECATION_SEMASI}\n{}\n",
        satir("DEP-001", "kalip", "0.9.0", "0.8.0", "duyuruldu")
    );
    assert!(deprecation_kayitlarini_coz(&geriye)
        .unwrap_err()
        .contains("duyurudan önce olamaz"));
    let ayni_seri = format!(
        "{DEPRECATION_SEMASI}\n{}\n",
        satir("DEP-001", "komut", "0.8.0-dev", "0.8.0", "duyuruldu")
    );
    assert!(deprecation_kayitlarini_coz(&ayni_seri)
        .unwrap_err()
        .contains("en az bir alt sürüm serisi sonra"));
    let ic_ayni_seri = format!(
        "{DEPRECATION_SEMASI}\n{}\n",
        satir("DEP-001", "ic", "0.8.0-dev", "0.8.0-dev", "kaldirildi")
    );
    assert!(deprecation_kayitlarini_coz(&ic_ayni_seri).is_ok());
    let atlayan_kimlik = format!(
        "{DEPRECATION_SEMASI}\n{}\n",
        satir("DEP-002", "ic", "0.8.0-dev", "0.8.0-dev", "kaldirildi")
    );
    assert!(deprecation_kayitlarini_coz(&atlayan_kimlik)
        .unwrap_err()
        .contains("ardışık olmalı"));
    let kisa_goc = format!(
        "{DEPRECATION_SEMASI}\nDEP-001\tic\tdil::eski\tkaldirma\t0.8.0-dev\t0.8.0-dev\tkısa\tadr/064-uyumluluk-kapisi.md\tkaldirildi\n"
    );
    assert!(deprecation_kayitlarini_coz(&kisa_goc)
        .unwrap_err()
        .contains("göç yolu"));
}

#[test]
fn kapi_kayitsiz_kaldirma_ve_kayitsiz_mezar_tasini_reddeder() {
    let yuzeyler = yuzey_kayitlarini_coz(&format!(
        "{YUZEY_SEMASI}\nkalip\teski\tkaldirildi\t0.7.0\tcompiler/src/lsp.rs\t-\n"
    ))
    .expect("geçerli fixture");
    let hata = capraz_denetle(&yuzeyler, &[], &BTreeSet::new()).unwrap_err();
    assert!(hata.contains("deprecation kaydı yok"), "{hata}");

    let kayitlar = deprecation_kayitlarini_coz(&format!(
        "{DEPRECATION_SEMASI}\nDEP-001\tkalip\teski\tkaldirma\t0.7.0\t0.8.0\tYeni biçim `yeni` kelimesiyle yazılır; göç otomatiktir.\tadr/064-uyumluluk-kapisi.md\tkaldirildi\n"
    ))
    .expect("geçerli kayıt");
    let hata = capraz_denetle(&[], &kayitlar, &BTreeSet::new()).unwrap_err();
    assert!(hata.contains("mezar taşı olarak kalmalı"), "{hata}");
    assert!(capraz_denetle(&yuzeyler, &kayitlar, &BTreeSet::new()).is_ok());

    let hata =
        capraz_denetle(&yuzeyler, &kayitlar, &BTreeSet::from(["S999".to_string()])).unwrap_err();
    assert!(hata.contains("S999 deprecation kaydı taşımıyor"), "{hata}");
}

// --- K-174: V1 syntax freeze penceresi ---

const SYNTAX_FREEZE: &str = include_str!("../../docs/v1-syntax-freeze-v1.tsv");

/// Freeze kaydı: alan → değer.
fn syntax_freeze_kaydi() -> BTreeMap<String, String> {
    let mut satirlar = SYNTAX_FREEZE.lines();
    assert_eq!(satirlar.next(), Some("# zee-v1-syntax-freeze-1"));
    satirlar
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .map(|s| {
            let (alan, deger) = s.split_once('\t').expect("alan\tdeger");
            (alan.to_string(), deger.to_string())
        })
        .collect()
}

#[test]
fn v1_syntax_freeze_penceresinde_kalip_kosul_komut_yuzeyi_degismez() {
    // K-174: 6 Eylül – 4 Ekim 2026 arasında yeni syntax açılmaz. Kalıp, koşul
    // ve komut satırlarının özeti kayıtla birebir olmalıdır; bilinçli bir
    // değişiklik kaydı, tarihi ve K-işini aynı committe günceller.
    use sha2::{Digest, Sha256};
    let kayit = syntax_freeze_kaydi();
    assert_eq!(kayit.get("karar").map(String::as_str), Some("K-174"));
    for alan in ["baslangic", "bitis"] {
        let tarih = &kayit[alan];
        assert!(
            tarih.len() == 10 && tarih.as_bytes()[4] == b'-' && tarih.as_bytes()[7] == b'-',
            "{alan} YYYY-AA-GG olmalı: {tarih}"
        );
    }
    assert!(kayit["baslangic"] < kayit["bitis"], "pencere ileri akmalı");
    let yuzeyler: Vec<&str> = kayit["dondurulmus_yuzeyler"].split(',').collect();
    assert_eq!(yuzeyler, vec!["kalip", "kosul", "komut"]);
    let fixture = std::fs::read_to_string(depo_koku().join(&kayit["yuzey_fixture"]))
        .expect("yüzey fixture'ı okunmalı");
    let mut ozet = Sha256::new();
    for satir in fixture.lines() {
        if yuzeyler
            .iter()
            .any(|y| satir.starts_with(&format!("{y}\t")))
        {
            ozet.update(satir.as_bytes());
            ozet.update(b"\n");
        }
    }
    let bulunan = ozet
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(
        bulunan, kayit["dondurulmus_sha256"],
        "V1 syntax freeze penceresinde ({}–{}) kalıp/koşul/komut yüzeyi değişti; K-174 kaydı bilinçli olarak güncellenmeden geçmez",
        kayit["baslangic"], kayit["bitis"]
    );
}
