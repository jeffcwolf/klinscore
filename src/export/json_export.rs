// json_export.rs
// Export calculation results as JSON

use super::ExportRecord;
use std::io::Write;

/// Export a single calculation result to JSON string
pub fn export_to_json(record: &ExportRecord) -> Result<String, String> {
    serde_json::to_string_pretty(record).map_err(|e| e.to_string())
}

/// Export a single result to JSON file
pub fn export_to_json_file(record: &ExportRecord, path: &str) -> Result<(), String> {
    let json = export_to_json(record)?;
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    file.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ExportFieldEntry;

    #[test]
    fn test_json_export() {
        let record = ExportRecord {
            score_name: "STOP-BANG Score".to_string(),
            total_score: 5,
            risk: "High Risk for OSA".to_string(),
            recommendation: "Consider sleep study".to_string(),
            details: "Score >=5 has 93% sensitivity".to_string(),
            field_breakdown: vec![ExportFieldEntry {
                field: "snoring".to_string(),
                label: "Loud snoring".to_string(),
                points: 1,
            }],
            timestamp: "2026-02-12 10:00:00".to_string(),
        };

        let json = export_to_json(&record).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["score_name"], "STOP-BANG Score");
        assert_eq!(parsed["total_score"], 5);
        assert_eq!(parsed["field_breakdown"][0]["label"], "Loud snoring");
    }

    #[test]
    fn test_json_roundtrip() {
        let record = ExportRecord {
            score_name: "Test".to_string(),
            total_score: 2,
            risk: "Low".to_string(),
            recommendation: "None".to_string(),
            details: String::new(),
            field_breakdown: vec![],
            timestamp: "2026-02-12 10:00:00".to_string(),
        };

        let json = export_to_json(&record).unwrap();
        let loaded: ExportRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.score_name, "Test");
        assert_eq!(loaded.total_score, 2);
    }
}
