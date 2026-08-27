use crate::FitError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct AnalysisOptions {
    pub retention_days: u32,
    pub growth_percent: f64,
    pub headroom_percent: f64,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            retention_days: 14,
            growth_percent: 0.0,
            headroom_percent: 30.0,
        }
    }
}

impl AnalysisOptions {
    pub(crate) fn validate(&self) -> Result<(), FitError> {
        if !(1..=3650).contains(&self.retention_days) {
            return Err(FitError("retention days must be between 1 and 3650".into()));
        }
        if !self.growth_percent.is_finite() || !(0.0..=500.0).contains(&self.growth_percent) {
            return Err(FitError("growth percent must be between 0 and 500".into()));
        }
        if !self.headroom_percent.is_finite() || !(0.0..=500.0).contains(&self.headroom_percent) {
            return Err(FitError(
                "headroom percent must be between 0 and 500".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: String,
    pub heuristic: bool,
    pub workload: Workload,
    pub cardinality: Cardinality,
    pub query_load: QueryLoad,
    pub assumptions: Assumptions,
    pub profiles: Vec<FitProfile>,
    pub warnings: Vec<String>,
    pub validation_plan: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workload {
    pub sample_bytes: u64,
    pub total_records: u64,
    pub sample_duration_seconds: f64,
    pub records_per_second: f64,
    pub estimated_raw_gib_per_day: f64,
    pub traces: SignalSummary,
    pub logs: SignalSummary,
    pub metrics: SignalSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalSummary {
    pub records: u64,
    pub share_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cardinality {
    pub unique_attribute_keys: usize,
    pub unique_attribute_pairs: usize,
    pub estimated_active_series: usize,
    pub highest_cardinality_keys: Vec<CardinalityKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardinalityKey {
    pub key: String,
    pub distinct_values: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryLoadBand {
    Light,
    Moderate,
    Heavy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLoad {
    pub band: QueryLoadBand,
    pub score: f64,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumptions {
    pub retention_days: u32,
    pub growth_percent_per_day: f64,
    pub headroom_percent: f64,
    pub minimum_sample_window_seconds: u32,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitLevel {
    GoodFit,
    Viable,
    ValidateCarefully,
}

impl FitLevel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::GoodFit => "good fit",
            Self::Viable => "viable",
            Self::ValidateCarefully => "validate carefully",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitProfile {
    pub slug: String,
    pub stack: String,
    pub fit: FitLevel,
    pub rationale: String,
    pub compression_index_factor: f64,
    pub replicas: u8,
    pub estimated_gib_per_day: f64,
    pub retention_budget_gib: f64,
    pub volume_gib: u64,
    pub vcpu: f64,
    pub memory_gib: f64,
    pub upstream_compose_url: String,
}
