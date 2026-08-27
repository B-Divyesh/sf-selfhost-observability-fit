use clap::Parser;
use observability_fit::{AnalysisOptions, Report, analyze, write_plan};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "obsfit",
    version,
    about = "Check an OTLP sample against self-hosted observability resource profiles",
    long_about = "Analyze a bounded, synthetic or redacted OTLP JSON/NDJSON sample locally. Estimates are heuristic planning aids, never vendor rankings or production guarantees."
)]
struct Cli {
    /// OTLP JSON/NDJSON file, or - for stdin
    input: String,

    /// Retention window to model
    #[arg(long, default_value_t = 14, value_parser = clap::value_parser!(u32).range(1..=3650))]
    retention_days: u32,

    /// Expected daily volume growth
    #[arg(long, default_value_t = 0.0, value_name = "PERCENT", value_parser = percent)]
    growth: f64,

    /// Capacity headroom
    #[arg(long, default_value_t = 30.0, value_name = "PERCENT", value_parser = percent)]
    headroom: f64,

    /// Refuse larger samples
    #[arg(long, default_value_t = 50, value_name = "MIB", value_parser = clap::value_parser!(u64).range(1..=1024))]
    max_sample_mib: u64,

    /// Write report.json, budgets.csv, and Compose overlays
    #[arg(long, value_name = "DIR")]
    emit_dir: Option<PathBuf>,

    /// Print only stable JSON to stdout
    #[arg(long)]
    json: bool,
}

fn percent(input: &str) -> Result<f64, String> {
    let value: f64 = input
        .parse()
        .map_err(|_| "must be a number between 0 and 500".to_owned())?;
    if value.is_finite() && (0.0..=500.0).contains(&value) {
        Ok(value)
    } else {
        Err("must be a number between 0 and 500".to_owned())
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err((code, message)) => {
            eprintln!("obsfit: {message}");
            ExitCode::from(code)
        }
    }
}

fn run(cli: &Cli) -> Result<(), (u8, String)> {
    let limit = cli
        .max_sample_mib
        .checked_mul(1024 * 1024)
        .ok_or((2, "sample limit is too large".to_owned()))?;
    let input = read_bounded(&cli.input, limit)?;
    let options = AnalysisOptions {
        retention_days: cli.retention_days,
        growth_percent: cli.growth,
        headroom_percent: cli.headroom,
    };
    let report = analyze(&input, &options).map_err(|error| (2, error.to_string()))?;
    if let Some(directory) = &cli.emit_dir {
        write_plan(directory, &report).map_err(|error| (1, error.to_string()))?;
    }
    if cli.json {
        let json = serde_json::to_string_pretty(&report)
            .map_err(|error| (1, format!("could not serialize report: {error}")))?;
        println!("{json}");
    } else {
        print_report(&report, cli.emit_dir.as_deref());
    }
    Ok(())
}

fn read_bounded(input: &str, limit: u64) -> Result<Vec<u8>, (u8, String)> {
    if input == "-" {
        let mut bytes = Vec::new();
        io::stdin()
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| (1, format!("could not read stdin: {error}")))?;
        return enforce_limit(bytes, limit);
    }
    let path = Path::new(input);
    let metadata = fs::metadata(path)
        .map_err(|error| (1, format!("could not read {}: {error}", path.display())))?;
    if metadata.len() > limit {
        return Err((
            2,
            format!(
                "{} is {:.1} MiB, above the {:.1} MiB safety limit; pass a smaller redacted sample or raise --max-sample-mib",
                path.display(),
                metadata.len() as f64 / 1_048_576.0,
                limit as f64 / 1_048_576.0
            ),
        ));
    }
    fs::read(path).map_err(|error| (1, format!("could not read {}: {error}", path.display())))
}

fn enforce_limit(bytes: Vec<u8>, limit: u64) -> Result<Vec<u8>, (u8, String)> {
    if bytes.len() as u64 > limit {
        Err((
            2,
            format!(
                "stdin exceeded the {:.1} MiB safety limit; pass a smaller redacted sample or raise --max-sample-mib",
                limit as f64 / 1_048_576.0
            ),
        ))
    } else {
        Ok(bytes)
    }
}

fn print_report(report: &Report, emitted: Option<&Path>) {
    let workload = &report.workload;
    println!("OBSERVABILITY FIT CHECK · HEURISTIC\n");
    println!(
        "Sample    {} records over {:.1} min · {:.2} records/s",
        workload.total_records,
        workload.sample_duration_seconds / 60.0,
        workload.records_per_second
    );
    println!(
        "Signals   {} traces · {} logs · {} metric points",
        workload.traces.records, workload.logs.records, workload.metrics.records
    );
    println!(
        "Volume    {:.3} raw GiB/day · {} active label sets · {:?} query load",
        workload.estimated_raw_gib_per_day,
        report.cardinality.estimated_active_series,
        report.query_load.band
    );
    println!(
        "Model     {} days · {:.1}% daily growth · {:.1}% headroom\n",
        report.assumptions.retention_days,
        report.assumptions.growth_percent_per_day,
        report.assumptions.headroom_percent
    );
    println!(
        "{:<16} {:<20} {:>8} {:>9} {:>8} {:>9}",
        "STACK", "FIT", "GiB/DAY", "VOLUME", "vCPU", "MEMORY"
    );
    for profile in &report.profiles {
        println!(
            "{:<16} {:<20} {:>8.2} {:>7}GiB {:>8.1} {:>7.1}GiB",
            profile.stack,
            profile.fit.label(),
            profile.estimated_gib_per_day,
            profile.volume_gib,
            profile.vcpu,
            profile.memory_gib
        );
        println!("  └─ {}", profile.rationale);
    }
    println!("\nCautions");
    for warning in &report.warnings {
        println!("  ! {warning}");
    }
    if let Some(directory) = emitted {
        println!("\nPlan written to {}", directory.display());
    }
}
