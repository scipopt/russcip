extern crate bindgen;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use glob::glob;

fn _build_from_scip_dir(path: String) -> bindgen::Builder {
    let lib_dir = PathBuf::from(&path).join("lib");
    let lib_dir_path = lib_dir.to_str().unwrap();

    if lib_dir.exists() {
        println!("cargo:warning=Using SCIP from {}", lib_dir_path);
        println!("cargo:rustc-link-search={}", lib_dir_path);
    } else {
        panic!(
            "{}",
            format!(
                "{}/lib does not exist, please check your SCIP installation",
                path
            )
        );
    }
    println!("cargo:rustc-link-search={}", lib_dir_path);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir_path);


    let include_dir = PathBuf::from(&path).join("include");
    let include_dir_path = include_dir.to_str().unwrap();
    let scip_header_file = PathBuf::from(&path)
        .join("include")
        .join("scip")
        .join("scip.h")
        .to_str()
        .unwrap()
        .to_owned();
    let scipdefplugins_header_file = PathBuf::from(&path)
        .join("include")
        .join("scip")
        .join("scipdefplugins.h")
        .to_str()
        .unwrap()
        .to_owned();

    bindgen::Builder::default()
        .header(scip_header_file)
        .header(scipdefplugins_header_file)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg(format!("-I{}", include_dir_path))
}

fn lib_scip_in_dir(path: &str) -> bool {
    glob(&format!("{}/lib/libscip*", path)).unwrap().count() > 0
}

fn main() -> Result<(), Box<dyn Error>> {
    if std::env::var("DOCS_RS").is_ok() {
        return Ok(()); // Don't build on docs.rs
    }

    let env_vars = vec![
        "SCIPOPTDIR",
        "CONDA_PREFIX",
    ];
    let mut builder = bindgen::Builder::default();
    let mut found_scip = false;
    for env_var_name in env_vars {
        println!("cargo:rerun-if-env-changed={}", env_var_name);
        let env_var = env::var(env_var_name);
        if let Ok(scip_dir) = env_var {
            println!("cargo:warning=Looking for SCIP in {}", scip_dir);
            if lib_scip_in_dir(&scip_dir) {
                builder = _build_from_scip_dir(scip_dir);
                found_scip = true;
                break;
            } else {
                println!("cargo:warning=SCIP was not found in {}", scip_dir);
            }
            
        } else {
            println!("cargo:warning={} is not set", env_var_name);
        }
    }
    
    if !found_scip {
        println!("cargo:warning=SCIP was not found in SCIPOPTDIR or in Conda environemnt, looking for SCIP in system libraries");
        let scip_header_file = "scip-wrapper.h";
        let scipdefplugins_header_file = "scipdefplugins-wrapper.h";
        builder = bindgen::Builder::default()
            .header(scip_header_file)
            .header(scipdefplugins_header_file)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    }

    println!("cargo:rustc-link-lib=scip");

    let builder = builder
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    let bindings = builder.generate()?;
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))?;

    Ok(())
}
