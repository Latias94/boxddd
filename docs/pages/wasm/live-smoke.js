const providerModule = "box3d-sys-v0";
const buttons = Array.from(document.querySelectorAll("[data-wasm-demo]"));
const statusText = document.querySelector("#wasm-status");
const detailText = document.querySelector("#wasm-detail");
const canvas = document.querySelector("#wasm-canvas");
const context = canvas.getContext("2d");

const demos = {
  drop: {
    exportName: "boxddd_provider_drop_millimeters",
    label: "Falling body",
    loading: "Stepping a dynamic body in a Box3D world.",
    detail: (value, imports) =>
      `Rust wasm called ${imports} Box3D provider imports and measured a ${value} mm fall.`,
  },
  ray: {
    exportName: "boxddd_provider_ray_hit_millimeters",
    label: "Closest ray query",
    loading: "Casting a closest-hit ray through two static shapes.",
    detail: (value, imports) =>
      `Rust wasm called ${imports} Box3D provider imports and hit after ${value} mm of ray travel.`,
  },
  cast: {
    exportName: "boxddd_provider_shape_cast_permyriad",
    label: "Shape cast",
    loading: "Sweeping one convex proxy against another.",
    detail: (value, imports) =>
      `Rust wasm called ${imports} Box3D provider imports and hit at ${(value / 100).toFixed(1)}% of the sweep.`,
  },
  joint: {
    exportName: "boxddd_provider_joint_error_millimeters",
    label: "Distance joint",
    loading: "Solving a distance joint after applying force.",
    detail: (value, imports) =>
      `Rust wasm called ${imports} Box3D provider imports and kept the joint within ${value} mm of its target length.`,
  },
};

let runtimePromise;
let animationFrame = 0;
let currentDemo = "drop";
let currentValue = 0;
let currentProgress = 0;

function setStatus(message, detail) {
  statusText.textContent = message;
  detailText.textContent = detail;
}

function setBusy(isBusy) {
  for (const button of buttons) {
    button.disabled = isBusy;
  }
}

function setActiveButton(demoKey) {
  for (const button of buttons) {
    const isActive = button.dataset.wasmDemo === demoKey;
    button.classList.toggle("primary", isActive);
    button.setAttribute("aria-pressed", String(isActive));
  }
}

function resizeCanvas() {
  const ratio = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = Math.max(1, Math.floor(rect.width * ratio));
  canvas.height = Math.max(1, Math.floor(rect.height * ratio));
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
}

function drawBackground() {
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

  return { width, height, floorY };
}

function drawLabel(label) {
  context.fillStyle = "#dbe7ef";
  context.font = "13px ui-monospace, SFMono-Regular, Consolas, monospace";
  context.fillText(label, 18, 26);
}

function drawBox(x, y, size, fill = "#6aa7ff") {
  context.fillStyle = fill;
  context.fillRect(x, y, size, size);
  context.strokeStyle = "#b8d4ff";
  context.lineWidth = 2;
  context.strokeRect(x, y, size, size);
}

function drawSphere(x, y, radius, fill = "#f4c95d") {
  context.fillStyle = fill;
  context.beginPath();
  context.arc(x, y, radius, 0, Math.PI * 2);
  context.fill();
  context.strokeStyle = "#fff0ad";
  context.lineWidth = 2;
  context.stroke();
}

function drawDrop(progress, value) {
  const { width, height, floorY } = drawBackground();
  const boxSize = Math.min(54, width * 0.16);
  const startY = height * 0.18;
  const endY = floorY - boxSize;
  const y = startY + (endY - startY) * progress;
  drawBox(width * 0.5 - boxSize / 2, y, boxSize);
  drawSphere(width * 0.76, floorY - 18, 11);
  drawLabel(`fall ${value} mm`);
}

function drawRay(progress, value) {
  const { width, height, floorY } = drawBackground();
  const y = floorY - 80;
  const startX = width * 0.16;
  const endX = width * 0.84;
  const hitX = startX + (endX - startX) * 0.3;
  drawSphere(hitX + 18, y, 28, "#6aa7ff");
  drawBox(width * 0.62, y - 26, 52, "#48b8a6");

  context.strokeStyle = "#f4c95d";
  context.lineWidth = 3;
  context.setLineDash([10, 8]);
  context.beginPath();
  context.moveTo(startX, y);
  context.lineTo(startX + (endX - startX) * progress, y);
  context.stroke();
  context.setLineDash([]);

  if (progress > 0.3) {
    drawSphere(hitX, y, 6, "#f4c95d");
  }
  drawLabel(`ray hit ${value} mm`);
}

function drawCast(progress, value) {
  const { width, floorY } = drawBackground();
  const y = floorY - 90;
  const startX = width * 0.78;
  const targetX = width * 0.26;
  const hitFraction = value / 10000;
  const hitX = startX + (targetX - startX) * hitFraction;
  const x = startX + (hitX - startX) * progress;

  drawSphere(targetX, y, 34, "#48b8a6");
  drawSphere(x, y, 34, "#f4c95d");
  context.strokeStyle = "#6aa7ff";
  context.lineWidth = 2;
  context.beginPath();
  context.moveTo(startX, y + 52);
  context.lineTo(targetX, y + 52);
  context.stroke();
  drawLabel(`shape cast ${(value / 100).toFixed(1)}%`);
}

function drawJoint(progress, value) {
  const { width, floorY } = drawBackground();
  const y = floorY - 92;
  const anchorX = width * 0.28;
  const restX = width * 0.62;
  const offset = Math.sin(progress * Math.PI) * Math.min(46, width * 0.08);
  const bodyX = restX + offset;

  drawSphere(anchorX, y, 12, "#48b8a6");
  context.strokeStyle = "#f4c95d";
  context.lineWidth = 4;
  context.beginPath();
  context.moveTo(anchorX, y);
  context.lineTo(bodyX, y);
  context.stroke();
  drawSphere(bodyX, y, 28, "#6aa7ff");
  drawLabel(`joint error ${value} mm`);
}

function drawScene(demoKey, progress, value) {
  if (demoKey === "ray") {
    drawRay(progress, value);
  } else if (demoKey === "cast") {
    drawCast(progress, value);
  } else if (demoKey === "joint") {
    drawJoint(progress, value);
  } else {
    drawDrop(progress, value);
  }
}

function animate(demoKey, value) {
  cancelAnimationFrame(animationFrame);
  const started = performance.now();
  const duration = demoKey === "joint" ? 1400 : 1100;

  function frame(now) {
    const progress = Math.min(1, (now - started) / duration);
    const eased = 1 - (1 - progress) ** 3;
    currentProgress = eased;
    drawScene(demoKey, eased, value);
    if (progress < 1) {
      animationFrame = requestAnimationFrame(frame);
    }
  }

  animationFrame = requestAnimationFrame(frame);
}

async function fetchBytes(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to fetch ${url}: ${response.status}`);
  }
  return response.arrayBuffer();
}

async function loadRuntime() {
  if (runtimePromise) {
    return runtimePromise;
  }

  runtimePromise = (async () => {
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
    if (typeof smoke !== "function") {
      throw new Error("boxddd_provider_smoke export is missing");
    }

    const code = smoke();
    if (code !== 0) {
      throw new Error(`provider smoke failed with code ${code}`);
    }

    return { exports: instance.exports, imports: imports.length };
  })();

  return runtimePromise;
}

async function runDemo(demoKey) {
  const demo = demos[demoKey] || demos.drop;
  currentDemo = demoKey;
  currentValue = 0;
  currentProgress = 0;
  setActiveButton(demoKey);
  setBusy(true);
  setStatus("Loading WASM...", demo.loading);
  drawScene(demoKey, 0, 0);

  try {
    const runtime = await loadRuntime();
    const exported = runtime.exports[demo.exportName];
    if (typeof exported !== "function") {
      throw new Error(`${demo.exportName} export is missing`);
    }
    const value = exported();
    if (value < 0) {
      throw new Error(`${demo.exportName} failed with code ${value}`);
    }
    currentValue = value;
    setStatus(`${demo.label} passed`, demo.detail(value, runtime.imports));
    animate(demoKey, value);
  } catch (error) {
    console.error(error);
    setStatus("WASM example failed", error instanceof Error ? error.message : String(error));
    drawLabel("failed");
  } finally {
    setBusy(false);
  }
}

for (const button of buttons) {
  button.addEventListener("click", () => runDemo(button.dataset.wasmDemo));
}

resizeCanvas();
setActiveButton(currentDemo);
drawScene(currentDemo, 0, currentValue);
window.addEventListener("resize", () => {
  resizeCanvas();
  drawScene(currentDemo, currentProgress, currentValue);
});
