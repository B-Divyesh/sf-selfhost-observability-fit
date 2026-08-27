use observability_fit::{AnalysisOptions, analyze};
use std::process::{Command, Stdio};

const SAMPLE: &[u8] = include_bytes!("fixtures/mixed-otlp.json");

#[test]
fn documented_sample_produces_all_signals_and_profiles() {
    let report = analyze(SAMPLE, &AnalysisOptions::default()).unwrap();
    assert_eq!(report.workload.total_records, 4);
    assert_eq!(report.workload.traces.records, 2);
    assert_eq!(report.workload.logs.records, 1);
    assert_eq!(report.workload.metrics.records, 1);
    assert_eq!(report.profiles.len(), 4);
    assert!(report.cardinality.unique_attribute_keys >= 4);
}

#[test]
fn cli_json_is_scriptable_and_emits_plan() {
    let output_dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_obsfit"))
        .args([
            "tests/fixtures/mixed-otlp.json",
            "--json",
            "--retention-days",
            "7",
            "--emit-dir",
            output_dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], "obsfit.report.v1");
    assert_eq!(report["assumptions"]["retention_days"], 7);
    assert!(output_dir.path().join("budgets.csv").exists());
}

#[test]
fn stdin_ndjson_and_invalid_input_have_documented_exit_codes() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_obsfit"))
        .args(["-", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"{\"resourceLogs\":[{\"scopeLogs\":[{\"logRecords\":[{\"timeUnixNano\":\"1\"}]}]}]}\n{\"resourceLogs\":[{\"scopeLogs\":[{\"logRecords\":[{\"timeUnixNano\":\"60000000001\"}]}]}]}",
        )
        .unwrap();
    assert!(child.wait_with_output().unwrap().status.success());

    let invalid = Command::new(env!("CARGO_BIN_EXE_obsfit"))
        .arg("tests/fixtures/does-not-exist.json")
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
}

#[test]
fn empty_and_non_otlp_samples_are_rejected() {
    assert!(analyze(b"  ", &AnalysisOptions::default()).is_err());
    assert!(analyze(br#"{"hello":"world"}"#, &AnalysisOptions::default()).is_err());
}
