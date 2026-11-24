//! K-147/K-155: Düzeltilmiş semantic bug'lar kaynak ve sürüm provenance'lıdır.

use dil::faz::KaynakMetni;
use dil::tani::Tani;
use dil::yetkinlik::YetkinlikPolitikasi;
use dil::yorumlayici::{
    calistir_baglanmis_io_kodla, IzKaydedenIo, IzYenidenOynatici, SahteBaglanti, ToplayanIo,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[path = "semantic_regresyon/smtp.rs"]
mod smtp_fixture;

#[path = "semantic_regresyon/lsp_uri.rs"]
mod lsp_uri_fixture;

#[path = "semantic_regresyon/lsp_utf16.rs"]
mod lsp_utf16_fixture;

#[path = "semantic_regresyon/native_ayri_port_birincil.rs"]
mod native_ayri_port_birincil_fixture;
#[path = "semantic_regresyon/native_coken_degistirme.rs"]
mod native_coken_degistirme_fixture;
#[path = "semantic_regresyon/native_dosya_exdev.rs"]
mod native_dosya_exdev_fixture;
#[path = "semantic_regresyon/native_dosya_okuma_penceresi.rs"]
mod native_dosya_okuma_penceresi_fixture;
#[path = "semantic_regresyon/native_eszamanli_ekleme.rs"]
mod native_eszamanli_ekleme_fixture;
#[path = "semantic_regresyon/native_eszamanli_paket_ekle.rs"]
mod native_eszamanli_paket_ekle_fixture;
#[path = "semantic_regresyon/native_eszamanli_tasima.rs"]
mod native_eszamanli_tasima_fixture;
#[path = "semantic_regresyon/native_isci_grubu.rs"]
mod native_isci_grubu_fixture;
#[path = "semantic_regresyon/native_isci_kabul_term.rs"]
mod native_isci_kabul_term_fixture;
#[path = "semantic_regresyon/native_isci_saglik_mirasi.rs"]
mod native_isci_saglik_mirasi_fixture;
#[path = "semantic_regresyon/native_listener_deadline.rs"]
mod native_listener_deadline_fixture;
#[path = "semantic_regresyon/native_metric_stdio.rs"]
mod native_metric_stdio_fixture;
#[path = "semantic_regresyon/native_proje_dosya_koku.rs"]
mod native_proje_dosya_koku_fixture;
#[path = "semantic_regresyon/native_registry_durum_cekismesi.rs"]
mod native_registry_durum_cekismesi_fixture;
#[path = "semantic_regresyon/native_unicode_kimlik.rs"]
mod native_unicode_kimlik_fixture;
#[path = "semantic_regresyon/native_web_depo_cekismesi.rs"]
mod native_web_depo_cekismesi_fixture;
#[path = "semantic_regresyon/native_web_kopma.rs"]
mod native_web_kopma_fixture;
#[path = "semantic_regresyon/native_web_oturum_deposu.rs"]
mod native_web_oturum_deposu_fixture;
#[path = "semantic_regresyon/native_web_yukleme.rs"]
mod native_web_yukleme_fixture;
#[path = "semantic_regresyon/native_yedek_artigi.rs"]
mod native_yedek_artigi_fixture;

const SEMA: &str = "# zee-semantic-regresyon-2";
const FAZLAR: &[&str] = &[
    "parser",
    "checker",
    "hir",
    "runtime",
    "morphology",
    "concurrency",
    "security",
    "cli",
    "lsp",
];
const KIPLER: &[&str] = &[
    "parser",
    "diagnostics",
    "compile",
    "hir",
    "definition",
    "run",
    "trace",
    "capability",
    "web",
    "smtp",
    "postgresql",
    "baglanti",
    "cli",
    "lsp-uri",
    "lsp-utf16",
    "native-listener-deadline",
    "native-metric-stdio",
    "native-web-kopma",
    "native-web-yazim-butcesi",
    "native-web-erken-red",
    "native-web-csp",
    "native-dosya-exdev",
    "native-web-yukleme",
    "native-dosya-okuma-penceresi",
    "native-dosya-okuma-izsiz-penceresi",
    "native-isci-grubu",
    "native-ayri-port-birincil",
    "native-eszamanli-ekleme",
    "native-isci-saglik-mirasi",
    "native-coken-degistirme",
    "native-yedek-artigi",
    "native-web-depo-cekismesi",
    "native-web-oturum-deposu",
    "native-isci-kabul-term",
    "native-eszamanli-tasima",
    "native-eszamanli-paket-ekle",
    "native-registry-durum-cekismesi",
    "native-unicode-kimlik",
    "native-proje-dosya-koku",
];

#[derive(Debug, Clone)]
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
        "hir" => matches!(kip, "hir" | "definition" | "run"),
        "runtime" => matches!(
            kip,
            "run"
                | "trace"
                | "postgresql"
                | "baglanti"
                | "native-dosya-exdev"
                | "native-dosya-okuma-penceresi"
                | "native-dosya-okuma-izsiz-penceresi"
                | "native-eszamanli-ekleme"
                | "native-eszamanli-tasima"
                | "native-coken-degistirme"
                | "native-yedek-artigi"
                | "native-proje-dosya-koku"
        ),
        "concurrency" => kip == "run",
        "security" => matches!(
            kip,
            "capability"
                | "web"
                | "smtp"
                | "native-listener-deadline"
                | "native-web-kopma"
                | "native-web-yazim-butcesi"
                | "native-web-erken-red"
                | "native-web-csp"
                | "native-web-yukleme"
                | "native-web-depo-cekismesi"
                | "native-web-oturum-deposu"
                | "native-registry-durum-cekismesi"
                | "native-unicode-kimlik"
        ),
        "cli" => matches!(
            kip,
            "cli"
                | "native-metric-stdio"
                | "native-isci-grubu"
                | "native-ayri-port-birincil"
                | "native-isci-saglik-mirasi"
                | "native-isci-kabul-term"
                | "native-eszamanli-paket-ekle"
        ),
        "lsp" => matches!(kip, "lsp-uri" | "lsp-utf16"),
        _ => false,
    }
}

fn native_deadline_manifest_sozlesmesi(vaka: &Vaka) -> bool {
    vaka.faz == "security"
        && vaka.cikis == 0
        && vaka.tani.is_none()
        && vaka.span.is_none()
        && vaka.cikti.is_empty()
}

fn cli_gozle(kaynak: &str) -> Gozlem {
    let argumanlar = kaynak
        .lines()
        .find_map(|satir| satir.strip_prefix("# cli-argumanlari: "))
        .expect("CLI fixture argüman başlığını taşımalı");
    let cikti = std::process::Command::new(env!("CARGO_BIN_EXE_dil"))
        .args(argumanlar.split_whitespace())
        .current_dir(depo())
        .output()
        .expect("CLI regresyon süreci çalışmalı");
    let akim = kaynak
        .lines()
        .find_map(|satir| satir.strip_prefix("# cli-cikti: "))
        .unwrap_or("stderr");
    let gozlenen = match akim {
        "stderr" => {
            assert!(cikti.stdout.is_empty(), "ret stdout üretmemeli");
            cikti.stderr
        }
        "stdout" => cikti.stdout,
        _ => panic!("CLI fixture çıktı akımı stdout veya stderr olmalı"),
    };
    Gozlem {
        tani: None,
        cikis: i64::from(cikti.status.code().expect("normal CLI exit")),
        cikti: String::from_utf8(gozlenen)
            .expect("CLI seçili çıktı UTF-8")
            .lines()
            // Sürüm kimliği seriyle değişir; beklenen çıktı `{surum}` yer tutucusu taşır.
            .map(|satir| satir.replace(env!("CARGO_PKG_VERSION"), "{surum}"))
            .collect(),
    }
}

fn vakayi_calistir(vaka: &Vaka, kaynak: &str) -> Gozlem {
    if vaka.kip == "lsp-utf16" {
        let (cikis, cikti) = lsp_utf16_fixture::calistir(kaynak);
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "lsp-uri" {
        let birim = depo()
            .join(&vaka.dosya)
            .parent()
            .unwrap()
            .join("yardimci.birim");
        let (cikis, cikti) = lsp_uri_fixture::calistir(kaynak, &birim);
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "cli" {
        return cli_gozle(kaynak);
    }
    if vaka.kip == "native-metric-stdio" {
        let (cikis, cikti) =
            native_metric_stdio_fixture::calistir(&depo(), &depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-kopma" {
        let (cikis, cikti) = native_web_kopma_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-yazim-butcesi" {
        let (cikis, cikti) = native_web_kopma_fixture::butce_calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-erken-red" {
        let (cikis, cikti) =
            native_web_kopma_fixture::erken_red_calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-csp" {
        let (cikis, cikti) = native_web_kopma_fixture::csp_calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-yukleme" {
        let (cikis, cikti) = native_web_yukleme_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-isci-kabul-term" {
        let (cikis, cikti) = native_isci_kabul_term_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-isci-saglik-mirasi" {
        let (cikis, cikti) = native_isci_saglik_mirasi_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-ayri-port-birincil" {
        let (cikis, cikti) = native_ayri_port_birincil_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-isci-grubu" {
        let (cikis, cikti) = native_isci_grubu_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-dosya-exdev" {
        let (cikis, cikti) = native_dosya_exdev_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-coken-degistirme" {
        let (cikis, cikti) = native_coken_degistirme_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-depo-cekismesi" {
        let (cikis, cikti) = native_web_depo_cekismesi_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-web-oturum-deposu" {
        let (cikis, cikti) = native_web_oturum_deposu_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-registry-durum-cekismesi" {
        let (cikis, cikti) =
            native_registry_durum_cekismesi_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-unicode-kimlik" {
        let (cikis, cikti) = native_unicode_kimlik_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-proje-dosya-koku" {
        let (cikis, cikti) = native_proje_dosya_koku_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-yedek-artigi" {
        let (cikis, cikti) = native_yedek_artigi_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-eszamanli-tasima" {
        let (cikis, cikti) = native_eszamanli_tasima_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-eszamanli-paket-ekle" {
        let (cikis, cikti) =
            native_eszamanli_paket_ekle_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-eszamanli-ekleme" {
        let (cikis, cikti) = native_eszamanli_ekleme_fixture::calistir(&depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-dosya-okuma-penceresi"
        || vaka.kip == "native-dosya-okuma-izsiz-penceresi"
    {
        let kaynak = depo().join(&vaka.dosya);
        let (cikis, cikti) = match vaka.kip.as_str() {
            "native-dosya-okuma-penceresi" => {
                native_dosya_okuma_penceresi_fixture::calistir(&kaynak)
            }
            _ => native_dosya_okuma_penceresi_fixture::calistir_izsiz(&kaynak),
        };
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
    if vaka.kip == "native-listener-deadline" {
        let (cikis, cikti) =
            native_listener_deadline_fixture::calistir(&depo(), &depo().join(&vaka.dosya));
        return Gozlem {
            tani: None,
            cikis,
            cikti,
        };
    }
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

    // HIR çok-kaynaklı vakalarının aynı dizindeki hermetik birim girdileri.
    // Yerel dosya/ürün/ağ bağımlılığı yok; bütün girdiler korpusta sürümlüdür.
    let mut yukleyici = |ad: &str| {
        if let Some(kaynak) = dil::gomulu_birim(ad) {
            return Ok(kaynak.to_string());
        }
        let dizin = depo()
            .join(&vaka.dosya)
            .parent()
            .expect("vaka dizini")
            .to_path_buf();
        std::fs::read_to_string(dizin.join(format!("{ad}.birim"))).map_err(|hata| hata.to_string())
    };
    let program = match dil::kaynagi_fazli_derle_birimlerle(kaynak, &mut yukleyici) {
        Ok(program) => program,
        Err(tani) => return hata_gozle(tani, Vec::new()),
    };
    if let Err(hata) = program.invariantleri_dogrula() {
        panic!("{}: bağlı HIR invarianti bozuk: {hata}", vaka.ad);
    }

    if vaka.kip == "definition" {
        for kullanim in program.hir().sembol_kullanimlari() {
            assert!(
                program.hir().sembol_tanimi(kullanim.kimlik()).is_some(),
                "{}: user SymbolId definition kaydı yok",
                vaka.ad
            );
        }
    }
    match vaka.kip.as_str() {
        "trace" => {
            let mut kaydeden = IzKaydedenIo::yeni(ToplayanIo::yeni(Vec::new()));
            let kod =
                calistir_baglanmis_io_kodla(&program, &mut kaydeden).expect("trace kayıt koşusu");
            let iz = kaydeden.iz_metni().expect("kanonik kayıt şeması");
            let mut oynatici = IzYenidenOynatici::yeni(&iz).expect("trace parse");
            let tekrar =
                calistir_baglanmis_io_kodla(&program, &mut oynatici).expect("trace replay");
            assert_eq!(kod, tekrar, "trace exit eşitliği");
            assert_eq!(kaydeden.ic.cikti, oynatici.cikti, "trace stdout eşitliği");
            oynatici.bitir().expect("trace tam tüketim");
            Gozlem {
                tani: None,
                cikis: kod,
                cikti: kaydeden.ic.cikti,
            }
        }
        "compile" | "hir" | "definition" => Gozlem {
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
        "postgresql" => {
            // K-375: hermetik PostgreSQL her sorguda başlıktaki sanal süreyi
            // harcar; adaptöre verilen bütçe ve geçen sanal süre çıktıya eklenir.
            let sure: i64 = kaynak
                .lines()
                .find_map(|satir| satir.strip_prefix("# postgresql-sorgu-suresi-ms: "))
                .and_then(|deger| deger.parse().ok())
                .unwrap_or_else(|| panic!("{}: postgresql sorgu süresi başlığı yok", vaka.ad));
            let mut io = ToplayanIo::yeni(Vec::new());
            io.postgresql.sorgu_suresi_ms = Some(sure);
            let sonuc = calistir_baglanmis_io_kodla(&program, &mut io);
            let butceler = io
                .postgresql
                .sureler
                .iter()
                .map(|butce| butce.map_or("-".to_string(), |ms| ms.to_string()))
                .collect::<Vec<_>>()
                .join(",");
            let gecen = dil::yorumlayici::GirdiCikti::an_ms(&mut io);
            let mut cikti = io.cikti;
            cikti.push(format!("[postgresql] bütçe={butceler} geçen={gecen}ms"));
            match sonuc {
                Ok(cikis) => Gozlem {
                    tani: None,
                    cikis,
                    cikti,
                },
                Err(tani) => hata_gozle(tani, cikti),
            }
        }
        "baglanti" => {
            // K-388: hermetik bağlantı adaptörü; başlıklar dinleyiciyi, gelen
            // satırları ve istemcinin susup susmadığını verir. Çıktı giden
            // satırlar ve `[günlük] <neden>` satırlarıdır.
            let mut dinleyici = None;
            let mut satirlar = Vec::new();
            let mut sessiz = false;
            for satir in kaynak.lines() {
                if let Some(ad) = satir.strip_prefix("# bağlantı-dinleyici: ") {
                    dinleyici = Some(ad);
                } else if let Some(gelen) = satir.strip_prefix("# bağlantı-satırı: ") {
                    satirlar.push(gelen);
                } else if satir == "# bağlantı-sessiz" {
                    sessiz = true;
                }
            }
            let dinleyici =
                dinleyici.unwrap_or_else(|| panic!("{}: bağlantı-dinleyici başlığı yok", vaka.ad));
            let mut baglanti = SahteBaglanti::satirlarla(dinleyici, "192.0.2.10", &satirlar);
            baglanti.sessiz_kalir = sessiz;
            let mut io = ToplayanIo::yeni(Vec::new());
            io.baglanti.bekleyenler.push_back(baglanti);
            let sonuc = calistir_baglanmis_io_kodla(&program, &mut io);
            let mut cikti: Vec<String> = io
                .baglanti
                .baglantilar
                .iter()
                .flat_map(|b| b.giden.iter().cloned())
                .collect();
            cikti.extend(
                io.baglanti
                    .gunluk
                    .iter()
                    .map(|(_, n)| format!("[günlük] {n}")),
            );
            match sonuc {
                Ok(cikis) => Gozlem {
                    tani: None,
                    cikis,
                    cikti,
                },
                Err(tani) => hata_gozle(tani, cikti),
            }
        }
        "smtp" => {
            let (sonuc, cikti) = smtp_fixture::calistir(&program, kaynak);
            match sonuc {
                Ok(cikis) => Gozlem {
                    tani: None,
                    cikis,
                    cikti,
                },
                Err(tani) => hata_gozle(tani, cikti),
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
        if vaka.kip == "native-metric-stdio" {
            assert!(
                native_metric_manifest_sozlesmesi(vaka),
                "native metric manifest CLI exit2/empty/no diagnostic contract"
            );
        }
        if vaka.kip == "native-listener-deadline" {
            assert!(
                native_deadline_manifest_sozlesmesi(vaka),
                "native deadline manifest exit0/empty HTTP/no diagnostic contract"
            );
        }
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
    for vaka in vakalari_oku().into_iter().filter(|vaka| {
        !matches!(
            vaka.kip.as_str(),
            "native-listener-deadline"
                | "native-metric-stdio"
                | "native-web-kopma"
                | "native-web-yazim-butcesi"
                | "native-web-erken-red"
                | "native-web-csp"
                | "native-dosya-exdev"
                | "native-web-yukleme"
                | "native-dosya-okuma-penceresi"
                | "native-dosya-okuma-izsiz-penceresi"
                | "native-isci-grubu"
                | "native-ayri-port-birincil"
                | "native-eszamanli-ekleme"
                | "native-isci-saglik-mirasi"
                | "native-coken-degistirme"
                | "native-yedek-artigi"
                | "native-proje-dosya-koku"
                | "native-web-depo-cekismesi"
                | "native-web-oturum-deposu"
                | "native-isci-kabul-term"
                | "native-eszamanli-tasima"
                | "native-eszamanli-paket-ekle"
                | "native-registry-durum-cekismesi"
                | "native-unicode-kimlik"
        )
    }) {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya))
            .unwrap_or_else(|hata| panic!("{} okunamadı: {hata}", vaka.dosya));
        // K-178: vaka resmî CLI yığınında koşar; debug çerçevesi 2 MiB test iş
        // parçacığına sığmaz, derinlik sözü ancak resmî yığında sınanabilir.
        let gozlem = {
            let vaka_kopyasi = vaka.clone();
            let kaynak_kopyasi = kaynak.clone();
            std::thread::Builder::new()
                .stack_size(
                    dil::kaynak_sinirlari::VARSAYILAN_KAYNAK_SINIRLARI.calistirma_yigin_bayti(),
                )
                .spawn(move || vakayi_calistir(&vaka_kopyasi, &kaynak_kopyasi))
                .expect("vaka iş parçacığı açılamadı")
                .join()
                .expect("vaka iş parçacığı düştü")
        };
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

#[test]
fn smtp_yavas_yanit_fixture_native_butce_ve_maskeyi_korur() {
    let kaynak = include_str!("../../regression/security/smtp-yavas-yanit.dil");
    let mut yukleyici = |ad: &str| {
        dil::gomulu_birim(ad)
            .map(str::to_string)
            .ok_or_else(|| "fixture birim yüklemez".to_string())
    };
    let program = dil::kaynagi_fazli_derle_birimlerle(kaynak, &mut yukleyici).unwrap();
    let (sonuc, cikti) = smtp_fixture::calistir(&program, kaynak);
    assert_eq!(sonuc.unwrap(), 0);
    assert_eq!(cikti, vec!["E-posta gönderilemedi; ayar geçersiz veya gönderim sonucu belirsiz; ayrıntılar gizlendi; otomatik tekrar yok"]);
}

/// K-371: düz metin `ileti göndermeyi dene` dış `içinde` son tarihinin
/// kalanını gerçek loopback SMTP bütçesine taşır; eski sabit 20 sn yoktur.
#[test]
fn smtp_son_tarih_fixture_dis_kalan_butceyi_kullanir() {
    let kaynak = "# smtp-vaka: son-tarih\nyarım saniye içinde\n    sonuç \"ali@ornek.invalid\" \"Konu\" \"Gövde\" ile ileti göndermeyi dene olsun\n    \"ulaşılmaz\" yaz\nyetişmezse\n    \"yetişmedi\" yaz\n";
    let mut yukleyici = |ad: &str| {
        dil::gomulu_birim(ad)
            .map(str::to_string)
            .ok_or_else(|| "fixture birim yüklemez".to_string())
    };
    let program = dil::kaynagi_fazli_derle_birimlerle(kaynak, &mut yukleyici).unwrap();
    let baslangic = std::time::Instant::now();
    let (sonuc, cikti) = smtp_fixture::calistir(&program, kaynak);
    assert_eq!(sonuc.unwrap(), 0);
    assert_eq!(cikti, vec!["yetişmedi"]);
    assert!(baslangic.elapsed() < std::time::Duration::from_secs(4));
}

#[test]
fn cli_stdout_alt_turu_gercek_basariyi_stderr_startup_metadatasindan_ayirir() {
    let kaynak = "# cli-argumanlari: çalıştır --web-proxy https://fixture.example --web-oran-deposu bellek golden/28-cli-araci.dil APP_ARG\n# cli-cikti: stdout\n";
    let gozlem = cli_gozle(kaynak);
    assert!(gozlem.tani.is_none());
    assert_eq!(gozlem.cikis, 0);
    assert_eq!(gozlem.cikti, vec!["Merhaba APP_ARG"]);
}

#[test]
fn native_deadline_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-listener-deadline"));
    assert!(faz_kipi_kabul_eder("security", "native-listener-deadline"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-listener-deadline"));
    }
}

#[test]
fn native_deadline_manifesti_sadece_terminal0_bos_http_gozlemini_kabul_eder() {
    let mut vaka = vakalari_oku().remove(0);
    vaka.faz = "security".into();
    vaka.kip = "native-listener-deadline".into();
    vaka.cikis = 0;
    vaka.tani = None;
    vaka.span = None;
    vaka.cikti.clear();
    assert!(native_deadline_manifest_sozlesmesi(&vaka));
    vaka.cikis = 1;
    assert!(!native_deadline_manifest_sozlesmesi(&vaka));
    vaka.cikis = 0;
    vaka.cikti.push("HTTP/1.1 200 OK".into());
    assert!(!native_deadline_manifest_sozlesmesi(&vaka));
    vaka.cikti.clear();
    vaka.tani = Some("C027".into());
    assert!(!native_deadline_manifest_sozlesmesi(&vaka));
    vaka.tani = None;
    vaka.faz = "cli".into();
    assert!(!native_deadline_manifest_sozlesmesi(&vaka));
}

// Follow-up only: add with real fixture + exact fixed_by row, never before.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned FD capability unavailable")]
fn native_listener_deadline_regresyonu_gercek_owned_host_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-listener-deadline")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "dedicated native regression cannot pass without a real manifest case"
    );
    for vaka in vakalar {
        assert!(native_deadline_manifest_sozlesmesi(&vaka));
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya))
            .expect("recorded minimal native source");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis);
        assert_eq!(gozlem.cikti, vaka.cikti);
    }
}

fn native_metric_manifest_sozlesmesi(v: &Vaka) -> bool {
    v.faz == "cli"
        && v.kip == "native-metric-stdio"
        && v.cikis == 2
        && v.cikti.is_empty()
        && v.tani.is_none()
        && v.span.is_none()
}
#[test]
fn native_metric_manifest_cli_owned_ve_sessiz_exit2_ile_sinirlidir() {
    assert!(KIPLER.contains(&"native-metric-stdio"));
    assert!(faz_kipi_kabul_eder("cli", "native-metric-stdio"));
    for phase in FAZLAR.iter().filter(|p| **p != "cli") {
        assert!(!faz_kipi_kabul_eder(phase, "native-metric-stdio"));
    }
    let mut v = vakalari_oku().into_iter().next().unwrap();
    v.faz = "cli".into();
    v.kip = "native-metric-stdio".into();
    v.cikis = 2;
    v.cikti.clear();
    v.tani = None;
    v.span = None;
    assert!(native_metric_manifest_sozlesmesi(&v));
    v.cikis = 0;
    assert!(!native_metric_manifest_sozlesmesi(&v));
    v.cikis = 2;
    v.cikti.push("not quiet".into());
    assert!(!native_metric_manifest_sozlesmesi(&v));
    v.cikti.clear();
    v.tani = Some("C000".into());
    v.span = Some((1, 1, 1));
    assert!(!native_metric_manifest_sozlesmesi(&v));
}

// Append only with actual .dil fixture and exact fixed_by manifest row.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned FD capability unavailable")]
fn native_metric_stdio_regresyonu_gercek_owned_host_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|v| v.kip == "native-metric-stdio")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "dedicated native regression requires real manifest case"
    );
    for vaka in vakalar {
        assert!(native_metric_manifest_sozlesmesi(&vaka));
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).unwrap();
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert_eq!(gozlem.cikis, vaka.cikis);
        assert_eq!(gozlem.cikti, vaka.cikti);
        assert!(gozlem.tani.is_none());
    }
}

#[test]
fn native_web_kopma_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-kopma"));
    assert!(faz_kipi_kabul_eder("security", "native-web-kopma"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-kopma"));
    }
}

/// K-327: yanıt yazımı ortasında RST alan gerçek native worker sonraki isteğe
/// 200 verir. ToplayanIo socket kopmasını taklit edemediği için toplu replay
/// bu kipi atlar; Unix dışında sahte başarı yoktur.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned socket capability unavailable")]
fn native_web_kopma_regresyonu_gercek_worker_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-kopma")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web kopma regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(
            gozlem.cikis, vaka.cikis,
            "{}: worker ayakta kalmalı",
            vaka.ad
        );
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: sonraki istek", vaka.ad);
    }
}

#[test]
fn native_web_yazim_butcesi_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-yazim-butcesi"));
    assert!(faz_kipi_kabul_eder("security", "native-web-yazim-butcesi"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-yazim-butcesi"));
    }
}

/// K-372: sürekli ama yavaş okuyan istemci tek işçiyi yalnız mutlak yanıt
/// yazım bütçesi kadar tutar; sonraki istek bütçe sonunda 200 alır.
/// ToplayanIo socket akış denetimini taklit edemediği için toplu replay bu
/// kipi atlar; Unix dışında sahte başarı yoktur.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned socket capability unavailable")]
fn native_web_yazim_butcesi_regresyonu_gercek_worker_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-yazim-butcesi")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web yazım bütçesi regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(
            gozlem.cikis, vaka.cikis,
            "{}: worker ayakta kalmalı",
            vaka.ad
        );
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: sonraki istek", vaka.ad);
    }
}

#[test]
fn native_web_csp_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-csp"));
    assert!(faz_kipi_kabul_eder("security", "native-web-csp"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-csp"));
    }
}

/// K-379: kaçışsız yansıyan `<script>` CSP özetini alamaz; kaynak dize
/// değişmezindeki betik izinli kalır. CSP başlığı yalnız gerçek native web
/// adaptöründe üretildiği için toplu replay bu kipi atlar; Unix dışında sahte
/// başarı yoktur.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned socket capability unavailable")]
fn native_web_csp_regresyonu_gercek_worker_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-csp")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web CSP regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(
            gozlem.cikis, vaka.cikis,
            "{}: worker ayakta kalmalı",
            vaka.ad
        );
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: betik CSP kararları", vaka.ad);
    }
}

#[test]
fn native_web_erken_red_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-erken-red"));
    assert!(faz_kipi_kabul_eder("security", "native-web-erken-red"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-erken-red"));
    }
}

/// K-374: gövde (413) ve başlık (431) sınırını aşan isteği tek seferde
/// gönderen istemci red yanıtını okuyabilir; damlatan istemci tek işçiyi
/// yalnız boşaltma bütçesi kadar tutar. ToplayanIo socket kapanışını taklit
/// edemediği için toplu replay bu kipi atlar; Unix dışında sahte başarı yok.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned socket capability unavailable")]
fn native_web_erken_red_regresyonu_gercek_worker_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-erken-red")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web erken red regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(
            gozlem.cikis, vaka.cikis,
            "{}: worker ayakta kalmalı",
            vaka.ad
        );
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: red yanıtları", vaka.ad);
    }
}

#[test]
fn native_isci_grubu_kipi_yalniz_cli_fazina_aittir() {
    assert!(KIPLER.contains(&"native-isci-grubu"));
    assert!(faz_kipi_kabul_eder("cli", "native-isci-grubu"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "cli") {
        assert!(!faz_kipi_kabul_eder(faz, "native-isci-grubu"));
    }
}

/// K-338: sürekli çöken ikincil yuva gerçek yöneticide tam 10. hızlı çıkışta
/// grubu durdurur. ToplayanIo süreç yönetimini taklit edemediği için toplu
/// replay bu kipi atlar; Unix dışında sahte başarı yoktur.
#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual Unix inherited listener capability unavailable"
)]
fn native_isci_grubu_regresyonu_gercek_yonetici_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-isci-grubu")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native işçi grubu regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: yönetici çıkışı", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: yuva sayımı", vaka.ad);
    }
}

#[test]
fn native_ayri_port_birincil_kipi_yalniz_cli_fazina_aittir() {
    assert!(KIPLER.contains(&"native-ayri-port-birincil"));
    assert!(faz_kipi_kabul_eder("cli", "native-ayri-port-birincil"));
    for faz in ["web", "runtime", "security", "lsp"] {
        assert!(!faz_kipi_kabul_eder(faz, "native-ayri-port-birincil"));
    }
}

#[test]
fn native_ayri_port_birincillik_regresyonu_gercek_sureclerle_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-ayri-port-birincil")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "ayrı port birincillik regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: roller", vaka.ad);
    }
}

#[test]
fn native_isci_saglik_mirasi_kipi_yalniz_cli_fazina_aittir() {
    assert!(KIPLER.contains(&"native-isci-saglik-mirasi"));
    assert!(faz_kipi_kabul_eder("cli", "native-isci-saglik-mirasi"));
    for faz in ["web", "runtime", "security", "lsp"] {
        assert!(!faz_kipi_kabul_eder(faz, "native-isci-saglik-mirasi"));
    }
}

#[test]
#[cfg_attr(
    not(unix),
    ignore = "actual Unix inherited listener capability unavailable"
)]
fn native_isci_saglik_mirasi_regresyonu_gercek_yonetici_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-isci-saglik-mirasi")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "işçi sağlık mirası regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: 32 işçi canlı", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: miras/asılı", vaka.ad);
    }
}

#[test]
fn native_isci_kabul_term_kipi_yalniz_cli_fazina_aittir() {
    assert!(KIPLER.contains(&"native-isci-kabul-term"));
    assert!(faz_kipi_kabul_eder("cli", "native-isci-kabul-term"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "cli") {
        assert!(!faz_kipi_kabul_eder(faz, "native-isci-kabul-term"));
    }
}

/// K-360: grup işçisi kabul ile SIGTERM denetimi arasına düşen bağlantıyı
/// düşürmez; ortak dinleyicide kabul edilen istek yanıtlanır.
#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "Linux ptrace accept window capability unavailable"
)]
fn native_isci_kabul_term_regresyonu_gercek_isci_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-isci-kabul-term")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "işçi kabul/SIGTERM regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: kayıpsız kabul", vaka.ad);
    }
}

#[test]
fn native_dosya_exdev_kipi_yalniz_runtime_fazina_aittir() {
    assert!(KIPLER.contains(&"native-dosya-exdev"));
    assert!(faz_kipi_kabul_eder("runtime", "native-dosya-exdev"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "runtime") {
        assert!(!faz_kipi_kabul_eder(faz, "native-dosya-exdev"));
    }
}

/// K-329: `.zee/yuklemeler` ile `medya` ayrı aygıtlardayken gerçek native
/// `dil` taşımayı tamamlar. Linux dışında `/dev/shm` aygıt sınırı yoktur; sahte
/// başarı yerine görünür ignored durumundadır.
#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "actual Linux /dev/shm device boundary unavailable"
)]
fn native_dosya_exdev_regresyonu_gercek_aygit_siniri_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-dosya-exdev")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native dosya EXDEV regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: taşıma sonucu", vaka.ad);
    }
}

#[test]
fn native_dosya_okuma_penceresi_kipi_yalniz_runtime_fazina_aittir() {
    for kip in [
        "native-dosya-okuma-penceresi",
        "native-dosya-okuma-izsiz-penceresi",
    ] {
        assert!(KIPLER.contains(&kip));
        assert!(faz_kipi_kabul_eder("runtime", kip));
        for faz in FAZLAR.iter().filter(|faz| **faz != "runtime") {
            assert!(!faz_kipi_kabul_eder(faz, kip));
        }
    }
}

/// K-342: Windows `ReplaceFileW` penceresi kısa bütçeyi aşsa da, yazarın
/// geçici dosya izi görünmese de gerçek native `dil` okuyucusu zee yazma
/// kilidini bekleyip yeni bütünü görür. Unix rename'inde pencere yoktur; sahte
/// başarı yerine görünür ignored durumundadır.
#[test]
#[cfg_attr(
    not(windows),
    ignore = "actual Windows ReplaceFileW window unavailable"
)]
fn native_dosya_okuma_penceresi_regresyonu_gercek_okuyucu_ile_korunur() {
    for kip in [
        "native-dosya-okuma-penceresi",
        "native-dosya-okuma-izsiz-penceresi",
    ] {
        let vakalar: Vec<_> = vakalari_oku()
            .into_iter()
            .filter(|vaka| vaka.kip == kip)
            .collect();
        assert!(
            !vakalar.is_empty(),
            "{kip} regresyonu gerçek manifest vakası olmadan geçemez"
        );
        for vaka in vakalar {
            let kaynak =
                std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
            let gozlem = vakayi_calistir(&vaka, &kaynak);
            assert!(gozlem.tani.is_none());
            assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
            assert_eq!(gozlem.cikti, vaka.cikti, "{}: okuma sonucu", vaka.ad);
        }
    }
}

#[test]
fn native_coken_degistirme_kipi_yalniz_runtime_fazina_aittir() {
    assert!(KIPLER.contains(&"native-coken-degistirme"));
    assert!(faz_kipi_kabul_eder("runtime", "native-coken-degistirme"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "runtime") {
        assert!(!faz_kipi_kabul_eder(faz, "native-coken-degistirme"));
    }
}

/// K-348: ReplaceFileW ortasında ölen yazarın bıraktığı durumda gerçek native
/// `dil` eski bütünü okur ve eklemesi önceki satırları silmez. POSIX rename'i
/// bu durumu üretmez; sahte başarı yerine görünür ignored durumundadır.
#[test]
#[cfg_attr(
    not(windows),
    ignore = "actual Windows ReplaceFileW backup recovery unavailable"
)]
fn native_coken_degistirme_regresyonu_gercek_surecle_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-coken-degistirme")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native çöken değiştirme regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: kurtarma sonucu", vaka.ad);
    }
}

#[test]
fn native_yedek_artigi_kipi_yalniz_runtime_fazina_aittir() {
    assert!(KIPLER.contains(&"native-yedek-artigi"));
    assert!(faz_kipi_kabul_eder("runtime", "native-yedek-artigi"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "runtime") {
        assert!(!faz_kipi_kabul_eder(faz, "native-yedek-artigi"));
    }
}

/// K-351: başarılı yazımın silinemeyen yedeği gerçek native `dil`in sonraki
/// yazımında silinir; hedef dışarıdan silinince eski içerik dirilmez. Yedek
/// yalnız Windows `ReplaceFileW` yazarında oluşur; diğerlerinde ignored.
#[test]
#[cfg_attr(
    not(windows),
    ignore = "actual Windows ReplaceFileW backup leftover unavailable"
)]
fn native_yedek_artigi_regresyonu_gercek_surecle_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-yedek-artigi")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native yedek artığı regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: artık temizliği", vaka.ad);
    }
}

#[test]
fn native_proje_dosya_koku_kipi_yalniz_runtime_fazina_aittir() {
    assert!(KIPLER.contains(&"native-proje-dosya-koku"));
    assert!(faz_kipi_kabul_eder("runtime", "native-proje-dosya-koku"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "runtime") {
        assert!(!faz_kipi_kabul_eder(faz, "native-proje-dosya-koku"));
    }
}

/// K-382: proje kipinde göreli yol giriş dosyasının klasöründen çözülür ve
/// yokluk iletisi eksik hedef klasörü / bulunamayan dosyayı adıyla söyler.
#[test]
fn native_proje_dosya_koku_regresyonu_gercek_surecle_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-proje-dosya-koku")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "proje dosya kökü regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: yokluk iletisi", vaka.ad);
    }
}

#[test]
fn native_web_depo_cekismesi_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-depo-cekismesi"));
    assert!(faz_kipi_kabul_eder("security", "native-web-depo-cekismesi"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-depo-cekismesi"));
    }
}

/// K-353: kalıcı web deposuna her turda araya giren başka yazar varken gerçek
/// native worker isteği 503 ile reddetmez; durum kilit içinde güncellenir.
#[test]
fn native_web_depo_cekismesi_regresyonu_gercek_surecle_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-depo-cekismesi")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web depo çekişmesi regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: kayıpsız istek", vaka.ad);
    }
}

#[test]
fn native_web_oturum_deposu_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-oturum-deposu"));
    assert!(faz_kipi_kabul_eder("security", "native-web-oturum-deposu"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-oturum-deposu"));
    }
}

/// K-370: kimlikli kayıtlarla dolu ortak depoda oturumsuz CSRF isteği gerçek
/// native worker'da 500 değil 503 alır ve worker sonraki isteği yanıtlar.
#[test]
fn native_web_oturum_deposu_regresyonu_gercek_surecle_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-oturum-deposu")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web oturum deposu regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(
            gozlem.cikti, vaka.cikti,
            "{}: 503 ve yaşayan worker",
            vaka.ad
        );
    }
}

#[test]
fn native_registry_durum_cekismesi_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-registry-durum-cekismesi"));
    assert!(faz_kipi_kabul_eder(
        "security",
        "native-registry-durum-cekismesi"
    ));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-registry-durum-cekismesi"));
    }
}

/// K-361: aynı registry cache'ine eşzamanlı iki çevrimiçi güncelleme sert
/// hata vermez; kalıcı durum kilit altında uzlaşır, geri gitmez ve aynı
/// sürümde farklı içerik P013 ile reddedilir.
#[test]
fn native_registry_durum_cekismesi_regresyonu_gercek_yarisla_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-registry-durum-cekismesi")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "registry durum çekişmesi regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: uzlaşan durum", vaka.ad);
    }
}

#[test]
fn native_unicode_kimlik_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-unicode-kimlik"));
    for faz in FAZLAR {
        assert_eq!(
            faz_kipi_kabul_eder(faz, "native-unicode-kimlik"),
            *faz == "security",
            "{faz}"
        );
    }
}

/// K-363/K-366: paket/birim/yeniden adlandırma kimlikleri ve `.zep` yolları
/// Unicode biçimine ve Windows ad kurallarına göre platformdan bağımsız
/// kabul/ret verir; her vaka tek senaryoyu seçer.
#[test]
fn native_unicode_kimlik_regresyonlari_calisma_aninda_uretilen_adlarla_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-unicode-kimlik")
        .collect();
    assert_eq!(
        vakalar.len(),
        8,
        "K-363/K-366 regresyonları gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: kimlik kararı", vaka.ad);
    }
}

#[test]
fn native_eszamanli_ekleme_kipi_yalniz_runtime_fazina_aittir() {
    assert!(KIPLER.contains(&"native-eszamanli-ekleme"));
    assert!(faz_kipi_kabul_eder("runtime", "native-eszamanli-ekleme"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "runtime") {
        assert!(!faz_kipi_kabul_eder(faz, "native-eszamanli-ekleme"));
    }
}

/// K-346: iki yazar süreci × 1000 düz ekleme, eylem içinde ekleyip geri alan
/// iki süreçle yarışır; hiçbir başarılı satır kaybolmaz, yarım satır olmaz.
#[test]
fn native_eszamanli_ekleme_regresyonu_gercek_surecler_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-eszamanli-ekleme")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native eşzamanlı ekleme regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: kalıcı satırlar", vaka.ad);
    }
}

#[test]
fn native_eszamanli_tasima_ve_paket_ekle_kipleri_tek_faza_aittir() {
    for (kip, sahip) in [
        ("native-eszamanli-tasima", "runtime"),
        ("native-eszamanli-paket-ekle", "cli"),
    ] {
        assert!(KIPLER.contains(&kip));
        for faz in FAZLAR {
            assert_eq!(
                faz_kipi_kabul_eder(faz, kip),
                *faz == sahip,
                "{kip} / {faz}"
            );
        }
    }
}

/// K-347: eylem içinde taşıyıp geri alan süreç, kaynağa düz ekleyen yazarın
/// hiçbir satırını silmez ve satırı iki dosyada bırakmaz; eşzamanlı `dil
/// ekle` komutları birbirinin bağımlılığını ezmez, son kilit geçerlidir.
#[test]
fn native_eszamanli_tasima_ve_paket_ekle_regresyonlari_gercek_surecler_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| {
            matches!(
                vaka.kip.as_str(),
                "native-eszamanli-tasima" | "native-eszamanli-paket-ekle"
            )
        })
        .collect();
    assert_eq!(
        vakalar.len(),
        2,
        "K-347 regresyonları gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(gozlem.cikis, vaka.cikis, "{}: çıkış", vaka.ad);
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: kalıcı durum", vaka.ad);
    }
}

#[test]
fn native_web_yukleme_kipi_yalniz_security_fazina_aittir() {
    assert!(KIPLER.contains(&"native-web-yukleme"));
    assert!(faz_kipi_kabul_eder("security", "native-web-yukleme"));
    for faz in FAZLAR.iter().filter(|faz| **faz != "security") {
        assert!(!faz_kipi_kabul_eder(faz, "native-web-yukleme"));
    }
}

/// K-332: reddedilen ya da tüketilmeyen binary yükleme gerçek native worker'da
/// `.parca` bırakmaz; taşıyan rota dosyasını bütün olarak korur.
#[test]
#[cfg_attr(not(unix), ignore = "actual Unix owned socket capability unavailable")]
fn native_web_yukleme_regresyonu_gercek_worker_ile_korunur() {
    let vakalar: Vec<_> = vakalari_oku()
        .into_iter()
        .filter(|vaka| vaka.kip == "native-web-yukleme")
        .collect();
    assert!(
        !vakalar.is_empty(),
        "native web yükleme regresyonu gerçek manifest vakası olmadan geçemez"
    );
    for vaka in vakalar {
        let kaynak = std::fs::read_to_string(depo().join(&vaka.dosya)).expect("fixture kaynağı");
        let gozlem = vakayi_calistir(&vaka, &kaynak);
        assert!(gozlem.tani.is_none());
        assert_eq!(
            gozlem.cikis, vaka.cikis,
            "{}: worker ayakta kalmalı",
            vaka.ad
        );
        assert_eq!(gozlem.cikti, vaka.cikti, "{}: yükleme sonucu", vaka.ad);
    }
}
