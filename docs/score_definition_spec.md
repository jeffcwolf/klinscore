# KlinScore YAML Score Definition Specification

## Overview

This document describes the YAML format for defining clinical scores in KlinScore. The format is designed to be:
- **Physician-friendly**: No coding required, just fill in the fields
- **Comprehensive**: Supports all common score types (boolean, numeric, dropdown)
- **Bilingual**: German and English labels for all fields
- **Validated**: Cites source guidelines for verification

## File Structure

Each score is defined in a single YAML file located in:
```
scores/
├── cardiology/
├── nephrology/
├── anesthesiology/
└── templates/
    └── score_template.yaml
```

## Required Fields

### Basic Information

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `name` | String | English name of the score | `"CHA2DS2-VA Score"` |
| `name_de` | String | German name of the score | `"CHA2DS2-VA-Score"` |
| `specialty` | Enum | Medical specialty (PascalCase) | `Cardiology` |
| `specialty_de` | String | German specialty name | `"Kardiologie"` |
| `version` | String | Version identifier | `"2024-v1"` |
| `guideline_source` | String | Source guideline | `"ESC 2024"` |
| `reference` | String | Full citation | `"Author et al. Journal. 2024..."` |
| `validation_status` | String | Status: `peer_reviewed`, `draft`, `community`, `experimental` | `"peer_reviewed"` |

### Optional Metadata

| Field | Type | Description |
|-------|------|-------------|
| `description` | String | Brief English description |
| `description_de` | String | Brief German description |
| `metadata` | Map | Additional key-value pairs (tags, DOI, etc.) |

## Input Fields

The `inputs` array defines all fields the user must fill in to calculate the score.

### Input Field Structure

```yaml
inputs:
  - field: "unique_identifier"      # Required: snake_case identifier
    type: "boolean"                 # Required: boolean, number, dropdown
    label: "English Label"          # Required
    label_de: "German Label"        # Required
    points: <points_value>          # Required: see Points Values section
    unit: "years"                   # Optional: unit of measurement
    unit_de: "Jahre"                # Optional: German unit
    help: "Help text"               # Optional: tooltip/help text
    help_de: "Hilfetext"            # Optional: German help text
    min: 0                          # Optional: for number type
    max: 120                        # Optional: for number type
    options: [...]                  # Required for dropdown type
    required: true                  # Optional: default true
```

### Input Types

#### 1. Boolean Input

Simple yes/no checkbox:

```yaml
- field: "hypertension"
  type: "boolean"
  label: "Hypertension"
  label_de: "Hypertonie"
  points: 1                         # Fixed points if checked
  required: true
```

#### 2. Number Input

Numeric value with optional range validation:

```yaml
- field: "age"
  type: "number"
  label: "Age"
  label_de: "Alter"
  unit: "years"
  unit_de: "Jahre"
  min: 0
  max: 120
  points: 1                         # Can be fixed or conditional
  required: true
```

#### 3. Dropdown Input

Selection from predefined options:

```yaml
- field: "asa_class"
  type: "dropdown"
  label: "ASA Physical Status"
  label_de: "ASA-Klassifikation"
  options:
    - value: "asa_i"
      label: "ASA I - Healthy"
      label_de: "ASA I - Gesund"
      points: 0
      description: "Normal healthy patient"
      description_de: "Normaler gesunder Patient"
    - value: "asa_ii"
      label: "ASA II - Mild disease"
      label_de: "ASA II - Leichte Erkrankung"
      points: 1
  required: true
```

### Points Values

Points can be **fixed** or **conditional**:

#### Fixed Points

```yaml
points: 1                           # Always 1 point
```

#### Conditional Points

Points based on numeric value ranges:

```yaml
points:
  - condition: ">= 75"
    points: 2
    label: "Age ≥75 years"
    label_de: "Alter ≥75 Jahre"
  - condition: ">= 65"
    points: 1
    label: "Age 65-74 years"
    label_de: "Alter 65-74 Jahre"
  # If no condition matches, 0 points
```

**Supported comparison operators:**
- `>` Greater than
- `>=` Greater than or equal
- `<` Less than
- `<=` Less than or equal
- `==` Equal to
- `!=` Not equal to

**Evaluation order:** Top to bottom, first match wins.

## Interpretation Rules

The `interpretation` array maps calculated scores to risk categories and clinical recommendations.

### Interpretation Structure

```yaml
interpretation:
  - score: 0                        # Can be number or range string
    risk: "Low Risk"
    risk_de: "Niedriges Risiko"
    risk_level: Low                 # For color coding
    recommendation: "Clinical recommendation in English"
    recommendation_de: "Klinische Empfehlung auf Deutsch"
    details: "Optional details"     # Optional
    details_de: "Optionale Details" # Optional
```

### Score Matching

| Format | Example | Matches |
|--------|---------|---------|
| Exact number | `0` | Score exactly 0 |
| Range | `"1-2"` | Score 1 or 2 (inclusive) |
| Greater/equal | `"≥3"` or `">= 3"` | Score 3 or higher |
| Greater than | `">5"` | Score strictly greater than 5 |
| Less/equal | `"<=2"` or `"≤2"` | Score 2 or lower |
| Less than | `"<10"` | Score strictly less than 10 |

### Risk Levels

Risk levels control color coding in the UI:

| Level | Color | Hex Code | Use Case |
|-------|-------|----------|----------|
| `VeryLow` | Dark green | `#4CAF50` | Very low risk |
| `Low` | Light green | `#8BC34A` | Low risk |
| `Moderate` | Yellow | `#FFC107` | Moderate risk |
| `High` | Orange | `#FF9800` | High risk |
| `VeryHigh` | Red | `#F44336` | Very high risk |
| `Critical` | Dark red | `#B71C1C` | Critical/life-threatening |
| `None` | Gray | `#9E9E9E` | Informational only |

## Complete Example: CHA2DS2-VA Score

```yaml
name: "CHA2DS2-VA Score"
name_de: "CHA2DS2-VA-Score"
specialty: Cardiology
specialty_de: "Kardiologie"
version: "2024-v1"
guideline_source: "ESC 2024"
reference: "ESC Guidelines for the management of atrial fibrillation. Eur Heart J. 2024;45:3314-3414."
validation_status: "peer_reviewed"
description: "Stroke risk stratification in atrial fibrillation"
description_de: "Schlaganfallrisiko-Stratifizierung bei Vorhofflimmern"

inputs:
  - field: "age"
    type: "number"
    label: "Age"
    label_de: "Alter"
    unit: "years"
    unit_de: "Jahre"
    min: 18
    max: 120
    points:
      - condition: ">= 75"
        points: 2
        label: "Age ≥75"
        label_de: "Alter ≥75"
      - condition: ">= 65"
        points: 1
        label: "Age 65-74"
        label_de: "Alter 65-74"
    required: true

  - field: "heart_failure"
    type: "boolean"
    label: "Congestive heart failure"
    label_de: "Herzinsuffizienz"
    points: 1
    help: "History of CHF or objective evidence of reduced LVEF"
    help_de: "Anamnese einer Herzinsuffizienz oder objektiver Nachweis einer reduzierten LVEF"
    required: true

  - field: "hypertension"
    type: "boolean"
    label: "Hypertension"
    label_de: "Hypertonie"
    points: 1
    help: "Blood pressure consistently >140/90 mmHg or on antihypertensive treatment"
    help_de: "Blutdruck durchgehend >140/90 mmHg oder antihypertensive Behandlung"
    required: true

  - field: "diabetes"
    type: "boolean"
    label: "Diabetes mellitus"
    label_de: "Diabetes mellitus"
    points: 1
    help: "Fasting glucose >125 mg/dL or on treatment"
    help_de: "Nüchternglukose >125 mg/dL oder unter Behandlung"
    required: true

  - field: "stroke_tia"
    type: "boolean"
    label: "Prior stroke or TIA"
    label_de: "Schlaganfall oder TIA in der Anamnese"
    points: 2
    help: "Previous stroke, TIA, or thromboembolism"
    help_de: "Früherer Schlaganfall, TIA oder Thromboembolie"
    required: true

  - field: "vascular_disease"
    type: "boolean"
    label: "Vascular disease"
    label_de: "Gefäßerkrankung"
    points: 1
    help: "Prior MI, peripheral artery disease, or aortic plaque"
    help_de: "Früherer MI, periphere arterielle Verschlusskrankheit oder Aortenplaque"
    required: true

interpretation:
  - score: 0
    risk: "Low"
    risk_de: "Niedrig"
    risk_level: Low
    recommendation: "Anticoagulation not recommended (may consider if additional risk factors present)"
    recommendation_de: "Antikoagulation nicht empfohlen (kann bei zusätzlichen Risikofaktoren erwogen werden)"
    details: "Annual stroke risk: 0-0.2%"
    details_de: "Jährliches Schlaganfallrisiko: 0-0,2%"

  - score: 1
    risk: "Low-Moderate"
    risk_de: "Niedrig-Mittel"
    risk_level: Moderate
    recommendation: "Consider anticoagulation based on individual assessment, bleeding risk, and patient preference"
    recommendation_de: "Antikoagulation erwägen basierend auf individueller Einschätzung, Blutungsrisiko und Patientenpräferenz"
    details: "Annual stroke risk: 0.6-2%"
    details_de: "Jährliches Schlaganfallrisiko: 0,6-2%"

  - score: "≥2"
    risk: "Moderate-High"
    risk_de: "Mittel-Hoch"
    risk_level: High
    recommendation: "Anticoagulation recommended unless contraindicated (ESC Class I)"
    recommendation_de: "Antikoagulation empfohlen, außer kontraindiziert (ESC Klasse I)"
    details: "Annual stroke risk: >2.2%"
    details_de: "Jährliches Schlaganfallrisiko: >2,2%"

metadata:
  tags: "atrial fibrillation, stroke risk, anticoagulation, NOAC, warfarin"
  keywords_de: "Vorhofflimmern, Schlaganfallrisiko, Antikoagulation"
  category: "risk_stratification"
  target_population: "Adults with atrial fibrillation"
  doi: "10.1093/eurheartj/ehad123"
  guideline_year: "2024"
```

## Validation Guidelines

### For Score Authors

1. **Cite Source**: Always reference the exact guideline or paper
2. **Include Examples**: Find published example calculations and verify your YAML produces the same results
3. **Use SI Units**: Primary units should be metric (μmol/L, mmol/L, kg, cm)
4. **Bilingual**: Always provide both English and German labels
5. **Test**: Create test cases before submitting

### For Reviewers

1. Verify reference is accurate and accessible
2. Check that examples from literature calculate correctly
3. Ensure German translations are medically accurate
4. Confirm risk categories match guideline recommendations
5. Validate that all required fields are present

## Common Patterns

### Pattern 1: Age-Based Scoring

```yaml
- field: "age"
  type: "number"
  label: "Age"
  label_de: "Alter"
  unit: "years"
  points:
    - condition: ">= 75"
      points: 2
    - condition: ">= 65"
      points: 1
```

### Pattern 2: Lab Value with Threshold

```yaml
- field: "creatinine"
  type: "number"
  label: "Serum Creatinine"
  label_de: "Serum-Kreatinin"
  unit: "μmol/L"
  help: "177 μmol/L = 2.0 mg/dL"
  points:
    - condition: "> 177"
      points: 1
```

### Pattern 3: Multiple Boolean Risk Factors

```yaml
- field: "hypertension"
  type: "boolean"
  label: "Hypertension"
  label_de: "Hypertonie"
  points: 1

- field: "diabetes"
  type: "boolean"
  label: "Diabetes"
  label_de: "Diabetes"
  points: 1
```

### Pattern 4: Ordinal Classification

```yaml
- field: "killip_class"
  type: "dropdown"
  label: "Killip Class"
  label_de: "Killip-Klassifikation"
  options:
    - value: "class_i"
      label: "Class I - No heart failure"
      label_de: "Klasse I - Keine Herzinsuffizienz"
      points: 0
    - value: "class_ii"
      label: "Class II - Rales, S3 gallop"
      label_de: "Klasse II - Rasselgeräusche, S3-Galopp"
      points: 1
    - value: "class_iii"
      label: "Class III - Pulmonary edema"
      label_de: "Klasse III - Lungenödem"
      points: 2
    - value: "class_iv"
      label: "Class IV - Cardiogenic shock"
      label_de: "Klasse IV - Kardiogener Schock"
      points: 3
```

## Best Practices

### Naming Conventions

- **Field identifiers**: `snake_case` (e.g., `heart_failure`, `systolic_bp`)
- **Enum values**: `snake_case` for dropdown options (e.g., `class_i`, `nyha_ii`)
- **Specialty**: `PascalCase` (e.g., `Cardiology`, `InternalMedicine`)

### German Medical Terminology

Use standard German medical terms:
- Herzinsuffizienz (heart failure)
- Hypertonie (hypertension)
- Schlaganfall (stroke)
- Diabetes mellitus (unchanged)
- Niereninsuffizienz (kidney failure)

### Units

Standard metric/SI units:
- Age: `years` / `Jahre`
- Weight: `kg` / `kg`
- Height: `cm` / `cm`
- Creatinine: `μmol/L` / `μmol/L`
- Glucose: `mmol/L` / `mmol/L`
- Blood pressure: `mmHg` / `mmHg`

### Help Text

Provide clarification for:
- Diagnostic criteria (e.g., "BP >140/90 mmHg or on treatment")
- Unit conversions (e.g., "177 μmol/L = 2.0 mg/dL")
- Clinical definitions (e.g., "LVEF <40%")

## Troubleshooting

### Common Issues

**Issue**: Score calculates incorrectly
**Solution**: Check condition order - first match wins. Most restrictive conditions should come first.

**Issue**: Dropdown doesn't show up
**Solution**: Ensure `options` array is present and has at least 2 items.

**Issue**: German text shows English
**Solution**: Verify `*_de` fields are filled in for all labels.

**Issue**: YAML parse error
**Solution**: Check indentation (use 2 spaces, not tabs). Ensure strings with special characters are quoted.

## File Naming

Convention: `{score_acronym}.yaml`

Examples:
- `cha2ds2_va.yaml`
- `has_bled.yaml`
- `egfr_ckd_epi_2021.yaml`
- `grace.yaml`

Use lowercase with underscores, not hyphens.

## Version Control

When updating a score:
1. Increment the `version` field (e.g., `"2024-v1"` → `"2024-v2"`)
2. Document changes in git commit message
3. Update `reference` if guideline changed
4. Re-run validation tests

## Questions?

For questions about the YAML format or contributing scores, see:
- `docs/contributing.md` - How to contribute
- `scores/templates/score_template.yaml` - Commented template
- GitHub Issues - Report problems or ask questions
