const statusPanel = document.querySelector("#bevy-status");

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
  const [{ default: createProvider }, { default: initBevyTestbed }, { setBox3dProvider }] =
    await Promise.all([
      import(new URL("box3d-sys-v0.mjs", providerGenerated).href),
      import(generatedUrl("generated/bevy_boxddd_testbed.js").href),
      import(generatedUrl("generated/box3d-provider-shim.js").href),
    ]);
  const memory = new WebAssembly.Memory({ initial: 4096, maximum: 8192 });

  setStatus("loading", "Loading Box3D provider", "Instantiating the Emscripten-built Box3D C module with shared memory.");
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
  setStatus("loading", "Loading Bevy + egui", "Starting the Rust testbed compiled from bevy_boxddd/examples/testbed_3d.");

  await initBevyTestbed({
    module_or_path: generatedUrl("generated/bevy_boxddd_testbed_bg.wasm"),
    memory,
  });

  window.BOXDDD_BEVY_TESTBED_READY = true;
  setStatus("running", "Bevy testbed running", "The scene browser, egui controls, picking, and Box3D simulation are running in this canvas.");
}

main().catch((error) => {
  console.error(error);
  const message = error instanceof Error ? error.message : String(error);
  setStatus("error", "Bevy Web testbed failed", message);
});
