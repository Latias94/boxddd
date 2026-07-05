const statusPanel = document.querySelector("#bevy-status");
const appRoot = document.querySelector("#bevy-app");
const sceneId = appRoot?.dataset.sceneId || "";
const sceneName = appRoot?.dataset.sceneName || "Bevy testbed";
const isExamplePage = Boolean(sceneId);

function setStatus(state, title, detail) {
  statusPanel.dataset.state = state;
  statusPanel.replaceChildren();

  const titleNode = document.createElement("strong");
  titleNode.textContent = title;
  const detailNode = document.createElement("span");
  detailNode.textContent = detail;
  statusPanel.append(titleNode, detailNode);
}

function generatedUrl(path) {
  return new URL(path, import.meta.url);
}

async function main() {
  const providerGenerated = new URL("../wasm/generated/", import.meta.url);
  const [
    { default: createProvider },
    { default: initBevyTestbed },
    { setBox3dProvider, setBoxdddAppExports },
  ] =
    await Promise.all([
      import(new URL("box3d-sys-v0.mjs", providerGenerated).href),
      import(generatedUrl("generated/bevy_boxddd_testbed.js").href),
      import(generatedUrl("generated/box3d-provider-shim.js").href),
    ]);
  const memory = new WebAssembly.Memory({ initial: 4096, maximum: 8192 });

  setStatus("loading", "Loading Box3D provider", `Preparing the shared Box3D C provider for ${sceneName}.`);
  const provider = await createProvider({
    wasmMemory: memory,
    locateFile: (path) => new URL(path, providerGenerated).href,
    print: (text) => console.log(`[box3d-sys-v0] ${text}`),
    printErr: (text) => console.warn(`[box3d-sys-v0] ${text}`),
  });

  if (provider.wasmMemory && provider.wasmMemory !== memory) {
    throw new Error("Box3D provider did not use the shared WebAssembly.Memory");
  }

  setBox3dProvider(provider);
  setStatus("loading", `Loading ${sceneName}`, "Starting the Rust Bevy + egui wasm module.");

  const bevyExports = await initBevyTestbed({
    module_or_path: generatedUrl("generated/bevy_boxddd_testbed_bg.wasm"),
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
