// score_definition.rs
// Core data structures for clinical score definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete definition of a clinical score, loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDefinition {
    /// Score name in English
    pub name: String,

    /// Score name in German
    pub name_de: String,

    /// Medical specialty
    pub specialty: Specialty,

    /// Medical specialty in German
    pub specialty_de: String,

    /// Version of the score definition
    pub version: String,

    /// Source guideline (e.g., "ESC 2024", "KDIGO 2024")
    pub guideline_source: String,

    /// Full reference citation
    pub reference: String,

    /// Validation status (e.g., "peer_reviewed", "draft")
    pub validation_status: String,

    /// Brief description in English
    #[serde(default)]
    pub description: String,

    /// Brief description in German
    #[serde(default)]
    pub description_de: String,

    /// List of input fields for the score
    pub inputs: Vec<InputField>,

    /// Interpretation rules mapping scores to risk categories
    pub interpretation: Vec<InterpretationRule>,

    /// Optional metadata (e.g., tags, keywords)
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Medical specialty classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Specialty {
    Cardiology,
    Nephrology,
    Anesthesiology,
    Emergency,
    InternalMedicine,
    Surgery,
    #[serde(other)]
    Other,
}

impl Specialty {
    /// Get German translation of specialty
    pub fn to_german(&self) -> &'static str {
        match self {
            Specialty::Cardiology => "Kardiologie",
            Specialty::Nephrology => "Nephrologie",
            Specialty::Anesthesiology => "Anästhesiologie",
            Specialty::Emergency => "Notfallmedizin",
            Specialty::InternalMedicine => "Innere Medizin",
            Specialty::Surgery => "Chirurgie",
            Specialty::Other => "Sonstiges",
        }
    }

    /// Get English name of specialty
    pub fn to_english(&self) -> &'static str {
        match self {
            Specialty::Cardiology => "Cardiology",
            Specialty::Nephrology => "Nephrology",
            Specialty::Anesthesiology => "Anesthesiology",
            Specialty::Emergency => "Emergency Medicine",
            Specialty::InternalMedicine => "Internal Medicine",
            Specialty::Surgery => "Surgery",
            Specialty::Other => "Other",
        }
    }
}

/// Definition of a single input field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputField {
    /// Unique identifier for this field (used as key in input map)
    pub field: String,

    /// Type of input
    #[serde(rename = "type")]
    pub input_type: InputType,

    /// Display label in English
    pub label: String,

    /// Display label in German
    pub label_de: String,

    /// Unit of measurement (e.g., "μmol/L", "years", "kg")
    #[serde(default)]
    pub unit: Option<String>,

    /// Unit in German (if different)
    #[serde(default)]
    pub unit_de: Option<String>,

    /// Points assigned based on this input
    pub points: PointsValue,

    /// Optional help text in English
    #[serde(default)]
    pub help: Option<String>,

    /// Optional help text in German
    #[serde(default)]
    pub help_de: Option<String>,

    /// For number inputs: minimum allowed value
    #[serde(default)]
    pub min: Option<f64>,

    /// For number inputs: maximum allowed value
    #[serde(default)]
    pub max: Option<f64>,

    /// For dropdown inputs: available options
    #[serde(default)]
    pub options: Vec<DropdownOption>,

    /// Whether this field is required
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

/// Type of input field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InputType {
    /// Boolean checkbox (yes/no)
    Boolean,

    /// Numeric input (integer or decimal)
    Number,

    /// Dropdown selection
    Dropdown,
}

/// Points value - can be fixed or conditional
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointsValue {
    /// Fixed point value (e.g., for boolean: true = 1 point)
    Fixed(i32),

    /// Conditional points based on value ranges
    Conditional(Vec<PointCondition>),
}

/// Conditional point assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCondition {
    /// Condition to check (e.g., ">= 65", "< 50", "== true")
    pub condition: String,

    /// Points awarded if condition is true
    pub points: i32,

    /// Optional label for this condition (e.g., "Age 65-74")
    #[serde(default)]
    pub label: Option<String>,

    /// Optional German label
    #[serde(default)]
    pub label_de: Option<String>,
}

/// Option for dropdown inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownOption {
    /// Internal value identifier
    pub value: String,

    /// Display label in English
    pub label: String,

    /// Display label in German
    pub label_de: String,

    /// Points awarded for this selection
    pub points: i32,

    /// Optional description in English
    #[serde(default)]
    pub description: Option<String>,

    /// Optional description in German
    #[serde(default)]
    pub description_de: Option<String>,
}

/// Interpretation rule mapping score to risk category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterpretationRule {
    /// Score value or range (e.g., 0, "0-1", "≥2")
    pub score: ScoreRange,

    /// Risk level in English
    pub risk: String,

    /// Risk level in German
    pub risk_de: String,

    /// Risk category for color coding
    pub risk_level: RiskLevel,

    /// Clinical recommendation in English
    pub recommendation: String,

    /// Clinical recommendation in German
    pub recommendation_de: String,

    /// Optional additional information in English
    #[serde(default)]
    pub details: Option<String>,

    /// Optional additional information in German
    #[serde(default)]
    pub details_de: Option<String>,
}

/// Score range for interpretation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScoreRange {
    /// Exact score value
    Exact(i32),

    /// Range string (e.g., "0-2", "≥3", "<5")
    Range(String),
}

/// Risk level for color coding and categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RiskLevel {
    /// Very low risk (green)
    VeryLow,

    /// Low risk (light green)
    Low,

    /// Moderate/medium risk (yellow)
    Moderate,

    /// High risk (orange)
    High,

    /// Very high risk (red)
    VeryHigh,

    /// Critical risk (dark red)
    Critical,

    /// Not applicable or informational
    None,
}

impl RiskLevel {
    /// Get color for UI display (hex color code)
    #[allow(dead_code)]
    pub fn color(&self) -> &'static str {
        match self {
            RiskLevel::VeryLow => "#4CAF50",  // Green
            RiskLevel::Low => "#8BC34A",      // Light green
            RiskLevel::Moderate => "#FFC107", // Yellow/amber
            RiskLevel::High => "#FF9800",     // Orange
            RiskLevel::VeryHigh => "#F44336", // Red
            RiskLevel::Critical => "#B71C1C", // Dark red
            RiskLevel::None => "#9E9E9E",     // Gray
        }
    }

    /// Get RGB color tuple for UI display
    pub fn rgb(&self) -> (f32, f32, f32) {
        match self {
            RiskLevel::VeryLow => (0.298, 0.686, 0.314),
            RiskLevel::Low => (0.545, 0.765, 0.290),
            RiskLevel::Moderate => (1.0, 0.757, 0.027),
            RiskLevel::High => (1.0, 0.596, 0.0),
            RiskLevel::VeryHigh => (0.957, 0.263, 0.212),
            RiskLevel::Critical => (0.718, 0.110, 0.110),
            RiskLevel::None => (0.620, 0.620, 0.620),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specialty_translations() {
        assert_eq!(Specialty::Cardiology.to_german(), "Kardiologie");
        assert_eq!(Specialty::Cardiology.to_english(), "Cardiology");
        assert_eq!(Specialty::Nephrology.to_german(), "Nephrologie");
    }

    #[test]
    fn test_risk_level_colors() {
        assert_eq!(RiskLevel::Low.color(), "#8BC34A");
        assert_eq!(RiskLevel::High.color(), "#FF9800");

        let (r, g, b) = RiskLevel::VeryHigh.rgb();
        assert!((r - 0.957).abs() < 0.001);
    }

    #[test]
    fn test_score_definition_serde() {
        let yaml = r#"
name: "Test Score"
name_de: "Test-Score"
specialty: Cardiology
specialty_de: "Kardiologie"
version: "1.0"
guideline_source: "Test 2024"
reference: "Test et al."
validation_status: "draft"
inputs:
  - field: "age"
    type: "number"
    label: "Age"
    label_de: "Alter"
    unit: "years"
    points: 1
    min: 0
    max: 120
interpretation:
  - score: 0
    risk: "Low"
    risk_de: "Niedrig"
    risk_level: Low
    recommendation: "No action needed"
    recommendation_de: "Keine Maßnahmen erforderlich"
"#;

        let score: Result<ScoreDefinition, _> = serde_yaml::from_str(yaml);
        assert!(score.is_ok());

        let score = score.unwrap();
        assert_eq!(score.name, "Test Score");
        assert_eq!(score.specialty, Specialty::Cardiology);
        assert_eq!(score.inputs.len(), 1);
        assert_eq!(score.inputs[0].field, "age");
        assert_eq!(score.interpretation.len(), 1);
    }

    #[test]
    fn test_conditional_points_parsing() {
        let yaml = r#"
name: "Age Test"
name_de: "Alter Test"
specialty: Cardiology
specialty_de: "Kardiologie"
version: "1.0"
guideline_source: "Test"
reference: "Test"
validation_status: "draft"
inputs:
  - field: "age"
    type: "number"
    label: "Age"
    label_de: "Alter"
    points:
      - condition: ">= 75"
        points: 2
      - condition: ">= 65"
        points: 1
interpretation:
  - score: 0
    risk: "Low"
    risk_de: "Niedrig"
    risk_level: Low
    recommendation: "Test"
    recommendation_de: "Test"
"#;

        let score: Result<ScoreDefinition, _> = serde_yaml::from_str(yaml);
        assert!(score.is_ok());

        let score = score.unwrap();
        if let PointsValue::Conditional(conditions) = &score.inputs[0].points {
            assert_eq!(conditions.len(), 2);
            assert_eq!(conditions[0].condition, ">= 75");
            assert_eq!(conditions[0].points, 2);
            assert_eq!(conditions[1].condition, ">= 65");
            assert_eq!(conditions[1].points, 1);
        } else {
            panic!("Expected conditional points");
        }
    }

    #[test]
    fn test_dropdown_options_parsing() {
        let yaml = r#"
name: "Dropdown Test"
name_de: "Dropdown Test"
specialty: Cardiology
specialty_de: "Kardiologie"
version: "1.0"
guideline_source: "Test"
reference: "Test"
validation_status: "draft"
inputs:
  - field: "severity"
    type: "dropdown"
    label: "Severity"
    label_de: "Schweregrad"
    points: 0
    options:
      - value: "mild"
        label: "Mild"
        label_de: "Leicht"
        points: 0
      - value: "severe"
        label: "Severe"
        label_de: "Schwer"
        points: 2
interpretation:
  - score: 0
    risk: "Low"
    risk_de: "Niedrig"
    risk_level: Low
    recommendation: "Test"
    recommendation_de: "Test"
"#;

        let score: Result<ScoreDefinition, _> = serde_yaml::from_str(yaml);
        assert!(score.is_ok());

        let score = score.unwrap();
        assert_eq!(score.inputs[0].options.len(), 2);
        assert_eq!(score.inputs[0].options[0].value, "mild");
        assert_eq!(score.inputs[0].options[0].points, 0);
        assert_eq!(score.inputs[0].options[1].value, "severe");
        assert_eq!(score.inputs[0].options[1].points, 2);
    }
}
