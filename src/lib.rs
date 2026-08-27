//! Workload analysis for bounded OTLP JSON samples.
//!
//! The public surface intentionally consists of one input type and one function:
//!
//! ```
//! use observability_fit::{analyze, AnalysisOptions};
//!
//! let sample = br#"{"resourceLogs":[{"scopeLogs":[{"logRecords":[
//!   {"timeUnixNano":"1000000000","body":{"stringValue":"ready"}}
//! ]}]}]}"#;
//! let report = analyze(sample, &AnalysisOptions::default()).unwrap();
//! assert_eq!(report.workload.total_records, 1);
//! ```

mod model;
mod otlp;
mod output;

pub use model::{
    AnalysisOptions, Cardinality, FitLevel, FitProfile, QueryLoad, Report, SignalSummary, Workload,
};
pub use output::write_plan;

use std::error::Error;
use std::fmt::{Display, Formatter};

/// An actionable analysis or plan-writing failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitError(pub String);

impl Display for FitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for FitError {}

/// Analyze one bounded OTLP JSON or NDJSON byte slice without network access.
pub fn analyze(input: &[u8], options: &AnalysisOptions) -> Result<Report, FitError> {
    options.validate()?;
    otlp::analyze(input, options)
}
