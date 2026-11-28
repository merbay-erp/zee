//! RFC-0047 (ADR-182) DAV istemci yüzeyi kapıları: uygulama parolası üretimi
//! `kriptografi` ister; trait yöntemlerini aynı politika ile açar.
macro_rules! yontemler {
    () => {
        fn uygulama_parolasi_uret(
            &mut self,
        ) -> Result<crate::guvenlik::uygulama_parolasi::UretilenParola, String> {
            self.politika.gerektir(Yetkinlik::Kriptografi)?;
            self.ic.uygulama_parolasi_uret()
        }
    };
}
pub(super) use yontemler;
