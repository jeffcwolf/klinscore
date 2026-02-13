// csv_export.rs
// Export calculation results as CSV

use super::ExportRecord;
use std::io::Write;

/// Export a single calculation result to CSV string
pub fn export_to_csv(record: &ExportRecord) -> Result<String, String> {
    let mut wtr = csv::Writer::from_writer(Vec::new());

    // Write header
    wtr.write_record(["Field", "Value"])
        .map_err(|e| e.to_string())?;

    // Write main result
    wtr.write_record(["Score", &record.score_name])
        .map_err(|e| e.to_string())?;
    wtr.write_record(["Total Score", &record.total_score.to_string()])
        .map_err(|e| e.to_string())?;
    wtr.write_record(["Risk", &record.risk])
        .map_err(|e| e.to_string())?;
    wtr.write_record(["Recommendation", &record.recommendation])
        .map_err(|e| e.to_string())?;
    if !record.details.is_empty() {
        wtr.write_record(["Details", &record.details])
            .map_err(|e| e.to_string())?;
    }
    wtr.write_record(["Timestamp", &record.timestamp])
        .map_err(|e| e.to_string())?;

    // Write field breakdown
    wtr.write_record(["", ""]).map_err(|e| e.to_string())?;
    wtr.write_record(["Factor", "Points"])
        .map_err(|e| e.to_string())?;

    for field in &record.field_breakdown {
        wtr.write_record([&field.label, &field.points.to_string()])
            .map_err(|e| e.to_string())?;
    }

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Export a single result to CSV and write to file
pub fn export_to_csv_file(record: &ExportRecord, path: &str) -> Result<(), String> {
    let csv = export_to_csv(record)?;
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    file.write_all(csv.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ExportFieldEntry;

    #[test]
    fn test_csv_export() {
        let record = ExportRecord {
            score_name: "CHA2DS2-VA Score".to_string(),
            total_score: 3,
            risk: "High Risk".to_string(),
            recommendation: "Anticoagulation recommended".to_string(),
            details: String::new(),
            field_breakdown: vec![
                ExportFieldEntry {
                    field: "age".to_string(),
                    label: "Age 65-74".to_string(),
                    points: 1,
                },
                ExportFieldEntry {
                    field: "hypertension".to_string(),
                    label: "Hypertension".to_string(),
                    points: 1,
                },
                ExportFieldEntry {
                    field: "heart_failure".to_string(),
                    label: "Heart failure".to_string(),
                    points: 1,
                },
            ],
            timestamp: "2026-02-12 10:00:00".to_string(),
        };

        let csv = export_to_csv(&record).unwrap();
        assert!(csv.contains("CHA2DS2-VA Score"));
        assert!(csv.contains("High Risk"));
        assert!(csv.contains("Hypertension"));
        assert!(csv.contains("3"));
    }
}
