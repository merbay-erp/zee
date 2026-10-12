#!/usr/bin/env bash
# K-199: gerçek Cargo, poisoned compiler/wrapper/env/config altında exact pini kullanır.
set -euo pipefail
depo_koku="$(cd "$(dirname "$0")/../.." && pwd)"
gecici="$(mktemp -d "${TMPDIR:-/tmp}/zee-rustc-pin.XXXXXX")"
trap 'rm -rf "$gecici"' EXIT
mkdir -p "$gecici/proje/src" "$gecici/cargo-home"
export CARGO_HOME="$gecici/cargo-home" CARGO_TARGET_DIR="$gecici/target"
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*)
        CARGO_HOME="$(cygpath -m "$CARGO_HOME")"
        CARGO_TARGET_DIR="$(cygpath -m "$CARGO_TARGET_DIR")"
        ;;
esac
# Config ve environment aynı anda yanlış compiler/wrapper seçer.
cat > "$gecici/cargo-home/config.toml" <<'CONFIG'
[build]
rustc = "zee-missing-rustc"
rustc-wrapper = "zee-missing-wrapper"
rustc-workspace-wrapper = "zee-missing-workspace-wrapper"
CONFIG
export RUSTC=zee-caller-rustc RUSTC_WRAPPER=zee-caller-wrapper RUSTC_WORKSPACE_WRAPPER=zee-caller-workspace-wrapper
export CARGO_BUILD_RUSTC=zee-config-rustc CARGO_BUILD_RUSTC_WRAPPER=zee-config-wrapper CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=zee-config-workspace-wrapper
source "$depo_koku/scripts/rust-arac-zinciri.sh"
zee_rust_arac_zinciri "$depo_koku/compiler/rust-toolchain.toml"
cat > "$gecici/proje/Cargo.toml" <<'TOML'
[package]
name = "zee_exact_pin_probe"
version = "0.0.0"
edition = "2021"
[workspace]
TOML
cat > "$gecici/proje/build.rs" <<'RUST'
fn main() {
    let rustc = std::env::var("RUSTC").unwrap();
    let output = std::process::Command::new(&rustc).arg("--version").output().unwrap();
    assert!(output.status.success());
    let version = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=ZEE_PIN_RUSTC_VERSION={}", version.trim());
}
RUST
cat > "$gecici/proje/src/main.rs" <<'RUST'
fn main() { println!("{}", env!("ZEE_PIN_RUSTC_VERSION")); }
RUST
manifest="$gecici/proje/Cargo.toml"
cargo generate-lockfile --offline --manifest-path "$manifest"
RUSTFLAGS="" CARGO_ENCODED_RUSTFLAGS="" cargo build --locked --offline --verbose --manifest-path "$manifest" > "$gecici/build.log" 2>&1
uzanti=""
[[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]] && uzanti=.exe
"$gecici/target/debug/zee_exact_pin_probe$uzanti" > "$gecici/actual-version"
"$RUSTC" --version > "$gecici/expected-version"
cmp "$gecici/actual-version" "$gecici/expected-version"
python_komutu=python
command -v python3 >/dev/null 2>&1 && python_komutu=python3
"$python_komutu" - "$gecici" "${ZEE_PIN_RAPOR:-$gecici/rapor.json}" "$RUSTC" "$depo_koku" <<'PY'
from pathlib import Path
import hashlib,json,os,platform,subprocess,sys
root,report,compiler,repo=map(Path,sys.argv[1:])
data={'sema':'zee-rustc-pin-prova-1','sonuc':'gecti','prova_git_sha':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
      'calisma_agaci_temiz':not subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True).strip(),
      'prova_araci_sha256':hashlib.sha256((repo/'scripts/tests/rust-arac-zinciri-gercek-test.sh').read_bytes()).hexdigest(),
      'helper_sha256':hashlib.sha256((repo/'scripts/rust-arac-zinciri.sh').read_bytes()).hexdigest(),
      'os':platform.platform(),'mimari':platform.machine(),'rustc_path':str(compiler),'rustc_sha256':hashlib.sha256(compiler.read_bytes()).hexdigest(),
      'pin_sha256':hashlib.sha256((repo/'compiler/rust-toolchain.toml').read_bytes()).hexdigest(),
      'rustc_version':(root/'actual-version').read_text().strip(),'cargo_version':subprocess.check_output(['cargo','--version'],text=True).strip(),
      'compiler_env':os.environ['RUSTC'],'wrapper':os.environ['RUSTC_WRAPPER'],'workspace_wrapper':os.environ['RUSTC_WORKSPACE_WRAPPER'],
      'compiler_config_poisoned':True,'build_log':(root/'build.log').read_text(encoding='utf-8'),
      'sinir':'Gerçek küçük Cargo/build-script/ikili derlemesi; tam Zee release veya tüm Cargo profil/linker config kaydı değildir.'}
report.parent.mkdir(parents=True,exist_ok=True)
report.write_bytes((json.dumps(data,ensure_ascii=False,indent=2)+'\n').encode('utf-8'))
PY
echo 'Exact Rust compiler/env/config probe: passed'
