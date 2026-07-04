use std::collections::BTreeSet;
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

#[derive(Clone, Debug, serde::Deserialize)]
struct PageSample {
    id: String,
    source: String,
    category: String,
    name: String,
    description: String,
    command: String,
    display: String,
    status: String,
    preview: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegistrySample {
    id: String,
    category: String,
    name: String,
    description: String,
}

#[derive(Default)]
struct PageSampleBuilder {
    id: Option<String>,
    category: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

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
        "validate-pages" => validate_pages(),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown xtask command: {other}").into()),
    }
}

fn print_help() {
    eprintln!(
        "Commands:\n  provider-smoke-app   Build the Rust wasm provider-smoke app and export list\n  provider-smoke       Build the Rust app, build the Box3D provider with emcc, and run Node smoke\n  validate-pages       Validate the static GitHub Pages site and sample catalog"
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

fn validate_pages() -> Result<()> {
    let root = project_root();
    let pages_dir = root.join("docs").join("pages");
    let index = ensure_file(&pages_dir.join("index.html"), "Pages index")?;
    let catalog = ensure_file(
        &pages_dir.join("sample-catalog.json"),
        "Pages sample catalog",
    )?;

    let catalog_json = fs::read_to_string(&catalog)?;
    let catalog_samples: Vec<PageSample> = serde_json::from_str(&catalog_json)?;
    validate_sample_catalog(&catalog_samples)?;

    let catalog_registry_samples = catalog_samples
        .iter()
        .filter(|sample| sample.source == "testbed-scene")
        .map(PageSample::registry_projection)
        .collect::<Vec<_>>();
    let registry_samples = read_testbed_registry(&root)?;
    if catalog_registry_samples != registry_samples {
        return Err(format!(
            "docs/pages/sample-catalog.json testbed-scene entries are out of sync with bevy_boxddd/examples/testbed_3d/scenes.rs ({} catalog entries, {} registry entries)",
            catalog_registry_samples.len(),
            registry_samples.len()
        )
        .into());
    }

    let html = fs::read_to_string(index)?;
    validate_html_links(&pages_dir, &html)?;

    eprintln!(
        "Validated Pages site: {} ({} samples)",
        pages_dir.display(),
        catalog_samples.len()
    );
    Ok(())
}

fn ensure_file(path: &Path, label: &str) -> Result<PathBuf> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(format!("{label} is missing: {}", path.display()).into())
    }
}

fn validate_sample_catalog(samples: &[PageSample]) -> Result<()> {
    if samples.is_empty() {
        return Err("sample catalog must contain at least one entry".into());
    }

    let mut seen = BTreeSet::new();
    for sample in samples {
        validate_sample_field(sample, "id", &sample.id)?;
        validate_sample_field(sample, "source", &sample.source)?;
        validate_sample_field(sample, "category", &sample.category)?;
        validate_sample_field(sample, "name", &sample.name)?;
        validate_sample_field(sample, "description", &sample.description)?;
        validate_sample_field(sample, "command", &sample.command)?;
        validate_sample_field(sample, "display", &sample.display)?;
        validate_sample_field(sample, "status", &sample.status)?;
        validate_sample_field(sample, "preview", &sample.preview)?;

        if !is_slug(&sample.id) {
            return Err(format!("sample id `{}` must be a lowercase ASCII slug", sample.id).into());
        }
        if !is_slug(&sample.source) {
            return Err(format!(
                "sample `{}` source must be a lowercase ASCII slug",
                sample.id
            )
            .into());
        }
        if !is_slug(&sample.preview) {
            return Err(format!(
                "sample `{}` preview must be a lowercase ASCII slug",
                sample.id
            )
            .into());
        }
        if !seen.insert(sample.id.as_str()) {
            return Err(format!("duplicate sample id `{}`", sample.id).into());
        }
    }

    Ok(())
}

fn validate_registry_catalog(samples: &[RegistrySample]) -> Result<()> {
    if samples.is_empty() {
        return Err("testbed registry must contain at least one entry".into());
    }

    let mut seen = BTreeSet::new();
    for sample in samples {
        validate_registry_field(sample, "id", &sample.id)?;
        validate_registry_field(sample, "category", &sample.category)?;
        validate_registry_field(sample, "name", &sample.name)?;
        validate_registry_field(sample, "description", &sample.description)?;

        if !is_slug(&sample.id) {
            return Err(format!(
                "testbed registry id `{}` must be a lowercase ASCII slug",
                sample.id
            )
            .into());
        }
        if !seen.insert(sample.id.as_str()) {
            return Err(format!("duplicate testbed registry id `{}`", sample.id).into());
        }
    }

    Ok(())
}

fn validate_sample_field(sample: &PageSample, field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(format!("sample `{}` has an empty `{field}` field", sample.id).into())
    } else {
        Ok(())
    }
}

fn validate_registry_field(sample: &RegistrySample, field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(format!(
            "testbed registry sample `{}` has an empty `{field}` field",
            sample.id
        )
        .into())
    } else {
        Ok(())
    }
}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

impl PageSample {
    fn registry_projection(&self) -> RegistrySample {
        RegistrySample {
            id: self.id.clone(),
            category: self.category.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

fn read_testbed_registry(root: &Path) -> Result<Vec<RegistrySample>> {
    let scenes = root
        .join("bevy_boxddd")
        .join("examples")
        .join("testbed_3d")
        .join("scenes.rs");
    let source = fs::read_to_string(&scenes)?;
    let mut samples = Vec::new();
    let mut current: Option<PageSampleBuilder> = None;
    let mut in_registry = false;

    for line in source.lines() {
        if line.contains("pub const SCENE_REGISTRY") {
            in_registry = true;
            continue;
        }
        if !in_registry {
            continue;
        }

        let trimmed = line.trim();
        if trimmed == "];" {
            break;
        }
        if trimmed.starts_with("TestbedSceneMetadata {") {
            current = Some(PageSampleBuilder::default());
            continue;
        }
        if trimmed == "}," {
            let builder = current.take().ok_or_else(|| {
                format!(
                    "unexpected registry entry terminator in {}",
                    scenes.display()
                )
            })?;
            samples.push(builder.build()?);
            continue;
        }

        let Some(builder) = current.as_mut() else {
            continue;
        };
        if let Some(value) = extract_string_field(trimmed, "id") {
            builder.id = Some(value);
        } else if let Some(value) = extract_string_field(trimmed, "category") {
            builder.category = Some(value);
        } else if let Some(value) = extract_string_field(trimmed, "name") {
            builder.name = Some(value);
        } else if let Some(value) = extract_string_field(trimmed, "description") {
            builder.description = Some(value);
        }
    }

    validate_registry_catalog(&samples)?;
    Ok(samples)
}

impl PageSampleBuilder {
    fn build(self) -> Result<RegistrySample> {
        Ok(RegistrySample {
            id: required_registry_field(self.id, "id")?,
            category: required_registry_field(self.category, "category")?,
            name: required_registry_field(self.name, "name")?,
            description: required_registry_field(self.description, "description")?,
        })
    }
}

fn required_registry_field(value: Option<String>, field: &str) -> Result<String> {
    value.ok_or_else(|| format!("SCENE_REGISTRY entry is missing `{field}`").into())
}

fn extract_string_field(line: &str, field: &str) -> Option<String> {
    let needle = format!("{field}: \"");
    let start = line.find(&needle)? + needle.len();
    let tail = &line[start..];
    let end = tail.find('"')?;
    Some(tail[..end].to_string())
}

fn validate_html_links(pages_dir: &Path, html: &str) -> Result<()> {
    let pages_root = fs::canonicalize(pages_dir)?;
    validate_attr_links(pages_dir, &pages_root, html, "href")?;
    validate_attr_links(pages_dir, &pages_root, html, "src")?;
    validate_fetch_links(pages_dir, &pages_root, html, "\"")?;
    validate_fetch_links(pages_dir, &pages_root, html, "'")?;
    Ok(())
}

fn validate_attr_links(pages_dir: &Path, pages_root: &Path, html: &str, attr: &str) -> Result<()> {
    let needle = format!("{attr}=\"");
    let mut remainder = html;
    while let Some(index) = remainder.find(&needle) {
        let after = &remainder[index + needle.len()..];
        let end = after
            .find('"')
            .ok_or_else(|| format!("unterminated `{attr}` attribute in docs/pages/index.html"))?;
        validate_local_link(pages_dir, pages_root, &after[..end])?;
        remainder = &after[end + 1..];
    }
    Ok(())
}

fn validate_fetch_links(
    pages_dir: &Path,
    pages_root: &Path,
    html: &str,
    quote: &str,
) -> Result<()> {
    let needle = format!("fetch({quote}");
    let mut remainder = html;
    while let Some(index) = remainder.find(&needle) {
        let after = &remainder[index + needle.len()..];
        let end = after
            .find(quote)
            .ok_or("unterminated fetch() URL in docs/pages/index.html")?;
        validate_local_link(pages_dir, pages_root, &after[..end])?;
        remainder = &after[end + quote.len()..];
    }
    Ok(())
}

fn validate_local_link(pages_dir: &Path, pages_root: &Path, value: &str) -> Result<()> {
    if is_external_or_fragment(value) {
        return Ok(());
    }

    let local = strip_url_suffix(value);
    if local.is_empty() {
        return Ok(());
    }

    let target = pages_dir.join(local);
    if !target.exists() {
        return Err(format!("docs/pages/index.html links missing local asset `{value}`").into());
    }

    let canonical = fs::canonicalize(&target)?;
    if !canonical.starts_with(pages_root) {
        return Err(format!("docs/pages/index.html link escapes docs/pages: `{value}`").into());
    }

    Ok(())
}

fn is_external_or_fragment(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("mailto:")
        || value.starts_with('#')
        || value.starts_with("javascript:")
}

fn strip_url_suffix(value: &str) -> &str {
    let query = value.find('?').unwrap_or(value.len());
    let fragment = value.find('#').unwrap_or(value.len());
    &value[..query.min(fragment)]
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
