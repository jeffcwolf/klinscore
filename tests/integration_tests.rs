// Integration tests for KlinScore
// Tests the complete workflow from loading scores to calculating results

use klinscore::config::Specialty;
use klinscore::scores::{calculate_score, load_all_scores, InputValue};
use std::collections::HashMap;

#[test]
fn test_end_to_end_workflow() {
    // Load all scores
    let library = load_all_scores("scores/").expect("Failed to load scores");

    // Should have loaded multiple scores
    assert!(library.count() >= 9, "Expected at least 9 scores, got {}", library.count());

    // Get Cardiology scores
    let cardio_scores = library.get_scores_for_specialty(Specialty::Cardiology);
    assert!(
        !cardio_scores.is_empty(),
        "No cardiology scores were loaded"
    );

    // Get the CHA2DS2-VA score
    let score = library
        .get_score("cha2ds2_va")
        .expect("CHA2DS2-VA score not found");

    // Verify score metadata
    assert_eq!(score.name, "CHA2DS2-VA Score");
    assert_eq!(score.specialty, Specialty::Cardiology);
    assert_eq!(score.inputs.len(), 6);

    // Calculate a score with patient data
    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(72.0)); // 1 point (65-74)
    inputs.insert("heart_failure".to_string(), InputValue::Boolean(true)); // 1 point
    inputs.insert("hypertension".to_string(), InputValue::Boolean(true)); // 1 point
    inputs.insert("diabetes".to_string(), InputValue::Boolean(false)); // 0 points
    inputs.insert("stroke_tia".to_string(), InputValue::Boolean(false)); // 0 points
    inputs.insert("vascular_disease".to_string(), InputValue::Boolean(false)); // 0 points

    let result = calculate_score(score, &inputs).expect("Calculation failed");

    // Verify result
    assert_eq!(result.total_score, 3);
    assert_eq!(result.field_points.get("age"), Some(&1));
    assert_eq!(result.field_points.get("heart_failure"), Some(&1));
    assert_eq!(result.field_points.get("hypertension"), Some(&1));

    // Should be moderate-high risk (score ≥2)
    assert_eq!(result.risk, "Moderate-High");
}

#[test]
fn test_low_risk_calculation() {
    let library = load_all_scores("scores/").expect("Failed to load scores");
    let score = library
        .get_score("cha2ds2_va")
        .expect("CHA2DS2-VA score not found");

    // Young patient with no risk factors = 0 points = low risk
    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(50.0)); // 0 points
    inputs.insert("heart_failure".to_string(), InputValue::Boolean(false));
    inputs.insert("hypertension".to_string(), InputValue::Boolean(false));
    inputs.insert("diabetes".to_string(), InputValue::Boolean(false));
    inputs.insert("stroke_tia".to_string(), InputValue::Boolean(false));
    inputs.insert("vascular_disease".to_string(), InputValue::Boolean(false));

    let result = calculate_score(score, &inputs).expect("Calculation failed");

    assert_eq!(result.total_score, 0);
    assert_eq!(result.risk, "Low");
    assert!(result.recommendation.contains("not"));
}

#[test]
fn test_age_thresholds() {
    let library = load_all_scores("scores/").expect("Failed to load scores");
    let score = library
        .get_score("cha2ds2_va")
        .expect("CHA2DS2-VA score not found");

    // Test age boundaries
    let test_cases = vec![
        (64.0, 0), // Just below 65
        (65.0, 1), // Exactly 65
        (70.0, 1), // Between 65-74
        (74.0, 1), // Exactly 74
        (75.0, 2), // Exactly 75
        (80.0, 2), // Above 75
    ];

    for (age, expected_points) in test_cases {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(age));
        inputs.insert("heart_failure".to_string(), InputValue::Boolean(false));
        inputs.insert("hypertension".to_string(), InputValue::Boolean(false));
        inputs.insert("diabetes".to_string(), InputValue::Boolean(false));
        inputs.insert("stroke_tia".to_string(), InputValue::Boolean(false));
        inputs.insert("vascular_disease".to_string(), InputValue::Boolean(false));

        let result = calculate_score(score, &inputs)
            .unwrap_or_else(|_| panic!("Calculation failed for age {}", age));

        assert_eq!(
            result.field_points.get("age"),
            Some(&expected_points),
            "Wrong points for age {}",
            age
        );
    }
}

#[test]
fn test_missing_required_field_error() {
    let library = load_all_scores("scores/").expect("Failed to load scores");
    let score = library
        .get_score("cha2ds2_va")
        .expect("CHA2DS2-VA score not found");

    // Missing required age field
    let mut inputs = HashMap::new();
    inputs.insert("heart_failure".to_string(), InputValue::Boolean(false));
    inputs.insert("hypertension".to_string(), InputValue::Boolean(false));

    let result = calculate_score(score, &inputs);
    assert!(
        result.is_err(),
        "Should fail with missing required field"
    );
}

#[test]
fn test_score_library_methods() {
    let library = load_all_scores("scores/").expect("Failed to load scores");

    // Should have 9 scores
    assert_eq!(library.count(), 9, "Should have loaded 9 scores");

    // Test get_specialties
    let specialties = library.get_specialties();
    assert!(
        specialties.contains(&Specialty::Cardiology),
        "Should have Cardiology specialty"
    );
    assert!(
        specialties.contains(&Specialty::Nephrology),
        "Should have Nephrology specialty"
    );
    assert!(
        specialties.contains(&Specialty::Anesthesiology),
        "Should have Anesthesiology specialty"
    );

    // Test get_scores_for_specialty
    let cardio_scores = library.get_scores_for_specialty(Specialty::Cardiology);
    assert_eq!(cardio_scores.len(), 3, "Should have 3 cardiology scores");

    let nephro_scores = library.get_scores_for_specialty(Specialty::Nephrology);
    assert_eq!(nephro_scores.len(), 2, "Should have 2 nephrology scores");

    let anesth_scores = library.get_scores_for_specialty(Specialty::Anesthesiology);
    assert_eq!(anesth_scores.len(), 4, "Should have 4 anesthesiology scores");

    // Test get_score
    assert!(
        library.get_score("cha2ds2_va").is_some(),
        "Should find CHA2DS2-VA score"
    );
    assert!(
        library.get_score("has_bled").is_some(),
        "Should find HAS-BLED score"
    );
    assert!(
        library.get_score("grace").is_some(),
        "Should find GRACE score"
    );
    assert!(
        library.get_score("caprini").is_some(),
        "Should find Caprini score"
    );
    assert!(
        library.get_score("kfre").is_some(),
        "Should find KFRE score"
    );
    assert!(
        library.get_score("nonexistent_score").is_none(),
        "Should return None for nonexistent score"
    );
}
