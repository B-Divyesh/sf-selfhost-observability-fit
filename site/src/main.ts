import "./style.css";
import { analyzeText, SYNTHETIC_SAMPLE, type BrowserReport } from "./analyzer";

const form = required<HTMLFormElement>("analyzer-form");
const fileInput = required<HTMLInputElement>("sample-file");
const filePrompt = required<HTMLElement>("file-prompt");
const retention = required<HTMLInputElement>("retention");
const retentionOutput = required<HTMLOutputElement>("retention-output");
const headroom = required<HTMLInputElement>("headroom");
const status = required<HTMLElement>("form-status");
const results = required<HTMLElement>("results");
const dropZone = document.querySelector<HTMLElement>(".drop-zone")!;
let sample: { name: string; text: string } | undefined;
let lastReport: BrowserReport | undefined;

retention.addEventListener("input", () => { retentionOutput.value = `${retention.value} days`; });

fileInput.addEventListener("change", async () => {
  const file = fileInput.files?.[0];
  if (file) await chooseFile(file);
});

for (const eventName of ["dragenter", "dragover"]) {
  dropZone.addEventListener(eventName, (event) => { event.preventDefault(); dropZone.classList.add("is-dragging"); });
}
for (const eventName of ["dragleave", "drop"]) {
  dropZone.addEventListener(eventName, (event) => { event.preventDefault(); dropZone.classList.remove("is-dragging"); });
}
dropZone.addEventListener("drop", async (event) => {
  const file = event.dataTransfer?.files[0];
  if (file) await chooseFile(file);
});

required<HTMLButtonElement>("synthetic-button").addEventListener("click", () => {
  sample = { name: "synthetic-garden.otlp.json", text: SYNTHETIC_SAMPLE };
  filePrompt.textContent = sample.name;
  setStatus("Synthetic specimen loaded. Ready to inspect.", "success");
  results.hidden = true;
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  if (!sample) {
    setStatus("Choose a sample or load the synthetic specimen first.", "error");
    fileInput.focus();
    return;
  }
  const retentionDays = Number(retention.value);
  const headroomPercent = Number(headroom.value);
  if (!Number.isFinite(headroomPercent) || headroomPercent < 0 || headroomPercent > 200) {
    setStatus("Headroom must be between 0% and 200%.", "error");
    headroom.focus();
    return;
  }
  setStatus("Reading the specimen…", "loading");
  requestAnimationFrame(() => {
    try {
      lastReport = analyzeText(sample!.text, sample!.name, retentionDays, headroomPercent);
      render(lastReport);
      setStatus(`Analysis complete: ${lastReport.sample.records} records found.`, "success");
    } catch (error) {
      results.hidden = true;
      setStatus(error instanceof Error ? error.message : "The sample could not be read.", "error");
    }
  });
});

required<HTMLButtonElement>("download-button").addEventListener("click", () => {
  if (!lastReport) return;
  const url = URL.createObjectURL(new Blob([JSON.stringify(lastReport, null, 2)], { type: "application/json" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = "obsfit-browser-report.json";
  anchor.click();
  URL.revokeObjectURL(url);
});

document.querySelectorAll<HTMLButtonElement>(".copy-button").forEach((button) => {
  button.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(button.dataset.copy ?? "");
      button.textContent = "Copied";
      setTimeout(() => { button.textContent = "Copy"; }, 1600);
    } catch { button.textContent = "Select command"; }
  });
});

const offlineNote = required<HTMLElement>("offline-note");
const updateOnlineState = () => { offlineNote.hidden = navigator.onLine; };
window.addEventListener("online", updateOnlineState);
window.addEventListener("offline", updateOnlineState);
updateOnlineState();

if ("serviceWorker" in navigator && import.meta.env.PROD) {
  window.addEventListener("load", () => navigator.serviceWorker.register("/sw.js").catch(() => undefined));
}

async function chooseFile(file: File): Promise<void> {
  if (file.size > 5 * 1024 * 1024) {
    sample = undefined;
    setStatus(`${file.name} is over 5 MB. Use the CLI for larger bounded samples.`, "error");
    return;
  }
  try {
    sample = { name: file.name, text: await file.text() };
    filePrompt.textContent = `${file.name} · ${formatBytes(file.size)}`;
    setStatus("Sample selected. Ready to inspect.", "success");
    results.hidden = true;
  } catch {
    setStatus("The browser could not read that file. Choose a local JSON or NDJSON file.", "error");
  }
}

function render(report: BrowserReport): void {
  setText("result-records", report.sample.records.toLocaleString());
  setText("result-window", report.sample.durationSeconds > 0 ? formatDuration(report.sample.durationSeconds) : "No span");
  setText("result-ingest", formatGib(report.rawGibPerDay));
  setText("result-series", report.activeSeries.toLocaleString());
  setText("signal-summary", `${report.signals.traces} / ${report.signals.logs} / ${report.signals.metrics}`);
  const total = report.sample.records;
  setWidth("trace-bar", report.signals.traces / total * 100);
  setWidth("log-bar", report.signals.logs / total * 100);
  setWidth("metric-bar", report.signals.metrics / total * 100);
  const body = required<HTMLTableSectionElement>("profile-body");
  body.replaceChildren(...report.profiles.map((profile) => {
    const row = document.createElement("tr");
    row.innerHTML = `<th scope="row">${escapeHtml(profile.stack)}</th><td data-label="Sample fit"><span class="fit ${profile.fitClass}">${escapeHtml(profile.fit)}</span></td><td data-label="Retained disk">${profile.diskGib} GiB</td><td data-label="vCPU">${profile.vcpu.toFixed(1)}</td><td data-label="Memory">${profile.memoryGib.toFixed(1)} GiB</td>`;
    return row;
  }));
  setText("result-warning", report.warnings.join(" "));
  results.hidden = false;
  results.scrollIntoView({ behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth", block: "start" });
}

function setStatus(message: string, state: "success" | "error" | "loading"): void {
  status.textContent = message;
  status.dataset.state = state;
}
function required<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (!element) throw new Error(`Missing #${id}`);
  return element as T;
}
function setText(id: string, text: string): void { required(id).textContent = text; }
function setWidth(id: string, percent: number): void { required<HTMLElement>(id).style.width = `${percent}%`; }
function formatGib(value: number): string { return value < 0.01 ? `${(value * 1024).toFixed(1)} MiB` : `${value.toFixed(2)} GiB`; }
function formatBytes(value: number): string { return value < 1024 ? `${value} B` : `${(value / 1024).toFixed(1)} KiB`; }
function formatDuration(seconds: number): string { return seconds >= 60 ? `${(seconds / 60).toFixed(1)} min` : `${Math.round(seconds)} sec`; }
function escapeHtml(value: string): string { const node = document.createElement("span"); node.textContent = value; return node.innerHTML; }
