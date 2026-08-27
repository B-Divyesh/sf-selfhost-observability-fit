use crate::FitError;
use crate::model::{
    AnalysisOptions, Assumptions, Cardinality, CardinalityKey, FitLevel, FitProfile, QueryLoad,
    QueryLoadBand, Report, SignalSummary, Workload,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
enum Signal {
    Trace,
    Log,
    Metric,
}

#[derive(Default)]
struct Evidence {
    traces: u64,
    logs: u64,
    metrics: u64,
    min_time: Option<u64>,
    max_time: Option<u64>,
    attributes: HashMap<String, HashSet<String>>,
    series: HashSet<String>,
}

impl Evidence {
    fn observe_time(&mut self, value: Option<&Value>) {
        let Some(timestamp) = value.and_then(parse_u64) else {
            return;
        };
        self.min_time = Some(self.min_time.map_or(timestamp, |old| old.min(timestamp)));
        self.max_time = Some(self.max_time.map_or(timestamp, |old| old.max(timestamp)));
    }

    fn observe_attributes(&mut self, arrays: &[Option<&Vec<Value>>], prefix: &str) {
        let mut series_parts = vec![prefix.to_owned()];
        for array in arrays.iter().flatten() {
            for attribute in *array {
                let Some(key) = attribute.get("key").and_then(Value::as_str) else {
                    continue;
                };
                let value = attribute
                    .get("value")
                    .map(compact_value)
                    .unwrap_or_else(|| "null".to_owned());
                self.attributes
                    .entry(key.to_owned())
                    .or_default()
                    .insert(value.clone());
                series_parts.push(format!("{key}={value}"));
            }
        }
        series_parts.sort();
        self.series.insert(series_parts.join("|"));
    }

    fn increment(&mut self, signal: Signal) {
        match signal {
            Signal::Trace => self.traces += 1,
            Signal::Log => self.logs += 1,
            Signal::Metric => self.metrics += 1,
        }
    }
}

pub(crate) fn analyze(input: &[u8], options: &AnalysisOptions) -> Result<Report, FitError> {
    if input.iter().all(u8::is_ascii_whitespace) {
        return Err(FitError(
            "the sample is empty; provide an OTLP JSON or NDJSON export".into(),
        ));
    }
    let envelopes = parse_envelopes(input)?;
    let mut evidence = Evidence::default();
    for envelope in &envelopes {
        walk_envelope(envelope, &mut evidence);
    }
    let total = evidence.traces + evidence.logs + evidence.metrics;
    if total == 0 {
        return Err(FitError(
            "no OTLP spans, log records, or metric data points were found; expected resourceSpans, resourceLogs, or resourceMetrics"
                .into(),
        ));
    }

    let observed_seconds = evidence
        .min_time
        .zip(evidence.max_time)
        .map(|(min, max)| max.saturating_sub(min) as f64 / 1_000_000_000.0)
        .unwrap_or_default();
    let duration_seconds = observed_seconds.max(60.0);
    let records_per_second = total as f64 / duration_seconds;
    let raw_bytes_per_day = input.len() as f64 / duration_seconds * 86_400.0;
    let raw_gib_per_day = raw_bytes_per_day / 1_073_741_824.0;

    let mut keys: Vec<_> = evidence
        .attributes
        .iter()
        .map(|(key, values)| CardinalityKey {
            key: key.clone(),
            distinct_values: values.len(),
        })
        .collect();
    keys.sort_by(|a, b| {
        b.distinct_values
            .cmp(&a.distinct_values)
            .then_with(|| a.key.cmp(&b.key))
    });
    let pair_count = keys.iter().map(|key| key.distinct_values).sum();
    let active_series = evidence.series.len();
    let query_score = records_per_second * (1.0 + (active_series.max(1) as f64).log10());
    let (query_band, explanation) = if query_score < 100.0 {
        (
            QueryLoadBand::Light,
            "Low event rate and label-set fan-out; interactive single-node queries should be modest.",
        )
    } else if query_score < 2_000.0 {
        (
            QueryLoadBand::Moderate,
            "Either event rate or label-set fan-out will make concurrent exploratory queries noticeable.",
        )
    } else {
        (
            QueryLoadBand::Heavy,
            "High ingest multiplied by label-set fan-out calls for query isolation and replay testing.",
        )
    };

    let mut warnings = vec![
        "Heuristic only: validate disk growth with a seven-day synthetic replay before production."
            .to_owned(),
        "Attribute values may contain secrets or personal data; analyze only synthetic or redacted samples."
            .to_owned(),
    ];
    if observed_seconds < 600.0 {
        warnings.push(format!(
            "The observed window is only {:.0} seconds; capture at least 10 minutes including a traffic peak.",
            observed_seconds
        ));
    }
    if active_series > 10_000 {
        warnings.push(
            "More than 10,000 distinct label sets were observed; inspect high-cardinality attributes before sizing."
                .to_owned(),
        );
    }

    let profiles = build_profiles(
        raw_gib_per_day,
        records_per_second,
        active_series,
        evidence.traces,
        evidence.logs,
        evidence.metrics,
        options,
    );

    Ok(Report {
        schema_version: "obsfit.report.v1".to_owned(),
        heuristic: true,
        workload: Workload {
            sample_bytes: input.len() as u64,
            total_records: total,
            sample_duration_seconds: observed_seconds,
            records_per_second,
            estimated_raw_gib_per_day: raw_gib_per_day,
            traces: summary(evidence.traces, total),
            logs: summary(evidence.logs, total),
            metrics: summary(evidence.metrics, total),
        },
        cardinality: Cardinality {
            unique_attribute_keys: keys.len(),
            unique_attribute_pairs: pair_count,
            estimated_active_series: active_series,
            highest_cardinality_keys: keys.into_iter().take(8).collect(),
        },
        query_load: QueryLoad {
            band: query_band,
            score: query_score,
            explanation: explanation.to_owned(),
        },
        assumptions: Assumptions {
            retention_days: options.retention_days,
            growth_percent_per_day: options.growth_percent,
            headroom_percent: options.headroom_percent,
            minimum_sample_window_seconds: 60,
            model: "sample byte-rate × signal-aware compression/index factor × geometric daily growth × replicas × headroom + stack floor".to_owned(),
        },
        profiles,
        warnings,
        validation_plan: vec![
            "Replay redacted telemetry at the measured peak rate for seven days.".to_owned(),
            "Record daily volume growth and compare it with the profile budget; investigate a variance above 25%.".to_owned(),
            "Run one representative trace, log, and metric query during ingest and record p95 latency.".to_owned(),
            "Confirm compaction, retention deletion, and restart recovery before choosing the profile.".to_owned(),
        ],
    })
}

fn parse_envelopes(input: &[u8]) -> Result<Vec<Value>, FitError> {
    if let Ok(value) = serde_json::from_slice::<Value>(input) {
        return Ok(match value {
            Value::Array(values) => values,
            value => vec![value],
        });
    }
    let text = std::str::from_utf8(input)
        .map_err(|_| FitError("sample is not valid UTF-8 JSON".into()))?;
    let mut values = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).map_err(|error| {
            FitError(format!(
                "invalid JSON on NDJSON line {}: {error}",
                index + 1
            ))
        })?);
    }
    if values.is_empty() {
        Err(FitError("the sample contains no JSON values".into()))
    } else {
        Ok(values)
    }
}

fn walk_envelope(root: &Value, evidence: &mut Evidence) {
    for resource in array(root, "resourceSpans") {
        let resource_attrs = attributes(resource.get("resource"));
        for scope in array(resource, "scopeSpans") {
            for record in array(scope, "spans") {
                evidence.increment(Signal::Trace);
                evidence.observe_time(record.get("startTimeUnixNano"));
                evidence.observe_time(record.get("endTimeUnixNano"));
                let own = record.get("attributes").and_then(Value::as_array);
                let name = record.get("name").and_then(Value::as_str).unwrap_or("span");
                evidence.observe_attributes(&[resource_attrs, own], &format!("trace:{name}"));
            }
        }
    }
    for resource in array(root, "resourceLogs") {
        let resource_attrs = attributes(resource.get("resource"));
        for scope in array(resource, "scopeLogs") {
            for record in array(scope, "logRecords") {
                evidence.increment(Signal::Log);
                evidence.observe_time(
                    record
                        .get("timeUnixNano")
                        .or_else(|| record.get("observedTimeUnixNano")),
                );
                let own = record.get("attributes").and_then(Value::as_array);
                let severity = record
                    .get("severityText")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                evidence.observe_attributes(&[resource_attrs, own], &format!("log:{severity}"));
            }
        }
    }
    for resource in array(root, "resourceMetrics") {
        let resource_attrs = attributes(resource.get("resource"));
        for scope in array(resource, "scopeMetrics") {
            for metric in array(scope, "metrics") {
                let name = metric
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("metric");
                for metric_type in [
                    "gauge",
                    "sum",
                    "histogram",
                    "exponentialHistogram",
                    "summary",
                ] {
                    for point in metric
                        .get(metric_type)
                        .into_iter()
                        .flat_map(|body| array(body, "dataPoints"))
                    {
                        evidence.increment(Signal::Metric);
                        evidence.observe_time(point.get("timeUnixNano"));
                        evidence.observe_time(point.get("startTimeUnixNano"));
                        let own = point.get("attributes").and_then(Value::as_array);
                        evidence
                            .observe_attributes(&[resource_attrs, own], &format!("metric:{name}"));
                    }
                }
            }
        }
    }
}

fn array<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn attributes(value: Option<&Value>) -> Option<&Vec<Value>> {
    value?.get("attributes")?.as_array()
}

fn parse_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str().and_then(|number| number.parse().ok()))
}

fn compact_value(value: &Value) -> String {
    if let Some(object) = value.as_object() {
        for key in [
            "stringValue",
            "intValue",
            "doubleValue",
            "boolValue",
            "bytesValue",
        ] {
            if let Some(inner) = object.get(key) {
                return inner
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| inner.to_string());
            }
        }
    }
    value.to_string()
}

fn summary(records: u64, total: u64) -> SignalSummary {
    SignalSummary {
        records,
        share_percent: records as f64 / total as f64 * 100.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_profiles(
    raw_gib_day: f64,
    records_second: f64,
    active_series: usize,
    traces: u64,
    logs: u64,
    metrics: u64,
    options: &AnalysisOptions,
) -> Vec<FitProfile> {
    struct Stack {
        slug: &'static str,
        name: &'static str,
        factor: f64,
        replicas: u8,
        floor_disk: f64,
        floor_cpu: f64,
        floor_memory: f64,
        url: &'static str,
    }
    let stacks = [
        Stack {
            slug: "grafana-lgtm",
            name: "Grafana LGTM",
            factor: 0.72,
            replicas: 1,
            floor_disk: 4.0,
            floor_cpu: 2.0,
            floor_memory: 4.0,
            url: "https://github.com/grafana/docker-otel-lgtm",
        },
        Stack {
            slug: "highlight",
            name: "Highlight",
            factor: 0.82,
            replicas: 1,
            floor_disk: 10.0,
            floor_cpu: 4.0,
            floor_memory: 8.0,
            url: "https://github.com/highlight/highlight/tree/main/docker",
        },
        Stack {
            slug: "openobserve",
            name: "OpenObserve",
            factor: 0.48,
            replicas: 1,
            floor_disk: 2.0,
            floor_cpu: 2.0,
            floor_memory: 2.0,
            url: "https://github.com/openobserve/openobserve",
        },
        Stack {
            slug: "signoz",
            name: "SigNoz",
            factor: 0.62,
            replicas: 1,
            floor_disk: 8.0,
            floor_cpu: 4.0,
            floor_memory: 8.0,
            url: "https://github.com/SigNoz/signoz/tree/main/deploy/docker",
        },
    ];
    let grown_days = geometric_days(options.retention_days, options.growth_percent / 100.0);
    let headroom = 1.0 + options.headroom_percent / 100.0;
    let scale = (records_second / 1_000.0).max(active_series as f64 / 50_000.0);
    let total = (traces + logs + metrics) as f64;

    stacks
        .into_iter()
        .map(|stack| {
            let daily = raw_gib_day * stack.factor;
            let retained = daily * grown_days * f64::from(stack.replicas) * headroom;
            let volume = (retained + stack.floor_disk).ceil().max(1.0) as u64;
            let vcpu = round_half(stack.floor_cpu + scale * 2.0);
            let memory = round_half(stack.floor_memory + scale * 4.0);
            let (fit, rationale) =
                profile_fit(stack.slug, traces, logs, metrics, total, active_series);
            FitProfile {
                slug: stack.slug.to_owned(),
                stack: stack.name.to_owned(),
                fit,
                rationale,
                compression_index_factor: stack.factor,
                replicas: stack.replicas,
                estimated_gib_per_day: daily,
                retention_budget_gib: retained,
                volume_gib: volume,
                vcpu,
                memory_gib: memory,
                upstream_compose_url: stack.url.to_owned(),
            }
        })
        .collect()
}

fn profile_fit(
    slug: &str,
    traces: u64,
    logs: u64,
    metrics: u64,
    total: f64,
    series: usize,
) -> (FitLevel, String) {
    let trace_share = traces as f64 / total;
    let log_share = logs as f64 / total;
    let metric_share = metrics as f64 / total;
    match slug {
        "grafana-lgtm" if traces > 0 && logs > 0 && metrics > 0 => (
            FitLevel::GoodFit,
            "All three signals are present; the integrated LGTM distribution matches the sample breadth."
                .to_owned(),
        ),
        "highlight" if trace_share > 0.5 => (
            FitLevel::Viable,
            "Trace-heavy application telemetry is compatible; validate the larger operational floor and any session-replay needs separately."
                .to_owned(),
        ),
        "highlight" => (
            FitLevel::ValidateCarefully,
            "The sample is not trace-dominant and contains no session-replay evidence; validate Highlight-specific product value before accepting its operational floor."
                .to_owned(),
        ),
        "openobserve" if log_share >= 0.4 => (
            FitLevel::GoodFit,
            "Logs are a large share of the sample; the compact single-node profile is a useful first replay target."
                .to_owned(),
        ),
        "openobserve" => (
            FitLevel::Viable,
            "The unified single-node shape fits a small team; exercise the sample's trace and metric queries during replay."
                .to_owned(),
        ),
        "signoz" if trace_share >= 0.35 => (
            FitLevel::GoodFit,
            "The trace share makes SigNoz's application-performance workflow a natural replay candidate."
                .to_owned(),
        ),
        "signoz" if series > 10_000 && metric_share > 0.3 => (
            FitLevel::ValidateCarefully,
            "Metric label-set fan-out is high; validate ClickHouse parts, merges, and query latency under replay."
                .to_owned(),
        ),
        "signoz" => (
            FitLevel::Viable,
            "The workload fits the general OTLP model; validate the multi-service operational floor on the target host."
                .to_owned(),
        ),
        _ => (
            FitLevel::Viable,
            "The sample is compatible; validate per-signal query latency and component memory during replay."
                .to_owned(),
        ),
    }
}

fn geometric_days(days: u32, growth: f64) -> f64 {
    if growth == 0.0 {
        f64::from(days)
    } else {
        ((1.0 + growth).powf(f64::from(days)) - 1.0) / growth
    }
}

fn round_half(value: f64) -> f64 {
    (value * 2.0).ceil() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometric_growth_is_compounded() {
        assert!((geometric_days(2, 0.1) - 2.1).abs() < 0.0001);
    }

    #[test]
    fn compact_otlp_values_are_readable() {
        let value = serde_json::json!({"stringValue": "api"});
        assert_eq!(compact_value(&value), "api");
    }
}
