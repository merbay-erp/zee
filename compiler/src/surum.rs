//! K-226: internal build record shared by CLI, LSP and worker logging.
//! This is not a supported api::v1 export.

pub const SURUM: &str = env!("CARGO_PKG_VERSION");
pub const KAYNAK_SHA: &str = env!("ZEE_BUILD_SHA");
pub const KAYNAK_TARIHI: &str = env!("ZEE_BUILD_DATE");
pub const KANAL: &str = env!("ZEE_BUILD_CHANNEL");
pub const KIRLI: &str = env!("ZEE_BUILD_DIRTY");

pub fn metin() -> String {
    let sha = KAYNAK_SHA.get(..12).unwrap_or(KAYNAK_SHA);
    let kirli = if KIRLI == "true" {
        ", kirli kaynak"
    } else {
        ""
    };
    format!("{SURUM} ({sha}, {KAYNAK_TARIHI}, {KANAL}{kirli})")
}

pub fn json() -> &'static str {
    // All strings are build-validated ASCII; version comes from Cargo.
    concat!(
        "{\"schema\":\"zee-build-v1\",\"version\":\"",
        env!("CARGO_PKG_VERSION"),
        "\",\"source_sha\":\"",
        env!("ZEE_BUILD_SHA"),
        "\",\"source_date\":\"",
        env!("ZEE_BUILD_DATE"),
        "\",\"channel\":\"",
        env!("ZEE_BUILD_CHANNEL"),
        "\",\"dirty\":",
        env!("ZEE_BUILD_DIRTY"),
        "}"
    )
}
