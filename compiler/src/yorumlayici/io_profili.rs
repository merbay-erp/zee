//! Deterministik IO dünyasının sürümlü rastgelelik çekirdeği (K-116).

/// Rastgelelik, sanal saat ve hermetik adaptör davranışının normatif profil
/// kimliği. Kırıcı değişiklik yeni bir profil adı gerektirir.
pub const DETERMINISTIK_IO_PROFILI: &str = "zee-io-1";

const TOHUM_KARISTIRICI: u64 = 0x5EED_2EE5;
const XORSHIFT64_CARPANI: u64 = 0x2545_F491_4F6C_DD1D;

/// `zee-io-1` profilinin taşınabilir, kriptografik olmayan üreteci.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurumluRastgele {
    durum: u64,
}

impl SurumluRastgele {
    /// Görünür tohumu profilin başlangıç durumuna dönüştürür. Sıfır dahil
    /// bütün `u64` tohumlar geçerlidir.
    pub fn yeni(tohum: u64) -> Self {
        Self {
            durum: tohum ^ TOHUM_KARISTIRICI,
        }
    }

    fn siradaki_u64(&mut self) -> u64 {
        // Her adımda tek durumlu xorshift64*; `| 1`, sıfır kilit durumunu
        // kaldırır ve eski playground dizisini profil-1'de korur.
        let mut x = self.durum | 1;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.durum = x;
        x.wrapping_mul(XORSHIFT64_CARPANI)
    }

    /// Uçları dahil `[alt, üst]` aralığında yansız bir değer üretir. Ters
    /// aralık normal dil hattında C006'dır; doğrudan adaptör çağrısında güvenli
    /// biçimde alt uca döner. Tam `i64` uzayı taşmadan desteklenir.
    pub fn aralikta(&mut self, alt: i64, ust: i64) -> i64 {
        if alt > ust {
            return alt;
        }
        let genislik = (ust as i128 - alt as i128 + 1) as u128;
        let ofset = if genislik == (1u128 << 64) {
            self.siradaki_u64() as u128
        } else {
            let genislik = genislik as u64;
            let esik = genislik.wrapping_neg() % genislik;
            loop {
                let aday = self.siradaki_u64();
                if aday >= esik {
                    break (aday % genislik) as u128;
                }
            }
        };
        (alt as i128 + ofset as i128) as i64
    }
}
