//! K-147/K-155: Düzeltilmiş semantic bug'lar kaynak ve sürüm provenance'lıdır.

use dil::faz::KaynakMetni;
use dil::tani::Tani;
use dil::yetkinlik::YetkinlikPolitikasi;
use dil::yorumlayici::{calistir_baglanmis_io_kodla, ToplayanIo};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SEMA: &str = "# zee-semantic-regresyon-2";
const FAZLAR: &[&str] = &[
    "parser",
    "checker",
    "hir",
    "runtime",
    "morphology",
    "concurrency",
    "security",
];
const KIPLER: &[&str] = &[
    "parser",
    "diagnostics",
    "compile",
    "hir",
    "run",
    "capability",
    "web",
];

#[derive(Debug)]
struct Vaka {
    ad: String,
    bug: String,
    fixed_by: String,
    introduced_by: Option<String>,
    guaranteed_since: String,
    faz: String,
    kip: String,
    tani: Option<String>,
    span: Option<(usize, usize, usize)>,
    cikis: i64,
    cikti: Vec<String>,
    dosya: String,
}

#[derive(Debug)]
struct Gozlem {
    tani: Option<Tani>,
    cikis: i64,
    cikti: Vec<String>,
}

fn depo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler depo içinde olmalı")
        .to_path_buf()
}

fn span_coz(metin: &str, satir: usize) -> Option<(usize, usize, usize)> {
    if metin == "-" {
        return None;
    }
    let sayilar = metin
        .split(':')
        .map(|parca| {
            parca
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("manifest satırı {satir}: bozuk span {metin:?}"))
        })
        .collect::<Vec<_>>();
    assert_eq!(sayilar.len(), 3, "manifest satırı {satir}: span üçlüdür");
    assert!(
        sayilar.iter().all(|deger| *deger > 0),
        "manifest satırı {satir}: span sıfır olamaz"
    );
    Some((sayilar[0], sayilar[1], sayilar[2]))
}

fn vakalari_oku() -> Vec<Vaka> {
    let yol = depo().join("regression/v2.tsv");
    let metin = std::fs::read_to_string(&yol).expect("regresyon manifesti okunmalı");
    assert!(metin.ends_with('\n'), "manifest LF ile bitmeli");
    assert!(!metin.contains('\r'), "manifest CR taşımamalı");
    assert_eq!(metin.lines().next(), Some(SEMA), "bilinmeyen şema");

    metin
        .lines()
        .enumerate()
        .filter(|(_, satir)| !satir.is_empty() && !satir.starts_with('#'))
        .map(|(indeks, satir)| {
            let sira = indeks + 1;
            let alanlar = satir.split('\t').collect::<Vec<_>>();
            assert_eq!(
                alanlar.len(),
                12,
                "manifest satırı {sira}: on iki alan gerekir"
            );
            let [ad, bug, fixed_by, introduced_by, guaranteed_since, faz, kip, tani, span, cikis, cikti, dosya] =
                alanlar.as_slice()
            else {
                unreachable!("alan sayısı doğrulandı")
            };
            Vaka {
                ad: (*ad).to_string(),
                bug: (*bug).to_string(),
                fixed_by: (*fixed_by).to_string(),
                introduced_by: (*introduced_by != "-").then(|| (*introduced_by).to_string()),
                guaranteed_since: (*guaranteed_since).to_string(),
                faz: (*faz).to_string(),
                kip: (*kip).to_string(),
                tani: (*tani != "-").then(|| (*tani).to_string()),
                span: span_coz(span, sira),
                cikis: cikis
                    .parse()
                    .unwrap_or_else(|_| panic!("manifest satırı {sira}: exit sayı olmalı")),
                cikti: if *cikti == "-" {
                    Vec::new()
                } else {
                    cikti.split("\\n").map(str::to_string).collect()
                },
                dosya: (*dosya).to_string(),
            }
        })
        .collect()
}

fn tam_git_sha(deger: &str) -> bool {
    deger.len() == 40
        && deger
            .bytes()
            .all(|bayt| bayt.is_ascii_hexdigit() && !bayt.is_ascii_uppercase())
}

fn git_commit_var(revizyon: &str) -> bool {
    std::process::Command::new("git")
        .args(["cat-file", "-e", &format!("{revizyon}^{{commit}}")])
        .current_dir(depo())
        .status()
        .is_ok_and(|durum| durum.success())
}

fn git_atasidir(ata: &str, torun: &str) -> bool {
    std::process::Command::new("git")
        .args(["merge-base", "--is-ancestor", ata, torun])
        .current_dir(depo())
        .status()
        .is_ok_and(|durum| durum.success())
}

fn dil_dosyalarini_topla(klasor: &Path, sonuc: &mut BTreeSet<String>) {
    let mut girdiler = std::fs::read_dir(klasor)
        .expect("regression klasörü okunmalı")
        .collect::<Result<Vec<_>, _>>()
        .expect("regression girdileri okunmalı");
    girdiler.sort_by_key(|girdi| girdi.file_name());
    for girdi in girdiler {
        let yol = girdi.path();
        if girdi.file_type().expect("dosya türü").is_dir() {
            dil_dosyalarini_topla(&yol, sonuc);
        } else if yol.extension().and_then(|uzanti| uzanti.to_str()) == Some("dil") {
            sonuc.insert(
                yol.strip_prefix(depo())
                    .expect("fixture depo içinde")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

fn hata_gozle(tani: Tani, cikti: Vec<String>) -> Gozlem {
    Gozlem {
        tani: Some(tani),
        cikis: 1,
        cikti,
    }
}

fn faz_kipi_kabul_eder(faz: &str, kip: &str) -> bool {
    match faz {
        "parser" => matches!(kip, "parser" | "diagnostics" | "run"),
        "checker" | "morphology" => matches!(kip, "compile" | "run" | "diagnostics"),
        "hir" => matches!(kip, "hir" | "run"),
        "runtime" | "concurrency" => kip == "run",
        "security" => matches!(kip, "capability" | "web"),
        _ => false,
    }
}

fn vakayi_calistir(vaka: &Vaka, kaynak: &str) -> Gozlem {
    if vaka.kip == "parser" {
        return match KaynakMetni::yeni(kaynak)
            .sozcukle()
            .and_then(|tokenlar| tokenlar.ayristir(Vec::new()).map(|_| ()))
        {
            Ok(()) => Gozlem {
                tani: None,
                cikis: 0,
                cikti: Vec::new(),
            },
            Err(tani) => hata_gozle(tani, Vec::new()),
        };
    }

    if vaka.kip == "diagnostics" {
        let mut yukleyici = |_: &str| Err("regresyon korpusu birim yüklemez".to_string());
        let mut tanilar = dil::kaynagi_tanilari(kaynak, &mut yukleyici);
        assert_eq!(
            tanilar.len(),
            1,
            "{}: diagnostics kipi tam bir tanı bekler: {tanilar:?}",
            vaka.ad
        );
        return hata_gozle(tanilar.remove(0), Vec::new());
    }

    let program = match dil::kaynagi_fazli_derle(kaynak) {
        Ok(program) => program,
        Err(tani) => return hata_gozle(tani, Vec::new()),
    };
    if let Err(hata) = program.invariantleri_dogrula() {
        panic!("{}: bağlı HIR invarianti bozuk: {hata}", vaka.ad);
    }

    match vaka.kip.as_str() {
        "compile" | "hir" => Gozlem {
            tani: None,
            cikis: 0,
            cikti: Vec::new(),
        },
        "capability" => {
            match dil::cozumleyici::yetkinlikleri_denetle(&program, &YetkinlikPolitikasi::kapali())
            {
                Ok(()) => Gozlem {
                    tani: None,
                    cikis: 0,
                    cikti: Vec::new(),
                },
                Err(tani) => hata_gozle(tani, Vec::new()),
            }
        }
        "run" => {
            let mut io = ToplayanIo::yeni(Vec::new());
            match calistir_baglanmis_io_kodla(&program, &mut io) {
                Ok(cikis) => Gozlem {
                    tani: None,
                    cikis,
                    cikti: io.cikti,
                },
                Err(tani) => hata_gozle(tani, io.cikti),
            }
        }
        "web" => {
            let istek = kaynak
                .lines()
                .next()
                .and_then(|satir| satir.strip_prefix("# web-istegi: "))
                .unwrap_or_else(|| panic!("{}: web fixture istek başlığı taşımıyor", vaka.ad));
            let mut io = ToplayanIo::yeni(Vec::new());
            io.istekler.push_back(istek.to_string());
            match calistir_baglanmis_io_kodla(&program, &mut io) {
                Ok(cikis) => Gozlem {
                    tani: None,
                    cikis,
                    cikti: io
                        .sunucu_durumlari
                        .iter()
                        .zip(&io.sunucu_yanitlari)
                        .map(|(durum, (_, yanit))| format!("{durum} {yanit}"))
                        .collect(),
                },
                Err(tani) => hata_gozle(tani, io.cikti),
            }
        }
        _ => panic!("{}: bilinmeyen kip {}", vaka.ad, vaka.kip),
    }
}

#[test]
fn semantic_regresyon_manifesti_tam_tekil_ve_minimaldir() {
    let vakalar = vakalari_oku();
    assert!(!vakalar.is_empty(), "korpus boş olamaz");
    let karar_gunlugu =
        std::fs::read_to_string(depo().join("kararlar/gunluk.md")).expect("karar günlüğü okunmalı");
    let mut adlar = BTreeSet::new();
    let mut dosyalar = BTreeSet::new();
    let mut gorulen_fazlar = BTreeSet::new();

    for vaka in &vakalar {
        assert!(adlar.insert(&vaka.ad), "yinelenen vaka: {}", vaka.ad);
        assert!(
            dosyalar.insert(vaka.dosya.clone()),
            "fixture iki kez sahiplenildi: {}",
            vaka.dosya
        );
        assert!(
            vaka.ad
                .chars()
                .all(|k| k.is_ascii_lowercase() || k.is_ascii_digit() || k == '-'),
            "{}: vaka kimliği küçük ASCII kebab-case olmalı",
            vaka.ad
        );
        assert!(
            vaka.bug.len() == 5
                && vaka.bug.starts_with("K-")
                && vaka.bug[2..].chars().all(|k| k.is_ascii_digit()),
            "{}: bug kimliği K-NNN olmalı",
            vaka.ad
        );
        assert!(
            karar_gunlugu.contains(&format!("## {} ", vaka.bug)),
            "{}: {} karar günlüğünde kayıtlı değil",
            vaka.ad,
            vaka.bug
        );
        assert!(
            tam_git_sha(&vaka.fixed_by),
            "{}: fixed_by tam SHA olmalı",
            vaka.ad
        );
        assert!(
            git_commit_var(&vaka.fixed_by),
            "{}: fixed_by commit değil",
            vaka.ad
        );
        assert!(
            git_atasidir(&vaka.fixed_by, "HEAD"),
            "{}: fixed_by HEAD atası değil",
            vaka.ad
        );
        if let Some(introduced_by) = &vaka.introduced_by {
            assert!(
                tam_git_sha(introduced_by),
                "{}: introduced_by tam SHA veya - olmalı",
                vaka.ad
            );
            assert!(
                git_commit_var(introduced_by),
                "{}: introduced_by commit değil",
                vaka.ad
            );
            assert!(
                git_atasidir(introduced_by, &vaka.fixed_by),
                "{}: introduced_by fixed_by atası değil",
                vaka.ad
            );
        }
        assert_eq!(
            vaka.guaranteed_since, "0.8.0-dev",
            "{}: garanti sürümü güncel geliştirme serisi olmalı",
            vaka.ad
        );
        assert!(
            FAZLAR.contains(&vaka.faz.as_str()),
            "{}: bilinmeyen faz",
            vaka.ad
        );
        assert!(
            KIPLER.contains(&vaka.kip.as_str()),
            "{}: bilinmeyen kip",
            vaka.ad
        );
        assert!(
            faz_kipi_kabul_eder(&vaka.faz, &vaka.kip),
            "{}: {} fazı {} kipini sahiplenemez",
            vaka.ad,
            vaka.faz,
            vaka.kip
        );
        gorulen_fazlar.insert(vaka.faz.as_str());
        assert_eq!(
            Path::new(&vaka.dosya).parent(),
            Some(Path::new("regression").join(&vaka.faz).as_path()),
            "{}: fixture birincil faz klasöründe olmalı",
            vaka.ad
        );
        assert_eq!(
            Path::new(&vaka.dosya)
                .file_stem()
                .and_then(|ad| ad.to_str()),
            Some(vaka.ad.as_str()),
            "{}: fixture adı vaka kimliğiyle aynı olmalı",
            vaka.ad
        );
        assert_eq!(
            Path::new(&vaka.dosya)
                .extension()
                .and_then(|uzanti| uzanti.to_str()),
            Some("dil"),
            "{}: fixture .dil olmalı",
            vaka.ad
        );
        assert_eq!(
            vaka.tani.is_some(),
            vaka.span.is_some(),
            "{}: tanı ve span birlikte bulunmalı",
            vaka.ad
        );
        if let Some(kod) = &vaka.tani {
            assert!(
                matches!(kod.as_bytes(), [b'S' | b'A' | b'T' | b'C' | b'D' | b'P', a, b, c]
                    if a.is_ascii_digit() && b.is_ascii_digit() && c.is_ascii_digit()),
                "{}: bozuk tanı kodu {}",
                vaka.ad,
                kod
            );
            assert_eq!(vaka.cikis, 1, "{}: hata sonucu exit 1 olmalı", vaka.ad);
        }
        assert!(
            (0..=255).contains(&vaka.cikis),
            "{}: exit süreç aralığında olmalı",
            vaka.ad
        );

        let yol = depo().join(&vaka.dosya);
        let kaynak = std::fs::read_to_string(&yol)
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", yol.display()));
        assert!(
            kaynak.ends_with('\n'),
            "{}: fixture LF ile bitmeli",
            vaka.ad
        );
        assert!(!kaynak.contains('\r'), "{}: fixture CR taşımamalı", vaka.ad);
        assert!(
            !kaynak.contains('\t'),
            "{}: fixture tab taşımamalı",
            vaka.ad
        );
        assert!(kaynak.len() <= 4_096, "{}: fixture 4 KiB'ı aşıyor", vaka.ad);
        assert!(
            kaynak
                .lines()
                .filter(|satir| !satir.trim().is_empty())
                .count()
                <= 32,
            "{}: fixture minimal 32 dolu satır sınırını aşıyor",
            vaka.ad
        );
    }

    assert_eq!(
        gorulen_fazlar,
        FAZLAR.iter().copied().collect(),
        "her semantic faz en az bir vaka taşımalı"
    );
    let mut agactaki_dosyalar = BTreeSet::new();
    dil_dosyalarini_topla(&depo().join("regression"), &mut agactaki_dosyalar);
    assert_eq!(
        dosyalar, agactaki_dosyalar,
        "manifest ile regression ağacı birebir olmalı"
    );
}

#[test]
fn duzeltilmis_semantic_buglar_tani_span_cikis_ve_ciktiyi_korur() {
    for vaka in vakalari_oku() {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya))
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", vaka.dosya));
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert_eq!(
            gozlem.tani.as_ref().map(|tani| tani.kod.as_str()),
            vaka.tani.as_deref(),
            "{} / {}: beklenmeyen tanı: {:?}",
            vaka.bug,
            vaka.ad,
            gozlem.tani
        );
        assert_eq!(
            gozlem
                .tani
                .as_ref()
                .map(|tani| (tani.satir, tani.sutun, tani.uzunluk)),
            vaka.span,
            "{} / {}: tanı spanı değişti: {:?}",
            vaka.bug,
            vaka.ad,
            gozlem.tani
        );
        assert_eq!(gozlem.cikis, vaka.cikis, "{} / {}: exit", vaka.bug, vaka.ad);
        assert_eq!(
            gozlem.cikti, vaka.cikti,
            "{} / {}: çıktı",
            vaka.bug, vaka.ad
        );
    }
}
