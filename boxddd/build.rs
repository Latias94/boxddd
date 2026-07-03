use std::env;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(boxddd_wasm_provider)");
    println!("cargo:rerun-if-env-changed=BOXDDD_SYS_WASM_MODE");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32"
        && env::var("BOXDDD_SYS_WASM_MODE")
            .ok()
            .is_some_and(|mode| is_provider_mode(&mode))
    {
        println!("cargo:rustc-cfg=boxddd_wasm_provider");
    }
}

fn is_provider_mode(mode: &str) -> bool {
    matches!(mode, "provider" | "import-provider" | "import_provider")
}
