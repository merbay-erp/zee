//! Kritik üretim işlevlerinin Clippy tabanlı boyut/karmaşıklık eğilimini izler.

use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const TABAN_YOLU: &str = "compiler/tests/fixtures/islev-egilimi-v1.tsv";
const RAPOR_YOLU: &str = "docs/islev-egilimi.md";
const SATIR_IZLEME_ESIGI: usize = 80;
const KARMASIKLIK_IZLEME_ESIGI: usize = 12;

#[derive(Clone, Debug, Eq, PartialEq)]
struct IslevOlcumu {
    yol: String,
    kimlik: String,
    satir: usize,
    karmasiklik: usize,
}

#[derive(Debug)]
struct Taban {
    inceleme: String,
    olcumler: BTreeMap<(String, String), IslevOlcumu>,
}

#[derive(Default)]
struct HamOlcum {
    yol: String,
    ad: String,
    satir: Option<usize>,
    karmasiklik: Option<usize>,
}

enum Kip {
    Yazdir,
    Denetle,
    RaporYaz,
    TabanYaz(String),
}

struct GeciciClippyAyari {
    klasor: PathBuf,
}

impl GeciciClippyAyari {
    fn yeni() -> Result<Self, String> {
        let klasor = env::temp_dir().join(format!("zee-islev-egilimi-{}", std::process::id()));
        fs::create_dir(&klasor)
            .map_err(|hata| format!("{} oluşturulamadı: {hata}", klasor.display()))?;
        fs::write(
            klasor.join("clippy.toml"),
            "too-many-lines-threshold = 0\ncognitive-complexity-threshold = 0\n",
        )
        .map_err(|hata| format!("geçici Clippy ayarı yazılamadı: {hata}"))?;
        Ok(Self { klasor })
    }
}

impl Drop for GeciciClippyAyari {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.klasor.join("clippy.toml"));
        let _ = fs::remove_dir(&self.klasor);
    }
}

fn depo_kokunu_bul(mut yol: PathBuf) -> Result<PathBuf, String> {
    loop {
        if yol.join("README.md").is_file() && yol.join("compiler/Cargo.toml").is_file() {
            return Ok(yol);
        }
        if !yol.pop() {
            return Err("README.md ve compiler/Cargo.toml taşıyan depo kökü bulunamadı".into());
        }
    }
}

fn argumanlari_oku() -> Result<(Kip, PathBuf), String> {
    let mut kip = Kip::Yazdir;
    let mut depo = None;
    let mut argumanlar = env::args().skip(1);
    while let Some(arguman) = argumanlar.next() {
        match arguman.as_str() {
            "--denetle" => kip = Kip::Denetle,
            "--rapor-yaz" => kip = Kip::RaporYaz,
            "--taban-yaz" => {
                let inceleme = argumanlar
                    .next()
                    .ok_or_else(|| "--taban-yaz bir inceleme kaydı ister".to_string())?;
                if inceleme.trim().is_empty() || inceleme.contains(['\t', '\n', '\r']) {
                    return Err("inceleme kaydı boş olamaz ve sekme/satır sonu taşıyamaz".into());
                }
                kip = Kip::TabanYaz(inceleme);
            }
            "--depo" => {
                depo = Some(PathBuf::from(
                    argumanlar
                        .next()
                        .ok_or_else(|| "--depo bir yol ister".to_string())?,
                ));
            }
            "--yardim" | "-h" => {
                return Err(
                    "kullanım: islev_egilimi [--denetle|--rapor-yaz|--taban-yaz KAYIT] [--depo YOL]"
                        .into(),
                );
            }
            _ => return Err(format!("bilinmeyen argüman: {arguman}")),
        }
    }
    let baslangic = depo.unwrap_or(env::current_dir().map_err(|hata| hata.to_string())?);
    Ok((kip, depo_kokunu_bul(baslangic)?))
}

fn clippy_ciktisi(depo: &Path) -> Result<String, String> {
    let ayar = GeciciClippyAyari::yeni()?;
    let cikti = Command::new("cargo")
        .args([
            "clippy",
            "--locked",
            "--lib",
            "--bin",
            "dil",
            "--message-format=json",
            "--",
            "--force-warn",
            "clippy::too_many_lines",
            "--force-warn",
            "clippy::cognitive_complexity",
        ])
        .current_dir(depo.join("compiler"))
        .env("CLIPPY_CONF_DIR", &ayar.klasor)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .map_err(|hata| format!("Clippy başlatılamadı: {hata}"))?;
    if !cikti.status.success() {
        return Err(format!(
            "Clippy ölçümü başarısız:\n{}",
            String::from_utf8_lossy(&cikti.stderr)
        ));
    }
    String::from_utf8(cikti.stdout).map_err(|_| "Clippy çıktısı UTF-8 değil".into())
}

fn parantezdeki_ilk_sayi(mesaj: &str) -> Option<usize> {
    let bas = mesaj.rfind('(')? + 1;
    mesaj[bas..].split('/').next()?.parse().ok()
}

fn islev_adini_oku(satir: &str) -> Option<String> {
    let (_, sonrasi) = satir.split_once("fn ")?;
    let ad: String = sonrasi
        .chars()
        .take_while(|karakter| karakter.is_alphanumeric() || *karakter == '_')
        .collect();
    (!ad.is_empty()).then_some(ad)
}

fn uretim_yolu_mu(yol: &str) -> bool {
    yol.starts_with("src/")
        && !yol.starts_with("src/bin/")
        && !yol.ends_with("/testler.rs")
        && !yol.ends_with("/tests.rs")
}

fn json_olcumlerini_oku(cikti: &str) -> Result<Vec<IslevOlcumu>, String> {
    let mut hamlar: BTreeMap<(String, usize, String), HamOlcum> = BTreeMap::new();
    for (sira, satir) in cikti.lines().enumerate() {
        if satir.trim().is_empty() {
            continue;
        }
        let deger: Value = serde_json::from_str(satir)
            .map_err(|hata| format!("Clippy JSON satırı {} bozuk: {hata}", sira + 1))?;
        if deger["reason"] != "compiler-message" {
            continue;
        }
        let Some(kod) = deger["message"]["code"]["code"].as_str() else {
            continue;
        };
        let tur = match kod {
            "clippy::too_many_lines" => "satir",
            "clippy::cognitive_complexity" => "karmasiklik",
            _ => continue,
        };
        let mesaj = deger["message"]["message"]
            .as_str()
            .ok_or_else(|| format!("Clippy {kod} tanısı mesaj taşımıyor"))?;
        let olcum = parantezdeki_ilk_sayi(mesaj)
            .ok_or_else(|| format!("Clippy {kod} ölçümü ayrıştırılamadı: {mesaj}"))?;
        let spanlar = deger["message"]["spans"]
            .as_array()
            .ok_or_else(|| format!("Clippy {kod} tanısı span taşımıyor"))?;
        let span = spanlar
            .iter()
            .find(|span| span["is_primary"] == true)
            .ok_or_else(|| format!("Clippy {kod} birincil span taşımıyor"))?;
        let yol = span["file_name"]
            .as_str()
            .ok_or_else(|| "Clippy span yolu yok".to_string())?
            .replace('\\', "/");
        if !uretim_yolu_mu(&yol) {
            continue;
        }
        let satir_no = span["line_start"]
            .as_u64()
            .and_then(|sayi| usize::try_from(sayi).ok())
            .ok_or_else(|| "Clippy span satırı geçersiz".to_string())?;
        let kaynak_satiri = span["text"]
            .as_array()
            .and_then(|satirlar| satirlar.first())
            .and_then(|satir| satir["text"].as_str())
            .unwrap_or_default();
        let Some(ad) = islev_adini_oku(kaynak_satiri) else {
            continue;
        };
        let anahtar = (yol.clone(), satir_no, ad.clone());
        let ham = hamlar.entry(anahtar).or_insert_with(|| HamOlcum {
            yol,
            ad,
            ..HamOlcum::default()
        });
        match tur {
            "satir" => ham.satir = Some(ham.satir.unwrap_or(0).max(olcum)),
            "karmasiklik" => {
                ham.karmasiklik = Some(ham.karmasiklik.unwrap_or(0).max(olcum));
            }
            _ => unreachable!(),
        }
    }

    let mut tekrarlar: HashMap<(String, String), usize> = HashMap::new();
    let mut sonuc = Vec::with_capacity(hamlar.len());
    for ham in hamlar.into_values() {
        let tekrar = tekrarlar
            .entry((ham.yol.clone(), ham.ad.clone()))
            .and_modify(|sayi| *sayi += 1)
            .or_insert(1);
        sonuc.push(IslevOlcumu {
            yol: ham.yol,
            kimlik: format!("{}#{}", ham.ad, *tekrar),
            satir: ham.satir.unwrap_or(0),
            karmasiklik: ham.karmasiklik.unwrap_or(0),
        });
    }
    if sonuc.is_empty() {
        return Err("Clippy çıktısında üretim işlevi ölçümü bulunamadı".into());
    }
    Ok(sonuc)
}

fn izlenen_mi(olcum: &IslevOlcumu) -> bool {
    olcum.satir >= SATIR_IZLEME_ESIGI || olcum.karmasiklik >= KARMASIKLIK_IZLEME_ESIGI
}

fn tabani_oku(yol: &Path) -> Result<Taban, String> {
    let metin =
        fs::read_to_string(yol).map_err(|hata| format!("{} okunamadı: {hata}", yol.display()))?;
    let inceleme = metin
        .lines()
        .find_map(|satir| satir.strip_prefix("# inceleme: "))
        .filter(|deger| !deger.trim().is_empty())
        .ok_or_else(|| "işlev tabanı inceleme kaydı taşımıyor".to_string())?
        .to_string();
    let mut olcumler = BTreeMap::new();
    for (sira, satir) in metin.lines().enumerate() {
        if satir.is_empty() || satir.starts_with('#') || satir.starts_with("yol\t") {
            continue;
        }
        let alanlar: Vec<_> = satir.split('\t').collect();
        let [yol, kimlik, satir_sayisi, karmasiklik] = alanlar.as_slice() else {
            return Err(format!(
                "işlev tabanı satırı {} dört alan taşımalı",
                sira + 1
            ));
        };
        let olcum = IslevOlcumu {
            yol: (*yol).to_string(),
            kimlik: (*kimlik).to_string(),
            satir: satir_sayisi
                .parse()
                .map_err(|_| format!("işlev tabanı satır ölçümü geçersiz: {satir_sayisi}"))?,
            karmasiklik: karmasiklik
                .parse()
                .map_err(|_| format!("işlev tabanı karmaşıklığı geçersiz: {karmasiklik}"))?,
        };
        if !uretim_yolu_mu(&olcum.yol) || !izlenen_mi(&olcum) {
            return Err(format!(
                "işlev tabanı kritik olmayan kayıt taşıyor: {}::{}",
                olcum.yol, olcum.kimlik
            ));
        }
        let anahtar = (olcum.yol.clone(), olcum.kimlik.clone());
        if olcumler.insert(anahtar, olcum).is_some() {
            return Err(format!("yinelenen işlev tabanı kaydı: {yol}::{kimlik}"));
        }
    }
    if olcumler.is_empty() {
        return Err("işlev tabanı boş olamaz".into());
    }
    Ok(Taban { inceleme, olcumler })
}

fn satir_artis_pay(base: usize) -> usize {
    base.div_ceil(10).clamp(8, 24)
}

fn karmasiklik_artis_pay(base: usize) -> usize {
    base.div_ceil(5).clamp(2, 5)
}

fn isaretli_fark(guncel: usize, taban: usize) -> String {
    if guncel >= taban {
        format!("+{}", guncel - taban)
    } else {
        format!("-{}", taban - guncel)
    }
}

fn durum_ve_ihlaller(
    taban: Option<&IslevOlcumu>,
    guncel: Option<&IslevOlcumu>,
) -> (String, Vec<String>) {
    let Some(taban) = taban else {
        let guncel = guncel.expect("yeni ölçüm bulunmalı");
        let ihlal = format!(
            "yeni kritik işlev inceleme tabanında yok: {}::{} ({} satır, karmaşıklık {})",
            guncel.yol, guncel.kimlik, guncel.satir, guncel.karmasiklik
        );
        return ("YENİ — inceleme gerekli".into(), vec![ihlal]);
    };
    let Some(guncel) = guncel else {
        let ihlal = format!(
            "tabandaki işlev ölçülemiyor; silme/yeniden adlandırma veya lint gizleme incelenmeli: {}::{}",
            taban.yol, taban.kimlik
        );
        return ("KAYIP — inceleme gerekli".into(), vec![ihlal]);
    };

    let mut durumlar = Vec::new();
    let mut ihlaller = Vec::new();
    let satir_pay = satir_artis_pay(taban.satir);
    let karmasiklik_pay = karmasiklik_artis_pay(taban.karmasiklik);
    if guncel.satir > taban.satir.saturating_add(satir_pay) {
        durumlar.push("SATIR EŞİĞİ");
        ihlaller.push(format!(
            "{}::{} satır büyümesi {} > +{}",
            taban.yol,
            taban.kimlik,
            isaretli_fark(guncel.satir, taban.satir),
            satir_pay
        ));
    }
    if guncel.karmasiklik > taban.karmasiklik.saturating_add(karmasiklik_pay) {
        durumlar.push("KARMAŞIKLIK EŞİĞİ");
        ihlaller.push(format!(
            "{}::{} karmaşıklık büyümesi {} > +{}",
            taban.yol,
            taban.kimlik,
            isaretli_fark(guncel.karmasiklik, taban.karmasiklik),
            karmasiklik_pay
        ));
    }
    if durumlar.is_empty() {
        if guncel.satir < taban.satir || guncel.karmasiklik < taban.karmasiklik {
            durumlar.push("İYİLEŞTİ");
        } else if guncel.satir == taban.satir && guncel.karmasiklik == taban.karmasiklik {
            durumlar.push("SABİT");
        } else {
            durumlar.push("PAY İÇİNDE");
        }
    }
    (durumlar.join(" + "), ihlaller)
}

fn raporu_uret(taban: &Taban, gunceller: &[IslevOlcumu]) -> (String, Vec<String>) {
    let guncel_harita: BTreeMap<_, _> = gunceller
        .iter()
        .map(|olcum| ((olcum.yol.clone(), olcum.kimlik.clone()), olcum))
        .collect();
    let mut anahtarlar = taban.olcumler.keys().cloned().collect::<Vec<_>>();
    for (anahtar, olcum) in &guncel_harita {
        if izlenen_mi(olcum) && !taban.olcumler.contains_key(anahtar) {
            anahtarlar.push(anahtar.clone());
        }
    }
    anahtarlar.sort();

    let mut rapor = format!(
        "# Kritik işlev boyutu ve karmaşıklık eğilimi\n\n\
<!-- `cd compiler && cargo run --locked --bin islev_egilimi -- --rapor-yaz` üretir. Elle değiştirme. -->\n\n\
Bu rapor K-144/ADR-041 makine kapısının güncel görünümüdür. Mutlak bir\n\
\"iyi işlev N satırdır\" kuralı koymaz; incelenmiş tabana göre büyümeyi görünür\n\
kılar. Clippy ölçümü sabit Rust araç zinciriyle üretim `lib` ve `dil` ikilisinde\n\
çalışır; `allow` öznitelikleri `--force-warn` nedeniyle kapıyı atlayamaz.\n\n\
- İncelenmiş taban: `{}`\n\
- İzlemeye giriş: en az {} satır veya bilişsel karmaşıklık {}\n\
- Satır gözden geçirme payı: tabanın %10'u; en az +8, en çok +24\n\
- Karmaşıklık gözden geçirme payı: tabanın %20'si; en az +2, en çok +5\n\n\
| İşlev | Satır taban→güncel (Δ/pay) | Karmaşıklık taban→güncel (Δ/pay) | Durum |\n\
|---|---:|---:|---|\n",
        taban.inceleme, SATIR_IZLEME_ESIGI, KARMASIKLIK_IZLEME_ESIGI
    );
    let mut ihlaller = Vec::new();
    for anahtar in anahtarlar {
        let eski = taban.olcumler.get(&anahtar);
        let yeni = guncel_harita.get(&anahtar).copied();
        let (durum, satir_ihlalleri) = durum_ve_ihlaller(eski, yeni);
        ihlaller.extend(satir_ihlalleri);
        let (eski_satir, eski_karmasiklik) = eski
            .map(|olcum| (olcum.satir, olcum.karmasiklik))
            .unwrap_or((0, 0));
        let (yeni_satir, yeni_karmasiklik) = yeni
            .map(|olcum| (olcum.satir, olcum.karmasiklik))
            .unwrap_or((0, 0));
        let satir_pay = eski.map(|olcum| satir_artis_pay(olcum.satir)).unwrap_or(0);
        let karmasiklik_pay = eski
            .map(|olcum| karmasiklik_artis_pay(olcum.karmasiklik))
            .unwrap_or(0);
        rapor.push_str(&format!(
            "| `compiler/{}::{}` | {}→{} ({}/{:+}) | {}→{} ({}/{:+}) | {} |\n",
            anahtar.0,
            anahtar.1,
            eski_satir,
            yeni_satir,
            isaretli_fark(yeni_satir, eski_satir),
            satir_pay,
            eski_karmasiklik,
            yeni_karmasiklik,
            isaretli_fark(yeni_karmasiklik, eski_karmasiklik),
            karmasiklik_pay,
            durum
        ));
    }
    rapor.push_str(
        "\nEşik aşımı otomatik bir tasarım hükmü değildir; işi durdurup işlevi bölme veya\n\
gerekçeli yeni tabanı aynı kod incelemesinde kabul etme zorunluluğudur. Düşüşler\n\
tabanı kendiliğinden aşağı çekmez; böylece küçük artışlarla borç gizlenemez.\n",
    );
    (rapor, ihlaller)
}

fn taban_metni(olcumler: &[IslevOlcumu], inceleme: &str) -> Result<String, String> {
    let mut izlenenler: Vec<_> = olcumler.iter().filter(|olcum| izlenen_mi(olcum)).collect();
    izlenenler.sort_by(|sol, sag| (&sol.yol, &sol.kimlik).cmp(&(&sag.yol, &sag.kimlik)));
    if izlenenler.is_empty() {
        return Err("incelenecek kritik işlev bulunamadı".into());
    }
    let mut metin = format!(
        "# zee-islev-egilimi-v1\n# inceleme: {inceleme}\n# yol\tkimlik\tsatir\tkarmasiklik\n"
    );
    for olcum in izlenenler {
        metin.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            olcum.yol, olcum.kimlik, olcum.satir, olcum.karmasiklik
        ));
    }
    Ok(metin)
}

fn atomik_yaz(yol: &Path, icerik: &str) -> Result<(), String> {
    dil::kalici_dosya::atomik_yaz(yol, icerik.as_bytes())
        .map_err(|hata| format!("{} yazılamadı: {hata}", yol.display()))
}

fn calistir() -> Result<(), String> {
    let (kip, depo) = argumanlari_oku()?;
    let gunceller = json_olcumlerini_oku(&clippy_ciktisi(&depo)?)?;
    let taban_yolu = depo.join(TABAN_YOLU);
    if let Kip::TabanYaz(inceleme) = &kip {
        atomik_yaz(&taban_yolu, &taban_metni(&gunceller, inceleme)?)?;
    }
    let taban = tabani_oku(&taban_yolu)?;
    let (rapor, ihlaller) = raporu_uret(&taban, &gunceller);
    let rapor_yolu = depo.join(RAPOR_YOLU);
    match kip {
        Kip::Yazdir => print!("{rapor}"),
        Kip::RaporYaz | Kip::TabanYaz(_) => {
            atomik_yaz(&rapor_yolu, &rapor)?;
            println!(
                "İşlev eğilim raporu güncellendi: {} kayıt.",
                taban.olcumler.len()
            );
        }
        Kip::Denetle => {
            let kayitli = fs::read_to_string(&rapor_yolu)
                .map_err(|hata| format!("{} okunamadı: {hata}", rapor_yolu.display()))?;
            if kayitli != rapor {
                return Err(
                    "işlev eğilim raporu bayat; `cargo run --locked --bin islev_egilimi -- --rapor-yaz` çalıştır"
                        .into(),
                );
            }
            if !ihlaller.is_empty() {
                return Err(format!(
                    "işlev eğilim incelemesi gerekli:\n- {}",
                    ihlaller.join("\n- ")
                ));
            }
            println!(
                "İşlev eğilimi temiz: {} kritik kayıt, bayat rapor yok.",
                taban.olcumler.len()
            );
        }
    }
    if !ihlaller.is_empty() {
        return Err(format!(
            "işlev eğilim incelemesi gerekli:\n- {}",
            ihlaller.join("\n- ")
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    match calistir() {
        Ok(()) => ExitCode::SUCCESS,
        Err(hata) => {
            eprintln!("HATA: {hata}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn tani(kod: &str, mesaj: &str, yol: &str, satir: usize, kaynak: &str) -> String {
        serde_json::json!({
            "reason": "compiler-message",
            "message": {
                "code": {"code": kod},
                "message": mesaj,
                "spans": [{
                    "is_primary": true,
                    "file_name": yol,
                    "line_start": satir,
                    "text": [{"text": kaynak}]
                }]
            }
        })
        .to_string()
    }

    #[test]
    fn clippy_olcumleri_yol_ve_ayni_ad_sirasiyla_kimliklenir() {
        let cikti = [
            tani(
                "clippy::too_many_lines",
                "this function has too many lines (90/0)",
                "src/ornek.rs",
                10,
                "fn isle() {",
            ),
            tani(
                "clippy::cognitive_complexity",
                "the function has a cognitive complexity of (13/0)",
                "src/ornek.rs",
                10,
                "fn isle() {",
            ),
            tani(
                "clippy::too_many_lines",
                "this function has too many lines (82/0)",
                "src/ornek.rs",
                80,
                "fn isle() {",
            ),
        ]
        .join("\n");
        let olcumler = json_olcumlerini_oku(&cikti).expect("ölçüm ayrıştırılmalı");
        assert_eq!(olcumler.len(), 2);
        assert_eq!(olcumler[0].kimlik, "isle#1");
        assert_eq!(olcumler[0].satir, 90);
        assert_eq!(olcumler[0].karmasiklik, 13);
        assert_eq!(olcumler[1].kimlik, "isle#2");
    }

    #[test]
    fn closure_ve_arac_binarysi_uretim_olcumune_girmez() {
        let cikti = [
            tani(
                "clippy::cognitive_complexity",
                "the function has a cognitive complexity of (30/0)",
                "src/ornek.rs",
                10,
                ".map(|deger| deger + 1)",
            ),
            tani(
                "clippy::too_many_lines",
                "this function has too many lines (200/0)",
                "src/bin/arac.rs",
                1,
                "fn main() {",
            ),
            tani(
                "clippy::too_many_lines",
                "this function has too many lines (81/0)",
                "src/urun.rs",
                1,
                "fn calistir() {",
            ),
        ]
        .join("\n");
        let olcumler = json_olcumlerini_oku(&cikti).expect("ölçüm ayrıştırılmalı");
        assert_eq!(olcumler.len(), 1);
        assert_eq!(olcumler[0].yol, "src/urun.rs");
    }

    #[test]
    fn artis_payi_oransal_ama_alt_ve_ust_sinirlidir() {
        assert_eq!(satir_artis_pay(20), 8);
        assert_eq!(satir_artis_pay(100), 10);
        assert_eq!(satir_artis_pay(500), 24);
        assert_eq!(karmasiklik_artis_pay(1), 2);
        assert_eq!(karmasiklik_artis_pay(20), 4);
        assert_eq!(karmasiklik_artis_pay(100), 5);
    }

    #[test]
    fn esik_icindeki_artis_ile_asim_ayrilir() {
        let taban = IslevOlcumu {
            yol: "src/ornek.rs".into(),
            kimlik: "isle#1".into(),
            satir: 100,
            karmasiklik: 20,
        };
        let pay_icinde = IslevOlcumu {
            satir: 110,
            karmasiklik: 24,
            ..taban.clone()
        };
        let (_, ihlaller) = durum_ve_ihlaller(Some(&taban), Some(&pay_icinde));
        assert!(ihlaller.is_empty());

        let asan = IslevOlcumu {
            satir: 111,
            karmasiklik: 25,
            ..taban.clone()
        };
        let (durum, ihlaller) = durum_ve_ihlaller(Some(&taban), Some(&asan));
        assert!(durum.contains("SATIR EŞİĞİ"));
        assert!(durum.contains("KARMAŞIKLIK EŞİĞİ"));
        assert_eq!(ihlaller.len(), 2);
    }

    #[test]
    fn yeni_ve_kayip_kritik_islev_inceleme_ister() {
        let olcum = IslevOlcumu {
            yol: "src/ornek.rs".into(),
            kimlik: "isle#1".into(),
            satir: 90,
            karmasiklik: 4,
        };
        assert_eq!(durum_ve_ihlaller(None, Some(&olcum)).1.len(), 1);
        assert_eq!(durum_ve_ihlaller(Some(&olcum), None).1.len(), 1);
    }
}
