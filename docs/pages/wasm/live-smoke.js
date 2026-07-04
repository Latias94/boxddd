const providerModule = "box3d-sys-v0";
const runButton = document.querySelector("#wasm-run");
const statusText = document.querySelector("#wasm-status");
const detailText = document.querySelector("#wasm-detail");
const canvas = document.querySelector("#wasm-canvas");
const context = canvas.getContext("2d");

function setStatus(message, detail) {
  statusText.textContent = message;
  detailText.textContent = detail;
}

function resizeCanvas() {
  const ratio = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = Math.max(1, Math.floor(rect.width * ratio));
  canvas.height = Math.max(1, Math.floor(rect.height * ratio));
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
}

function drawScene(progress, label) {
  const width = canvas.clientWidth;
  const height = canvas.clientHeight;
  context.clearRect(0, 0, width, height);

  context.fillStyle = "#0a1017";
  context.fillRect(0, 0, width, height);

  const floorY = height * 0.78;
  context.fillStyle = "#1b2a34";
  context.fillRect(0, floorY, width, height - floorY);

  context.strokeStyle = "#334b58";
  context.lineWidth = 1;
  for (let i = 0; i < 4; i += 1) {
    const y = floorY + i * 18;
    context.beginPath();
    context.moveTo(24, y);
    context.lineTo(width - 24, y - 12);
    context.stroke();
  }

  const boxSize = Math.min(54, width * 0.16);
  const startY = height * 0.18;
  const endY = floorY - boxSize;
  const y = startY + (endY - startY) * Math.min(1, Math.max(0, progress));
  const x = width * 0.5 - boxSize / 2;

  context.fillStyle = "#6aa7ff";
  context.fillRect(x, y, boxSize, boxSize);
  context.strokeStyle = "#b8d4ff";
  context.lineWidth = 2;
  context.strokeRect(x, y, boxSize, boxSize);

  context.fillStyle = "#f4c95d";
  context.beginPath();
  context.arc(width * 0.76, floorY - 18, 11, 0, Math.PI * 2);
  context.fill();

  context.strokeStyle = "#f4c95d";
  context.setLineDash([10, 9]);
  context.beginPath();
  context.moveTo(width * 0.18, height * 0.22);
  context.lineTo(width * 0.76, floorY - 18);
  context.stroke();
  context.setLineDash([]);

  context.fillStyle = "#dbe7ef";
  context.font = "13px ui-monospace, SFMono-Regular, Consolas, monospace";
  context.fillText(label, 18, 26);
}

async function fetchBytes(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to fetch ${url}: ${response.status}`);
  }
  return response.arrayBuffer();
}

async function runSmoke() {
  const generated = new URL("./generated/", import.meta.url);
  const providerUrl = new URL("box3d-sys-v0.mjs", generated);
  const rustWasmUrl = new URL("boxddd_provider_smoke.wasm", generated);
  const { default: createProvider } = await import(providerUrl.href);
  const memory = new WebAssembly.Memory({ initial: 2048, maximum: 4096 });
  const provider = await createProvider({
    wasmMemory: memory,
    locateFile: (path) => new URL(path, generated).href,
    print: (text) => console.log(`[box3d-sys-v0] ${text}`),
    printErr: (text) => console.warn(`[box3d-sys-v0] ${text}`),
  });

  if (provider.wasmMemory && provider.wasmMemory !== memory) {
    throw new Error("provider did not use the shared WebAssembly.Memory");
  }

  const appBytes = await fetchBytes(rustWasmUrl.href);
  const module = await WebAssembly.compile(appBytes);
  const imports = WebAssembly.Module.imports(module)
    .filter((item) => item.kind === "function" && item.module === providerModule)
    .map((item) => item.name)
    .sort();

  const importObject = {
    env: { memory },
    [providerModule]: {},
  };

  for (const name of imports) {
    const exported = provider[`_${name}`] || provider[name];
    if (typeof exported !== "function") {
      throw new Error(`provider is missing export for ${name}`);
    }
    importObject[providerModule][name] = exported;
  }

  const instance = await WebAssembly.instantiate(module, importObject);
  const smoke = instance.exports.boxddd_provider_smoke;
  const drop = instance.exports.boxddd_provider_drop_millimeters;
  if (typeof smoke !== "function" || typeof drop !== "function") {
    throw new Error("Rust wasm exports are missing");
  }

  const code = smoke();
  if (code !== 0) {
    throw new Error(`provider smoke failed with code ${code}`);
  }

  const dropMillimeters = drop();
  if (dropMillimeters < 0) {
    throw new Error(`drop probe failed with code ${dropMillimeters}`);
  }

  return { dropMillimeters, imports: imports.length };
}

function animateDrop(dropMillimeters) {
  const started = performance.now();
  const duration = 1100;
  const label = `drop ${dropMillimeters} mm`;

  function frame(now) {
    const progress = Math.min(1, (now - started) / duration);
    const eased = 1 - (1 - progress) ** 3;
    drawScene(eased, label);
    if (progress < 1) {
      requestAnimationFrame(frame);
    }
  }

  requestAnimationFrame(frame);
}

runButton.addEventListener("click", async () => {
  runButton.disabled = true;
  setStatus("Loading WASM...", "Fetching the Rust wasm app and Box3D provider module.");
  drawScene(0, "loading");

  try {
    const result = await runSmoke();
    setStatus(
      "WASM smoke passed",
      `Rust wasm called ${result.imports} Box3D provider imports and measured a ${result.dropMillimeters} mm fall.`
    );
    animateDrop(result.dropMillimeters);
  } catch (error) {
    console.error(error);
    setStatus("WASM smoke failed", error instanceof Error ? error.message : String(error));
    drawScene(0, "failed");
  } finally {
    runButton.disabled = false;
  }
});

resizeCanvas();
drawScene(0, "ready");
window.addEventListener("resize", () => {
  resizeCanvas();
  drawScene(0, "ready");
});
