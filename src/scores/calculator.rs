// calculator.rs
// Generic calculation engine for clinical scores

use crate::config::{
    InputField, InputType, InterpretationRule, PointCondition, PointsValue, RiskLevel,
    ScoreDefinition, ScoreRange,
};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during score calculation
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CalculationError {
    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },

    #[error("Invalid input value for field '{field}': {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Field '{field}' is out of range: {value} (allowed: {min} - {max})")]
    OutOfRange {
        field: String,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("Unknown dropdown option '{option}' for field '{field}'")]
    UnknownDropdownOption { field: String, option: String },

    #[error("Failed to parse condition '{condition}': {reason}")]
    ConditionParseError { condition: String, reason: String },

    #[error("No interpretation found for score {score}")]
    NoInterpretation { score: i32 },
}

/// Input value types
#[derive(Debug, Clone, PartialEq)]
pub enum InputValue {
    Boolean(bool),
    Number(f64),
    Dropdown(String),
}

impl InputValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            InputValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            InputValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            InputValue::Dropdown(s) => Some(s),
            _ => None,
        }
    }
}

/// Points breakdown for a single field
#[derive(Debug, Clone, PartialEq)]
pub struct FieldScore {
    pub field: String,
    pub label: String,
    pub label_de: String,
    pub points: i32,
}

/// Result of a score calculation
#[derive(Debug, Clone, PartialEq)]
pub struct CalculationResult {
    /// Total calculated score
    pub total_score: i32,

    /// Ordered breakdown of points by field (preserves YAML definition order)
    pub field_scores: Vec<FieldScore>,

    /// Matched interpretation rule
    pub interpretation: InterpretationRule,

    /// Risk level (for color coding)
    pub risk_level: RiskLevel,

    /// Risk description (English)
    pub risk: String,

    /// Risk description (German)
    pub risk_de: String,

    /// Clinical recommendation (English)
    pub recommendation: String,

    /// Clinical recommendation (German)
    pub recommendation_de: String,

    /// Optional details (English)
    pub details: Option<String>,

    /// Optional details (German)
    pub details_de: Option<String>,
}

impl CalculationResult {
    /// Get points for a field by name (for testing)
    #[allow(dead_code)]
    pub fn get_field_points(&self, field_name: &str) -> Option<i32> {
        self.field_scores
            .iter()
            .find(|fs| fs.field == field_name)
            .map(|fs| fs.points)
    }
}

/// Calculate a score based on user inputs
///
/// This is the core calculation engine. It:
/// 1. Validates all inputs
/// 2. Calculates points for each field
/// 3. Sums the total score
/// 4. Finds the matching interpretation rule
/// 5. Returns the complete result
///
/// # Arguments
///
/// * `score_def` - The score definition (from YAML)
/// * `inputs` - User-provided input values
///
/// # Returns
///
/// `CalculationResult` with total score, breakdown, and interpretation
///
/// # Errors
///
/// Returns `CalculationError` if:
/// - Required fields are missing
/// - Input values are invalid
/// - Numbers are out of range
/// - Dropdown options don't exist
/// - No interpretation matches the calculated score
pub fn calculate_score(
    score_def: &ScoreDefinition,
    inputs: &HashMap<String, InputValue>,
) -> Result<CalculationResult, CalculationError> {
    // If this score uses a formula, dispatch to formula engine
    if let Some(ref formula) = score_def.formula {
        return calculate_formula_score(score_def, inputs, formula);
    }

    let mut total_score = 0;
    let mut field_scores = Vec::new();

    // Calculate points for each input field
    for input_field in &score_def.inputs {
        let field_name = &input_field.field;

        // Check if required field is present
        if input_field.required && !inputs.contains_key(field_name) {
            return Err(CalculationError::MissingRequiredField {
                field: field_name.clone(),
            });
        }

        // Get input value
        let points = match inputs.get(field_name) {
            Some(input_value) => calculate_field_points(input_field, input_value)?,
            None => 0, // Field not provided (e.g., unchecked boolean) = 0 points
        };

        field_scores.push(FieldScore {
            field: field_name.clone(),
            label: input_field.label.clone(),
            label_de: input_field.label_de.clone(),
            points,
        });
        total_score += points;
    }

    // Find matching interpretation
    let interpretation = find_interpretation(score_def, total_score)?;

    Ok(CalculationResult {
        total_score,
        field_scores,
        risk_level: interpretation.risk_level,
        risk: interpretation.risk.clone(),
        risk_de: interpretation.risk_de.clone(),
        recommendation: interpretation.recommendation.clone(),
        recommendation_de: interpretation.recommendation_de.clone(),
        details: interpretation.details.clone(),
        details_de: interpretation.details_de.clone(),
        interpretation: interpretation.clone(),
    })
}

/// Calculate points for a single input field
fn calculate_field_points(
    input_field: &InputField,
    input_value: &InputValue,
) -> Result<i32, CalculationError> {
    match input_field.input_type {
        InputType::Boolean => calculate_boolean_points(input_field, input_value),
        InputType::Number => calculate_number_points(input_field, input_value),
        InputType::Dropdown => calculate_dropdown_points(input_field, input_value),
    }
}

/// Calculate points for a boolean input
fn calculate_boolean_points(
    input_field: &InputField,
    input_value: &InputValue,
) -> Result<i32, CalculationError> {
    let value = input_value
        .as_bool()
        .ok_or_else(|| CalculationError::InvalidInput {
            field: input_field.field.clone(),
            reason: "Expected boolean value".to_string(),
        })?;

    if value {
        match &input_field.points {
            PointsValue::Fixed(points) => Ok(*points),
            PointsValue::Conditional(_) => {
                // For boolean, conditional doesn't make sense, but handle it
                Ok(0)
            }
        }
    } else {
        Ok(0)
    }
}

/// Calculate points for a numeric input
fn calculate_number_points(
    input_field: &InputField,
    input_value: &InputValue,
) -> Result<i32, CalculationError> {
    let value = input_value
        .as_number()
        .ok_or_else(|| CalculationError::InvalidInput {
            field: input_field.field.clone(),
            reason: "Expected numeric value".to_string(),
        })?;

    // Validate range if specified
    if let Some(min) = input_field.min {
        if value < min {
            return Err(CalculationError::OutOfRange {
                field: input_field.field.clone(),
                value,
                min,
                max: input_field.max.unwrap_or(f64::MAX),
            });
        }
    }

    if let Some(max) = input_field.max {
        if value > max {
            return Err(CalculationError::OutOfRange {
                field: input_field.field.clone(),
                value,
                min: input_field.min.unwrap_or(0.0),
                max,
            });
        }
    }

    // Calculate points based on value
    match &input_field.points {
        PointsValue::Fixed(points) => Ok(*points),
        PointsValue::Conditional(conditions) => evaluate_conditions(conditions, value),
    }
}

/// Calculate points for a dropdown input
fn calculate_dropdown_points(
    input_field: &InputField,
    input_value: &InputValue,
) -> Result<i32, CalculationError> {
    let selected = input_value
        .as_string()
        .ok_or_else(|| CalculationError::InvalidInput {
            field: input_field.field.clone(),
            reason: "Expected dropdown/string value".to_string(),
        })?;

    // Find the matching option
    let option = input_field
        .options
        .iter()
        .find(|opt| opt.value == selected)
        .ok_or_else(|| CalculationError::UnknownDropdownOption {
            field: input_field.field.clone(),
            option: selected.to_string(),
        })?;

    Ok(option.points)
}

/// Evaluate conditional point rules
fn evaluate_conditions(conditions: &[PointCondition], value: f64) -> Result<i32, CalculationError> {
    // Evaluate conditions in order, return first match
    for condition in conditions {
        if evaluate_condition(&condition.condition, value)? {
            return Ok(condition.points);
        }
    }

    // No condition matched, return 0 points
    Ok(0)
}

/// Evaluate a single condition (e.g., ">= 65", "< 50", ">= 30 && < 40")
fn evaluate_condition(condition: &str, value: f64) -> Result<bool, CalculationError> {
    let condition = condition.trim();

    // Handle compound conditions with && (AND)
    if condition.contains("&&") {
        let parts: Vec<&str> = condition.split("&&").collect();
        for part in parts {
            if !evaluate_single_condition(part.trim(), value)? {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    evaluate_single_condition(condition, value)
}

/// Evaluate a single comparison (e.g., ">= 65", "< 50")
fn evaluate_single_condition(condition: &str, value: f64) -> Result<bool, CalculationError> {
    let condition = condition.trim();

    if let Some(threshold_str) = condition.strip_prefix(">=") {
        let threshold: f64 =
            threshold_str
                .trim()
                .parse()
                .map_err(|_| CalculationError::ConditionParseError {
                    condition: condition.to_string(),
                    reason: "Invalid number after '>='".to_string(),
                })?;
        Ok(value >= threshold)
    } else if let Some(threshold_str) = condition.strip_prefix("<=") {
        let threshold: f64 =
            threshold_str
                .trim()
                .parse()
                .map_err(|_| CalculationError::ConditionParseError {
                    condition: condition.to_string(),
                    reason: "Invalid number after '<='".to_string(),
                })?;
        Ok(value <= threshold)
    } else if let Some(threshold_str) = condition.strip_prefix('>') {
        let threshold: f64 =
            threshold_str
                .trim()
                .parse()
                .map_err(|_| CalculationError::ConditionParseError {
                    condition: condition.to_string(),
                    reason: "Invalid number after '>'".to_string(),
                })?;
        Ok(value > threshold)
    } else if let Some(threshold_str) = condition.strip_prefix('<') {
        let threshold: f64 =
            threshold_str
                .trim()
                .parse()
                .map_err(|_| CalculationError::ConditionParseError {
                    condition: condition.to_string(),
                    reason: "Invalid number after '<'".to_string(),
                })?;
        Ok(value < threshold)
    } else if let Some(threshold_str) = condition.strip_prefix("==") {
        let threshold: f64 =
            threshold_str
                .trim()
                .parse()
                .map_err(|_| CalculationError::ConditionParseError {
                    condition: condition.to_string(),
                    reason: "Invalid number after '=='".to_string(),
                })?;
        Ok((value - threshold).abs() < f64::EPSILON)
    } else if let Some(threshold_str) = condition.strip_prefix("!=") {
        let threshold: f64 =
            threshold_str
                .trim()
                .parse()
                .map_err(|_| CalculationError::ConditionParseError {
                    condition: condition.to_string(),
                    reason: "Invalid number after '!='".to_string(),
                })?;
        Ok((value - threshold).abs() >= f64::EPSILON)
    } else {
        Err(CalculationError::ConditionParseError {
            condition: condition.to_string(),
            reason: "Unknown operator (expected: >=, <=, >, <, ==, !=)".to_string(),
        })
    }
}

/// Find the interpretation rule that matches the calculated score
fn find_interpretation(
    score_def: &ScoreDefinition,
    total_score: i32,
) -> Result<InterpretationRule, CalculationError> {
    for interp in &score_def.interpretation {
        if matches_score_range(&interp.score, total_score)? {
            return Ok(interp.clone());
        }
    }

    Err(CalculationError::NoInterpretation { score: total_score })
}

/// Check if a score matches a range specification
fn matches_score_range(range: &ScoreRange, score: i32) -> Result<bool, CalculationError> {
    match range {
        ScoreRange::Exact(value) => Ok(score == *value),
        ScoreRange::Range(range_str) => {
            let range_str = range_str.trim();

            // Handle ranges like "1-3"
            if range_str.contains('-') {
                let parts: Vec<&str> = range_str.split('-').collect();
                if parts.len() == 2 {
                    let min: i32 = parts[0].trim().parse().map_err(|_| {
                        CalculationError::ConditionParseError {
                            condition: range_str.to_string(),
                            reason: "Invalid range format".to_string(),
                        }
                    })?;
                    let max: i32 = parts[1].trim().parse().map_err(|_| {
                        CalculationError::ConditionParseError {
                            condition: range_str.to_string(),
                            reason: "Invalid range format".to_string(),
                        }
                    })?;
                    return Ok(score >= min && score <= max);
                }
            }

            // Handle comparisons like "≥3", ">=3", ">5", "<=2", "<10"
            if range_str.starts_with("≥") || range_str.starts_with(">=") {
                let threshold_str = range_str
                    .trim_start_matches("≥")
                    .trim_start_matches(">=")
                    .trim();
                let threshold: i32 =
                    threshold_str
                        .parse()
                        .map_err(|_| CalculationError::ConditionParseError {
                            condition: range_str.to_string(),
                            reason: "Invalid number in range".to_string(),
                        })?;
                return Ok(score >= threshold);
            }

            if range_str.starts_with("≤") || range_str.starts_with("<=") {
                let threshold_str = range_str
                    .trim_start_matches("≤")
                    .trim_start_matches("<=")
                    .trim();
                let threshold: i32 =
                    threshold_str
                        .parse()
                        .map_err(|_| CalculationError::ConditionParseError {
                            condition: range_str.to_string(),
                            reason: "Invalid number in range".to_string(),
                        })?;
                return Ok(score <= threshold);
            }

            if range_str.starts_with('>') && !range_str.starts_with(">=") {
                let threshold_str = range_str.trim_start_matches('>').trim();
                let threshold: i32 =
                    threshold_str
                        .parse()
                        .map_err(|_| CalculationError::ConditionParseError {
                            condition: range_str.to_string(),
                            reason: "Invalid number in range".to_string(),
                        })?;
                return Ok(score > threshold);
            }

            if range_str.starts_with('<') && !range_str.starts_with("<=") {
                let threshold_str = range_str.trim_start_matches('<').trim();
                let threshold: i32 =
                    threshold_str
                        .parse()
                        .map_err(|_| CalculationError::ConditionParseError {
                            condition: range_str.to_string(),
                            reason: "Invalid number in range".to_string(),
                        })?;
                return Ok(score < threshold);
            }

            // Handle plain number as exact match (e.g., "2" in YAML becomes Range("2"))
            if let Ok(exact_value) = range_str.parse::<i32>() {
                return Ok(score == exact_value);
            }

            Err(CalculationError::ConditionParseError {
                condition: range_str.to_string(),
                reason: "Unrecognized range format".to_string(),
            })
        }
    }
}

/// Calculate a formula-based score (e.g., eGFR, KFRE)
fn calculate_formula_score(
    score_def: &ScoreDefinition,
    inputs: &HashMap<String, InputValue>,
    formula: &str,
) -> Result<CalculationResult, CalculationError> {
    // Validate required fields
    for input_field in &score_def.inputs {
        if input_field.required && !inputs.contains_key(&input_field.field) {
            return Err(CalculationError::MissingRequiredField {
                field: input_field.field.clone(),
            });
        }
    }

    let result = crate::scores::formulas::calculate_formula(formula, inputs)?;

    // Find matching interpretation using the formula result
    let interpretation = find_interpretation(score_def, result.value)?;

    Ok(CalculationResult {
        total_score: result.value,
        field_scores: result.field_scores,
        risk_level: interpretation.risk_level,
        risk: interpretation.risk.clone(),
        risk_de: interpretation.risk_de.clone(),
        recommendation: interpretation.recommendation.clone(),
        recommendation_de: interpretation.recommendation_de.clone(),
        details: interpretation.details.clone(),
        details_de: interpretation.details_de.clone(),
        interpretation: interpretation.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DropdownOption, Specialty};

    fn create_test_score() -> ScoreDefinition {
        ScoreDefinition {
            name: "Test Score".to_string(),
            name_de: "Test-Score".to_string(),
            specialty: Specialty::Cardiology,
            specialty_de: "Kardiologie".to_string(),
            version: "1.0".to_string(),
            guideline_source: "Test".to_string(),
            reference: "Test".to_string(),
            validation_status: "draft".to_string(),
            description: String::new(),
            description_de: String::new(),
            inputs: vec![
                InputField {
                    field: "age".to_string(),
                    input_type: InputType::Number,
                    label: "Age".to_string(),
                    label_de: "Alter".to_string(),
                    unit: Some("years".to_string()),
                    unit_de: Some("Jahre".to_string()),
                    points: PointsValue::Conditional(vec![
                        PointCondition {
                            condition: ">= 75".to_string(),
                            points: 2,
                            label: Some("Age ≥75".to_string()),
                            label_de: Some("Alter ≥75".to_string()),
                        },
                        PointCondition {
                            condition: ">= 65".to_string(),
                            points: 1,
                            label: Some("Age 65-74".to_string()),
                            label_de: Some("Alter 65-74".to_string()),
                        },
                    ]),
                    help: None,
                    help_de: None,
                    min: Some(0.0),
                    max: Some(120.0),
                    options: vec![],
                    required: true,
                },
                InputField {
                    field: "hypertension".to_string(),
                    input_type: InputType::Boolean,
                    label: "Hypertension".to_string(),
                    label_de: "Hypertonie".to_string(),
                    unit: None,
                    unit_de: None,
                    points: PointsValue::Fixed(1),
                    help: None,
                    help_de: None,
                    min: None,
                    max: None,
                    options: vec![],
                    required: true,
                },
            ],
            interpretation: vec![
                InterpretationRule {
                    score: ScoreRange::Exact(0),
                    risk: "Low".to_string(),
                    risk_de: "Niedrig".to_string(),
                    risk_level: RiskLevel::Low,
                    recommendation: "No action".to_string(),
                    recommendation_de: "Keine Maßnahmen".to_string(),
                    details: None,
                    details_de: None,
                },
                InterpretationRule {
                    score: ScoreRange::Range("≥1".to_string()),
                    risk: "High".to_string(),
                    risk_de: "Hoch".to_string(),
                    risk_level: RiskLevel::High,
                    recommendation: "Take action".to_string(),
                    recommendation_de: "Maßnahmen ergreifen".to_string(),
                    details: None,
                    details_de: None,
                },
            ],
            formula: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_calculate_score_basic() {
        let score_def = create_test_score();
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(70.0));
        inputs.insert("hypertension".to_string(), InputValue::Boolean(true));

        let result = calculate_score(&score_def, &inputs).unwrap();

        assert_eq!(result.total_score, 2); // 1 for age 65-74, 1 for hypertension
        assert_eq!(result.risk, "High");
        assert_eq!(result.get_field_points("age"), Some(1));
        assert_eq!(result.get_field_points("hypertension"), Some(1));
    }

    #[test]
    fn test_calculate_score_age_thresholds() {
        let score_def = create_test_score();

        // Age 80 (≥75) = 2 points
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(80.0));
        inputs.insert("hypertension".to_string(), InputValue::Boolean(false));
        let result = calculate_score(&score_def, &inputs).unwrap();
        assert_eq!(result.get_field_points("age"), Some(2));

        // Age 70 (65-74) = 1 point
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(70.0));
        inputs.insert("hypertension".to_string(), InputValue::Boolean(false));
        let result = calculate_score(&score_def, &inputs).unwrap();
        assert_eq!(result.get_field_points("age"), Some(1));

        // Age 60 (<65) = 0 points
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(60.0));
        inputs.insert("hypertension".to_string(), InputValue::Boolean(false));
        let result = calculate_score(&score_def, &inputs).unwrap();
        assert_eq!(result.get_field_points("age"), Some(0));
    }

    #[test]
    fn test_missing_required_field() {
        let score_def = create_test_score();
        let inputs = HashMap::new(); // No inputs provided

        let result = calculate_score(&score_def, &inputs);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CalculationError::MissingRequiredField { .. }
        ));
    }

    #[test]
    fn test_out_of_range() {
        let score_def = create_test_score();
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(150.0)); // > max
        inputs.insert("hypertension".to_string(), InputValue::Boolean(false));

        let result = calculate_score(&score_def, &inputs);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CalculationError::OutOfRange { .. }
        ));
    }

    #[test]
    fn test_evaluate_condition() {
        assert!(evaluate_condition(">= 65", 65.0).unwrap());
        assert!(evaluate_condition(">= 65", 70.0).unwrap());
        assert!(!evaluate_condition(">= 65", 60.0).unwrap());

        assert!(evaluate_condition("> 50", 51.0).unwrap());
        assert!(!evaluate_condition("> 50", 50.0).unwrap());

        assert!(evaluate_condition("< 100", 99.0).unwrap());
        assert!(!evaluate_condition("< 100", 100.0).unwrap());

        assert!(evaluate_condition("<= 100", 100.0).unwrap());
        assert!(!evaluate_condition("<= 100", 101.0).unwrap());
    }

    #[test]
    fn test_evaluate_compound_condition() {
        // Test && (AND) conditions used in GRACE score
        assert!(evaluate_condition(">= 30 && < 40", 35.0).unwrap());
        assert!(evaluate_condition(">= 30 && < 40", 30.0).unwrap());
        assert!(!evaluate_condition(">= 30 && < 40", 40.0).unwrap());
        assert!(!evaluate_condition(">= 30 && < 40", 29.0).unwrap());

        // Boundary test
        assert!(evaluate_condition(">= 50 && < 60", 50.0).unwrap());
        assert!(evaluate_condition(">= 50 && < 60", 59.9).unwrap());
        assert!(!evaluate_condition(">= 50 && < 60", 60.0).unwrap());

        // Three conditions
        assert!(evaluate_condition(">= 10 && < 20 && != 15", 12.0).unwrap());
        assert!(!evaluate_condition(">= 10 && < 20 && != 15", 15.0).unwrap());
    }

    #[test]
    fn test_matches_score_range() {
        // Exact match
        assert!(matches_score_range(&ScoreRange::Exact(5), 5).unwrap());
        assert!(!matches_score_range(&ScoreRange::Exact(5), 6).unwrap());

        // Range match
        assert!(matches_score_range(&ScoreRange::Range("1-3".to_string()), 2).unwrap());
        assert!(matches_score_range(&ScoreRange::Range("1-3".to_string()), 1).unwrap());
        assert!(matches_score_range(&ScoreRange::Range("1-3".to_string()), 3).unwrap());
        assert!(!matches_score_range(&ScoreRange::Range("1-3".to_string()), 4).unwrap());

        // Comparison match
        assert!(matches_score_range(&ScoreRange::Range("≥3".to_string()), 3).unwrap());
        assert!(matches_score_range(&ScoreRange::Range("≥3".to_string()), 5).unwrap());
        assert!(!matches_score_range(&ScoreRange::Range("≥3".to_string()), 2).unwrap());
    }

    #[test]
    fn test_dropdown_calculation() {
        let mut score_def = create_test_score();
        score_def.inputs.push(InputField {
            field: "severity".to_string(),
            input_type: InputType::Dropdown,
            label: "Severity".to_string(),
            label_de: "Schweregrad".to_string(),
            unit: None,
            unit_de: None,
            points: PointsValue::Fixed(0),
            help: None,
            help_de: None,
            min: None,
            max: None,
            options: vec![
                DropdownOption {
                    value: "mild".to_string(),
                    label: "Mild".to_string(),
                    label_de: "Leicht".to_string(),
                    points: 0,
                    description: None,
                    description_de: None,
                },
                DropdownOption {
                    value: "severe".to_string(),
                    label: "Severe".to_string(),
                    label_de: "Schwer".to_string(),
                    points: 3,
                    description: None,
                    description_de: None,
                },
            ],
            required: false,
        });

        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(60.0));
        inputs.insert("hypertension".to_string(), InputValue::Boolean(false));
        inputs.insert(
            "severity".to_string(),
            InputValue::Dropdown("severe".to_string()),
        );

        let result = calculate_score(&score_def, &inputs).unwrap();
        assert_eq!(result.get_field_points("severity"), Some(3));
    }
}
