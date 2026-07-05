use std::collections::BTreeSet;
use std::env;
use std::error::Error;
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
const PAGES_WASM_DIR: &str = "wasm/generated";
const BEVY_EXAMPLES_DIR: &str = "examples";
const BEVY_WEB_EXAMPLE: &str = "testbed_3d";
const BEVY_WEB_OUT_DIR: &str = "bevy-testbed/generated";
const BEVY_WEB_OUT_NAME: &str = "bevy_boxddd_testbed";
const BEVY_WEB_JS: &str = "bevy_boxddd_testbed.js";
const BEVY_WEB_WASM: &str = "bevy_boxddd_testbed_bg.wasm";
const BEVY_PROVIDER_SHIM: &str = "box3d-provider-shim.js";
const SAMPLE_MATRIX_PATH: &str = "docs/upstream-parity/box3d-sample-matrix.md";
const SAMPLE_CASE_TABLE_HEADER: &str =
    "| Category | Official sample | Source location | Parity mode | Target | Notes |";
const SAMPLE_PARITY_MODES: &[&str] = &[
    "FaithfulPort",
    "TeachingAdaptation",
    "TestOnly",
    "Deferred",
    "UpstreamReference",
];
const PROVIDER_SMOKE_EXPORTS: &[&str] = &[
    "boxddd_provider_smoke",
    "boxddd_provider_drop_millimeters",
    "boxddd_provider_ray_hit_millimeters",
    "boxddd_provider_shape_cast_permyriad",
    "boxddd_provider_joint_error_millimeters",
];
const DEBUG_BRIDGE_EXPORTS: &[&str] = &[
    "boxddd_debug_report_error",
    "boxddd_debug_shape_create",
    "boxddd_debug_shape_destroy",
    "boxddd_debug_draw_shape",
    "boxddd_debug_draw_segment",
    "boxddd_debug_draw_transform",
    "boxddd_debug_draw_point",
    "boxddd_debug_draw_sphere",
    "boxddd_debug_draw_capsule",
    "boxddd_debug_draw_bounds",
    "boxddd_debug_draw_box",
    "boxddd_debug_draw_string",
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegistrySample {
    id: String,
    category: String,
    name: String,
    description: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct OfficialSample {
    category: String,
    name: String,
    source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SampleParityRow {
    category: String,
    name: String,
    source: String,
    mode: String,
    target: String,
    notes: String,
    line_number: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum BuildProfile {
    Debug,
    Release,
}

#[derive(Debug)]
struct BevyWebArtifacts {
    out_dir: PathBuf,
    imports: Vec<String>,
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
        "build-pages-wasm" => build_pages_wasm(),
        "generate-pages" => generate_pages(),
        "validate-pages" => validate_pages(),
        "sample-parity" => sample_parity(args.collect()),
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown xtask command: {other}").into()),
    }
}

fn print_help() {
    eprintln!(
        "Commands:\n  provider-smoke-app   Build the Rust wasm provider-smoke app and export list\n  provider-smoke       Build the Rust app, build the Box3D provider with emcc, and run Node smoke\n  build-pages-wasm     Build Bevy example WASM artifacts into docs/pages/wasm/generated and docs/pages/bevy-testbed/generated\n  generate-pages       Generate static Bevy example entry pages from the Rust scene registry\n  validate-pages       Validate the static GitHub Pages site\n  sample-parity        Validate the official Box3D sample parity matrix"
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

fn pages_wasm_generated_dir() -> PathBuf {
    project_root()
        .join("docs")
        .join("pages")
        .join(PAGES_WASM_DIR)
}

fn pages_bevy_generated_dir() -> PathBuf {
    project_root()
        .join("docs")
        .join("pages")
        .join(BEVY_WEB_OUT_DIR)
}

fn pages_bevy_examples_dir() -> PathBuf {
    project_root()
        .join("docs")
        .join("pages")
        .join(BEVY_EXAMPLES_DIR)
}

fn sample_parity(args: Vec<String>) -> Result<()> {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("Usage: cargo run -p xtask -- sample-parity [--check]");
        return Ok(());
    }
    if !args.is_empty() && args != ["--check"] {
        return Err(format!("unknown sample-parity arguments: {}", args.join(" ")).into());
    }

    let root = project_root();
    let official_samples = read_official_samples(&root)?;
    let parity_rows = read_sample_parity_rows(&root)?;
    validate_sample_parity(&root, &official_samples, &parity_rows)?;

    eprintln!(
        "Validated official sample parity matrix: {} cases",
        official_samples.len()
    );
    Ok(())
}

fn read_official_samples(root: &Path) -> Result<Vec<OfficialSample>> {
    let samples_dir = root
        .join("boxddd-sys")
        .join("third-party")
        .join("box3d")
        .join("samples");
    let mut files = fs::read_dir(&samples_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;
    files.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("sample_") && name.ends_with(".cpp"))
    });
    files.sort();

    let mut samples = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("invalid UTF-8 sample path: {}", path.display()))?;
        for (line_index, line) in source.lines().enumerate() {
            if let Some((category, name)) = parse_sample_registration(line) {
                samples.push(OfficialSample {
                    category,
                    name,
                    source: format!("{file_name}:{}", line_index + 1),
                });
            }
        }
    }

    if samples.is_empty() {
        Err(format!(
            "no official sample registrations found under {}",
            samples_dir.display()
        )
        .into())
    } else {
        Ok(samples)
    }
}

fn parse_sample_registration(line: &str) -> Option<(String, String)> {
    let tail = if let Some(index) = line.find("RegisterSample(") {
        &line[index + "RegisterSample(".len()..]
    } else if let Some(index) = line.find("RegisterReplay(") {
        &line[index + "RegisterReplay(".len()..]
    } else {
        return None;
    };

    let (category, tail) = parse_cpp_string_literal(tail)?;
    let tail = tail.trim_start();
    let tail = tail.strip_prefix(',')?;
    let (name, _) = parse_cpp_string_literal(tail)?;
    Some((category, name))
}

fn parse_cpp_string_literal(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    let input = input.strip_prefix('"')?;
    let end = input.find('"')?;
    Some((input[..end].to_string(), &input[end + 1..]))
}

fn read_sample_parity_rows(root: &Path) -> Result<Vec<SampleParityRow>> {
    let matrix = root.join(SAMPLE_MATRIX_PATH);
    let source = fs::read_to_string(&matrix)?;
    let mut rows = Vec::new();
    let mut in_case_table = false;

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == SAMPLE_CASE_TABLE_HEADER {
            in_case_table = true;
            continue;
        }
        if !in_case_table {
            continue;
        }
        if trimmed.starts_with("|---") {
            continue;
        }
        if !trimmed.starts_with('|') {
            if !rows.is_empty() {
                break;
            }
            continue;
        }

        let cells = split_markdown_table_row(trimmed);
        if cells.len() != 6 {
            return Err(format!(
                "{}:{} has {} cells, expected 6",
                matrix.display(),
                line_index + 1,
                cells.len()
            )
            .into());
        }
        rows.push(SampleParityRow {
            category: cells[0].clone(),
            name: cells[1].clone(),
            source: strip_code_ticks(&cells[2]),
            mode: cells[3].clone(),
            target: cells[4].clone(),
            notes: cells[5].clone(),
            line_number: line_index + 1,
        });
    }

    if rows.is_empty() {
        Err(format!(
            "{} is missing the official case table header `{SAMPLE_CASE_TABLE_HEADER}`",
            matrix.display()
        )
        .into())
    } else {
        Ok(rows)
    }
}

fn split_markdown_table_row(row: &str) -> Vec<String> {
    row.trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn strip_code_ticks(value: &str) -> String {
    value.trim().trim_matches('`').to_string()
}

fn validate_sample_parity(
    root: &Path,
    official_samples: &[OfficialSample],
    parity_rows: &[SampleParityRow],
) -> Result<()> {
    let official_keys = official_samples.iter().collect::<BTreeSet<_>>();
    let mut row_keys = BTreeSet::new();
    for row in parity_rows {
        validate_sample_parity_row(root, row)?;
        let key = OfficialSample {
            category: row.category.clone(),
            name: row.name.clone(),
            source: row.source.clone(),
        };
        if !row_keys.insert(key) {
            return Err(format!(
                "{}:{} duplicates sample `{}` / `{}` at `{}`",
                SAMPLE_MATRIX_PATH, row.line_number, row.category, row.name, row.source
            )
            .into());
        }
    }

    let missing = official_samples
        .iter()
        .filter(|sample| !row_keys.contains(*sample))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "{} is missing {} official sample case(s): {}",
            SAMPLE_MATRIX_PATH,
            missing.len(),
            format_sample_key_list(&missing)
        )
        .into());
    }

    let extra = row_keys
        .iter()
        .filter(|sample| !official_keys.contains(sample))
        .collect::<Vec<_>>();
    if !extra.is_empty() {
        return Err(format!(
            "{} contains {} unknown sample case(s): {}",
            SAMPLE_MATRIX_PATH,
            extra.len(),
            format_sample_key_list(&extra)
        )
        .into());
    }

    Ok(())
}

fn validate_sample_parity_row(root: &Path, row: &SampleParityRow) -> Result<()> {
    if row.category.is_empty() || row.name.is_empty() || row.source.is_empty() {
        return Err(format!(
            "{}:{} has an empty category, sample, or source field",
            SAMPLE_MATRIX_PATH, row.line_number
        )
        .into());
    }
    if !SAMPLE_PARITY_MODES.contains(&row.mode.as_str()) {
        return Err(format!(
            "{}:{} uses unknown parity mode `{}`",
            SAMPLE_MATRIX_PATH, row.line_number, row.mode
        )
        .into());
    }
    if row.target.is_empty() || row.notes.is_empty() {
        return Err(format!(
            "{}:{} has an empty target or notes field",
            SAMPLE_MATRIX_PATH, row.line_number
        )
        .into());
    }
    if row.mode != "Deferred" && row.mode != "UpstreamReference" {
        validate_target_code_spans(root, row)?;
    }
    Ok(())
}

fn validate_target_code_spans(root: &Path, row: &SampleParityRow) -> Result<()> {
    let targets = extract_code_spans(&row.target);
    if targets.is_empty() {
        return Err(format!(
            "{}:{} target must contain at least one repo-relative code span",
            SAMPLE_MATRIX_PATH, row.line_number
        )
        .into());
    }

    for target in targets {
        let path = target.split('#').next().unwrap_or(&target).trim();
        if path.is_empty() {
            return Err(format!(
                "{}:{} target `{target}` has an empty path",
                SAMPLE_MATRIX_PATH, row.line_number
            )
            .into());
        }
        if !root.join(path).exists() {
            return Err(format!(
                "{}:{} target path does not exist: `{path}`",
                SAMPLE_MATRIX_PATH, row.line_number
            )
            .into());
        }
    }
    Ok(())
}

fn extract_code_spans(value: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = value;
    while let Some(start) = rest.find('`') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('`') else {
            break;
        };
        spans.push(after_start[..end].to_string());
        rest = &after_start[end + 1..];
    }
    spans
}

fn format_sample_key_list<T>(samples: &[T]) -> String
where
    T: std::borrow::Borrow<OfficialSample>,
{
    samples
        .iter()
        .take(10)
        .map(|sample| {
            let sample = sample.borrow();
            format!("{}/{} ({})", sample.category, sample.name, sample.source)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn generate_pages() -> Result<()> {
    let root = project_root();
    let pages_dir = root.join("docs").join("pages");
    let registry_samples = read_testbed_registry(&root)?;
    generate_bevy_example_pages(&pages_dir, &registry_samples)?;
    eprintln!(
        "Generated {} Bevy example pages under {}",
        registry_samples.len(),
        pages_bevy_examples_dir().display()
    );
    Ok(())
}

fn validate_pages() -> Result<()> {
    let root = project_root();
    let pages_dir = root.join("docs").join("pages");
    let index = ensure_file(&pages_dir.join("index.html"), "Pages index")?;
    ensure_file(
        &pages_dir.join("bevy-testbed").join("index.html"),
        "Bevy Web testbed page",
    )?;
    let loader = ensure_file(
        &pages_dir.join("bevy-testbed").join("loader.js"),
        "Bevy Web testbed loader",
    )?;

    let registry_samples = read_testbed_registry(&root)?;
    validate_bevy_example_pages(&pages_dir, &registry_samples)?;

    let html = fs::read_to_string(&index)?;
    validate_html_links(&index, &html)?;
    validate_bevy_loader(&loader)?;

    eprintln!(
        "Validated Pages site: {} ({} Bevy examples)",
        pages_dir.display(),
        registry_samples.len()
    );
    Ok(())
}

fn validate_bevy_loader(loader: &Path) -> Result<()> {
    let js = fs::read_to_string(loader)?;
    for required in [
        "box3d-provider-shim.js",
        "setBox3dProvider",
        "setBoxdddAppExports",
        "bevyExports",
    ] {
        if !js.contains(required) {
            return Err(format!(
                "{} is missing required Bevy provider glue `{required}`",
                loader.display()
            )
            .into());
        }
    }
    Ok(())
}

fn generate_bevy_example_pages(pages_dir: &Path, samples: &[RegistrySample]) -> Result<()> {
    let examples_dir = pages_dir.join(BEVY_EXAMPLES_DIR);
    fs::create_dir_all(&examples_dir)?;
    fs::write(
        pages_dir.join("index.html"),
        example_index_page(samples, ExampleIndexLocation::Root),
    )?;
    fs::write(
        examples_dir.join("index.html"),
        example_index_page(samples, ExampleIndexLocation::ExamplesDirectory),
    )?;

    for sample in samples {
        let dir = examples_dir.join(&sample.id);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join("index.html"), example_page(sample))?;
    }

    Ok(())
}

fn validate_bevy_example_pages(pages_dir: &Path, samples: &[RegistrySample]) -> Result<()> {
    let examples_dir = ensure_file(
        &pages_dir.join(BEVY_EXAMPLES_DIR).join("index.html"),
        "Bevy examples index",
    )?;
    let examples_html = fs::read_to_string(&examples_dir)?;
    validate_html_links(&examples_dir, &examples_html)?;

    for sample in samples {
        let page = ensure_file(
            &pages_dir
                .join(BEVY_EXAMPLES_DIR)
                .join(&sample.id)
                .join("index.html"),
            &format!("Bevy example page `{}`", sample.id),
        )?;
        let html = fs::read_to_string(&page)?;
        if !html.contains(&format!("data-scene-id=\"{}\"", sample.id)) {
            return Err(format!("{} is missing its scene id", page.display()).into());
        }
        if !html.contains(&escape_html(&sample.name)) {
            return Err(format!("{} is missing its scene title", page.display()).into());
        }
        validate_html_links(&page, &html)?;
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ExampleIndexLocation {
    Root,
    ExamplesDirectory,
}

impl ExampleIndexLocation {
    fn home_href(self) -> &'static str {
        match self {
            Self::Root => "./",
            Self::ExamplesDirectory => "../",
        }
    }

    fn scene_href(self, id: &str) -> String {
        match self {
            Self::Root => format!("examples/{id}/"),
            Self::ExamplesDirectory => format!("{id}/"),
        }
    }
}

fn example_index_page(samples: &[RegistrySample], location: ExampleIndexLocation) -> String {
    let links = samples
        .iter()
        .map(|sample| {
            format!(
                "        <a class=\"card\" href=\"{href}\"><span>{category}</span><strong>{name}</strong><small>{description}</small></a>",
                href = location.scene_href(&sample.id),
                category = escape_html(&sample.category),
                name = escape_html(&sample.name),
                description = escape_html(&sample.description)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>boxddd Bevy Examples</title>
  <link rel="icon" href="data:,">
  <meta name="description" content="Direct Bevy Web examples for boxddd.">
  <style>{example_page_css}</style>
</head>
<body>
  <div class="directory">
    <header class="topbar">
      <a href="{home_href}">boxddd Examples</a>
      <nav>
        <a href="https://github.com/Latias94/boxddd">GitHub</a>
        <a href="https://docs.rs/boxddd">Docs.rs</a>
      </nav>
    </header>
    <main class="directory-main">
      <p class="eyebrow">Bevy Web examples</p>
      <h1>Run a Box3D scene</h1>
      <p class="lead">Each entry opens a dedicated Bevy + egui WASM page backed by the same Box3D provider runtime.</p>
      <section class="card-grid">
{links}
      </section>
    </main>
  </div>
</body>
</html>
"#,
        example_page_css = example_page_css(),
        home_href = location.home_href(),
        links = links
    )
}

fn example_page(sample: &RegistrySample) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{name} - boxddd Bevy Example</title>
  <link rel="icon" href="data:,">
  <meta name="description" content="{description}">
  <style>{example_page_css}</style>
</head>
<body>
  <div class="shell">
    <header class="topbar">
      <div>
        <a href="../../">boxddd Examples</a>
        <h1>{name}</h1>
        <p><span>{category}</span>{description}</p>
      </div>
      <nav>
        <a href="../">All Bevy examples</a>
        <a href="https://github.com/Latias94/boxddd/tree/main/bevy_boxddd/examples/testbed_3d">Source</a>
      </nav>
    </header>
    <main id="bevy-app" data-scene-id="{id}" data-scene-name="{name}" data-scene-category="{category}">
      <canvas id="bevy-canvas" tabindex="0"></canvas>
      <div id="bevy-status" role="status" aria-live="polite">
        <strong>Loading {name}</strong>
        <span>Preparing the shared Box3D provider and the Rust Bevy wasm module.</span>
      </div>
    </main>
  </div>
  <script type="module" src="../../bevy-testbed/loader.js"></script>
</body>
</html>
"#,
        id = sample.id,
        name = escape_html(&sample.name),
        category = escape_html(&sample.category),
        description = escape_html(&sample.description),
        example_page_css = example_page_css()
    )
}

fn example_page_css() -> &'static str {
    r#"
:root {
  color-scheme: dark;
  --background: #09090b;
  --foreground: #fafafa;
  --card: #0f0f12;
  --muted: #a1a1aa;
  --border: #27272a;
  --accent: #84cc16;
  --danger: #f87171;
}
* { box-sizing: border-box; }
html, body { width: 100%; height: 100%; margin: 0; background: var(--background); color: var(--foreground); font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
a { color: var(--foreground); text-decoration: none; }
a:hover { text-decoration: underline; text-underline-offset: 4px; }
.shell { display: grid; grid-template-rows: auto minmax(0, 1fr); width: 100%; height: 100%; }
.topbar { display: flex; flex-wrap: wrap; gap: 14px; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--border); background: rgba(9, 9, 11, 0.94); padding: 14px 18px; }
.topbar h1 { margin: 4px 0 0; font-size: 20px; line-height: 1.2; letter-spacing: 0; }
.topbar p { display: flex; flex-wrap: wrap; gap: 8px; margin: 5px 0 0; color: var(--muted); font-size: 13px; }
.topbar p span, .eyebrow { color: var(--accent); font-weight: 700; text-transform: uppercase; }
.topbar nav { display: flex; flex-wrap: wrap; gap: 12px; color: var(--muted); font-size: 14px; }
#bevy-app { position: relative; min-width: 0; min-height: 0; background: #020617; }
#bevy-canvas { display: block; width: 100%; height: 100%; outline: none; touch-action: none; }
#bevy-status { position: absolute; left: 18px; bottom: 18px; max-width: min(560px, calc(100% - 36px)); border: 1px solid var(--border); border-radius: 8px; background: rgba(15, 15, 18, 0.94); padding: 12px 14px; color: var(--muted); font-size: 14px; line-height: 1.45; }
#bevy-status strong { display: block; margin-bottom: 4px; color: var(--foreground); font-size: 15px; }
#bevy-status[data-state="error"] strong { color: var(--danger); }
#bevy-status[data-state="running"] { opacity: 0; pointer-events: none; transition: opacity 180ms ease; }
.directory { min-height: 100%; }
.directory-main { width: min(1180px, calc(100% - 32px)); margin: 0 auto; padding: 54px 0; }
.directory-main h1 { margin: 0; font-size: clamp(34px, 6vw, 58px); line-height: 1; letter-spacing: 0; }
.lead { max-width: 720px; color: var(--muted); font-size: 17px; }
.card-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; margin-top: 28px; }
.card { display: grid; min-height: 150px; gap: 8px; border: 1px solid var(--border); border-radius: 8px; background: var(--card); padding: 16px; }
.card:hover { border-color: #52525b; text-decoration: none; }
.card span { color: var(--accent); font-size: 12px; font-weight: 700; text-transform: uppercase; }
.card strong { font-size: 18px; }
.card small { color: var(--muted); font-size: 13px; line-height: 1.5; }
"#
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn ensure_file(path: &Path, label: &str) -> Result<PathBuf> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(format!("{label} is missing: {}", path.display()).into())
    }
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

impl BuildProfile {
    fn from_env() -> Self {
        match env::var("PROFILE").as_deref() {
            Ok("release") => Self::Release,
            _ => Self::Debug,
        }
    }

    fn cargo_args(self) -> &'static [&'static str] {
        match self {
            Self::Debug => &[],
            Self::Release => &["--release"],
        }
    }

    fn target_dir(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
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

fn validate_html_links(html_file: &Path, html: &str) -> Result<()> {
    let pages_dir = project_root().join("docs").join("pages");
    let pages_root = fs::canonicalize(&pages_dir)?;
    let base_dir = html_file
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", html_file.display()))?;
    validate_attr_links(html_file, base_dir, &pages_root, html, "href")?;
    validate_attr_links(html_file, base_dir, &pages_root, html, "src")?;
    validate_fetch_links(html_file, base_dir, &pages_root, html, "\"")?;
    validate_fetch_links(html_file, base_dir, &pages_root, html, "'")?;
    Ok(())
}

fn validate_attr_links(
    html_file: &Path,
    base_dir: &Path,
    pages_root: &Path,
    html: &str,
    attr: &str,
) -> Result<()> {
    let needle = format!("{attr}=\"");
    let mut remainder = html;
    while let Some(index) = remainder.find(&needle) {
        let after = &remainder[index + needle.len()..];
        let end = after
            .find('"')
            .ok_or_else(|| format!("unterminated `{attr}` attribute in {}", html_file.display()))?;
        validate_local_link(html_file, base_dir, pages_root, &after[..end])?;
        remainder = &after[end + 1..];
    }
    Ok(())
}

fn validate_fetch_links(
    html_file: &Path,
    base_dir: &Path,
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
            .ok_or_else(|| format!("unterminated fetch() URL in {}", html_file.display()))?;
        validate_local_link(html_file, base_dir, pages_root, &after[..end])?;
        remainder = &after[end + quote.len()..];
    }
    Ok(())
}

fn validate_local_link(
    html_file: &Path,
    base_dir: &Path,
    pages_root: &Path,
    value: &str,
) -> Result<()> {
    if is_external_or_fragment(value) {
        return Ok(());
    }

    let local = strip_url_suffix(value);
    if local.is_empty() {
        return Ok(());
    }

    let target = base_dir.join(local);
    if !target.exists() {
        return Err(format!(
            "{} links missing local asset `{value}`",
            html_file.display()
        )
        .into());
    }

    let canonical = fs::canonicalize(&target)?;
    if !canonical.starts_with(pages_root) {
        return Err(format!("{} link escapes docs/pages: `{value}`", html_file.display()).into());
    }

    Ok(())
}

fn is_external_or_fragment(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("mailto:")
        || value.starts_with("data:")
        || value.starts_with('#')
        || value.starts_with("javascript:")
}

fn strip_url_suffix(value: &str) -> &str {
    let query = value.find('?').unwrap_or(value.len());
    let fragment = value.find('#').unwrap_or(value.len());
    &value[..query.min(fragment)]
}

fn build_provider_smoke_app() -> Result<PathBuf> {
    build_provider_smoke_app_for(BuildProfile::from_env())
}

fn build_provider_smoke_app_for(profile: BuildProfile) -> Result<PathBuf> {
    let root = project_root();
    let mut command = Command::new("cargo");
    command
        .arg("rustc")
        .arg("-p")
        .arg(SMOKE_PACKAGE)
        .arg("--lib")
        .arg("--target")
        .arg(TARGET)
        .args(profile.cargo_args())
        .env("BOXDDD_SYS_WASM_MODE", "provider");
    add_wasm_app_link_args(
        &mut command,
        &[PROVIDER_SMOKE_EXPORTS, DEBUG_BRIDGE_EXPORTS],
    );
    run_command(&mut command, "build provider-smoke Rust wasm")?;

    let wasm = root
        .join("target")
        .join(TARGET)
        .join(profile.target_dir())
        .join(SMOKE_WASM);
    if !wasm.exists() {
        return Err(format!("provider-smoke wasm artifact not found: {}", wasm.display()).into());
    }

    let out_dir = provider_smoke_dir();
    fs::create_dir_all(&out_dir)?;
    fs::copy(&wasm, out_dir.join(SMOKE_WASM))?;
    Ok(wasm)
}

fn add_wasm_app_link_args(command: &mut Command, export_groups: &[&[&str]]) {
    command.arg("--").arg("-C").arg("link-arg=--import-memory");
    for export in export_groups.iter().flat_map(|exports| exports.iter()) {
        command.arg("-C").arg(format!("link-arg=--export={export}"));
    }
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
        .collect::<BTreeSet<_>>()
        .into_iter()
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

fn build_bevy_web_app() -> Result<BevyWebArtifacts> {
    ensure_tool(
        "wasm-bindgen",
        "--version",
        "wasm-bindgen-cli is required for Bevy Web examples",
    )?;

    let root = project_root();
    let out_dir = root.join("target").join("boxddd-bevy-testbed-web");
    replace_dir_under(&out_dir, &root.join("target"))?;

    let mut command = Command::new("cargo");
    command
        .arg("rustc")
        .arg("-p")
        .arg("bevy_boxddd")
        .arg("--features")
        .arg("debug-gizmos physics-picking")
        .arg("--example")
        .arg(BEVY_WEB_EXAMPLE)
        .arg("--target")
        .arg(TARGET)
        .arg("--release")
        .env("BOXDDD_SYS_WASM_MODE", "provider");
    add_wasm_app_link_args(&mut command, &[DEBUG_BRIDGE_EXPORTS]);
    run_command(&mut command, "build Bevy testbed wasm")?;

    let wasm = root
        .join("target")
        .join(TARGET)
        .join("release")
        .join("examples")
        .join(format!("{BEVY_WEB_EXAMPLE}.wasm"));
    ensure_file(&wasm, "Bevy testbed wasm")?;

    let mut bindgen = Command::new("wasm-bindgen");
    bindgen
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--out-name")
        .arg(BEVY_WEB_OUT_NAME)
        .arg(&wasm);
    run_command(&mut bindgen, "run wasm-bindgen for Bevy testbed")?;

    patch_bevy_bindgen_imports(&out_dir.join(BEVY_WEB_JS))?;
    let bevy_wasm = out_dir.join(BEVY_WEB_WASM);
    let imports = collect_provider_imports(&bevy_wasm)?;
    write_browser_provider_shim(&out_dir, &imports)?;

    Ok(BevyWebArtifacts { out_dir, imports })
}

fn patch_bevy_bindgen_imports(js: &Path) -> Result<()> {
    let source = fs::read_to_string(js)?;
    let patched_imports = source.replace(
        &format!("from \"{PROVIDER_MODULE}\""),
        &format!("from \"./{BEVY_PROVIDER_SHIM}\""),
    );
    if patched_imports == source {
        return Err(format!(
            "wasm-bindgen output does not import {PROVIDER_MODULE}: {}",
            js.display()
        )
        .into());
    }
    let patched = patched_imports.replace(
        "    wasm = instance.exports;\n",
        "    wasm = instance.exports;\n    if (typeof import1.setBoxdddAppExports === \"function\") {\n        import1.setBoxdddAppExports(wasm);\n    }\n",
    );
    if patched == patched_imports {
        return Err(format!(
            "wasm-bindgen output does not assign instance exports: {}",
            js.display()
        )
        .into());
    }
    fs::write(js, patched)?;
    Ok(())
}

fn write_browser_provider_shim(out_dir: &Path, imports: &[String]) -> Result<PathBuf> {
    let exports = imports
        .iter()
        .map(|name| {
            format!("export function {name}(...args) {{ return callProvider(\"{name}\", args); }}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let shim = format!(
        r#"let provider;

export function setBox3dProvider(nextProvider) {{
  provider = nextProvider;
}}

export function setBoxdddAppExports(exports) {{
  if (!provider) {{
    throw new Error("Box3D provider is not initialized");
  }}
  provider.boxdddAppExports = exports;
}}

function resolveProviderExport(name) {{
  if (!provider) {{
    throw new Error("Box3D provider is not initialized");
  }}
  const exported = provider[`_${{name}}`] || provider[name];
  if (typeof exported !== "function") {{
    throw new Error(`Box3D provider is missing export ${{name}}`);
  }}
  return exported;
}}

function callProvider(name, args) {{
  return resolveProviderExport(name)(...args);
}}

{exports}
"#
    );
    let path = out_dir.join(BEVY_PROVIDER_SHIM);
    fs::write(&path, shim)?;
    Ok(path)
}

fn copy_bevy_web_artifacts(artifacts: &BevyWebArtifacts) -> Result<()> {
    let generated = pages_bevy_generated_dir();
    replace_dir_under(&generated, &project_root().join("docs").join("pages"))?;

    for file in [BEVY_WEB_JS, BEVY_WEB_WASM, BEVY_PROVIDER_SHIM] {
        fs::copy(artifacts.out_dir.join(file), generated.join(file))?;
    }

    Ok(())
}

fn replace_dir_under(dir: &Path, allowed_root: &Path) -> Result<()> {
    fs::create_dir_all(allowed_root)?;
    if dir.exists() {
        let canonical_dir = fs::canonicalize(dir)?;
        let canonical_root = fs::canonicalize(allowed_root)?;
        if !canonical_dir.starts_with(&canonical_root) {
            return Err(format!(
                "refusing to remove directory outside {}: {}",
                canonical_root.display(),
                canonical_dir.display()
            )
            .into());
        }
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;
    Ok(())
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

fn build_pages_wasm() -> Result<()> {
    generate_pages()?;
    let bevy_artifacts = build_bevy_web_app()?;
    let out_dir = provider_smoke_dir();
    let exports = write_exports_json(&out_dir, &bevy_artifacts.imports)?;
    let provider = build_box3d_provider(&out_dir, &exports)?;
    let provider_wasm = provider.with_extension("wasm");
    ensure_file(&provider, "Box3D provider module")?;
    ensure_file(&provider_wasm, "Box3D provider wasm")?;

    let generated = pages_wasm_generated_dir();
    replace_dir_under(&generated, &project_root().join("docs").join("pages"))?;

    fs::copy(&provider, generated.join("box3d-sys-v0.mjs"))?;
    fs::copy(&provider_wasm, generated.join("box3d-sys-v0.wasm"))?;
    copy_bevy_web_artifacts(&bevy_artifacts)?;

    eprintln!(
        "Pages WASM assets ready: {} and {} ({} Bevy imports, {} provider exports)",
        generated.display(),
        pages_bevy_generated_dir().display(),
        bevy_artifacts.imports.len(),
        bevy_artifacts.imports.len()
    );
    Ok(())
}

fn build_box3d_provider(out_dir: &Path, exports_json: &Path) -> Result<PathBuf> {
    let emcc = find_emcc()?;
    let root = project_root();
    let box3d_root = root.join("boxddd-sys").join("third-party").join("box3d");
    let include_dir = box3d_root.join("include");
    let src_dir = box3d_root.join("src");
    let provider_helper = root
        .join("boxddd-sys")
        .join("provider")
        .join("debug_callbacks.c");
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
        .arg("MAXIMUM_MEMORY=536870912")
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
    command.arg(provider_helper);
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
provider.boxdddAppExports = instance.exports;
if (typeof instance.exports.boxddd_provider_smoke !== 'function') {{
  throw new Error('boxddd_provider_smoke export is missing from Rust wasm');
}}

const code = instance.exports.boxddd_provider_smoke();
if (code !== 0) {{
  throw new Error(`boxddd provider smoke failed with code ${{code}}`);
}}

const metricExports = {{
  dropMillimeters: 'boxddd_provider_drop_millimeters',
  rayHitMillimeters: 'boxddd_provider_ray_hit_millimeters',
  shapeCastPermyriad: 'boxddd_provider_shape_cast_permyriad',
  jointErrorMillimeters: 'boxddd_provider_joint_error_millimeters',
}};
const metrics = {{}};
for (const [label, exportName] of Object.entries(metricExports)) {{
  const exported = instance.exports[exportName];
  if (typeof exported !== 'function') {{
    throw new Error(`${{exportName}} export is missing from Rust wasm`);
  }}
  const value = exported();
  if (value < 0) {{
    throw new Error(`${{exportName}} failed with code ${{value}}`);
  }}
  metrics[label] = value;
}}

console.log(
  `boxddd provider smoke passed: drop_mm=${{metrics.dropMillimeters}}, ` +
    `ray_hit_mm=${{metrics.rayHitMillimeters}}, ` +
    `shape_cast_permyriad=${{metrics.shapeCastPermyriad}}, ` +
    `joint_error_mm=${{metrics.jointErrorMillimeters}}`
);
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
