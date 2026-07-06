const statusPanel = document.querySelector("#bevy-status");
const appRoot = document.querySelector("#bevy-app");
const sceneId = appRoot?.dataset.sceneId || "";
const sceneName = appRoot?.dataset.sceneName || "Bevy testbed";
const isExamplePage = Boolean(sceneId);

function setStatus(state, title, detail, progress) {
  statusPanel.dataset.state = state;
  statusPanel.replaceChildren();

  const titleNode = document.createElement("strong");
  titleNode.textContent = title;
  const detailNode = document.createElement("span");
  detailNode.textContent = detail;
  statusPanel.append(titleNode, detailNode);

  if (progress) {
    const progressNode = document.createElement("progress");
    progressNode.value = progress.loaded;
    if (progress.total) {
      progressNode.max = progress.total;
    } else {
      progressNode.removeAttribute("value");
    }

    const progressText = document.createElement("small");
    progressText.textContent = progressTextFor(progress.loaded, progress.total);
    statusPanel.append(progressNode, progressText);
  }
}

function generatedUrl(path) {
  return new URL(path, import.meta.url);
}

function progressTextFor(loaded, total) {
  if (total) {
    const percent = Math.min(100, Math.round((loaded / total) * 100));
    return `${formatBytes(loaded)} / ${formatBytes(total)} (${percent}%)`;
  }
  return `${formatBytes(loaded)} downloaded`;
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return "0 B";
  }
  const units = ["B", "KiB", "MiB", "GiB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${value} ${units[unit]}` : `${value.toFixed(2)} ${units[unit]}`;
}

async function fetchArrayBufferWithProgress(url, label) {
  setStatus("loading", `Downloading ${label}`, "Starting download.", { loaded: 0, total: 0 });
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${label} download failed with HTTP ${response.status}`);
  }

  const total = Number(response.headers.get("Content-Length")) || 0;
  if (!response.body) {
    const buffer = await response.arrayBuffer();
    setStatus("loading", `Downloading ${label}`, "Download complete.", {
      loaded: buffer.byteLength,
      total: total || buffer.byteLength,
    });
    return buffer;
  }

  const reader = response.body.getReader();
  const chunks = [];
  let loaded = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    chunks.push(value);
    loaded += value.byteLength;
    setStatus("loading", `Downloading ${label}`, "Downloading runtime asset.", { loaded, total });
  }

  const bytes = new Uint8Array(loaded);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  setStatus("loading", `Downloading ${label}`, "Download complete.", { loaded, total: total || loaded });
  return bytes.buffer;
}

async function main() {
  const providerGenerated = new URL("../wasm/generated/", import.meta.url);
  const providerWasmUrl = new URL("box3d-sys-v0.wasm", providerGenerated);
  const bevyWasmUrl = generatedUrl("generated/bevy_boxddd_testbed_bg.wasm");

  setStatus("loading", "Loading JavaScript modules", `Preparing the browser runtime for ${sceneName}.`);
  const [
    { default: createProvider },
    { default: initBevyTestbed },
    { setBox3dProvider, setBoxdddAppExports },
  ] =
    await Promise.all([
      import(new URL("box3d-sys-v0.js", providerGenerated).href),
      import(generatedUrl("generated/bevy_boxddd_testbed.js").href),
      import(generatedUrl("generated/box3d-provider-shim.js").href),
    ]);
  const memory = new WebAssembly.Memory({ initial: 4096, maximum: 8192 });

  const providerWasm = await fetchArrayBufferWithProgress(providerWasmUrl, "Box3D provider wasm");
  setStatus("loading", "Starting Box3D provider", `Instantiating the shared Box3D C provider for ${sceneName}.`);
  const provider = await createProvider({
    wasmMemory: memory,
    wasmBinary: providerWasm,
    locateFile: (path) => new URL(path, providerGenerated).href,
    print: (text) => console.log(`[box3d-sys-v0] ${text}`),
    printErr: (text) => console.warn(`[box3d-sys-v0] ${text}`),
  });

  if (provider.wasmMemory && provider.wasmMemory !== memory) {
    throw new Error("Box3D provider did not use the shared WebAssembly.Memory");
  }

  setBox3dProvider(provider);
  const bevyWasm = await fetchArrayBufferWithProgress(bevyWasmUrl, `${sceneName} Bevy wasm`);
  setStatus("loading", `Starting ${sceneName}`, "Instantiating the Rust Bevy + egui wasm module.");

  const bevyExports = await initBevyTestbed({
    module_or_path: bevyWasm,
    memory,
  });
  setBoxdddAppExports(bevyExports);

  window.BOXDDD_BEVY_TESTBED_READY = true;
  window.BOXDDD_BEVY_EXAMPLE_READY = true;
  window.BOXDDD_BEVY_SCENE_ID = sceneId;
  setStatus(
    "running",
    `${sceneName} running`,
    isExamplePage
      ? "This dedicated example page is running the selected Box3D scene in Bevy."
      : "The scene browser, egui controls, picking, and Box3D simulation are running in this canvas.",
  );
}

main().catch((error) => {
  console.error(error);
  const message = error instanceof Error ? error.message : String(error);
  setStatus("error", `${sceneName} failed`, message);
});
