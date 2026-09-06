//! K-173/ADR-069 uzun soak kapısı: derleyici döngüsü (golden korpus derlemesi
//! ve sabit yürütme) ile gerçek `dillsp` süreci (didOpen/didChange döngüsü)
//! süre bütçesi boyunca koşar; iki sürecin RSS'i pencere pencere örneklenir ve
//! ısınma sonrası büyüme sınırı aşılırsa kapı kırılır. Sonuç exact Git
//! provenance'ıyla `docs/soak-gecmisi-v1.tsv` tarihçesine eklenir.

use serde_json::json;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode, Stdio};
use std::time::{Duration, Instant};

const GECMIS_SEMASI: &str = "# zee-soak-gecmisi-1";
const GECMIS_BASLIK: &str = "# git_sha\ttarih\tplatform\trustc\tsure_sn\tpencere_sn\tderleme_donemi\tlsp_donemi\tderleyici_rss_ilk_kib\tderleyici_rss_son_kib\tdillsp_rss_ilk_kib\tdillsp_rss_son_kib\tderleyici_buyume_yuzde\tdillsp_buyume_yuzde\tazami_buyume_yuzde\tazami_buyume_kib\tsonuc";
/// Isınma sonrası ilk pencere medyanına göre izin verilen büyüme.
const AZAMI_BUYUME_YUZDE: u64 = 10;
/// Küçük süreçlerde yüzdeyi anlamlı kılan mutlak tavan.
const AZAMI_BUYUME_KIB: u64 = 32 * 1024;
const ISINMA_PENCERESI: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ornek {
    saniye: u64,
    derleyici_rss_kib: u64,
    dillsp_rss_kib: u64,
    derleme_donemi: u64,
    lsp_donemi: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Karar {
    derleyici_ilk: u64,
    derleyici_son: u64,
    dillsp_ilk: u64,
    dillsp_son: u64,
    derleyici_buyume_yuzde: u64,
    dillsp_buyume_yuzde: u64,
    gecti: bool,
    neden: String,
}

struct Ayarlar {
    sure: Duration,
    pencere: Duration,
    dillsp: PathBuf,
    gecmis: Option<PathBuf>,
    rapor: Option<PathBuf>,
    depo: PathBuf,
}

fn depo_kokunu_bul(mut yol: PathBuf) -> Result<PathBuf, String> {
    loop {
        if yol.join("README.md").is_file() && yol.join("compiler/Cargo.toml").is_file() {
            return Ok(yol);
        }
        if !yol.pop() {
            return Err("depo kökü bulunamadı".into());
        }
    }
}

fn argumanlari_oku() -> Result<Ayarlar, String> {
    let mut sure = 600u64;
    let mut pencere = 60u64;
    let mut dillsp = None;
    let mut gecmis = None;
    let mut rapor = None;
    let mut argumanlar = std::env::args().skip(1);
    while let Some(arguman) = argumanlar.next() {
        let mut deger = || {
            argumanlar
                .next()
                .ok_or_else(|| format!("{arguman} bir değer ister"))
        };
        match arguman.as_str() {
            "--sure-sn" => sure = deger()?.parse().map_err(|_| "--sure-sn sayı olmalı")?,
            "--pencere-sn" => pencere = deger()?.parse().map_err(|_| "--pencere-sn sayı olmalı")?,
            "--dillsp" => dillsp = Some(PathBuf::from(deger()?)),
            "--gecmis" => gecmis = Some(PathBuf::from(deger()?)),
            "--rapor" => rapor = Some(PathBuf::from(deger()?)),
            _ => return Err("kullanım: soak --sure-sn N [--pencere-sn N] [--dillsp YOL] [--gecmis TSV] [--rapor MD]".into()),
        }
    }
    if pencere == 0 || sure < pencere * 3 {
        return Err("süre en az üç pencere olmalı; pencere sıfır olamaz".into());
    }
    let depo = depo_kokunu_bul(std::env::current_dir().map_err(|h| h.to_string())?)?;
    let dillsp = dillsp.unwrap_or_else(|| depo.join("compiler/target/release/dillsp"));
    if !dillsp.is_file() {
        return Err(format!(
            "dillsp ikilisi yok: {} (önce cargo build --release --bin dillsp)",
            dillsp.display()
        ));
    }
    Ok(Ayarlar {
        sure: Duration::from_secs(sure),
        pencere: Duration::from_secs(pencere),
        dillsp,
        gecmis,
        rapor,
        depo,
    })
}

fn rss_kib(pid: u32) -> Option<u64> {
    let cikti = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    String::from_utf8(cikti.stdout).ok()?.trim().parse().ok()
}

fn golden_kaynaklari(depo: &Path) -> Result<(PathBuf, Vec<String>), String> {
    let mut yollar = std::fs::read_dir(depo.join("golden"))
        .map_err(|h| format!("golden okunamadı: {h}"))?
        .filter_map(|g| g.ok().map(|g| g.path()))
        .filter(|y| y.extension().and_then(|u| u.to_str()) == Some("dil"))
        .collect::<Vec<_>>();
    yollar.sort();
    let kaynaklar = yollar
        .iter()
        .map(|y| std::fs::read_to_string(y).map_err(|h| format!("{} okunamadı: {h}", y.display())))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((depo.join("golden"), kaynaklar))
}

fn derleme_donemi(klasor: &Path, kaynaklar: &[String]) -> Result<(), String> {
    let mut yukleyici = |ad: &str| -> Result<String, String> {
        std::fs::read_to_string(klasor.join(format!("{ad}.dil"))).map_err(|h| h.to_string())
    };
    for kaynak in kaynaklar {
        dil::kaynagi_derle_birimlerle(kaynak, &mut yukleyici)
            .map_err(|t| format!("golden derlenemedi: {} {}", t.kod, t.mesaj))?;
    }
    let cikti = dil::kaynagi_calistir(
        "toplam 0 olsun\n2000 kez tekrarla\n    toplamı 1 artır\ntoplam yaz\n",
    )
    .map_err(|t| format!("yürütme dönemi başarısız: {} {}", t.kod, t.mesaj))?;
    if cikti != ["2000"] {
        return Err("yürütme dönemi beklenen çıktıyı vermedi".into());
    }
    Ok(())
}

fn satirli_kaynak(satir_sayisi: usize, tohum: u64) -> String {
    let mut kaynak = String::new();
    for sira in 0..satir_sayisi / 4 {
        kaynak.push_str(&format!(
            "değer{sira} {} olsun\nkatı{sira} değer{sira} ile 3 ün çarpımı olsun\nkatı{sira} 0 dan büyükse\n    toplam{sira} katı{sira} ile 1 in toplamı olsun\n",
            sira as u64 + tohum
        ));
    }
    kaynak
}

struct LspSureci {
    cocuk: Child,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
    surum: u64,
}

impl LspSureci {
    fn baslat(dillsp: &Path) -> Result<Self, String> {
        let mut cocuk = Command::new(dillsp)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|h| format!("{} başlatılamadı: {h}", dillsp.display()))?;
        let stdin = cocuk.stdin.take().ok_or("dillsp stdin")?;
        let stdout = cocuk.stdout.take().ok_or("dillsp stdout")?;
        let mut surec = LspSureci {
            cocuk,
            stdin,
            stdout,
            surum: 1,
        };
        surec.gonder(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)?;
        let yanit = surec.cerceve_oku()?;
        if !yanit.contains("\"capabilities\"") {
            return Err("dillsp initialize capabilities vermedi".into());
        }
        surec.gonder(&json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/zee-soak.dil","languageId":"dil","version":1,"text":satirli_kaynak(2_000, 0)}}}).to_string())?;
        surec.cerceve_oku()?;
        Ok(surec)
    }

    fn gonder(&mut self, govde: &str) -> Result<(), String> {
        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n{}",
            govde.len(),
            govde
        )
        .and_then(|_| self.stdin.flush())
        .map_err(|h| format!("dillsp yazılamadı: {h}"))
    }

    fn cerceve_oku(&mut self) -> Result<String, String> {
        let mut baslik = Vec::new();
        let mut son_dort = [0u8; 4];
        while &son_dort != b"\r\n\r\n" {
            if baslik.len() >= 8 * 1024 {
                return Err("LSP başlığı 8 KiB'ı aştı".into());
            }
            let mut bayt = [0u8; 1];
            self.stdout
                .read_exact(&mut bayt)
                .map_err(|h| format!("dillsp okunamadı: {h}"))?;
            baslik.push(bayt[0]);
            son_dort.rotate_left(1);
            son_dort[3] = bayt[0];
        }
        let baslik = String::from_utf8_lossy(&baslik).into_owned();
        let uzunluk = baslik
            .split("\r\n")
            .find_map(|s| {
                s.split_once(':')
                    .filter(|(ad, _)| ad.eq_ignore_ascii_case("Content-Length"))
                    .and_then(|(_, d)| d.trim().parse::<usize>().ok())
            })
            .ok_or("Content-Length yok")?;
        if uzunluk > 8 * 1024 * 1024 {
            return Err("LSP gövdesi 8 MiB'ı aştı".into());
        }
        let mut govde = vec![0u8; uzunluk];
        self.stdout
            .read_exact(&mut govde)
            .map_err(|h| format!("dillsp gövdesi okunamadı: {h}"))?;
        String::from_utf8(govde).map_err(|_| "LSP gövdesi UTF-8 değil".into())
    }

    fn lsp_donemi(&mut self) -> Result<(), String> {
        self.surum += 1;
        self.gonder(&json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":"file:///tmp/zee-soak.dil","version":self.surum},"contentChanges":[{"text":satirli_kaynak(2_000, self.surum)}]}}).to_string())?;
        let yanit = self.cerceve_oku()?;
        if !yanit.contains("publishDiagnostics") {
            return Err("didChange tanı bildirimi üretmedi".into());
        }
        Ok(())
    }

    fn pid(&self) -> u32 {
        self.cocuk.id()
    }
}

impl Drop for LspSureci {
    fn drop(&mut self) {
        let _ = self.cocuk.kill();
        let _ = self.cocuk.wait();
    }
}

fn medyan(degerler: &mut [u64]) -> u64 {
    degerler.sort_unstable();
    degerler[degerler.len() / 2]
}

fn buyume_yuzde(ilk: u64, son: u64) -> u64 {
    if son <= ilk || ilk == 0 {
        return 0;
    }
    ((son - ilk) * 100).div_ceil(ilk)
}

/// Isınma penceresi atıldıktan sonra ilk ve son pencere medyanlarını
/// karşılaştırır; iki süreçten biri hem yüzde hem mutlak tavanı aşarsa kalır.
fn karar_ver(ornekler: &[Ornek], pencere_sn: u64) -> Result<Karar, String> {
    let pencereler = ornekler
        .iter()
        .fold(Vec::<Vec<Ornek>>::new(), |mut acc, o| {
            let sira = (o.saniye / pencere_sn) as usize;
            while acc.len() <= sira {
                acc.push(Vec::new());
            }
            acc[sira].push(*o);
            acc
        })
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();
    if pencereler.len() < ISINMA_PENCERESI + 2 {
        return Err(format!(
            "en az {} dolu pencere gerekir, {} var",
            ISINMA_PENCERESI + 2,
            pencereler.len()
        ));
    }
    let ilk = &pencereler[ISINMA_PENCERESI];
    let son = pencereler.last().expect("en az bir pencere");
    let derleyici_ilk = medyan(&mut ilk.iter().map(|o| o.derleyici_rss_kib).collect::<Vec<_>>());
    let derleyici_son = medyan(&mut son.iter().map(|o| o.derleyici_rss_kib).collect::<Vec<_>>());
    let dillsp_ilk = medyan(&mut ilk.iter().map(|o| o.dillsp_rss_kib).collect::<Vec<_>>());
    let dillsp_son = medyan(&mut son.iter().map(|o| o.dillsp_rss_kib).collect::<Vec<_>>());
    let derleyici_buyume_yuzde = buyume_yuzde(derleyici_ilk, derleyici_son);
    let dillsp_buyume_yuzde = buyume_yuzde(dillsp_ilk, dillsp_son);
    let asti = |ilk: u64, son: u64, yuzde: u64| {
        yuzde > AZAMI_BUYUME_YUZDE && son.saturating_sub(ilk) > AZAMI_BUYUME_KIB
    };
    let mut nedenler = Vec::new();
    if asti(derleyici_ilk, derleyici_son, derleyici_buyume_yuzde) {
        nedenler.push(format!(
            "derleyici RSS {derleyici_ilk}→{derleyici_son} KiB (+%{derleyici_buyume_yuzde})"
        ));
    }
    if asti(dillsp_ilk, dillsp_son, dillsp_buyume_yuzde) {
        nedenler.push(format!(
            "dillsp RSS {dillsp_ilk}→{dillsp_son} KiB (+%{dillsp_buyume_yuzde})"
        ));
    }
    Ok(Karar {
        derleyici_ilk,
        derleyici_son,
        dillsp_ilk,
        dillsp_son,
        derleyici_buyume_yuzde,
        dillsp_buyume_yuzde,
        gecti: nedenler.is_empty(),
        neden: if nedenler.is_empty() {
            "ısınma sonrası büyüme sınır içinde".into()
        } else {
            nedenler.join("; ")
        },
    })
}

fn komut_satiri(program: &str, argumanlar: &[&str]) -> Option<String> {
    let cikti = Command::new(program).args(argumanlar).output().ok()?;
    cikti
        .status
        .success()
        .then(|| String::from_utf8_lossy(&cikti.stdout).trim().to_string())
}

fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn rapor_metni(ornekler: &[Ornek], karar: &Karar, sure_sn: u64, pencere_sn: u64) -> String {
    let son = ornekler.last().copied().unwrap_or(Ornek {
        saniye: 0,
        derleyici_rss_kib: 0,
        dillsp_rss_kib: 0,
        derleme_donemi: 0,
        lsp_donemi: 0,
    });
    let mut rapor = format!(
        "# Soak raporu\n\nSüre {sure_sn} sn, pencere {pencere_sn} sn, {} örnek; derleme dönemi {}, LSP dönemi {}.\n\n| Süreç | Isınma sonrası ilk pencere medyanı (KiB) | Son pencere medyanı (KiB) | Büyüme |\n|---|---:|---:|---:|\n| derleyici (soak süreci) | {} | {} | %{} |\n| dillsp | {} | {} | %{} |\n\nSonuç: **{}** — {}\n\nSınır: ısınma sonrası büyüme hem %{AZAMI_BUYUME_YUZDE} hem {} KiB tavanını aşarsa kalır.\n",
        ornekler.len(), son.derleme_donemi, son.lsp_donemi,
        karar.derleyici_ilk, karar.derleyici_son, karar.derleyici_buyume_yuzde,
        karar.dillsp_ilk, karar.dillsp_son, karar.dillsp_buyume_yuzde,
        if karar.gecti { "GEÇTİ" } else { "KALDI" }, karar.neden, AZAMI_BUYUME_KIB
    );
    rapor.push_str(
        "\n| sn | derleyici KiB | dillsp KiB | derleme | lsp |\n|---:|---:|---:|---:|---:|\n",
    );
    for o in ornekler {
        rapor.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            o.saniye, o.derleyici_rss_kib, o.dillsp_rss_kib, o.derleme_donemi, o.lsp_donemi
        ));
    }
    rapor
}

fn gecmis_satiri(
    sha: &str,
    tarih: &str,
    rustc: &str,
    sure_sn: u64,
    pencere_sn: u64,
    son: &Ornek,
    karar: &Karar,
) -> String {
    format!(
        "{sha}\t{tarih}\t{}\t{rustc}\t{sure_sn}\t{pencere_sn}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{AZAMI_BUYUME_YUZDE}\t{AZAMI_BUYUME_KIB}\t{}\n",
        platform(), son.derleme_donemi, son.lsp_donemi,
        karar.derleyici_ilk, karar.derleyici_son, karar.dillsp_ilk, karar.dillsp_son,
        karar.derleyici_buyume_yuzde, karar.dillsp_buyume_yuzde,
        if karar.gecti { "gecti" } else { "kaldi" }
    )
}

fn gecmise_ekle(yol: &Path, satir: &str) -> Result<(), String> {
    let mut metin = std::fs::read_to_string(yol).unwrap_or_default();
    if metin.is_empty() {
        metin = format!("{GECMIS_SEMASI}\n{GECMIS_BASLIK}\n");
    } else if !metin.starts_with(GECMIS_SEMASI) {
        return Err("soak tarihçesi şeması bilinmiyor".into());
    }
    metin.push_str(satir);
    dil::kalici_dosya::atomik_yaz(yol, metin.as_bytes())
        .map_err(|h| format!("{} yazılamadı: {h}", yol.display()))
}

fn calistir() -> Result<bool, String> {
    let ayarlar = argumanlari_oku()?;
    let (golden_klasoru, kaynaklar) = golden_kaynaklari(&ayarlar.depo)?;
    let mut lsp = LspSureci::baslat(&ayarlar.dillsp)?;
    let kendi_pid = std::process::id();
    let baslangic = Instant::now();
    let mut ornekler = Vec::new();
    let mut derleme = 0u64;
    let mut lsp_donemi = 0u64;
    let mut sonraki_ornek = Duration::from_secs(5);
    eprintln!(
        "== Soak: {} sn, pencere {} sn, dillsp pid {}",
        ayarlar.sure.as_secs(),
        ayarlar.pencere.as_secs(),
        lsp.pid()
    );
    while baslangic.elapsed() < ayarlar.sure {
        derleme_donemi(&golden_klasoru, &kaynaklar)?;
        derleme += 1;
        lsp.lsp_donemi()?;
        lsp_donemi += 1;
        if baslangic.elapsed() >= sonraki_ornek {
            let ornek = Ornek {
                saniye: baslangic.elapsed().as_secs(),
                derleyici_rss_kib: rss_kib(kendi_pid).ok_or("kendi RSS okunamadı")?,
                dillsp_rss_kib: rss_kib(lsp.pid()).ok_or("dillsp RSS okunamadı")?,
                derleme_donemi: derleme,
                lsp_donemi,
            };
            eprintln!(
                "{} sn: derleyici {} KiB, dillsp {} KiB, derleme {}, lsp {}",
                ornek.saniye, ornek.derleyici_rss_kib, ornek.dillsp_rss_kib, derleme, lsp_donemi
            );
            ornekler.push(ornek);
            sonraki_ornek += Duration::from_secs(5);
        }
    }
    let karar = karar_ver(&ornekler, ayarlar.pencere.as_secs())?;
    let rapor = rapor_metni(
        &ornekler,
        &karar,
        ayarlar.sure.as_secs(),
        ayarlar.pencere.as_secs(),
    );
    if let Some(yol) = &ayarlar.rapor {
        dil::kalici_dosya::atomik_yaz(yol, rapor.as_bytes())
            .map_err(|h| format!("{} yazılamadı: {h}", yol.display()))?;
    }
    print!("{rapor}");
    if let Some(yol) = &ayarlar.gecmis {
        let kirli = komut_satiri(
            "git",
            &[
                "-C",
                ayarlar.depo.to_str().ok_or("depo yolu")?,
                "status",
                "--porcelain",
            ],
        )
        .ok_or("git status")?;
        if !kirli.is_empty() {
            return Err("soak tarihçesi kirli çalışma ağacından yazılamaz".into());
        }
        let sha = komut_satiri(
            "git",
            &[
                "-C",
                ayarlar.depo.to_str().ok_or("depo yolu")?,
                "rev-parse",
                "HEAD",
            ],
        )
        .ok_or("git sha")?;
        let tarih = komut_satiri("date", &["-u", "+%Y-%m-%d"]).ok_or("tarih")?;
        let rustc = komut_satiri("rustc", &["--version"]).unwrap_or_else(|| "rustc ?".into());
        let son = ornekler.last().ok_or("örnek yok")?;
        gecmise_ekle(
            yol,
            &gecmis_satiri(
                &sha,
                &tarih,
                &rustc,
                ayarlar.sure.as_secs(),
                ayarlar.pencere.as_secs(),
                son,
                &karar,
            ),
        )?;
    }
    Ok(karar.gecti)
}

fn main() -> ExitCode {
    match calistir() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!("HATA: soak kaynak büyümesi sınırı aştı");
            ExitCode::FAILURE
        }
        Err(hata) => {
            eprintln!("HATA: {hata}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn ornek(saniye: u64, derleyici: u64, dillsp: u64) -> Ornek {
        Ornek {
            saniye,
            derleyici_rss_kib: derleyici,
            dillsp_rss_kib: dillsp,
            derleme_donemi: saniye,
            lsp_donemi: saniye,
        }
    }

    #[test]
    fn sabit_rss_gecer_isinma_penceresi_atilir() {
        let mut ornekler = vec![ornek(5, 900_000, 900_000), ornek(30, 950_000, 950_000)];
        for sn in (65..=180).step_by(5) {
            ornekler.push(ornek(sn, 100_000, 50_000));
        }
        let karar = karar_ver(&ornekler, 60).unwrap();
        assert!(karar.gecti, "{}", karar.neden);
        assert_eq!(karar.derleyici_ilk, 100_000);
        assert_eq!(karar.derleyici_buyume_yuzde, 0);
    }

    #[test]
    fn yuzde_ve_mutlak_tavani_birlikte_asan_buyume_kalir() {
        let mut ornekler = Vec::new();
        for sn in (5..=240).step_by(5) {
            let buyume = if sn > 180 { 60_000 } else { 0 };
            ornekler.push(ornek(sn, 100_000 + buyume, 50_000));
        }
        let karar = karar_ver(&ornekler, 60).unwrap();
        assert!(!karar.gecti);
        assert!(karar.neden.contains("derleyici RSS"));
        // Yüzde aşan ama 32 MiB tavanı aşmayan küçük süreç geçer.
        let mut kucuk = Vec::new();
        for sn in (5..=240).step_by(5) {
            kucuk.push(ornek(sn, 10_000 + if sn > 180 { 5_000 } else { 0 }, 5_000));
        }
        assert!(karar_ver(&kucuk, 60).unwrap().gecti);
        assert!(karar_ver(&[ornek(5, 1, 1)], 60).is_err());
    }

    #[test]
    fn buyume_yuzdesi_ve_medyan_kararlidir() {
        assert_eq!(buyume_yuzde(100, 110), 10);
        assert_eq!(buyume_yuzde(100, 111), 11);
        assert_eq!(buyume_yuzde(100, 90), 0);
        assert_eq!(buyume_yuzde(0, 5), 0);
        assert_eq!(medyan(&mut [3, 1, 2]), 2);
        assert_eq!(medyan(&mut [4, 1, 3, 2]), 3);
    }

    #[test]
    fn tarihce_satiri_semayi_ve_provenance_alanlarini_tasir() {
        let karar = Karar {
            derleyici_ilk: 1,
            derleyici_son: 2,
            dillsp_ilk: 3,
            dillsp_son: 4,
            derleyici_buyume_yuzde: 100,
            dillsp_buyume_yuzde: 34,
            gecti: true,
            neden: String::new(),
        };
        let satir = gecmis_satiri(
            "a".repeat(40).as_str(),
            "2026-09-06",
            "rustc 1.93.1",
            1800,
            60,
            &ornek(1800, 2, 4),
            &karar,
        );
        let alanlar = satir.trim_end().split('\t').collect::<Vec<_>>();
        assert_eq!(
            alanlar.len(),
            GECMIS_BASLIK.trim_start_matches("# ").split('\t').count()
        );
        assert_eq!(alanlar[16], "gecti");
        assert!(rapor_metni(&[ornek(5, 1, 1)], &karar, 10, 5).contains("GEÇTİ"));
    }
}
