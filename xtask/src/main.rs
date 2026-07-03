use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type DynError = Box<dyn Error>;
type Result<T> = std::result::Result<T, DynError>;

const PROVIDER_MODULE: &str = "box3d-sys-v0";
const TARGET: &str = "wasm32-unknown-unknown";
const SMOKE_PACKAGE: &str = "boxddd-provider-smoke";
const SMOKE_WASM: &str = "boxddd_provider_smoke.wasm";

fn main() {
    if let Err(err) = run() {
        eprintln!("xtask error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());
    match command.as_str() {
        "provider-smoke-app" => {
            let app = build_provider_smoke_app()?;
            let imports = collect_provider_imports(&app)?;
            write_exports_json(&provider_smoke_dir(), &imports)?;
            eprintln!(
                "Provider smoke app ready: {} ({} provider imports)",
                app.display(),
                imports.len()
            );
            Ok(())
        }
        "provider-smoke" => run_provider_smoke(),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown xtask command: {other}").into()),
    }
}

fn print_help() {
    eprintln!(
        "Commands:\n  provider-smoke-app   Build the Rust wasm provider-smoke app and export list\n  provider-smoke       Build the Rust app, build the Box3D provider with emcc, and run Node smoke"
    );
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask should live under the workspace root")
        .to_path_buf()
}

fn provider_smoke_dir() -> PathBuf {
    project_root().join("target").join("boxddd-provider-smoke")
}

fn build_provider_smoke_app() -> Result<PathBuf> {
    let root = project_root();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("-p")
        .arg(SMOKE_PACKAGE)
        .arg("--target")
        .arg(TARGET)
        .env("BOXDDD_SYS_WASM_MODE", "provider")
        .env("RUSTFLAGS", provider_rustflags());
    run_command(&mut command, "build provider-smoke Rust wasm")?;

    let wasm = root
        .join("target")
        .join(TARGET)
        .join(profile)
        .join(SMOKE_WASM);
    if !wasm.exists() {
        return Err(format!("provider-smoke wasm artifact not found: {}", wasm.display()).into());
    }

    let out_dir = provider_smoke_dir();
    fs::create_dir_all(&out_dir)?;
    fs::copy(&wasm, out_dir.join(SMOKE_WASM))?;
    Ok(wasm)
}

fn provider_rustflags() -> OsString {
    let mut flags = env::var_os("RUSTFLAGS").unwrap_or_default();
    if !flags.is_empty() {
        flags.push(" ");
    }
    flags.push("-C link-arg=--import-memory -C link-arg=--export=boxddd_provider_smoke");
    flags
}

fn collect_provider_imports(wasm: &Path) -> Result<Vec<String>> {
    ensure_tool(
        "node",
        "--version",
        "Node.js is required for provider smoke",
    )?;
    let script = r#"
const fs = require('node:fs');
const wasmPath = process.argv[1];
const providerModule = process.argv[2];
const module = new WebAssembly.Module(fs.readFileSync(wasmPath));
const names = WebAssembly.Module.imports(module)
  .filter((i) => i.kind === 'function' && i.module === providerModule)
  .map((i) => i.name)
  .sort();
for (const name of names) console.log(name);
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(script)
        .arg(wasm)
        .arg(PROVIDER_MODULE)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "failed to inspect wasm imports with node: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let imports = String::from_utf8(output.stdout)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if imports.is_empty() {
        return Err(format!(
            "{} does not import any functions from {PROVIDER_MODULE}",
            wasm.display()
        )
        .into());
    }
    Ok(imports)
}

fn write_exports_json(out_dir: &Path, imports: &[String]) -> Result<PathBuf> {
    fs::create_dir_all(out_dir)?;
    let mut exported = imports
        .iter()
        .map(|name| format!("\"_{name}\""))
        .collect::<Vec<_>>();
    exported.sort();
    let path = out_dir.join("box3d-provider-exports.json");
    fs::write(&path, format!("[{}]", exported.join(",")))?;
    Ok(path)
}

fn run_provider_smoke() -> Result<()> {
    let app_wasm = build_provider_smoke_app()?;
    let imports = collect_provider_imports(&app_wasm)?;
    let out_dir = provider_smoke_dir();
    let exports = write_exports_json(&out_dir, &imports)?;
    let provider = build_box3d_provider(&out_dir, &exports)?;
    let app_copy = out_dir.join(SMOKE_WASM);
    write_node_runner(&out_dir, &provider, &app_copy, &imports)?;

    let runner = out_dir.join("run-provider-smoke.mjs");
    let mut command = Command::new("node");
    command.arg(runner);
    run_command(&mut command, "run provider shared-memory smoke")?;
    Ok(())
}

fn build_box3d_provider(out_dir: &Path, exports_json: &Path) -> Result<PathBuf> {
    let emcc = find_emcc()?;
    let root = project_root();
    let box3d_root = root.join("boxddd-sys").join("third-party").join("box3d");
    let include_dir = box3d_root.join("include");
    let src_dir = box3d_root.join("src");
    let provider = out_dir.join("box3d-sys-v0.mjs");

    let mut c_files = Vec::new();
    collect_c_files(&src_dir, &mut c_files)?;
    c_files.sort();

    let mut command = Command::new(emcc);
    command
        .arg("-std=c17")
        .arg("-O2")
        .arg("-s")
        .arg("MODULARIZE=1")
        .arg("-s")
        .arg("EXPORT_ES6=1")
        .arg("-s")
        .arg("ENVIRONMENT=node,web")
        .arg("-s")
        .arg("GLOBAL_BASE=67108864")
        .arg("-s")
        .arg("IMPORTED_MEMORY=1")
        .arg("-s")
        .arg("ALLOW_MEMORY_GROWTH=1")
        .arg("-s")
        .arg("INITIAL_MEMORY=134217728")
        .arg("-s")
        .arg("MAXIMUM_MEMORY=268435456")
        .arg("-s")
        .arg("FILESYSTEM=0")
        .arg("-s")
        .arg("NO_EXIT_RUNTIME=1")
        .arg("-s")
        .arg("MALLOC=emmalloc")
        .arg("-s")
        .arg("ASSERTIONS=1")
        .arg("-s")
        .arg("STACK_SIZE=1048576")
        .arg("-s")
        .arg("ERROR_ON_UNDEFINED_SYMBOLS=1")
        .arg("-s")
        .arg(format!(
            "EXPORTED_FUNCTIONS=@{}",
            exports_json.to_string_lossy().replace('\\', "/")
        ))
        .arg("-DBOX3D_DISABLE_SIMD")
        .arg("-DBOX3D_WASM_SINGLE_THREADED")
        .arg("-I")
        .arg(&include_dir)
        .arg("-I")
        .arg(&src_dir);
    for file in c_files {
        command.arg(file);
    }
    command.arg("-o").arg(&provider);
    run_command(&mut command, "build Box3D provider wasm")?;
    Ok(provider)
}

fn collect_c_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_c_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "c") {
            out.push(path);
        }
    }
    Ok(())
}

fn write_node_runner(
    out_dir: &Path,
    provider: &Path,
    app_wasm: &Path,
    imports: &[String],
) -> Result<()> {
    let provider_name = provider
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid provider file name")?;
    let app_name = app_wasm
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("invalid app wasm file name")?;
    let imports_array = imports
        .iter()
        .map(|name| format!("  \"{name}\""))
        .collect::<Vec<_>>()
        .join(",\n");
    let runner = format!(
        r#"import fs from 'node:fs';
import {{ dirname, join }} from 'node:path';
import {{ fileURLToPath }} from 'node:url';
import createProvider from './{provider_name}';

const here = dirname(fileURLToPath(import.meta.url));
const memory = new WebAssembly.Memory({{ initial: 2048, maximum: 4096 }});
const provider = await createProvider({{
  wasmMemory: memory,
  locateFile: (path) => join(here, path),
  print: (text) => console.log(`[box3d-sys-v0] ${{text}}`),
  printErr: (text) => console.warn(`[box3d-sys-v0] ${{text}}`),
}});

if (provider.wasmMemory && provider.wasmMemory !== memory) {{
  throw new Error('provider did not use the shared WebAssembly.Memory');
}}

const providerImports = [
{imports_array}
];
const importObject = {{
  env: {{ memory }},
  '{PROVIDER_MODULE}': {{}},
}};

for (const name of providerImports) {{
  const exported = provider[`_${{name}}`] || provider[name];
  if (typeof exported !== 'function') {{
    throw new Error(`provider is missing export for ${{name}}`);
  }}
  importObject['{PROVIDER_MODULE}'][name] = exported;
}}

const appBytes = fs.readFileSync(join(here, '{app_name}'));
const {{ instance }} = await WebAssembly.instantiate(appBytes, importObject);
if (typeof instance.exports.boxddd_provider_smoke !== 'function') {{
  throw new Error('boxddd_provider_smoke export is missing from Rust wasm');
}}

const code = instance.exports.boxddd_provider_smoke();
if (code !== 0) {{
  throw new Error(`boxddd provider smoke failed with code ${{code}}`);
}}

console.log('boxddd provider smoke passed');
"#
    );
    fs::write(out_dir.join("run-provider-smoke.mjs"), runner)?;
    Ok(())
}

fn find_emcc() -> Result<PathBuf> {
    if let Some(path) = runnable_tool("emcc", "--version") {
        return Ok(path);
    }

    if let Ok(root) = env::var("EMSDK") {
        let emscripten = PathBuf::from(root).join("upstream").join("emscripten");
        let emcc = if cfg!(windows) {
            emscripten.join("emcc.bat")
        } else {
            emscripten.join("emcc")
        };
        if emcc.exists() {
            return Ok(emcc);
        }
    }

    Err(
        "failed to locate emcc; install emsdk, run emsdk_env, or set EMSDK to the emsdk root"
            .into(),
    )
}

fn ensure_tool(name: &str, arg: &str, message: &str) -> Result<()> {
    if runnable_tool(name, arg).is_some() {
        Ok(())
    } else {
        Err(format!("{message}: `{name} {arg}` failed").into())
    }
}

fn runnable_tool(name: &str, arg: &str) -> Option<PathBuf> {
    Command::new(name)
        .arg(arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .filter(|status| status.success())
        .map(|_| PathBuf::from(name))
}

fn run_command(command: &mut Command, label: &str) -> Result<()> {
    eprintln!("running {label}: {command:?}");
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with status {status}").into())
    }
}
