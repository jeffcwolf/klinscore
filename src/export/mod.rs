// Export module - CSV, JSON, and PDF export of calculation results

pub mod csv_export;
pub mod json_export;
pub mod pdf_export;

use crate::scores::CalculationResult;
use chrono::Local;
use serde::{Deserialize, Serialize};

/// Flattened export data for a single calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRecord {
    pub score_name: String,
    pub total_score: i32,
    pub risk: String,
    pub recommendation: String,
    pub details: String,
    pub field_breakdown: Vec<ExportFieldEntry>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFieldEntry {
    pub field: String,
    pub label: String,
    pub points: i32,
}

impl ExportRecord {
    pub fn from_result(result: &CalculationResult, score_name: &str, use_german: bool) -> Self {
        let field_breakdown = result
            .field_scores
            .iter()
            .filter(|fs| fs.points != 0)
            .map(|fs| ExportFieldEntry {
                field: fs.field.clone(),
                label: if use_german {
                    fs.label_de.clone()
                } else {
                    fs.label.clone()
                },
                points: fs.points,
            })
            .collect();

        Self {
            score_name: score_name.to_string(),
            total_score: result.total_score,
            risk: if use_german {
                result.risk_de.clone()
            } else {
                result.risk.clone()
            },
            recommendation: if use_german {
                result.recommendation_de.clone()
            } else {
                result.recommendation.clone()
            },
            details: if use_german {
                result.details_de.clone().unwrap_or_default()
            } else {
                result.details.clone().unwrap_or_default()
            },
            field_breakdown,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Generate a default filename for export
pub fn default_filename(score_name: &str, extension: &str) -> String {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let safe_name: String = score_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("klinscore_{}_{}.{}", safe_name, timestamp, extension)
}
