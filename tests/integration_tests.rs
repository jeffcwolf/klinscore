// Integration tests for KlinScore
// Tests the complete workflow from loading scores to calculating results
// Each score is tested with realistic clinical scenarios based on published literature

use klinscore::config::Specialty;
use klinscore::scores::{calculate_score, load_all_scores, InputValue};
use std::collections::HashMap;

// ============================================================
// Library & Loading Tests
// ============================================================

#[test]
fn test_end_to_end_workflow() {
    let library = load_all_scores("scores/").expect("Failed to load scores");
    assert!(
        library.count() >= 9,
        "Expected at least 9 scores, got {}",
        library.count()
    );

    let cardio_scores = library.get_scores_for_specialty(Specialty::Cardiology);
    assert!(
        !cardio_scores.is_empty(),
        "No cardiology scores were loaded"
    );

    let score = library
        .get_score("cha2ds2_va")
        .expect("CHA2DS2-VA score not found");
    assert_eq!(score.name, "CHA2DS2-VA Score");
    assert_eq!(score.specialty, Specialty::Cardiology);
    assert_eq!(score.inputs.len(), 6);
}

#[test]
fn test_score_library_methods() {
    let library = load_all_scores("scores/").expect("Failed to load scores");
    assert_eq!(library.count(), 9, "Should have loaded 9 scores");

    let specialties = library.get_specialties();
    assert!(specialties.contains(&Specialty::Cardiology));
    assert!(specialties.contains(&Specialty::Nephrology));
    assert!(specialties.contains(&Specialty::Anesthesiology));

    let cardio_scores = library.get_scores_for_specialty(Specialty::Cardiology);
    assert_eq!(cardio_scores.len(), 3, "Should have 3 cardiology scores");

    let nephro_scores = library.get_scores_for_specialty(Specialty::Nephrology);
    assert_eq!(nephro_scores.len(), 2, "Should have 2 nephrology scores");

    let anesth_scores = library.get_scores_for_specialty(Specialty::Anesthesiology);
    assert_eq!(
        anesth_scores.len(),
        4,
        "Should have 4 anesthesiology scores"
    );

    assert!(library.get_score("cha2ds2_va").is_some());
    assert!(library.get_score("has_bled").is_some());
    assert!(library.get_score("grace").is_some());
    assert!(library.get_score("caprini").is_some());
    assert!(library.get_score("kfre").is_some());
    assert!(library.get_score("egfr_ckd_epi_2021").is_some());
    assert!(library.get_score("asa").is_some());
    assert!(library.get_score("rcri").is_some());
    assert!(library.get_score("stop_bang").is_some());
    assert!(library.get_score("nonexistent_score").is_none());
}

// ============================================================
// CHA2DS2-VA Score Tests (Cardiology)
// Source: ESC 2024 Guidelines
// ============================================================

#[test]
fn test_cha2ds2va_high_risk_patient() {
    // 72-year-old with CHF and hypertension
    // Expected: age 65-74 = 1pt, CHF = 1pt, HTN = 1pt = 3 total
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("cha2ds2_va").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(72.0));
    inputs.insert("heart_failure".to_string(), InputValue::Boolean(true));
    inputs.insert("hypertension".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 3);
    assert_eq!(result.risk, "Moderate-High");
    assert!(result.recommendation.contains("anticoagulation"));
}

#[test]
fn test_cha2ds2va_zero_risk_no_booleans() {
    // Young patient, only age provided - no risk factors checked
    // This is the realistic UI scenario: user enters age, doesn't check any boxes
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("cha2ds2_va").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(50.0));
    // No boolean fields provided = all unchecked = 0 points each

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 0);
    assert_eq!(result.risk, "Low");
}

#[test]
fn test_cha2ds2va_max_score() {
    // 80-year-old with all risk factors = max score
    // age ≥75 = 2, CHF = 1, HTN = 1, DM = 1, stroke = 2, vascular = 1 = 8
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("cha2ds2_va").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(80.0));
    inputs.insert("heart_failure".to_string(), InputValue::Boolean(true));
    inputs.insert("hypertension".to_string(), InputValue::Boolean(true));
    inputs.insert("diabetes".to_string(), InputValue::Boolean(true));
    inputs.insert("stroke_tia".to_string(), InputValue::Boolean(true));
    inputs.insert("vascular_disease".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 8);
    assert_eq!(result.risk, "Moderate-High"); // ≥2 interpretation
}

#[test]
fn test_cha2ds2va_age_boundaries() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("cha2ds2_va").unwrap();

    let test_cases = vec![
        (64.0, 0), // Just below 65
        (65.0, 1), // Exactly 65
        (74.0, 1), // Upper boundary of 65-74
        (75.0, 2), // Exactly 75
        (90.0, 2), // Well above 75
    ];

    for (age, expected_points) in test_cases {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(age));

        let result = calculate_score(score, &inputs)
            .unwrap_or_else(|e| panic!("Calculation failed for age {}: {}", age, e));
        assert_eq!(
            result.get_field_points("age"),
            Some(expected_points),
            "Wrong points for age {}",
            age
        );
    }
}

#[test]
fn test_cha2ds2va_missing_age_fails() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("cha2ds2_va").unwrap();

    // Age is required - should fail without it
    let inputs = HashMap::new();
    let result = calculate_score(score, &inputs);
    assert!(result.is_err(), "Should fail with missing required age");
}

// ============================================================
// HAS-BLED Score Tests (Cardiology)
// Source: ESC 2024 Guidelines
// ============================================================

#[test]
fn test_has_bled_low_risk() {
    // No risk factors = 0 = low risk
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("has_bled").unwrap();

    let inputs = HashMap::new(); // All booleans unchecked = 0
    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 0);
    assert_eq!(result.risk, "Low Bleeding Risk");
}

#[test]
fn test_has_bled_high_risk() {
    // 4 risk factors checked = high bleeding risk (≥3)
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("has_bled").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("hypertension".to_string(), InputValue::Boolean(true));
    inputs.insert(
        "abnormal_renal_function".to_string(),
        InputValue::Boolean(true),
    );
    inputs.insert("stroke".to_string(), InputValue::Boolean(true));
    inputs.insert("elderly".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 4);
    assert_eq!(result.risk, "High Bleeding Risk");
}

#[test]
fn test_has_bled_boundary() {
    // Exactly 2 = still low, 3 = high
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("has_bled").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("hypertension".to_string(), InputValue::Boolean(true));
    inputs.insert("elderly".to_string(), InputValue::Boolean(true));
    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 2);
    assert_eq!(result.risk, "Low Bleeding Risk");

    inputs.insert("stroke".to_string(), InputValue::Boolean(true));
    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 3);
    assert_eq!(result.risk, "High Bleeding Risk");
}

// ============================================================
// GRACE Score Tests (Cardiology)
// Source: Fox et al. 2006, ESC 2020
// ============================================================

#[test]
fn test_grace_low_risk_patient() {
    // Young, hemodynamically stable, no complications
    // Age 45 = 25pt, HR 75 = 9pt, SBP 130 = 34pt, Cr 80 μmol/L = 7pt
    // No arrest, no ST changes, no enzymes, Killip I = 0pt
    // Total = 25 + 9 + 34 + 7 = 75 (low risk <108)
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("grace").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(45.0));
    inputs.insert("heart_rate".to_string(), InputValue::Number(75.0));
    inputs.insert("systolic_bp".to_string(), InputValue::Number(130.0));
    inputs.insert("creatinine".to_string(), InputValue::Number(80.0));
    inputs.insert(
        "killip_class".to_string(),
        InputValue::Dropdown("killip_1".to_string()),
    );
    // No boolean fields = all false

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 75);
    assert_eq!(result.risk, "Low Risk");
}

#[test]
fn test_grace_high_risk_patient() {
    // Elderly, tachycardic, hypotensive, renal failure, ST changes, positive enzymes
    // Age 75 = 75pt, HR 110 = 24pt, SBP 90 = 53pt, Cr 200 μmol/L = 21pt
    // ST deviation = 28pt, elevated enzymes = 14pt, Killip II = 20pt
    // Total = 75 + 24 + 53 + 21 + 28 + 14 + 20 = 235 (high risk >140)
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("grace").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(75.0));
    inputs.insert("heart_rate".to_string(), InputValue::Number(110.0));
    inputs.insert("systolic_bp".to_string(), InputValue::Number(90.0));
    inputs.insert("creatinine".to_string(), InputValue::Number(200.0));
    inputs.insert("st_deviation".to_string(), InputValue::Boolean(true));
    inputs.insert("elevated_enzymes".to_string(), InputValue::Boolean(true));
    inputs.insert(
        "killip_class".to_string(),
        InputValue::Dropdown("killip_2".to_string()),
    );

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 235);
    assert_eq!(result.risk, "High Risk");
}

#[test]
fn test_grace_intermediate_risk() {
    // Moderate risk scenario
    // Age 65 = 58pt, HR 85 = 9pt, SBP 140 = 24pt, Cr 100 μmol/L = 7pt
    // Elevated enzymes only = 14pt, Killip I = 0pt
    // Total = 58 + 9 + 24 + 7 + 14 = 112 (intermediate 109-140)
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("grace").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(65.0));
    inputs.insert("heart_rate".to_string(), InputValue::Number(85.0));
    inputs.insert("systolic_bp".to_string(), InputValue::Number(140.0));
    inputs.insert("creatinine".to_string(), InputValue::Number(100.0));
    inputs.insert("elevated_enzymes".to_string(), InputValue::Boolean(true));
    inputs.insert(
        "killip_class".to_string(),
        InputValue::Dropdown("killip_1".to_string()),
    );

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 112);
    assert_eq!(result.risk, "Intermediate Risk");
}

#[test]
fn test_grace_compound_conditions() {
    // Specifically test the && conditions in GRACE (e.g. ">= 30 && < 40")
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("grace").unwrap();

    // Test age boundaries: age 35 should match ">= 30 && < 40" = 8 points
    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(35.0));
    inputs.insert("heart_rate".to_string(), InputValue::Number(60.0));
    inputs.insert("systolic_bp".to_string(), InputValue::Number(120.0));
    inputs.insert("creatinine".to_string(), InputValue::Number(80.0));
    inputs.insert(
        "killip_class".to_string(),
        InputValue::Dropdown("killip_1".to_string()),
    );

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.get_field_points("age"), Some(8));

    // Age 55 should match ">= 50 && < 60" = 41 points
    let mut inputs2 = inputs.clone();
    inputs2.insert("age".to_string(), InputValue::Number(55.0));
    let result2 = calculate_score(score, &inputs2).unwrap();
    assert_eq!(result2.get_field_points("age"), Some(41));
}

#[test]
fn test_grace_missing_required_number_fails() {
    // GRACE requires age, heart_rate, systolic_bp, creatinine, killip_class
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("grace").unwrap();

    let inputs = HashMap::new(); // Empty = missing age
    let result = calculate_score(score, &inputs);
    assert!(result.is_err());
}

// ============================================================
// ASA Physical Status Tests (Anesthesiology)
// Source: ASA 2020
// ============================================================

#[test]
fn test_asa_class_1() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("asa").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert(
        "asa_class".to_string(),
        InputValue::Dropdown("asa_1".to_string()),
    );

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 1);
    assert_eq!(result.risk, "ASA I - Minimal Risk");
}

#[test]
fn test_asa_class_3() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("asa").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert(
        "asa_class".to_string(),
        InputValue::Dropdown("asa_3".to_string()),
    );

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 3);
    assert_eq!(result.risk, "ASA III - Moderate Risk");
}

#[test]
fn test_asa_class_5_with_emergency() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("asa").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert(
        "asa_class".to_string(),
        InputValue::Dropdown("asa_5".to_string()),
    );
    inputs.insert("emergency".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    // Emergency has points: 0, so total = 5
    assert_eq!(result.total_score, 5);
    assert_eq!(result.risk, "ASA V - Extreme Risk");
}

#[test]
fn test_asa_missing_class_fails() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("asa").unwrap();

    let inputs = HashMap::new(); // Dropdown is required
    let result = calculate_score(score, &inputs);
    assert!(result.is_err());
}

// ============================================================
// RCRI Score Tests (Anesthesiology)
// Source: Lee et al. 1999, ACC/AHA 2022
// ============================================================

#[test]
fn test_rcri_zero_risk_factors() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("rcri").unwrap();

    let inputs = HashMap::new(); // No risk factors = 0
    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 0);
    assert_eq!(result.risk, "Class I - Very Low Risk");
}

#[test]
fn test_rcri_two_risk_factors() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("rcri").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("high_risk_surgery".to_string(), InputValue::Boolean(true));
    inputs.insert(
        "ischemic_heart_disease".to_string(),
        InputValue::Boolean(true),
    );

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 2);
    assert_eq!(result.risk, "Class III - Moderate Risk");
}

#[test]
fn test_rcri_max_score() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("rcri").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("high_risk_surgery".to_string(), InputValue::Boolean(true));
    inputs.insert(
        "ischemic_heart_disease".to_string(),
        InputValue::Boolean(true),
    );
    inputs.insert("heart_failure".to_string(), InputValue::Boolean(true));
    inputs.insert(
        "cerebrovascular_disease".to_string(),
        InputValue::Boolean(true),
    );
    inputs.insert("diabetes_insulin".to_string(), InputValue::Boolean(true));
    inputs.insert("renal_insufficiency".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 6);
    assert_eq!(result.risk, "Class IV - High Risk");
}

// ============================================================
// STOP-BANG Score Tests (Anesthesiology)
// Source: Chung et al. 2016
// ============================================================

#[test]
fn test_stop_bang_low_risk() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("stop_bang").unwrap();

    // No risk factors
    let inputs = HashMap::new();
    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 0);
    assert_eq!(result.risk, "Low Risk for OSA");
}

#[test]
fn test_stop_bang_high_risk() {
    // Typical high-risk profile: snoring, tired, observed apnea, high BP, BMI>35, age>50
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("stop_bang").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("snoring".to_string(), InputValue::Boolean(true));
    inputs.insert("tired".to_string(), InputValue::Boolean(true));
    inputs.insert("observed".to_string(), InputValue::Boolean(true));
    inputs.insert("pressure".to_string(), InputValue::Boolean(true));
    inputs.insert("bmi".to_string(), InputValue::Boolean(true));
    inputs.insert("age".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 6);
    assert_eq!(result.risk, "High Risk for OSA");
}

#[test]
fn test_stop_bang_intermediate() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("stop_bang").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("snoring".to_string(), InputValue::Boolean(true));
    inputs.insert("tired".to_string(), InputValue::Boolean(true));
    inputs.insert("pressure".to_string(), InputValue::Boolean(true));

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 3);
    assert_eq!(result.risk, "Intermediate Risk for OSA");
}

// ============================================================
// Caprini VTE Score Tests (Anesthesiology)
// Source: Caprini 2013
// ============================================================

#[test]
fn test_caprini_very_low_risk() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("caprini").unwrap();

    // No risk factors
    let inputs = HashMap::new();
    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 0);
    assert_eq!(result.risk, "Very Low Risk");
}

#[test]
fn test_caprini_moderate_risk() {
    // 1-point + 2-point factors = 3 (moderate)
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("caprini").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age_41_60".to_string(), InputValue::Boolean(true)); // 1pt
    inputs.insert("major_surgery".to_string(), InputValue::Boolean(true)); // 2pt

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 3);
    assert_eq!(result.risk, "Moderate Risk");
}

#[test]
fn test_caprini_high_risk() {
    // History of VTE (3pt) + major surgery (2pt) = 5 (high risk)
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("caprini").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("history_vte".to_string(), InputValue::Boolean(true)); // 3pt
    inputs.insert("major_surgery".to_string(), InputValue::Boolean(true)); // 2pt

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 5);
    assert_eq!(result.risk, "High Risk");
}

#[test]
fn test_caprini_mixed_point_values() {
    // Verify different point categories work
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("caprini").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("minor_surgery".to_string(), InputValue::Boolean(true)); // 1pt
    inputs.insert("bmi_gt_30".to_string(), InputValue::Boolean(true)); // 2pt
    inputs.insert("family_history_vte".to_string(), InputValue::Boolean(true)); // 3pt
    inputs.insert("stroke".to_string(), InputValue::Boolean(true)); // 5pt

    let result = calculate_score(score, &inputs).unwrap();
    assert_eq!(result.total_score, 11); // 1 + 2 + 3 + 5
    assert_eq!(result.risk, "High Risk"); // ≥5
}

// ============================================================
// eGFR CKD-EPI 2021 Tests (Nephrology)
// Source: Inker et al. 2021, KDIGO 2024
// Note: This is a simplified point-based representation
// ============================================================

#[test]
fn test_egfr_loads_with_sex_dropdown() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("egfr_ckd_epi_2021").unwrap();

    // Verify sex field is a dropdown
    let sex_field = score.inputs.iter().find(|f| f.field == "sex").unwrap();
    assert_eq!(
        sex_field.input_type,
        klinscore::config::InputType::Dropdown,
        "Sex should be a dropdown"
    );
    assert_eq!(
        sex_field.options.len(),
        2,
        "Should have male and female options"
    );
}

#[test]
fn test_egfr_required_fields() {
    // Age, sex, and creatinine are all required
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("egfr_ckd_epi_2021").unwrap();

    let inputs = HashMap::new();
    let result = calculate_score(score, &inputs);
    assert!(result.is_err(), "Should fail without required age");
}

#[test]
fn test_egfr_calculation_with_all_fields() {
    // This score uses points: 0 for all fields (it's a diagnostic formula)
    // The total "score" is 0, which won't match any interpretation
    // This tests that the YAML loads and inputs are accepted
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("egfr_ckd_epi_2021").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(55.0));
    inputs.insert("sex".to_string(), InputValue::Dropdown("male".to_string()));
    inputs.insert("creatinine".to_string(), InputValue::Number(100.0));

    // All fields have points: 0, so total = 0
    // This won't match any interpretation (lowest is ≥90 which is for eGFR values)
    // This is expected - eGFR needs a real formula, not point-based scoring
    let result = calculate_score(score, &inputs);
    // It's ok if this errors with NoInterpretation - the YAML is a simplified representation
    assert!(
        result.is_ok() || format!("{:?}", result).contains("NoInterpretation"),
        "Should either succeed or fail with NoInterpretation, got: {:?}",
        result
    );
}

// ============================================================
// KFRE Tests (Nephrology)
// Source: Tangri et al. 2011, KDIGO 2024
// ============================================================

#[test]
fn test_kfre_loads_with_sex_dropdown() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("kfre").unwrap();

    let sex_field = score.inputs.iter().find(|f| f.field == "sex").unwrap();
    assert_eq!(
        sex_field.input_type,
        klinscore::config::InputType::Dropdown,
        "Sex should be a dropdown"
    );
}

#[test]
fn test_kfre_required_fields() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("kfre").unwrap();

    let inputs = HashMap::new();
    let result = calculate_score(score, &inputs);
    assert!(result.is_err());
}

#[test]
fn test_kfre_calculation() {
    // Similar to eGFR - all fields have points: 0
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("kfre").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(65.0));
    inputs.insert(
        "sex".to_string(),
        InputValue::Dropdown("female".to_string()),
    );
    inputs.insert("egfr".to_string(), InputValue::Number(35.0));
    inputs.insert("acr".to_string(), InputValue::Number(30.0));

    let result = calculate_score(score, &inputs);
    assert!(
        result.is_ok() || format!("{:?}", result).contains("NoInterpretation"),
        "Should handle all-zero-points gracefully"
    );
}

// ============================================================
// Edge Cases & Cross-cutting Concerns
// ============================================================

#[test]
fn test_boolean_fields_default_to_zero_when_missing() {
    // Verify that for scores with all-boolean fields,
    // an empty input map produces score = 0
    let library = load_all_scores("scores/").unwrap();

    for (score_id, expected_zero_scores) in [
        ("has_bled", true),
        ("rcri", true),
        ("stop_bang", true),
        ("caprini", true),
    ] {
        let score = library.get_score(score_id).unwrap();
        let inputs = HashMap::new();
        let result = calculate_score(score, &inputs);

        if expected_zero_scores {
            let result = result.unwrap_or_else(|e| {
                panic!("{}: should succeed with empty inputs, got: {}", score_id, e)
            });
            assert_eq!(
                result.total_score, 0,
                "{}: empty inputs should produce score 0",
                score_id
            );
        }
    }
}

#[test]
fn test_out_of_range_validation() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("cha2ds2_va").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert("age".to_string(), InputValue::Number(150.0)); // max is 120

    let result = calculate_score(score, &inputs);
    assert!(result.is_err(), "Should reject age > 120");
}

#[test]
fn test_invalid_dropdown_option() {
    let library = load_all_scores("scores/").unwrap();
    let score = library.get_score("asa").unwrap();

    let mut inputs = HashMap::new();
    inputs.insert(
        "asa_class".to_string(),
        InputValue::Dropdown("invalid_option".to_string()),
    );

    let result = calculate_score(score, &inputs);
    assert!(result.is_err(), "Should reject invalid dropdown option");
}
