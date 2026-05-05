import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { chromium } from "playwright";

const url =
  process.env.BOARD_RENDERING_HARNESS_URL ??
  "http://127.0.0.1:8080/board-rendering-perf-harness.html?fixture=board_rendering_baseline&seed=board-rendering-baseline-v1";
const screenshotPath = resolve(
  "production/qa/evidence/captures/board-rendering-baseline-1920x1080.png",
);
const tracePath = resolve(
  "production/qa/evidence/captures/board-rendering-baseline-timing.json",
);

await mkdir(dirname(screenshotPath), { recursive: true });

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
const consoleMessages = [];
page.on("console", (message) => {
  consoleMessages.push(`[${message.type()}] ${message.text()}`);
});

await page.goto(url, { waitUntil: "domcontentloaded" });
await page.waitForSelector("canvas");
await page.waitForFunction(
  () => globalThis.__boardRenderingPerf?.harnessReport?.ready_for_capture === true,
  { timeout: 15000 },
);
await page.waitForFunction(
  () =>
    globalThis.__boardRenderingPerf?.sampleCount >= 120 &&
    globalThis.__boardRenderingPerf?.totalFrameBudgetPass !== null,
  { timeout: 15000 },
);
await page.screenshot({ path: screenshotPath, fullPage: false });

const browserFrameTiming = await page.evaluate(
  () => globalThis.__boardRenderingPerf ?? null,
);
const harnessReport = browserFrameTiming?.harnessReport ?? null;
const budgetVerdict = {
  totalFrameSource: "browser_raf_sampler",
  totalFrameBudgetPass: browserFrameTiming?.totalFrameBudgetPass === true,
  steadyStatePresentationBudgetPass:
    harnessReport?.status?.steady_state_presentation === "pass",
  reconnectSnapshotRebuildBudgetPass:
    harnessReport?.status?.reconnect_snapshot_rebuild === "pass",
  phaseBoundaryPresentationSpikeStatus:
    harnessReport?.status?.phase_boundary_presentation_spike ?? "not_sampled",
  board012BudgetPass:
    browserFrameTiming?.totalFrameBudgetPass === true &&
    harnessReport?.status?.steady_state_presentation === "pass" &&
    harnessReport?.status?.reconnect_snapshot_rebuild === "pass",
};
const harnessConsole = consoleMessages.filter((line) =>
  line.includes("BOARD-012 harness"),
);

await browser.close();

const result = {
  url,
  viewport: { width: 1920, height: 1080 },
  seed: "board-rendering-baseline-v1",
  screenshotPath:
    "production/qa/evidence/captures/board-rendering-baseline-1920x1080.png",
  tracePath:
    "production/qa/evidence/captures/board-rendering-baseline-timing.json",
  browserFrameTiming,
  harnessReport,
  budgetVerdict,
  harnessConsole,
  capturedAt: new Date().toISOString(),
};

await writeFile(tracePath, `${JSON.stringify(result, null, 2)}\n`, "utf8");
console.log(JSON.stringify(result, null, 2));
