use std::path::Path;
use std::{env, fs};

fn main() {
    let script_path = env::var("KUBER_SCRIPT_PATH").unwrap_or("src/script.sh".to_string());
    let out_dir = env::var("OUT_DIR").expect("OUTDIR not set");
    let dest_path = Path::new(&out_dir).join("script.sh");
    let script = fs::read(script_path).expect("invalid kuber script path");

    fs::write(&dest_path, script).expect("unable to write to OUTDIR");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=KUBER_SCRIPT_PATH");
}
