use super::*;

#[test]
fn baglanti_urlsi_cozulur() {
    let ayar = EpostaAyari::ayristir(
        "smtps://info%40example.invalid:gi%3Azli@mail.example.invalid:465?from=info@example.invalid&name=Site+E",
    )
    .unwrap();
    assert!(ayar.implicit_tls);
    assert_eq!(ayar.kapi, 465);
    assert_eq!(ayar.kullanici, "info@example.invalid");
    assert_eq!(ayar.parola, "gi:zli");
    assert_eq!(ayar.gonderen_adi, "Site E");
    let duz = EpostaAyari::ayristir("smtp://u:p@127.0.0.1:2525?from=a@b.c").unwrap();
    assert!(!duz.implicit_tls);
    assert_eq!(duz.kapi, 2525);
    assert!(EpostaAyari::ayristir("http://u:p@x?from=a@b.c").is_err());
    assert!(
        EpostaAyari::ayristir("smtps://u:p@x").is_err(),
        "from zorunlu"
    );
}

#[test]
fn ileti_rfc2047_ve_base64() {
    let ayar = EpostaAyari::ayristir(
        "smtps://u:p@mail.example.invalid?from=info@example.invalid&name=Site+E",
    )
    .unwrap();
    let ileti = ileti_kur(
        &ayar,
        &Ileti {
            alici: "ali@ornek.com",
            konu: "Yeni mesaj: Şalça",
            govde: "Merhaba\r\n.nokta",
            kimlik: "abc",
            tarih: "Tue, 09 Sep 2026 12:00:00 +0000",
        },
    );
    assert!(ileti.contains("Subject: =?UTF-8?B?"));
    assert!(ileti.contains("Content-Transfer-Encoding: base64"));
    assert!(ileti.contains("Message-ID: <abc@example.invalid>"));
    assert!(!ileti.contains("Şalça"));
    assert!(adres_gecerli("a.b@c.com"));
    assert!(!adres_gecerli("a b@c.com"));
    assert!(!adres_gecerli("a@c"));
    assert!(!adres_gecerli("<a@c.com>"));
}
struct ReadSayaci<R> {
    ic: R,
    sayac: std::rc::Rc<OkumaSayaci>,
    parca: usize,
    hata_sonra: Option<usize>,
}

#[derive(Default)]
struct OkumaSayaci {
    bayt: std::cell::Cell<usize>,
    cagri: std::cell::Cell<usize>,
    en_buyuk_talep: std::cell::Cell<usize>,
}

impl<R: Read> Read for ReadSayaci<R> {
    fn read(&mut self, tampon: &mut [u8]) -> std::io::Result<usize> {
        self.sayac.cagri.set(self.sayac.cagri.get() + 1);
        self.sayac
            .en_buyuk_talep
            .set(self.sayac.en_buyuk_talep.get().max(tampon.len()));
        let mut boyut = tampon.len().min(self.parca);
        if let Some(sinir) = self.hata_sonra {
            if self.sayac.bayt.get() >= sinir {
                return Err(std::io::ErrorKind::TimedOut.into());
            }
            boyut = boyut.min(sinir - self.sayac.bayt.get());
        }
        let okunan = self.ic.read(&mut tampon[..boyut])?;
        self.sayac.bayt.set(self.sayac.bayt.get() + okunan);
        Ok(okunan)
    }
}

impl<R> Write for ReadSayaci<R> {
    fn write(&mut self, baytlar: &[u8]) -> std::io::Result<usize> {
        Ok(baytlar.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn sayilan_oturum<R: Read + 'static>(
    ic: R,
    kapasite: usize,
    parca: usize,
    hata_sonra: Option<usize>,
) -> (Oturum, std::rc::Rc<OkumaSayaci>) {
    let sayac = std::rc::Rc::new(OkumaSayaci::default());
    let akis = ReadSayaci {
        ic,
        sayac: sayac.clone(),
        parca,
        hata_sonra,
    };
    (
        Oturum {
            okuyucu: BufReader::with_capacity(kapasite, Box::new(akis)),
        },
        sayac,
    )
}

fn tasma_sinirini_dogrula(oturum: &mut Oturum, sayac: &OkumaSayaci) {
    assert_eq!(
        oturum.yanit().unwrap_err(),
        "SMTP yanıtı 16 KiB sınırını aşıyor"
    );
    // Transport BufReader kapasitesi kadar ön okuyabilir. Take yalnız
    // 16385 baytı tüketir; socket'e dönen talep/ön okuma ayrıca sınırlıdır.
    assert_eq!(
        sayac.bayt.get() - oturum.okuyucu.buffer().len(),
        16 * 1024 + 1
    );
    assert!(sayac.bayt.get() <= 16 * 1024 + oturum.okuyucu.capacity());
    assert!(sayac.en_buyuk_talep.get() <= oturum.okuyucu.capacity());
}

#[test]
fn smtp_yanit_lfsiz_mebibayt_ve_bitmeyen_akista_sinirda_durur() {
    for kapasite in [1, 7, 8192] {
        for satir_sonu in [false, true] {
            let mut veri = vec![b'x'; 1024 * 1024];
            veri[..4].copy_from_slice(b"220 ");
            if satir_sonu {
                veri.push(b'\n');
            }
            let (mut oturum, sayac) =
                sayilan_oturum(std::io::Cursor::new(veri), kapasite, usize::MAX, None);
            tasma_sinirini_dogrula(&mut oturum, &sayac);
            assert!(sayac.bayt.get() < 1024 * 1024, "EOF veya LF beklenmedi");
        }
        let (mut oturum, sayac) = sayilan_oturum(std::io::repeat(b'x'), kapasite, usize::MAX, None);
        tasma_sinirini_dogrula(&mut oturum, &sayac);
    }
}

#[test]
fn smtp_yanit_tam_sinir_lf_ve_eof_baytlarini_aynen_korur() {
    for boyut in [16 * 1024 - 1, 16 * 1024] {
        for lf in [false, true] {
            let mut veri = vec![b'x'; boyut];
            veri[..4].copy_from_slice(b"220 ");
            if lf {
                veri[boyut - 1] = b'\n';
            }
            let (mut oturum, sayac) =
                sayilan_oturum(std::io::Cursor::new(veri.clone()), 7, 3, None);
            let (kod, yanit) = oturum.yanit().unwrap();
            assert_eq!(kod, 220);
            assert_eq!(yanit.as_bytes(), veri);
            assert_eq!(
                yanit.capacity(),
                16 * 1024 + 1,
                "tek yanıt tamponu büyümedi"
            );
            assert_eq!(sayac.bayt.get(), boyut);
        }
    }
}

#[test]
fn smtp_yanit_cok_satir_toplamini_ve_ardisik_yanitlari_korur() {
    let ilk = "250-merhaba\r\n250-özellik\r\n250 tamam\r\n";
    let ikinci = "221 kapat\r\n";
    let (mut oturum, _) = sayilan_oturum(
        std::io::Cursor::new(format!("{ilk}{ikinci}").into_bytes()),
        8192,
        usize::MAX,
        None,
    );
    assert_eq!(oturum.yanit().unwrap(), (250, ilk.into()));
    assert_eq!(oturum.yanit().unwrap(), (221, ikinci.into()));
    for tasma in [false, true] {
        let mut veri = b"250-devam\n".to_vec();
        let ikinci_baslangic = veri.len();
        veri.extend_from_slice(b"250 ");
        veri.resize(16 * 1024 + usize::from(tasma), b'x');
        let son = veri.len() - 1;
        veri[son] = b'\n';
        let (mut oturum, sayac) = sayilan_oturum(std::io::Cursor::new(veri.clone()), 17, 2, None);
        if tasma {
            tasma_sinirini_dogrula(&mut oturum, &sayac);
        } else {
            let (kod, yanit) = oturum.yanit().unwrap();
            assert_eq!(kod, 250);
            assert_eq!(yanit.as_bytes(), veri);
            assert!(yanit.as_bytes()[ikinci_baslangic + 3] == b' ');
        }
    }
    let veri = "250-devam\n".repeat(3000);
    let (mut oturum, sayac) = sayilan_oturum(std::io::Cursor::new(veri.into_bytes()), 19, 1, None);
    tasma_sinirini_dogrula(&mut oturum, &sayac);
}

#[test]
fn smtp_yanit_parcali_utf8_ve_tasma_kesigini_dogru_tanilar() {
    let veri = "220 şalça😀\r\n".as_bytes().to_vec();
    let (mut oturum, _) = sayilan_oturum(std::io::Cursor::new(veri.clone()), 3, 1, None);
    assert_eq!(oturum.yanit().unwrap().1.as_bytes(), veri);
    // UTF-8 karakteri limitte ikiye bölünse de taşma önce tanılanır.
    let mut veri = b"220 ".to_vec();
    veri.resize(16 * 1024 - 1, b'x');
    veri.extend_from_slice("😀\n".as_bytes());
    let (mut oturum, sayac) = sayilan_oturum(std::io::Cursor::new(veri), 7, 1, None);
    tasma_sinirini_dogrula(&mut oturum, &sayac);
    let (mut oturum, sayac) =
        sayilan_oturum(std::io::Cursor::new(b"220 \xff\n".to_vec()), 1, 1, None);
    assert!(oturum
        .yanit()
        .unwrap_err()
        .starts_with("SMTP yanıtı okunamadı:"));
    assert_eq!(sayac.bayt.get(), 6);
}

#[test]
fn smtp_yanit_tek_bayt_sel_timeout_ve_bos_eof_buyumeden_durur() {
    // Gecikme yerine her transport read'inde tek bayt: LF/EOF vermeyen
    // yavaş selin algoritmik okuma sınırı, zamanlamaya bağlı test yok.
    let (mut oturum, sayac) = sayilan_oturum(std::io::repeat(b'x'), 8192, 1, None);
    tasma_sinirini_dogrula(&mut oturum, &sayac);
    assert_eq!(sayac.cagri.get(), 16 * 1024 + 1);
    let (mut oturum, sayac) = sayilan_oturum(std::io::repeat(b'x'), 8192, 1, Some(100));
    assert!(oturum
        .yanit()
        .unwrap_err()
        .starts_with("SMTP yanıtı okunamadı:"));
    assert_eq!(sayac.bayt.get(), 100);
    let (mut oturum, sayac) = sayilan_oturum(std::io::empty(), 8192, 1, None);
    assert_eq!(
        oturum.yanit().unwrap_err(),
        "SMTP sunucusu bağlantıyı kapattı"
    );
    assert_eq!(sayac.bayt.get(), 0);
}
