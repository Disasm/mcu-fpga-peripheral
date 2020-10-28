use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::env;
use svd2ral::{generate, AddressSize};

const SVD_FILE: &str = "soc.svd";

fn main() {
    let xml = &mut String::new();
    File::open(SVD_FILE).unwrap().read_to_string(xml).unwrap();

    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    generate(&xml, crate_dir.join("src"), AddressSize::U32, &["IDENTIFIER_MEM"]).unwrap();

    println!("cargo:rerun-if-changed={}", SVD_FILE);
    println!("cargo:rerun-if-env-changed=FORCE");
}
