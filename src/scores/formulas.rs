// formulas.rs
// Built-in formula calculations for scores that aren't simple point sums
// (e.g., eGFR CKD-EPI 2021, KFRE)

use crate::scores::calculator::{CalculationError, FieldScore, InputValue};
use std::collections::HashMap;

/// Result from a formula calculation
pub struct FormulaResult {
    /// The computed value (e.g., eGFR in mL/min/1.73m², KFRE risk %)
    pub value: i32,
    /// Field scores for breakdown display
    pub field_scores: Vec<FieldScore>,
}

/// Dispatch to the right formula by name
pub fn calculate_formula(
    formula: &str,
    inputs: &HashMap<String, InputValue>,
) -> Result<FormulaResult, CalculationError> {
    match formula {
        "ckd_epi_2021" => calculate_egfr_ckd_epi_2021(inputs),
        "kfre_4var" => calculate_kfre_4var(inputs),
        _ => Err(CalculationError::InvalidInput {
            field: "formula".to_string(),
            reason: format!("Unknown formula: {}", formula),
        }),
    }
}

/// CKD-EPI 2021 race-free eGFR equation
///
/// Reference: Inker LA, et al. NEJM 2021;385(19):1737-1749
///
/// eGFR = 142 × min(Scr/κ, 1)^α × max(Scr/κ, 1)^(-1.200) × 0.9938^age × (1.012 if female)
///
/// Where:
///   Female: κ = 0.7, α = -0.241
///   Male:   κ = 0.9, α = -0.302
///
/// Scr in mg/dL (input is μmol/L, converted by dividing by 88.4)
fn calculate_egfr_ckd_epi_2021(
    inputs: &HashMap<String, InputValue>,
) -> Result<FormulaResult, CalculationError> {
    let age = get_required_number(inputs, "age")?;
    let sex = get_required_dropdown(inputs, "sex")?;
    let creatinine_umol = get_required_number(inputs, "creatinine")?;

    // Convert μmol/L to mg/dL
    let scr = creatinine_umol / 88.4;

    let is_female = sex == "female";

    let (kappa, alpha) = if is_female {
        (0.7, -0.241)
    } else {
        (0.9, -0.302)
    };

    let scr_over_kappa = scr / kappa;

    let term1 = scr_over_kappa.min(1.0).powf(alpha);
    let term2 = scr_over_kappa.max(1.0).powf(-1.200);
    let term3 = 0.9938_f64.powf(age);
    let sex_factor = if is_female { 1.012 } else { 1.0 };

    let egfr = 142.0 * term1 * term2 * term3 * sex_factor;

    // Round to nearest integer for interpretation matching
    let egfr_rounded = egfr.round() as i32;

    let sex_label = if is_female { "Female" } else { "Male" };
    let sex_label_de = if is_female { "Weiblich" } else { "Männlich" };

    let field_scores = vec![
        FieldScore {
            field: "age".to_string(),
            label: format!("Age: {:.0} years", age),
            label_de: format!("Alter: {:.0} Jahre", age),
            points: 0,
        },
        FieldScore {
            field: "sex".to_string(),
            label: format!("Sex: {}", sex_label),
            label_de: format!("Geschlecht: {}", sex_label_de),
            points: 0,
        },
        FieldScore {
            field: "creatinine".to_string(),
            label: format!(
                "Creatinine: {:.0} μmol/L ({:.2} mg/dL)",
                creatinine_umol, scr
            ),
            label_de: format!(
                "Kreatinin: {:.0} μmol/L ({:.2} mg/dL)",
                creatinine_umol, scr
            ),
            points: 0,
        },
        FieldScore {
            field: "result".to_string(),
            label: format!("eGFR: {} mL/min/1.73m²", egfr_rounded),
            label_de: format!("eGFR: {} mL/min/1,73m²", egfr_rounded),
            points: egfr_rounded,
        },
    ];

    Ok(FormulaResult {
        value: egfr_rounded,
        field_scores,
    })
}

/// KFRE 4-variable equation (2-year risk)
///
/// Reference: Tangri N, et al. JAMA 2011;305(15):1553-9
///
/// Risk score = 1 - 0.9832^exp(sum)
/// where sum = -0.2201×(age/10-7.036) + 0.2467×(male-0.5642)
///             - 0.5567×(eGFR/5-7.222) + 0.4510×(ln(ACR)-5.137)
///
/// ACR in mg/g (input is mg/mmol, converted by multiplying by 8.84)
fn calculate_kfre_4var(
    inputs: &HashMap<String, InputValue>,
) -> Result<FormulaResult, CalculationError> {
    let age = get_required_number(inputs, "age")?;
    let sex = get_required_dropdown(inputs, "sex")?;
    let egfr = get_required_number(inputs, "egfr")?;
    let acr_mg_mmol = get_required_number(inputs, "acr")?;

    // Validate ACR > 0 for ln()
    if acr_mg_mmol <= 0.0 {
        return Err(CalculationError::InvalidInput {
            field: "acr".to_string(),
            reason: "ACR must be greater than 0".to_string(),
        });
    }

    let is_male = sex == "male";
    let male_val = if is_male { 1.0 } else { 0.0 };

    // Convert ACR from mg/mmol to mg/g
    let acr_mg_g = acr_mg_mmol * 8.84;

    let sum = -0.2201 * (age / 10.0 - 7.036) + 0.2467 * (male_val - 0.5642)
        - 0.5567 * (egfr / 5.0 - 7.222)
        + 0.4510 * (acr_mg_g.ln() - 5.137);

    let risk_2yr = 1.0 - 0.9832_f64.powf(sum.exp());

    // Convert to percentage and round to integer for interpretation
    let risk_percent = (risk_2yr * 100.0).round() as i32;
    // Clamp to 0-100 range
    let risk_percent = risk_percent.clamp(0, 100);

    let sex_label = if is_male { "Male" } else { "Female" };
    let sex_label_de = if is_male { "Männlich" } else { "Weiblich" };

    let field_scores = vec![
        FieldScore {
            field: "age".to_string(),
            label: format!("Age: {:.0} years", age),
            label_de: format!("Alter: {:.0} Jahre", age),
            points: 0,
        },
        FieldScore {
            field: "sex".to_string(),
            label: format!("Sex: {}", sex_label),
            label_de: format!("Geschlecht: {}", sex_label_de),
            points: 0,
        },
        FieldScore {
            field: "egfr".to_string(),
            label: format!("eGFR: {:.0} mL/min/1.73m²", egfr),
            label_de: format!("eGFR: {:.0} mL/min/1,73m²", egfr),
            points: 0,
        },
        FieldScore {
            field: "acr".to_string(),
            label: format!("ACR: {:.1} mg/mmol ({:.0} mg/g)", acr_mg_mmol, acr_mg_g),
            label_de: format!("ACR: {:.1} mg/mmol ({:.0} mg/g)", acr_mg_mmol, acr_mg_g),
            points: 0,
        },
        FieldScore {
            field: "result".to_string(),
            label: format!("2-year risk: {}%", risk_percent),
            label_de: format!("2-Jahres-Risiko: {}%", risk_percent),
            points: risk_percent,
        },
    ];

    Ok(FormulaResult {
        value: risk_percent,
        field_scores,
    })
}

/// Helper: get a required number from inputs
fn get_required_number(
    inputs: &HashMap<String, InputValue>,
    field: &str,
) -> Result<f64, CalculationError> {
    inputs
        .get(field)
        .and_then(|v| v.as_number())
        .ok_or_else(|| CalculationError::MissingRequiredField {
            field: field.to_string(),
        })
}

/// Helper: get a required dropdown value from inputs
fn get_required_dropdown(
    inputs: &HashMap<String, InputValue>,
    field: &str,
) -> Result<String, CalculationError> {
    inputs
        .get(field)
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .ok_or_else(|| CalculationError::MissingRequiredField {
            field: field.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test case from CKD-EPI 2021 online calculator
    /// 55-year-old female, creatinine 80 μmol/L -> eGFR ~77
    #[test]
    fn test_egfr_ckd_epi_2021_female() {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(55.0));
        inputs.insert(
            "sex".to_string(),
            InputValue::Dropdown("female".to_string()),
        );
        inputs.insert("creatinine".to_string(), InputValue::Number(80.0));

        let result = calculate_egfr_ckd_epi_2021(&inputs).unwrap();
        // Expected: ~77 mL/min/1.73m² (varies slightly by calculator)
        assert!(
            result.value >= 74 && result.value <= 80,
            "eGFR for 55F Scr=80μmol/L should be ~77, got {}",
            result.value
        );
    }

    /// 70-year-old male, creatinine 120 μmol/L (1.36 mg/dL) -> eGFR ~56 (G3a)
    #[test]
    fn test_egfr_ckd_epi_2021_male() {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(70.0));
        inputs.insert("sex".to_string(), InputValue::Dropdown("male".to_string()));
        inputs.insert("creatinine".to_string(), InputValue::Number(120.0));

        let result = calculate_egfr_ckd_epi_2021(&inputs).unwrap();
        // CKD-EPI 2021: 70M, Scr=1.36mg/dL -> eGFR ~56 mL/min/1.73m² (G3a)
        assert!(
            result.value >= 53 && result.value <= 59,
            "eGFR for 70M Scr=120μmol/L should be ~56, got {}",
            result.value
        );
    }

    /// Young patient with normal creatinine -> high eGFR
    #[test]
    fn test_egfr_ckd_epi_2021_young_healthy() {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(25.0));
        inputs.insert("sex".to_string(), InputValue::Dropdown("male".to_string()));
        inputs.insert("creatinine".to_string(), InputValue::Number(80.0));

        let result = calculate_egfr_ckd_epi_2021(&inputs).unwrap();
        // Young + low creatinine = high eGFR (>100)
        assert!(
            result.value > 100,
            "eGFR for 25M Scr=80μmol/L should be >100, got {}",
            result.value
        );
    }

    /// KFRE test: 65-year-old male, eGFR 25, ACR 30 mg/mmol -> significant risk
    #[test]
    fn test_kfre_high_risk() {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(65.0));
        inputs.insert("sex".to_string(), InputValue::Dropdown("male".to_string()));
        inputs.insert("egfr".to_string(), InputValue::Number(25.0));
        inputs.insert("acr".to_string(), InputValue::Number(30.0));

        let result = calculate_kfre_4var(&inputs).unwrap();
        // Low eGFR + high ACR = significant risk (>5%)
        assert!(
            result.value > 5,
            "KFRE for 65M eGFR=25 ACR=30 should be >5%, got {}%",
            result.value
        );
    }

    /// KFRE test: 50-year-old female, eGFR 55, ACR 3 mg/mmol -> low risk
    #[test]
    fn test_kfre_low_risk() {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(50.0));
        inputs.insert(
            "sex".to_string(),
            InputValue::Dropdown("female".to_string()),
        );
        inputs.insert("egfr".to_string(), InputValue::Number(55.0));
        inputs.insert("acr".to_string(), InputValue::Number(3.0));

        let result = calculate_kfre_4var(&inputs).unwrap();
        // Higher eGFR + low ACR = low risk (<5%)
        assert!(
            result.value < 5,
            "KFRE for 50F eGFR=55 ACR=3 should be <5%, got {}%",
            result.value
        );
    }

    /// KFRE: ACR must be > 0
    #[test]
    fn test_kfre_invalid_acr() {
        let mut inputs = HashMap::new();
        inputs.insert("age".to_string(), InputValue::Number(60.0));
        inputs.insert("sex".to_string(), InputValue::Dropdown("male".to_string()));
        inputs.insert("egfr".to_string(), InputValue::Number(40.0));
        inputs.insert("acr".to_string(), InputValue::Number(0.0));

        let result = calculate_kfre_4var(&inputs);
        assert!(result.is_err());
    }
}
