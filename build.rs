// build.rs

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("exp_table.rs");

    let values = (0..256).map(|n| {
        let val = 1.0 - (-9.0 * ((n as f64) / 255.0)).exp();
        (val * 255.0) as u8
    });

    fs::write(
        &dest_path,
        format!(
            "const ONE_MINUS_EXP_MINUS_X_TABLE: [u8; 256] = [{}];",
            values.map(|n| n.to_string()).collect::<Vec<_>>().join(",\n")
        ),
    )
    .unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
