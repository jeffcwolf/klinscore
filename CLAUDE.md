CLAUDE.md for KlinScore
markdown# CLAUDE.md

## Git Commit Strategy

**Auto-commit guidelines:**
- Commit after completing each discrete task or feature
- Commit after tests pass and code compiles without warnings
- Use descriptive, conventional commit messages
- Follow format: `<type>: <description>`

**Commit message types:**
- `feat:` - New feature (e.g., "feat: Add CHA2DS2-VA score calculator")
- `fix:` - Bug fix (e.g., "fix: Correct HAS-BLED point calculation")
- `refactor:` - Code restructuring (e.g., "refactor: Extract score loading into module")
- `test:` - Add/update tests (e.g., "test: Add unit tests for eGFR calculation")
- `docs:` - Documentation (e.g., "docs: Add user guide in German")
- `style:` - Formatting (e.g., "style: Run cargo fmt")
- `chore:` - Maintenance (e.g., "chore: Update dependencies")
- `score:` - Score definitions (e.g., "score: Add Wells Score for DVT")
- `ui:` - UI changes (e.g., "ui: Improve specialty selection layout")

**When to commit:**
1. After implementing a complete feature (e.g., score calculator working)
2. After all tests pass (`cargo test`)
3. After fixing bugs or compilation errors
4. Before switching to unrelated task
5. Every 3-5 significant changes

**Commit message examples:**
- `feat: Implement YAML score definition loader`
- `fix: Handle missing age field in CHA2DS2-VA calculation`
- `test: Add test cases from ESC 2024 guidelines`
- `refactor: Extract UI components into separate modules`
- `score: Add ASA Physical Status classification`
- `docs: Add German user guide`

**Don't commit:**
- Work in progress with compilation errors
- Code with clippy warnings (unless intentionally suppressed)
- Experimental changes you're not sure about
- Incomplete score definitions

## Docker Isolation Context

This workspace runs in an **isolated Docker container** at `/work`. The container is ephemeral and fully isolated from the host system — it is destroyed on exit and cannot access anything outside `/work`.

**What this means for autonomy:**
- Auto-run all Rust commands (cargo build, cargo test, cargo run, cargo clippy) freely — everything is sandboxed
- Create/edit `.rs`, `.toml`, `.yaml`, `.md` files without confirmation
- Ask before git commits so changes can be reviewed
- No destructive operations (`rm -rf`, `sudo`) are permitted
- Experimentation is safe — worst case is rebuilding the container

**Rust development:**
- Use `cargo add <crate>` to add dependencies
- All builds happen in `/work/target/` (isolated in container)
- Your Mac's Rust installation is completely untouched

This isolation means you can be more autonomous than usual. Compile, test, and iterate quickly without confirmation for routine development actions.

## Project Overview

**KlinScore: Clinical Score Calculator for German Hospitals**

This project demonstrates ability to build physician-facing tools for the **BIH@Charité Datenintegrationszentrum** position.

**Key aspects:**
- **Multi-specialty clinical score calculator** (Cardiology, Nephrology, Anesthesiology)
- Uses **German/European clinical guidelines** (ESC 2024, AWMF, KDIGO)
- **Open source** and **physician-maintainable** (scores via YAML config)
- **German + English interface** with proper medical terminology
- **Offline-first** desktop application (Rust + Iced)

**Core features:**
- Fast, keyboard-driven workflow (<30 seconds per calculation)
- Standardized UI across all scores
- German clinical units (mmol/L, μmol/L, etc.)
- Export to PDF/CSV for documentation
- Calculation history
- Community-driven score library

**Tech stack:**
- **Rust + Iced** (declarative UI, cross-platform)
- **YAML** score definitions (physician-editable)
- **serde** for config parsing
- **printpdf** for PDF export
- **fluent** for German/English i18n

## Autonomy Guidelines

This project runs in an **isolated Docker container**. Claude Code has high autonomy:

**✅ NO PERMISSION NEEDED (auto-execute):**
- All Rust commands (cargo build, cargo run, cargo test, cargo clippy, cargo fmt)
- Creating/editing/deleting project files (.rs, .toml, .yaml, .md)
- Running the application (cargo run)
- Adding dependencies (cargo add)
- Creating score definition files

**⚠️ ASK PERMISSION FIRST:**
- Git commits (so changes can be reviewed)
- Large deletions (rm -rf with many files)

**Rationale:** The Docker container is fully isolated and ephemeral. Maximum autonomy enables rapid iteration on UI and calculation logic.

## Development Standards

### Testing Requirements
**IMPORTANT: All score calculations MUST have tests.**

When implementing features:
1. Write the implementation
2. Write tests using examples from published guidelines
3. Run `cargo test` to ensure tests pass
4. Tests should cover:
   - Calculation correctness (using published examples)
   - Edge cases (minimum/maximum scores)
   - Input validation (missing fields, out-of-range values)
   - Unit conversions (μmol/L ↔ mg/dL)

### Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cha2ds2va_example_from_esc_2024() {
        // Example from ESC Guidelines 2024, page 3345
        let inputs = HashMap::from([
            ("age".to_string(), 72),
            ("heart_failure".to_string(), true),
            ("hypertension".to_string(), true),
        ]);
        
        let result = calculate_score("cha2ds2va", &inputs);
        assert_eq!(result.score, 3);
        assert_eq!(result.risk_category, "High");
    }
}
```

### Code Quality
- All code must compile without warnings
- All code must pass clippy: `cargo clippy -- -D warnings`
- All code must be formatted: `cargo fmt`
- All tests must pass: `cargo test`
- Document public APIs with `///` doc comments

### Score Validation
**Critical: Every score MUST be validated against published literature**

For each score:
1. Find published example in guidelines (e.g., ESC 2024)
2. Add test case with exact values from publication
3. Verify calculation matches published result
4. Document source in test comment

## Development Commands

### Rust Development
```bash
# Build the project
cargo build

# Run the application
cargo run

# Build optimized release version
cargo build --release

# Run with detailed logging
RUST_LOG=debug cargo run
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_cha2ds2va

# Run tests and show coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Code Quality
```bash
# Format code (auto-fix)
cargo fmt

# Check code style (without fixing)
cargo fmt -- --check

# Run clippy (linter)
cargo clippy

# Clippy with strict warnings
cargo clippy -- -D warnings

# Check without building
cargo check
```

### Dependencies
```bash
# Add a dependency
cargo add 

# Add dev dependency (for testing)
cargo add --dev 

# Update dependencies
cargo update
```

### Documentation
```bash
# Build and open documentation
cargo doc --open

# Build docs for all dependencies
cargo doc --no-deps --open
```

## Project Structure
```
klinscore/
├── README.md                      # Project overview
├── Cargo.toml                     # Rust dependencies
├── CLAUDE.md                     # This file (instructions for Claude)
├── .gitignore                    # Git ignore rules
│
├── src/                          # Source code
│   ├── main.rs                   # Application entry point
│   ├── ui/                       # UI components (Iced)
│   │   ├── mod.rs
│   │   ├── welcome.rs            # Welcome screen
│   │   ├── specialty.rs          # Specialty selection
│   │   └── calculator.rs         # Score calculator view
│   ├── scores/                   # Score calculation logic
│   │   ├── mod.rs
│   │   ├── loader.rs             # Load YAML definitions
│   │   ├── calculator.rs         # Generic calculation engine
│   │   └── validator.rs          # Input validation
│   ├── config/                   # Configuration management
│   │   ├── mod.rs
│   │   └── score_definition.rs  # YAML schema structs
│   ├── export/                   # Export functionality
│   │   ├── mod.rs
│   │   ├── pdf.rs                # PDF generation
│   │   └── csv.rs                # CSV export
│   └── i18n/                     # Internationalization
│       ├── mod.rs
│       ├── de.rs                 # German strings
│       └── en.rs                 # English strings
│
├── scores/                       # Score definitions (YAML)
│   ├── cardiology/
│   │   ├── cha2ds2_va.yaml       # CHA2DS2-VA Score
│   │   ├── has_bled.yaml         # HAS-BLED Score
│   │   └── grace.yaml            # GRACE Score
│   ├── nephrology/
│   │   ├── egfr_ckd_epi_2021.yaml
│   │   └── kfre.yaml
│   ├── anesthesiology/
│   │   ├── asa.yaml
│   │   └── stop_bang.yaml
│   └── templates/
│       └── score_template.yaml   # Template for new scores
│
├── tests/                        # Integration tests
│   ├── score_tests.rs            # Test all score calculations
│   ├── ui_tests.rs               # UI state tests
│   └── fixtures/                 # Test data
│       └── sample_scores.yaml
│
├── docs/                         # Documentation
│   ├── user_guide_de.md          # German user guide
│   ├── user_guide_en.md          # English user guide
│   ├── contributing.md           # How to add scores
│   └── score_definition_spec.md # YAML format specification
│
├── assets/                       # Application assets
│   ├── icon.png
│   ├── icon.icns                 # macOS
│   └── icon.ico                  # Windows
│
└── examples/                     # Example code
    └── basic_calculator.rs       # Simple calculator example
```

## Development Phases

### Phase 1: Core Application Framework (8-10 hours)
**Goal:** Basic Iced app with navigation and score loading

**Tasks:**
1. Set up Iced application structure (2 hours)
   - Welcome screen
   - Specialty selection
   - Basic navigation
2. Implement YAML score loader (2 hours)
   - Define score definition schema
   - Parse YAML files
   - Load all scores on startup
3. Language switching (German/English) (2 hours)
4. Basic UI styling and layout (2 hours)

**Deliverables:**
- Working Iced app with navigation
- YAML score loader
- German/English interface toggle
- 3 sample score definitions (for testing)

### Phase 2: Score Calculator Engine (10-12 hours)
**Goal:** Generic calculation engine that works with any score

**Tasks:**
1. Design calculation engine (3 hours)
   - Generic input handling (boolean, number, dropdown)
   - Point calculation logic
   - Risk category determination
2. Implement first 5-6 priority scores (4 hours)
   - CHA2DS2-VA (Cardiology)
   - HAS-BLED (Cardiology)
   - eGFR CKD-EPI 2021 (Nephrology)
   - ASA Physical Status (Anesthesia)
   - RCRI (Anesthesia)
3. Write comprehensive tests (3 hours)
   - Test cases from clinical guidelines
   - Edge case handling
   - Input validation

**Deliverables:**
- Generic calculation engine
- 5-6 working score calculators
- Test suite with >90% coverage
- Tests using published examples

### Phase 3: UI Polish & Features (8-10 hours)
**Goal:** Professional, usable interface with export

**Tasks:**
1. Score calculator UI (4 hours)
   - Input forms (keyboard shortcuts)
   - Real-time calculation
   - Risk category display with color coding
   - Guideline recommendations
2. Calculation history (2 hours)
   - Save recent calculations
   - View/load previous results
3. Export functionality (2 hours)
   - PDF export (printable)
   - CSV export (for records)

**Deliverables:**
- Polished calculator interface
- Calculation history
- PDF/CSV export
- Keyboard shortcuts working

### Phase 4: Complete Score Library (6-8 hours)
**Goal:** 15-20 scores across 3 specialties

**Tasks:**
1. Add remaining cardiology scores (2 hours)
   - GRACE, TIMI
2. Add nephrology scores (2 hours)
   - KFRE, RIFLE
3. Add anesthesiology scores (2 hours)
   - STOP-BANG, Caprini, Apfel
4. Validate all scores against literature (2 hours)

**Deliverables:**
- 15-20 validated score definitions
- All tests passing
- Documentation for each score

### Phase 5: Documentation & Release (4-6 hours)
**Goal:** Production-ready release

**Tasks:**
1. User guides (German + English) (2 hours)
2. Contributing guide (for physicians to add scores) (1 hour)
3. Score definition specification (1 hour)
4. Build release binaries (Windows, Mac, Linux) (1 hour)
5. Create GitHub release (1 hour)

**Deliverables:**
- Complete documentation
- Standalone executables (3 platforms)
- GitHub repository polished
- README with screenshots

## Key Technical Standards

### German Clinical Standards
- **Units:** Metric (kg, cm, mmol/L, μmol/L)
- **Lab values:** Display both German and international units
  - Creatinine: μmol/L AND mg/dL
  - Glucose: mmol/L (primary), mg/dL (secondary)
  - HbA1c: % AND mmol/mol
- **Language:** German primary, English secondary
- **Guidelines:** ESC, AWMF, KDIGO (European/German first)

### Score Definition Format
```yaml
name: "Score Name"
name_de: "German Name"
specialty: "Cardiology"
specialty_de: "Kardiologie"
version: "2024-v1"
guideline_source: "ESC"
reference: "Journal citation"
validation_status: "peer_reviewed"

inputs:
  - field: "field_name"
    type: "boolean" | "number" | "dropdown"
    label: "English label"
    label_de: "German label"
    points:  or 

interpretation:
  - score: 
    risk: "Low" | "Medium" | "High"
    risk_de: "Niedrig" | "Mittel" | "Hoch"
    recommendation: "English text"
    recommendation_de: "German text"
```

### Code Organization
- **Separation of concerns:** UI, logic, data separate
- **Testability:** All calculation logic is pure functions
- **State management:** Iced's Elm architecture
- **Error handling:** Use `Result<T, E>` and `thiserror`
- **Documentation:** Public APIs have doc comments

## Interview Talking Points

When presenting this project:

1. **"Built tool physicians actually asked for"** - Validated need with practicing physician
2. **"German/European clinical guidelines"** - ESC 2024, AWMF, KDIGO standards
3. **"Physician-maintainable without coding"** - YAML config files for new scores
4. **"Multi-specialty approach"** - Solves real workflow problem (physicians need scores from multiple specialties)
5. **"Open source and extensible"** - Can be adopted department-wide at no cost
6. **"Fast, offline, privacy-respecting"** - Addresses MDCalc pain points
7. **"Demonstrates software design for clinical users"** - Not just for engineers

**Technical highlights:**
- Rust for reliability (medical calculations can't have bugs)
- Iced for modern, cross-platform UI
- Declarative state management (Elm architecture)
- Comprehensive test coverage with published examples
- German localization and clinical units

## Notes

### Why This Approach?

**Desktop-first, not web:**
- ✅ Works offline (hospital internet unreliable)
- ✅ Faster (no network latency)
- ✅ Privacy (no data sent to servers)
- ✅ Professional tool feel

**YAML config for scores:**
- ✅ Physicians can add scores without coding
- ✅ Easy to share (one file per score)
- ✅ Version control friendly (Git can diff YAML)
- ✅ Community contributions (GitHub PRs)

**Rust + Iced:**
- ✅ Native performance
- ✅ Memory safety (critical for medical tools)
- ✅ Cross-platform (Windows, Mac, Linux)
- ✅ Modern UI framework (Iced)
- ✅ Excellent testing support

### Current Status

**✅ Completed:**
- Project structure defined
- Cargo.toml with dependencies
- Basic Iced app running
- Language toggle working

**👉 Next Steps:**
- Implement YAML score loader
- Create first score definitions
- Build calculator UI
- Add first 3-5 scores

**Timeline:** ~36-44 hours total (3-4 weekends)

### Quality Checklist

Before each commit:
- [ ] `cargo fmt` - Code formatted
- [ ] `cargo clippy` - No warnings
- [ ] `cargo test` - All tests pass
- [ ] `cargo build --release` - Release builds
- [ ] Documentation updated if needed
- [ ] Score definitions validated against literature

---

**Ready to build KlinScore!** 🎯