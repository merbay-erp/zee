//! K-227/C3: mevcut local paket ve rota birimiyle anonim katalog kur/kaldır.

use dil::paket::ProjeGrafigi;
use dil::yorumlayici::{calistir_io, ToplayanIo};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static SIRA: AtomicU64 = AtomicU64::new(0);
const KOK: &str = "../dogfood/zee-cekirdek/modul-kompozisyonu";
const MODULSUZ: &str =
    include_str!("../../dogfood/zee-cekirdek/modul-kompozisyonu/site/ana-modulsuz.dil");
const MODULSUZ_BILDIRIM: &str = "# K-227: anonim modülsüz site kabulü.\nproje \"modulkabulu\" olsun\nsürüm \"0.1.0\" olsun\nmorfoloji \"zee-tr-1\" olsun\ngiriş \"ana.dil\" olsun\nyetkinlikler \"ağ-sunucusu\" listesi olsun\nağ_hedefleri boş liste olsun\nyerel_bağımlılıklar boş liste olsun\n";

struct KabulKopyasi(PathBuf);

impl KabulKopyasi {
    fn yeni() -> Self {
        let yol = std::env::temp_dir().join(format!(
            "zee-k227-kabul-{}-{}",
            std::process::id(),
            SIRA.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&yol).unwrap();
        let kaynak = Path::new(env!("CARGO_MANIFEST_DIR")).join(KOK);
        for klasor in ["site", "katalog"] {
            std::fs::create_dir(yol.join(klasor)).unwrap();
            for dosya in std::fs::read_dir(kaynak.join(klasor)).unwrap() {
                let dosya = dosya.unwrap();
                if dosya.path().extension().and_then(|s| s.to_str()) == Some("dil") {
                    std::fs::copy(dosya.path(), yol.join(klasor).join(dosya.file_name())).unwrap();
                }
            }
        }
        Self(yol)
    }

    fn site(&self) -> PathBuf {
        self.0.join("site")
    }

    fn komut(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dil"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .expect("mevcut paket CLI")
    }

    fn basarili(&self, args: &[&str]) -> Output {
        let sonuc = self.komut(args);
        assert!(
            sonuc.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&sonuc.stderr)
        );
        sonuc
    }

    fn istekler(&self) -> ToplayanIo {
        let grafik = ProjeGrafigi::cozumle(&self.site()).expect("paket grafiği");
        grafik.kilidi_denetle().expect("exact kilit");
        let giris = grafik.ana_giris().unwrap();
        let program =
            dil::kaynagi_derle_kokenlerle(&giris.kaynak, Some(&giris.koken), &mut |istek| {
                grafik.yukle(istek)
            })
            .expect("kökenli paket/rota kompozisyonu");
        dil::cozumleyici::yetkinlikleri_denetle(
            &program,
            &grafik.ana_bildirim().yetkinlik_politikasi(),
        )
        .expect("host politikası aşılmamalı");
        let mut io = ToplayanIo::yeni(Vec::new());
        io.istekler = ["GET /", "GET /katalog"].map(String::from).into();
        calistir_io(&program, &mut io).expect("hermetik GET kabulü");
        io
    }

    fn kutuphane_kaynaklari(&self) -> Vec<(String, Vec<u8>)> {
        let mut kaynaklar = std::fs::read_dir(self.0.join("katalog"))
            .unwrap()
            .map(|dosya| {
                let dosya = dosya.unwrap();
                (
                    dosya.file_name().to_string_lossy().into_owned(),
                    std::fs::read(dosya.path()).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        kaynaklar.sort();
        kaynaklar
    }
}

impl Drop for KabulKopyasi {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("yalnız geçici kabul kopyası silinmeli");
    }
}

#[test]
fn katalog_paketi_ve_rota_birimi_metin_yamasi_olmadan_kurulur_kaldirilir() {
    let kopya = KabulKopyasi::yeni();
    let kutuphane = kopya.kutuphane_kaynaklari();
    let ana = std::fs::read(kopya.site().join("ana.dil")).unwrap();
    let adapter = std::fs::read(kopya.site().join("katalog_rotalari.dil")).unwrap();
    std::fs::write(kopya.site().join("proje.dil"), MODULSUZ_BILDIRIM).unwrap();
    std::fs::write(kopya.site().join("ana.dil"), MODULSUZ).unwrap();
    std::fs::remove_file(kopya.site().join("katalog_rotalari.dil")).unwrap();
    kopya.basarili(&["kilitle", "site"]);
    kopya.basarili(&["denetle", "site"]);
    kopya.basarili(&["dene", "site"]);
    let baslangic = kopya.istekler();
    assert_eq!(baslangic.sunucu_durumlari, [200, 404]);
    assert_eq!(baslangic.sunucu_yanitlari[0].1, "Anonim çekirdek");
    // Kurulum önceden gözden geçirilmiş bütün kaynakları seçer; başka
    // modüllerin ortak satırlarını aramaz veya değiştirmez.
    std::fs::write(kopya.site().join("ana.dil"), &ana).unwrap();
    std::fs::write(kopya.site().join("katalog_rotalari.dil"), &adapter).unwrap();
    kopya.basarili(&["ekle", "katalog", "site"]);
    let paketler = kopya.basarili(&["paketler", "site"]);
    assert!(String::from_utf8_lossy(&paketler.stdout).contains("katalog"));
    kopya.basarili(&["denetle", "site"]);
    kopya.basarili(&["dene", "site"]);
    assert_eq!(std::fs::read(kopya.site().join("ana.dil")).unwrap(), ana);
    assert_eq!(
        std::fs::read(kopya.site().join("katalog_rotalari.dil")).unwrap(),
        adapter
    );
    let io = kopya.istekler();
    assert_eq!(io.sunucu_durumlari, [200, 200]);
    assert_eq!(io.sunucu_yanitlari[0].1, "Anonim çekirdek");
    assert_eq!(io.sunucu_yanitlari[1].1, "Anonim katalog");
    let bildirim = std::fs::read(kopya.site().join("proje.dil")).unwrap();
    let kilit = std::fs::read(kopya.site().join("proje.kilit")).unwrap();
    let red = kopya.komut(&["çıkar", "katalog", "site"]);
    assert!(!red.status.success());
    assert!(String::from_utf8_lossy(&red.stderr).contains("P010"));
    assert_eq!(
        std::fs::read(kopya.site().join("proje.dil")).unwrap(),
        bildirim
    );
    assert_eq!(
        std::fs::read(kopya.site().join("proje.kilit")).unwrap(),
        kilit
    );

    // Tam, önceden gözden geçirilmiş giriş seçilir. Satır arama, anchor veya
    // başka modülün kaynağına replace uygulanmaz; yalnız bu modülün adapter'ı gider.
    std::fs::write(kopya.site().join("ana.dil"), MODULSUZ).unwrap();
    std::fs::remove_file(kopya.site().join("katalog_rotalari.dil")).unwrap();
    kopya.basarili(&["kilitle", "site"]);
    kopya.basarili(&["çıkar", "katalog", "site"]);
    kopya.basarili(&["denetle", "site"]);
    kopya.basarili(&["dene", "site"]);
    let grafik = ProjeGrafigi::cozumle(&kopya.site()).unwrap();
    assert_eq!(grafik.paket_sayisi(), 0);
    let io = kopya.istekler();
    assert_eq!(io.sunucu_durumlari, [200, 404]);
    assert_eq!(io.sunucu_yanitlari[0].1, "Anonim çekirdek");
    assert_eq!(kopya.kutuphane_kaynaklari(), kutuphane);
}

#[test]
fn eksik_ve_bayat_katalog_paketi_sessiz_kirilma_yerine_reddedilir() {
    let kopya = KabulKopyasi::yeni();
    kopya.basarili(&["kilitle", "site"]);
    let kutuphane = kopya.0.join("katalog/ana.dil");
    let eski = std::fs::read(&kutuphane).unwrap();
    std::fs::write(&kutuphane, [eski.as_slice(), b"\n# yeni kaynak\n"].concat()).unwrap();
    let bayat = kopya.komut(&["denetle", "site"]);
    assert!(!bayat.status.success());
    assert!(String::from_utf8_lossy(&bayat.stderr).contains("P008"));
    std::fs::write(&kutuphane, eski).unwrap();
    kopya.basarili(&["denetle", "site"]);
    std::fs::remove_dir_all(kopya.0.join("katalog")).unwrap();
    let eksik = kopya.komut(&["denetle", "site"]);
    assert!(!eksik.status.success());
    let tani = String::from_utf8_lossy(&eksik.stderr);
    assert!(tani.contains("P006"), "{tani}");
}
