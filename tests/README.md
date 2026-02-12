# KlinScore Testing Documentation

## Test Structure

KlinScore uses a comprehensive testing strategy with three levels:

### 1. Unit Tests (in `src/`)
Located in `#[cfg(test)] mod tests` blocks within each module:
- **`src/config/score_definition.rs`**: Data structure serialization/deserialization
- **`src/scores/loader.rs`**: File loading, validation, directory scanning
- **`src/scores/calculator.rs`**: Score calculation logic, condition evaluation

Run unit tests:
```bash
cargo test --lib
```

### 2. Integration Tests (in `tests/`)
Test complete workflows from loading to calculation:
- **`integration_tests.rs`**: End-to-end scenarios

Run integration tests:
```bash
cargo test --test integration_tests
```

### 3. Clinical Validation Tests
Each score MUST have tests using published examples from clinical guidelines:

```rust
#[test]
fn test_cha2ds2va_example_from_esc_2024() {
    // Example from ESC Guidelines 2024, page 3345
    let inputs = HashMap::from([
        ("age", 72),
        ("heart_failure", true),
        ("hypertension", true),
    ]);

    let result = calculate_score("cha2ds2va", &inputs);
    assert_eq!(result.score, 3);
    assert_eq!(result.risk, "High");
}
```

## Test Coverage Goals

- **Unit tests**: >90% coverage for all calculation logic
- **Integration tests**: Cover all critical workflows
- **Clinical validation**: Every score has ≥1 test from published literature

## Running All Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_end_to_end_workflow

# Run tests for a specific module
cargo test scores::calculator

# Check test coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Test Fixtures

Test data files are stored in `tests/fixtures/`:
- Sample YAML score definitions
- Example patient data sets
- Expected calculation results

## Adding New Tests

When adding a new score:

1. **Find published examples** from the source guideline
2. **Create test case** with exact values from publication
3. **Document source** in test comment
4. **Verify calculation** matches published result

Example:
```rust
#[test]
fn test_has_bled_example_from_guideline() {
    // Example from ESC AF Guidelines 2024, Table 8
    // Patient: Age 75, HTN, abnormal liver, prior stroke
    // Expected: HAS-BLED score = 4 (high bleeding risk)

    let inputs = HashMap::from([
        ("age_over_65", true),          // 1 point
        ("hypertension", true),         // 1 point
        ("abnormal_liver", true),       // 1 point
        ("stroke_history", true),       // 1 point
        // Other fields...
    ]);

    let result = calculate_score("has_bled", &inputs).unwrap();
    assert_eq!(result.total_score, 4);
    assert_eq!(result.risk, "High");
}
```

## Continuous Integration

Tests run automatically on:
- Every commit (pre-commit hook)
- Pull requests (GitHub Actions)
- Before release builds

All tests must pass before merging.

## Test Quality Standards

✅ **Good test:**
- Uses real clinical examples from guidelines
- Documents source with page number
- Tests edge cases (boundaries, thresholds)
- Clear assertion messages

❌ **Bad test:**
- Made-up example values
- No source documentation
- Tests only happy path
- Unclear what's being validated

## Current Test Status

```
✅ config::score_definition::tests (5 tests)
✅ scores::loader::tests (5 tests)
✅ scores::calculator::tests (7 tests)
✅ integration_tests (5 tests)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 22 tests passing
```

Run `cargo test` to verify current status.
