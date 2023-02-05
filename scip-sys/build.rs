extern crate bindgen;

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=SCIPOPTDIR");
    let scip_dir = env::var("SCIPOPTDIR");

    match &scip_dir {
        Ok(scip_dir) => {
            println!(
                "cargo:warning=SCIPOPTDIR was defined, using SCIP from {}",
                scip_dir
            );
            let lib_dir = PathBuf::from(&scip_dir).join("lib");
            let lib_dir_path = lib_dir.to_str().unwrap();

            if lib_dir.exists() {
                println!("cargo:warning=Using SCIP from {}", lib_dir_path);
                println!("cargo:rustc-link-search={}", lib_dir_path);
            } else {
                panic!(
                    "{}",
                    format!(
                        "{}/lib does not exist, please check your SCIP installation",
                        scip_dir
                    )
                );
            }
            println!("cargo:rustc-link-search={}", lib_dir_path);
            println!("cargo:rustc-link-lib=scip");
        }
        Err(_) => {
            println!(
                "cargo:warning=SCIPOPTDIR was not defined, looking for SCIP in system libraries"
            );
            println!("cargo:rustc-link-lib=scip");
        }
    };

    let scip_header_file = &scip_dir.clone().map_or("scip-wrapper.h".to_owned(), |scip_dir| { 
        PathBuf::from(scip_dir)
        .join("include")
        .join("scip")
        .join("scip.h")
        .to_str().unwrap().to_owned()
    });

    let scipdefplugins_header_file = scip_dir.map_or("scipdefplugins-wrapper.h".to_owned(), |scip_dir| { 
        PathBuf::from(scip_dir)
        .join("include")
        .join("scip")
        .join("scipdefplugins.h")
        .to_str().unwrap().to_owned()
    });

    let bindings = bindgen::Builder::default()
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .header(scip_header_file)
        .header(scipdefplugins_header_file)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()?;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
