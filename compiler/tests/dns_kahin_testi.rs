//! K-390 (ADR-169): `dns_teli` ayrıştırıcısının bağımsız kâhinle diferansiyel
//! testi. `hickory-proto` yalnız dev-dependency'dir. Sözleşme: Zee'nin kabul
//! ettiği her paketi hickory de kabul eder ve cevap bölümünü aynı okur
//! (Zee kabul kümesi ⊆ hickory kabul kümesi). Tohumlar elle kurulmuş geçerli
//! yanıtlardır; bozucu deterministik xorshift ile bayt değiştirir, keser,
//! dilim çoğaltır ve sayaç alanlarını oynatır.

use dil::dns_teli;
use hickory_proto::op::Message;

#[path = "ortak/dns_kahini.rs"]
mod dns_kahini;
use dns_kahini::karsilastir;

fn ad_yaz(p: &mut Vec<u8>, ad: &str) {
    for etiket in ad.split('.').filter(|e| !e.is_empty()) {
        p.push(etiket.len() as u8);
        p.extend_from_slice(etiket.as_bytes());
    }
    p.push(0);
}

fn baslik(kimlik: u16, bayrak: u16, an: u16, ns: u16, ar: u16) -> Vec<u8> {
    let mut p = Vec::new();
    for alan in [kimlik, bayrak, 1, an, ns, ar] {
        p.extend_from_slice(&alan.to_be_bytes());
    }
    p
}

fn soru(p: &mut Vec<u8>, ad: &str, tur: u16) {
    ad_yaz(p, ad);
    p.extend_from_slice(&tur.to_be_bytes());
    p.extend_from_slice(&1u16.to_be_bytes());
}

fn kayit(p: &mut Vec<u8>, ad: &[u8], tur: u16, sinif: u16, ttl: u32, veri: &[u8]) {
    p.extend_from_slice(ad);
    p.extend_from_slice(&tur.to_be_bytes());
    p.extend_from_slice(&sinif.to_be_bytes());
    p.extend_from_slice(&ttl.to_be_bytes());
    p.extend_from_slice(&(veri.len() as u16).to_be_bytes());
    p.extend_from_slice(veri);
}

fn tohumlar() -> Vec<Vec<u8>> {
    let mut hepsi = Vec::new();
    // MX + sıkıştırma + ek bölümde A ve OPT.
    let mut p = baslik(0x1234, 0x81A0, 2, 0, 2);
    soru(&mut p, "Ornek.test", 15);
    kayit(&mut p, &[0xC0, 0x0C], 15, 1, 300, b"\x00\x0a\x02mx\xc0\x0c");
    kayit(
        &mut p,
        &[0xC0, 0x0C],
        15,
        1,
        300,
        b"\x00\x14\x03mx2\xc0\x0c",
    );
    kayit(&mut p, b"\x02mx\xc0\x0c", 1, 1, 60, &[192, 0, 2, 1]);
    kayit(&mut p, &[0], 41, 1232, 0, &[0, 10, 0, 2, 0xAA, 0xBB]);
    hepsi.push(p);
    // TXT çok parçalı.
    let mut p = baslik(7, 0x8180, 2, 0, 0);
    soru(&mut p, "ornek.test", 16);
    kayit(
        &mut p,
        &[0xC0, 0x0C],
        16,
        1,
        30,
        b"\x06v=spf1\x0a include:x\x05 -all",
    );
    kayit(&mut p, &[0xC0, 0x0C], 16, 1, 30, b"\x00\x03abc");
    hepsi.push(p);
    // CNAME zinciri + AAAA.
    let mut p = baslik(9, 0x8180, 2, 0, 0);
    soru(&mut p, "www.ornek.test", 28);
    kayit(&mut p, &[0xC0, 0x0C], 5, 1, 5, b"\x03web\xc0\x10");
    let mut aaaa = vec![0x20, 0x01, 0x0d, 0xb8];
    aaaa.extend_from_slice(&[0; 11]);
    aaaa.push(1);
    kayit(&mut p, b"\x03web\xc0\x10", 28, 1, 5, &aaaa);
    hepsi.push(p);
    // NXDOMAIN + yetki bölümünde SOA.
    let mut p = baslik(11, 0x8183, 0, 1, 0);
    soru(&mut p, "yok.ornek.test", 1);
    let mut soa = b"\x02ns\xc0\x10\x0ahostmaster\xc0\x10".to_vec();
    soa.extend_from_slice(&[
        0, 0, 0, 1, 0, 0, 0x0e, 0x10, 0, 0, 0x07, 0x08, 0, 0x09, 0x3a, 0x80, 0, 0, 0, 60,
    ]);
    kayit(&mut p, &[0xC0, 0x10], 6, 1, 60, &soa);
    hepsi.push(p);
    // TLSA + NS yetki.
    let mut p = baslik(13, 0x81A0, 1, 1, 0);
    soru(&mut p, "_25._tcp.mx.ornek.test", 52);
    let mut tlsa = vec![3, 1, 1];
    tlsa.extend_from_slice(&[0x5a; 32]);
    kayit(&mut p, &[0xC0, 0x0C], 52, 1, 3600, &tlsa);
    kayit(
        &mut p,
        b"\x05ornek\x04test\x00",
        2,
        1,
        3600,
        b"\x02ns\x05ornek\x04test\x00",
    );
    hepsi.push(p);
    hepsi
}

struct Xorshift(u64);
impl Xorshift {
    fn sonraki(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn aralik(&mut self, ust: usize) -> usize {
        (self.sonraki() % ust.max(1) as u64) as usize
    }
}

fn boz(tohum: &[u8], r: &mut Xorshift) -> Vec<u8> {
    let mut p = tohum.to_vec();
    for _ in 0..=r.aralik(3) {
        match r.aralik(6) {
            0 | 1 if !p.is_empty() => {
                let i = r.aralik(p.len());
                p[i] = r.sonraki() as u8;
            }
            2 => {
                let son = r.aralik(p.len() + 1);
                p.truncate(son);
            }
            3 if !p.is_empty() => {
                let a = r.aralik(p.len());
                let b = (a + 1 + r.aralik(8)).min(p.len());
                let dilim = p[a..b].to_vec();
                let i = r.aralik(p.len() + 1);
                p.splice(i..i, dilim);
            }
            4 if p.len() >= 12 => {
                let alan = 4 + 2 * r.aralik(4);
                p[alan] = 0;
                p[alan + 1] = r.aralik(4) as u8;
            }
            _ if p.len() > 12 => {
                // Sıkıştırma işaretçisi ek.
                let i = 12 + r.aralik(p.len() - 12);
                p[i] = 0xC0 | (r.sonraki() as u8 & 0x01);
            }
            _ => {}
        }
    }
    p
}

#[test]
fn tohumlar_iki_ayristiricida_ayni_okunur() {
    for tohum in tohumlar() {
        assert!(
            karsilastir(&tohum),
            "tohum Zee'de kabul edilmeli: {tohum:02x?}"
        );
    }
}

#[test]
fn bozulmus_paketlerde_zee_kabul_kumesi_kahinin_alt_kumesidir() {
    let tohumlar = tohumlar();
    let mut r = Xorshift(0x9E37_79B9_7F4A_7C15);
    let (mut kabul, mut ret) = (0usize, 0usize);
    for i in 0..40_000 {
        let paket = boz(&tohumlar[i % tohumlar.len()], &mut r);
        if karsilastir(&paket) {
            kabul += 1;
        } else {
            ret += 1;
        }
    }
    // Bozucu iki yönü de gerçekten sınamalı.
    assert!(kabul > 1_000, "kabul edilen bozulmuş paket az: {kabul}");
    assert!(ret > 10_000, "reddedilen bozulmuş paket az: {ret}");
}

#[test]
fn zee_reddettigi_ama_kahinin_kabul_ettigi_sinifler_bilincli() {
    // Bilinmeyen kayıt türü (HINFO): hickory kabul eder, Zee fail-closed reddeder.
    let mut p = baslik(1, 0x8180, 1, 0, 0);
    soru(&mut p, "a.test", 1);
    kayit(&mut p, &[0xC0, 0x0C], 13, 1, 1, b"\x01x\x01y");
    assert!(Message::from_vec(&p).is_ok());
    assert!(dns_teli::yanit_ayristir(&p).is_err());
    // Sondaki fazla bayt.
    let mut p = baslik(1, 0x8180, 0, 0, 0);
    soru(&mut p, "a.test", 1);
    p.push(0);
    assert!(dns_teli::yanit_ayristir(&p).is_err());
}

#[test]
fn kalici_fuzz_korpusu_kahinle_uyusur() {
    let dizin = concat!(env!("CARGO_MANIFEST_DIR"), "/fuzz/corpus/dns_yaniti");
    let mut sayi = 0;
    for giris in std::fs::read_dir(dizin).expect("korpus") {
        let paket = std::fs::read(giris.unwrap().path()).unwrap();
        karsilastir(&paket);
        sayi += 1;
    }
    assert!(sayi >= 6, "korpus tohumu eksik: {sayi}");
}
