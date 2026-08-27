import { describe, expect, it } from "vitest";
import { analyzeText, SYNTHETIC_SAMPLE } from "../src/analyzer";

describe("browser analyzer", () => {
  it("reads the documented mixed-signal synthetic sample", () => {
    const report = analyzeText(SYNTHETIC_SAMPLE, "synthetic.json", 14, 30);
    expect(report.sample.records).toBe(4);
    expect(report.signals).toEqual({ traces: 2, logs: 1, metrics: 1 });
    expect(report.profiles).toHaveLength(4);
    expect(report.activeSeries).toBe(4);
  });

  it("accepts NDJSON envelopes", () => {
    const line = JSON.stringify({ resourceLogs: [{ scopeLogs: [{ logRecords: [{ timeUnixNano: "1000000000" }] }] }] });
    const report = analyzeText(`${line}\n${line.replace("1000000000", "61000000000")}`, "sample.ndjson", 7, 20);
    expect(report.sample.records).toBe(2);
    expect(report.sample.durationSeconds).toBe(60);
  });

  it("rejects empty, malformed, and non-OTLP samples", () => {
    expect(() => analyzeText("", "empty.json", 14, 30)).toThrow(/empty/i);
    expect(() => analyzeText("{nope", "bad.json", 14, 30)).toThrow(/line 1/i);
    expect(() => analyzeText('{"hello":"world"}', "other.json", 14, 30)).toThrow(/No OTLP/i);
  });
});
