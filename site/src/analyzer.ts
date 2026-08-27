export type SignalKind = "traces" | "logs" | "metrics";

export interface BrowserReport {
  heuristic: true;
  sample: { name: string; bytes: number; records: number; durationSeconds: number };
  signals: Record<SignalKind, number>;
  activeSeries: number;
  rawGibPerDay: number;
  retentionDays: number;
  headroomPercent: number;
  profiles: Array<{
    stack: string;
    fit: string;
    fitClass: "good" | "viable" | "caution";
    diskGib: number;
    vcpu: number;
    memoryGib: number;
  }>;
  warnings: string[];
}

const STACKS = [
  { stack: "Grafana LGTM", factor: 0.72, diskFloor: 4, cpu: 2, memory: 4 },
  { stack: "Highlight", factor: 0.82, diskFloor: 10, cpu: 4, memory: 8 },
  { stack: "OpenObserve", factor: 0.48, diskFloor: 2, cpu: 2, memory: 2 },
  { stack: "SigNoz", factor: 0.62, diskFloor: 8, cpu: 4, memory: 8 },
] as const;

interface Evidence {
  traces: number;
  logs: number;
  metrics: number;
  minTime?: bigint;
  maxTime?: bigint;
  series: Set<string>;
}

export function analyzeText(
  text: string,
  name: string,
  retentionDays: number,
  headroomPercent: number,
): BrowserReport {
  if (!text.trim()) throw new Error("The sample is empty. Choose an OTLP JSON or NDJSON export.");
  const envelopes = parseEnvelopes(text);
  const evidence: Evidence = { traces: 0, logs: 0, metrics: 0, series: new Set() };
  for (const envelope of envelopes) walkEnvelope(envelope, evidence);
  const records = evidence.traces + evidence.logs + evidence.metrics;
  if (!records) {
    throw new Error("No OTLP records found. Expected resourceSpans, resourceLogs, or resourceMetrics.");
  }
  const observed = evidence.minTime !== undefined && evidence.maxTime !== undefined
    ? Number(evidence.maxTime - evidence.minTime) / 1e9
    : 0;
  const modeledDuration = Math.max(observed, 60);
  const bytes = new TextEncoder().encode(text).byteLength;
  const rawGibPerDay = (bytes / modeledDuration * 86_400) / 1_073_741_824;
  const scale = Math.max(records / modeledDuration / 1_000, evidence.series.size / 50_000);
  const total = records;
  const profiles = STACKS.map((stack) => {
    const retained = rawGibPerDay * stack.factor * retentionDays * (1 + headroomPercent / 100);
    const fit = fitFor(stack.stack, evidence, total);
    return {
      stack: stack.stack,
      ...fit,
      diskGib: Math.max(1, Math.ceil(retained + stack.diskFloor)),
      vcpu: roundHalf(stack.cpu + scale * 2),
      memoryGib: roundHalf(stack.memory + scale * 4),
    };
  });
  const warnings = ["Heuristic only—validate with a seven-day synthetic replay before production."];
  if (observed < 600) warnings.push(`Only ${Math.round(observed)} seconds were observed; capture at least 10 minutes including a peak.`);
  return {
    heuristic: true,
    sample: { name, bytes, records, durationSeconds: observed },
    signals: { traces: evidence.traces, logs: evidence.logs, metrics: evidence.metrics },
    activeSeries: evidence.series.size,
    rawGibPerDay,
    retentionDays,
    headroomPercent,
    profiles,
    warnings,
  };
}

function parseEnvelopes(text: string): unknown[] {
  try {
    const value: unknown = JSON.parse(text);
    return Array.isArray(value) ? value : [value];
  } catch {
    return text.split(/\r?\n/).filter((line) => line.trim()).map((line, index) => {
      try { return JSON.parse(line) as unknown; }
      catch { throw new Error(`Invalid JSON on NDJSON line ${index + 1}. Export OTLP as JSON and try again.`); }
    });
  }
}

function walkEnvelope(value: unknown, evidence: Evidence): void {
  const root = object(value);
  for (const resource of array(root.resourceSpans)) {
    const resourceObject = object(resource);
    const resourceAttrs = attributes(object(resourceObject.resource).attributes);
    for (const scope of array(resourceObject.scopeSpans)) for (const span of array(object(scope).spans)) {
      const record = object(span);
      evidence.traces += 1;
      observeTime(evidence, record.startTimeUnixNano);
      observeTime(evidence, record.endTimeUnixNano);
      observeSeries(evidence, `trace:${string(record.name, "span")}`, resourceAttrs, attributes(record.attributes));
    }
  }
  for (const resource of array(root.resourceLogs)) {
    const resourceObject = object(resource);
    const resourceAttrs = attributes(object(resourceObject.resource).attributes);
    for (const scope of array(resourceObject.scopeLogs)) for (const log of array(object(scope).logRecords)) {
      const record = object(log);
      evidence.logs += 1;
      observeTime(evidence, record.timeUnixNano ?? record.observedTimeUnixNano);
      observeSeries(evidence, `log:${string(record.severityText, "unknown")}`, resourceAttrs, attributes(record.attributes));
    }
  }
  for (const resource of array(root.resourceMetrics)) {
    const resourceObject = object(resource);
    const resourceAttrs = attributes(object(resourceObject.resource).attributes);
    for (const scope of array(resourceObject.scopeMetrics)) for (const metricValue of array(object(scope).metrics)) {
      const metric = object(metricValue);
      for (const type of ["gauge", "sum", "histogram", "exponentialHistogram", "summary"]) {
        for (const pointValue of array(object(metric[type]).dataPoints)) {
          const point = object(pointValue);
          evidence.metrics += 1;
          observeTime(evidence, point.timeUnixNano);
          observeTime(evidence, point.startTimeUnixNano);
          observeSeries(evidence, `metric:${string(metric.name, "metric")}`, resourceAttrs, attributes(point.attributes));
        }
      }
    }
  }
}

function observeTime(evidence: Evidence, value: unknown): void {
  if (typeof value !== "string" && typeof value !== "number") return;
  try {
    const time = BigInt(value);
    evidence.minTime = evidence.minTime === undefined || time < evidence.minTime ? time : evidence.minTime;
    evidence.maxTime = evidence.maxTime === undefined || time > evidence.maxTime ? time : evidence.maxTime;
  } catch { /* Ignore malformed optional timestamps; the CLI reports more detail. */ }
}

function observeSeries(evidence: Evidence, prefix: string, ...groups: Array<Array<[string, string]>>): void {
  const parts = [prefix, ...groups.flat().map(([key, value]) => `${key}=${value}`)].sort();
  evidence.series.add(parts.join("|"));
}

function attributes(value: unknown): Array<[string, string]> {
  return array(value).flatMap((item) => {
    const attribute = object(item);
    if (typeof attribute.key !== "string") return [];
    const wrapped = object(attribute.value);
    const inner = wrapped.stringValue ?? wrapped.intValue ?? wrapped.doubleValue ?? wrapped.boolValue ?? attribute.value;
    return [[attribute.key, typeof inner === "object" ? JSON.stringify(inner) : String(inner)]];
  });
}

function fitFor(stack: string, e: Evidence, total: number): { fit: string; fitClass: "good" | "viable" | "caution" } {
  if (stack === "Grafana LGTM" && e.traces && e.logs && e.metrics) return { fit: "Good fit · all signals", fitClass: "good" };
  if (stack === "Highlight" && e.traces / total <= 0.5) return { fit: "Validate · not trace-led", fitClass: "caution" };
  if (stack === "OpenObserve" && e.logs / total >= 0.4) return { fit: "Good fit · log-heavy", fitClass: "good" };
  if (stack === "SigNoz" && e.traces / total >= 0.35) return { fit: "Good fit · trace-led", fitClass: "good" };
  return { fit: "Viable · replay first", fitClass: "viable" };
}

function object(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
}
function array(value: unknown): unknown[] { return Array.isArray(value) ? value : []; }
function string(value: unknown, fallback: string): string { return typeof value === "string" ? value : fallback; }
function roundHalf(value: number): number { return Math.ceil(value * 2) / 2; }

export const SYNTHETIC_SAMPLE = JSON.stringify({
  resourceSpans: [{ resource: { attributes: [{ key: "service.name", value: { stringValue: "garden-api" } }] }, scopeSpans: [{ spans: [
    { name: "GET /plants", startTimeUnixNano: "1000000000", endTimeUnixNano: "1900000000", attributes: [{ key: "http.response.status_code", value: { intValue: "200" } }] },
    { name: "SELECT specimen", startTimeUnixNano: "300000000000", endTimeUnixNano: "301000000000", attributes: [{ key: "db.system", value: { stringValue: "postgresql" } }] },
  ] }] }],
  resourceLogs: [{ resource: { attributes: [{ key: "service.name", value: { stringValue: "garden-api" } }] }, scopeLogs: [{ logRecords: [
    { timeUnixNano: "610000000000", severityText: "INFO", body: { stringValue: "synthetic specimen indexed" }, attributes: [{ key: "deployment.environment", value: { stringValue: "test" } }] },
  ] }] }],
  resourceMetrics: [{ resource: { attributes: [{ key: "service.name", value: { stringValue: "garden-api" } }] }, scopeMetrics: [{ metrics: [
    { name: "http.server.request.duration", histogram: { dataPoints: [{ timeUnixNano: "901000000000", count: "4", sum: 0.8, attributes: [{ key: "http.request.method", value: { stringValue: "GET" } }] }] } },
  ] }] }],
}, null, 2);
