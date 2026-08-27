use crate::{FitError, FitProfile, Report};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Write a reproducible local capacity plan and Compose resource overlays.
pub fn write_plan(directory: &Path, report: &Report) -> Result<(), FitError> {
    fs::create_dir_all(directory).map_err(|error| {
        FitError(format!(
            "could not create plan directory {}: {error}",
            directory.display()
        ))
    })?;
    write(
        &directory.join("report.json"),
        &serde_json::to_string_pretty(report)
            .map_err(|error| FitError(format!("could not serialize report: {error}")))?,
    )?;

    let mut csv = String::from(
        "stack,fit,raw_gib_per_day,estimated_gib_per_day,retention_budget_gib,volume_gib,vcpu,memory_gib\n",
    );
    for profile in &report.profiles {
        writeln!(
            csv,
            "{},{},{:.4},{:.4},{:.2},{},{:.1},{:.1}",
            profile.slug,
            profile.fit.label(),
            report.workload.estimated_raw_gib_per_day,
            profile.estimated_gib_per_day,
            profile.retention_budget_gib,
            profile.volume_gib,
            profile.vcpu,
            profile.memory_gib
        )
        .expect("writing to a string cannot fail");
        write(
            &directory.join(format!("compose.{}.yaml", profile.slug)),
            &compose_overlay(profile),
        )?;
    }
    write(&directory.join("budgets.csv"), &csv)?;
    write(&directory.join("README.md"), &plan_readme(report))?;
    Ok(())
}

fn write(path: &Path, contents: &str) -> Result<(), FitError> {
    fs::write(path, contents)
        .map_err(|error| FitError(format!("could not write {}: {error}", path.display())))
}

fn compose_overlay(profile: &FitProfile) -> String {
    let services: &[(&str, f64, f64)] = match profile.slug.as_str() {
        "grafana-lgtm" => &[("lgtm", 1.0, 1.0)],
        "openobserve" => &[("openobserve", 1.0, 1.0)],
        "signoz" => &[("clickhouse", 0.75, 0.75), ("otel-collector", 0.25, 0.25)],
        "highlight" => &[("clickhouse", 0.75, 0.75), ("otel-collector", 0.25, 0.25)],
        _ => &[("observability", 1.0, 1.0)],
    };
    let mut yaml = format!(
        "# obsfit heuristic resource overlay for {}\n# Merge with the upstream Compose file from:\n# {}\n# Validate with seven days of synthetic replay before production.\nservices:\n",
        profile.stack, profile.upstream_compose_url
    );
    for (service, cpu_share, memory_share) in services {
        let cpu = (profile.vcpu * cpu_share * 2.0).ceil() / 2.0;
        let memory = (profile.memory_gib * memory_share * 2.0).ceil() / 2.0;
        writeln!(
            yaml,
            "  {service}:\n    cpus: \"{cpu:.1}\"\n    mem_limit: \"{memory:.1}g\""
        )
        .expect("writing to a string cannot fail");
    }
    yaml
}

fn plan_readme(report: &Report) -> String {
    let mut text = format!(
        "# obsfit capacity plan\n\nGenerated from a local OTLP sample. These are **heuristic resource overlays**, not standalone deployments. Retention: {} days; growth: {:.1}%/day; headroom: {:.1}%.\n\n",
        report.assumptions.retention_days,
        report.assumptions.growth_percent_per_day,
        report.assumptions.headroom_percent
    );
    text.push_str("For a candidate stack, fetch its upstream Compose project, review its service names, then merge the matching overlay:\n\n```sh\ndocker compose -f upstream-compose.yaml -f /path/to/compose.<stack>.yaml config\ndocker compose -f upstream-compose.yaml -f /path/to/compose.<stack>.yaml up -d\n```\n\nThe overlay constrains CPU and memory. Provision the `volume_gib` value from `budgets.csv` using the upstream stack's documented storage path. Multi-service overlays allocate 75% to ClickHouse and 25% to the collector; other upstream services still need their documented minimums.\n\nDo not send unredacted production data. Replay a synthetic workload for seven days and investigate measured disk growth that differs from the estimate by more than 25%.\n");
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnalysisOptions, analyze};

    #[test]
    fn writes_all_plan_files() {
        let temp = tempfile::tempdir().unwrap();
        let report = analyze(
            br#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[{"timeUnixNano":"1000000000"}]}]}]}"#,
            &AnalysisOptions::default(),
        )
        .unwrap();
        write_plan(temp.path(), &report).unwrap();
        assert!(temp.path().join("report.json").exists());
        assert!(temp.path().join("budgets.csv").exists());
        assert!(temp.path().join("compose.grafana-lgtm.yaml").exists());
    }
}
