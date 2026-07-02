use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn parse_bool_env(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(
            v.as_str(),
            "1" | "true" | "yes" | "on" | "TRUE" | "YES" | "ON"
        ),
        Err(_) => false,
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=third-party/box3d/include/box3d/box3d.h");
    println!("cargo:rerun-if-changed=third-party/box3d");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_SKIP_CC");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_FORCE_BINDGEN");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_DOCSRS");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "release".into());
    let is_debug = profile == "debug";
    let is_docsrs = env::var("DOCS_RS").is_ok() || env::var("CARGO_CFG_DOCSRS").is_ok();

    if parse_bool_env("BOXDDD_SYS_FORCE_BINDGEN") {
        #[cfg(feature = "bindgen")]
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        #[cfg(feature = "bindgen")]
        generate_bindings(&manifest_dir, &out_dir);
        #[cfg(not(feature = "bindgen"))]
        panic!("BOXDDD_SYS_FORCE_BINDGEN=1 requires the `bindgen` feature");
    }

    if is_docsrs || parse_bool_env("BOXDDD_SYS_SKIP_CC") {
        return;
    }

    if target_arch == "wasm32" {
        println!("cargo:warning=boxddd-sys does not build Box3D C sources for wasm32 yet");
        return;
    }

    build_box3d_from_source(&manifest_dir, &target_env, &target_os, is_debug);
}

#[cfg(feature = "bindgen")]
fn generate_bindings(manifest_dir: &Path, out_dir: &Path) {
    let include_root = manifest_dir
        .join("third-party")
        .join("box3d")
        .join("include");
    let header = include_root.join("box3d").join("box3d.h");
    let bindings = bindgen::Builder::default()
        .header(header.to_string_lossy())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(["-x", "c", "-std=c17"])
        .clang_arg(format!("-I{}", include_root.display()))
        .allowlist_function("b3.*")
        .allowlist_type("b3.*")
        .allowlist_var("B3_.*")
        .layout_tests(false)
        .generate()
        .expect("failed to generate Box3D bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write Box3D bindings");
}

fn add_msvc_c_standard_flag(build: &mut cc::Build) {
    match build.is_flag_supported("/std:c17") {
        Ok(true) => {
            build.flag("/std:c17");
        }
        Ok(false) | Err(_) => {
            build.flag_if_supported("/std:c11");
        }
    }
}

fn build_box3d_from_source(manifest_dir: &Path, target_env: &str, target_os: &str, is_debug: bool) {
    let box3d_root = manifest_dir.join("third-party").join("box3d");
    let box3d_include = box3d_root.join("include");
    let box3d_src = box3d_root.join("src");

    let mut build = cc::Build::new();
    build.include(&box3d_include);
    build.include(&box3d_src);

    let mut files = Vec::new();
    collect_c_files(&box3d_src, &mut files);
    for file in files {
        build.file(file);
    }

    if target_env == "msvc" {
        let use_static_crt = env::var("CARGO_CFG_TARGET_FEATURE")
            .unwrap_or_default()
            .split(',')
            .any(|feature| feature == "crt-static");
        build.static_crt(use_static_crt);
        build.debug(is_debug);
        build.opt_level(if is_debug { 0 } else { 2 });
        add_msvc_c_standard_flag(&mut build);
    } else {
        build.flag_if_supported("-std=c17");
        build.flag_if_supported("-ffp-contract=off");
        build.debug(is_debug);
        build.opt_level(if is_debug { 0 } else { 2 });
        if target_os == "linux" {
            build.define("_POSIX_C_SOURCE", Some("199309L"));
            build.flag_if_supported("-pthread");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
        }
    }

    if cfg!(feature = "disable-simd") {
        build.define("BOX3D_DISABLE_SIMD", None);
    }
    if cfg!(feature = "validate") {
        build.define("BOX3D_VALIDATE", None);
    }

    build.compile("box3d");
}

fn collect_c_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_c_files(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "c") {
                out.push(path);
            }
        }
    }
}
