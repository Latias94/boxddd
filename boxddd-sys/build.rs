use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WasmMode {
    CompileOnly,
    Source,
    Provider,
}

#[derive(Debug)]
struct BuildConfig {
    manifest_dir: PathBuf,
    #[cfg_attr(not(feature = "bindgen"), allow(dead_code))]
    out_dir: PathBuf,
    target_env: String,
    target_os: String,
    profile: String,
    is_docsrs: bool,
    skip_cc: bool,
    force_bindgen: bool,
    wasm_mode: Option<WasmMode>,
}

impl BuildConfig {
    fn from_env() -> Self {
        let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
        let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        let is_docsrs = env::var("DOCS_RS").is_ok() || env::var("CARGO_CFG_DOCSRS").is_ok();
        let skip_cc = parse_bool_env("BOXDDD_SYS_SKIP_CC");
        let force_bindgen = parse_bool_env("BOXDDD_SYS_FORCE_BINDGEN");
        let wasm_mode = (target_arch == "wasm32").then(|| {
            env::var("BOXDDD_SYS_WASM_MODE")
                .ok()
                .map(|value| parse_wasm_mode(&value))
                .unwrap_or_else(|| default_wasm_mode(&target_os))
        });

        Self {
            manifest_dir: PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()),
            out_dir: PathBuf::from(env::var("OUT_DIR").unwrap()),
            target_env,
            target_os,
            profile: env::var("PROFILE").unwrap_or_else(|_| "release".into()),
            is_docsrs,
            skip_cc,
            force_bindgen,
            wasm_mode,
        }
    }

    fn is_debug(&self) -> bool {
        self.profile == "debug"
    }

    fn pregenerated_bindings(&self) -> PathBuf {
        if cfg!(feature = "double-precision") {
            self.manifest_dir
                .join("src")
                .join("bindings_pregenerated_double.rs")
        } else {
            self.manifest_dir
                .join("src")
                .join("bindings_pregenerated.rs")
        }
    }
}

fn parse_bool_env(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(
            v.as_str(),
            "1" | "true" | "yes" | "on" | "TRUE" | "YES" | "ON"
        ),
        Err(_) => false,
    }
}

fn parse_wasm_mode(value: &str) -> WasmMode {
    match value {
        "compile-only" | "compile_only" | "check" => WasmMode::CompileOnly,
        "source" | "c-backed" | "c_backed" | "wasi" => WasmMode::Source,
        "provider" | "import-provider" | "import_provider" => WasmMode::Provider,
        other => panic!(
            "unsupported BOXDDD_SYS_WASM_MODE={other:?}; expected compile-only, source, or provider"
        ),
    }
}

fn default_wasm_mode(target_os: &str) -> WasmMode {
    if target_os == "wasi" {
        WasmMode::Source
    } else {
        WasmMode::CompileOnly
    }
}

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_pregenerated)");
    println!("cargo:rustc-check-cfg=cfg(force_bindgen)");
    println!("cargo:rustc-check-cfg=cfg(boxddd_sys_wasm_provider)");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=third-party/box3d/include/box3d/box3d.h");
    println!("cargo:rerun-if-changed=third-party/box3d");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_SKIP_CC");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_FORCE_BINDGEN");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_WASM_MODE");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_LINK_LIB");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_LINK_SEARCH");
    println!("cargo:rerun-if-env-changed=WASI_SYSROOT");
    println!("cargo:rerun-if-env-changed=WASI_SDK_PATH");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=CARGO_CFG_DOCSRS");

    let config = BuildConfig::from_env();
    let pregenerated = config.pregenerated_bindings();
    let has_pregenerated = pregenerated.exists();
    if config.force_bindgen {
        println!("cargo:rustc-cfg=force_bindgen");
    } else if has_pregenerated {
        println!("cargo:rustc-cfg=has_pregenerated");
    }
    if config.wasm_mode == Some(WasmMode::Provider) {
        println!("cargo:rustc-cfg=boxddd_sys_wasm_provider");
        if config.force_bindgen {
            panic!(
                "BOXDDD_SYS_WASM_MODE=provider cannot be combined with BOXDDD_SYS_FORCE_BINDGEN=1 yet"
            );
        }
        if !has_pregenerated {
            panic!("BOXDDD_SYS_WASM_MODE=provider requires checked-in pregenerated bindings");
        }
        generate_wasm_provider_bindings(&pregenerated, &config.out_dir);
    }

    if config.force_bindgen || (!has_pregenerated && !config.is_docsrs) {
        #[cfg(feature = "bindgen")]
        generate_bindings(&config.manifest_dir, &config.out_dir);
        #[cfg(not(feature = "bindgen"))]
        {
            if config.force_bindgen {
                panic!("BOXDDD_SYS_FORCE_BINDGEN=1 requires the `bindgen` feature");
            }
            panic!(
                "pregenerated Box3D bindings are missing for the selected ABI mode; enable `bindgen` or refresh checked-in bindings"
            );
        }
    }

    if config.is_docsrs || config.skip_cc {
        if config.is_docsrs {
            println!("cargo:warning=DOCS_RS detected: skipping native Box3D C build");
        } else {
            println!("cargo:warning=Skipping native Box3D C build due to BOXDDD_SYS_SKIP_CC");
        }
        return;
    }

    if handle_wasm_build(&config) {
        return;
    }

    if !cfg!(feature = "build-from-source") {
        emit_external_link_directives();
        println!(
            "cargo:warning=build-from-source disabled: not compiling vendored Box3D C sources"
        );
        return;
    }

    build_box3d_from_source(&config);
}

fn handle_wasm_build(config: &BuildConfig) -> bool {
    let Some(mode) = config.wasm_mode else {
        return false;
    };

    match mode {
        WasmMode::CompileOnly => {
            println!(
                "cargo:warning=boxddd-sys is using compile-only WASM mode; Box3D C sources are not linked"
            );
            true
        }
        WasmMode::Provider => {
            println!(
                "cargo:warning=boxddd-sys WASM provider mode is opt-in scaffold only; Box3D symbols are expected from the provider module"
            );
            true
        }
        WasmMode::Source => {
            if config.target_os != "wasi" {
                panic!(
                    "BOXDDD_SYS_WASM_MODE=source currently supports wasm32-wasip1 only; use provider mode for wasm32-unknown-unknown browser builds"
                );
            }
            if !cfg!(feature = "build-from-source") {
                panic!(
                    "BOXDDD_SYS_WASM_MODE=source requires the default `build-from-source` feature"
                );
            }
            build_box3d_from_source(config);
            true
        }
    }
}

fn emit_external_link_directives() {
    if let Ok(path) = env::var("BOXDDD_SYS_LINK_SEARCH") {
        if !path.is_empty() {
            println!("cargo:rustc-link-search=native={path}");
        }
    }

    let lib = env::var("BOXDDD_SYS_LINK_LIB").unwrap_or_else(|_| "box3d".into());
    if !lib.is_empty() {
        println!("cargo:rustc-link-lib={lib}");
    }
}

fn generate_wasm_provider_bindings(pregenerated: &Path, out_dir: &Path) {
    const IMPORT_MODULE: &str = "box3d-sys-v0";
    let source = fs::read_to_string(pregenerated).unwrap_or_else(|err| {
        panic!(
            "failed to read pregenerated bindings at {}: {err}",
            pregenerated.display()
        )
    });
    let rewritten = source.replace(
        "unsafe extern \"C\" {",
        &format!("#[link(wasm_import_module = \"{IMPORT_MODULE}\")]\nunsafe extern \"C\" {{"),
    );
    if rewritten == source {
        panic!(
            "failed to generate WASM provider bindings from {}; no extern blocks were found",
            pregenerated.display()
        );
    }
    fs::write(out_dir.join("wasm_provider_bindings.rs"), rewritten)
        .expect("failed to write WASM provider bindings");
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
        .clang_args(double_precision_clang_args())
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

#[cfg(feature = "bindgen")]
fn double_precision_clang_args() -> Vec<&'static str> {
    if cfg!(feature = "double-precision") {
        vec!["-DBOX3D_DOUBLE_PRECISION"]
    } else {
        Vec::new()
    }
}

#[cfg(not(feature = "bindgen"))]
#[allow(dead_code)]
fn generate_bindings(_manifest_dir: &Path, _out_dir: &Path) {
    unreachable!("generate_bindings is only available with the `bindgen` feature enabled");
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

fn build_box3d_from_source(config: &BuildConfig) {
    let box3d_root = config.manifest_dir.join("third-party").join("box3d");
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

    if config.target_env == "msvc" {
        let use_static_crt = env::var("CARGO_CFG_TARGET_FEATURE")
            .unwrap_or_default()
            .split(',')
            .any(|feature| feature == "crt-static");
        build.static_crt(use_static_crt);
        build.debug(config.is_debug());
        build.opt_level(if config.is_debug() { 0 } else { 2 });
        add_msvc_c_standard_flag(&mut build);
    } else {
        build.flag_if_supported("-std=c17");
        build.flag_if_supported("-ffp-contract=off");
        build.debug(config.is_debug());
        build.opt_level(if config.is_debug() { 0 } else { 2 });
        if config.target_os == "linux" {
            build.define("_POSIX_C_SOURCE", Some("199309L"));
            build.flag_if_supported("-pthread");
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=m");
        }
        if config.wasm_mode == Some(WasmMode::Source) {
            build.define("BOX3D_WASM_SINGLE_THREADED", None);
            configure_wasi_sysroot(&mut build);
        }
    }

    if cfg!(feature = "disable-simd") {
        build.define("BOX3D_DISABLE_SIMD", None);
    }
    if cfg!(feature = "validate") {
        build.define("BOX3D_VALIDATE", None);
    }
    if cfg!(feature = "double-precision") {
        build.define("BOX3D_DOUBLE_PRECISION", None);
    }

    build.compile("box3d");
}

fn configure_wasi_sysroot(build: &mut cc::Build) {
    let sysroot = env::var_os("WASI_SYSROOT")
        .map(PathBuf::from)
        .or_else(|| env::var_os("WASI_SDK_PATH").map(|path| PathBuf::from(path).join("share/wasi-sysroot")))
        .unwrap_or_else(|| {
            panic!(
                "wasm32-wasip1 source builds require WASI_SYSROOT or WASI_SDK_PATH so clang can find WASI libc headers"
            )
        });

    let has_libc_headers = sysroot.join("include").join("math.h").exists()
        || sysroot
            .join("include")
            .join("wasm32-wasi")
            .join("math.h")
            .exists();
    if !has_libc_headers {
        panic!(
            "WASI sysroot at {} does not contain WASI libc headers",
            sysroot.display()
        );
    }

    build.flag(format!("--sysroot={}", sysroot.display()));
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
