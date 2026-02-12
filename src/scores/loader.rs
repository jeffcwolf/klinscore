// loader.rs
// Loads clinical score definitions from YAML files

use crate::config::{ScoreDefinition, Specialty};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when loading score definitions
#[derive(Error, Debug)]
pub enum ScoreLoadError {
    #[error("Failed to read directory {path}: {source}")]
    DirectoryRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to read file {path}: {source}")]
    FileRead {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to parse YAML in {path}: {source}")]
    YamlParse {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("Invalid score definition in {path}: {reason}")]
    InvalidScore { path: String, reason: String },

    #[error("No scores directory found at {path}")]
    ScoresDirectoryNotFound { path: String },
}

/// Collection of loaded score definitions organized by specialty
#[derive(Debug, Clone)]
pub struct ScoreLibrary {
    /// All loaded scores, keyed by score ID (filename without extension)
    pub scores: HashMap<String, ScoreDefinition>,

    /// Scores organized by specialty for quick filtering
    pub by_specialty: HashMap<Specialty, Vec<String>>,

    /// Path where scores were loaded from
    #[allow(dead_code)]
    pub source_path: PathBuf,
}

impl ScoreLibrary {
    /// Get a score by its ID
    pub fn get_score(&self, score_id: &str) -> Option<&ScoreDefinition> {
        self.scores.get(score_id)
    }

    /// Get all scores for a specific specialty
    pub fn get_scores_for_specialty(&self, specialty: Specialty) -> Vec<&ScoreDefinition> {
        self.by_specialty
            .get(&specialty)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.scores.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all available specialties
    #[allow(dead_code)]
    pub fn get_specialties(&self) -> Vec<Specialty> {
        let mut specialties: Vec<_> = self.by_specialty.keys().copied().collect();
        specialties.sort_by_key(|s| format!("{:?}", s));
        specialties
    }

    /// Get total number of loaded scores
    pub fn count(&self) -> usize {
        self.scores.len()
    }
}

/// Load all score definitions from a directory
///
/// Recursively scans the directory for .yaml files and loads them as score definitions.
/// Returns a ScoreLibrary containing all successfully loaded scores.
///
/// # Arguments
///
/// * `scores_dir` - Path to the scores directory (e.g., "scores/")
///
/// # Errors
///
/// Returns `ScoreLoadError` if:
/// - The scores directory doesn't exist
/// - Files cannot be read
/// - YAML parsing fails
/// - Score definitions are invalid
///
/// # Example
///
/// ```no_run
/// use klinscore::scores::load_all_scores;
///
/// let library = load_all_scores("scores/").expect("Failed to load scores");
/// println!("Loaded {} scores", library.count());
/// ```
pub fn load_all_scores<P: AsRef<Path>>(scores_dir: P) -> Result<ScoreLibrary, ScoreLoadError> {
    let scores_dir = scores_dir.as_ref();

    // Check if directory exists
    if !scores_dir.exists() {
        return Err(ScoreLoadError::ScoresDirectoryNotFound {
            path: scores_dir.display().to_string(),
        });
    }

    let mut scores = HashMap::new();
    let mut by_specialty: HashMap<Specialty, Vec<String>> = HashMap::new();

    // Recursively find all .yaml files
    let yaml_files = find_yaml_files(scores_dir)?;

    for file_path in yaml_files {
        // Skip template files
        if file_path.to_string_lossy().contains("template") {
            continue;
        }

        // Try to load the score
        match load_score_from_file(&file_path) {
            Ok(score) => {
                // Generate score ID from filename
                let score_id = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Add to specialty index
                by_specialty
                    .entry(score.specialty)
                    .or_default()
                    .push(score_id.clone());

                // Store the score
                scores.insert(score_id, score);
            }
            Err(e) => {
                // Log warning but continue loading other scores
                eprintln!("Warning: Failed to load score from {:?}: {}", file_path, e);
            }
        }
    }

    Ok(ScoreLibrary {
        scores,
        by_specialty,
        source_path: scores_dir.to_path_buf(),
    })
}

/// Load a single score definition from a YAML file
pub fn load_score_from_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<ScoreDefinition, ScoreLoadError> {
    let file_path = file_path.as_ref();

    // Read file contents
    let contents = fs::read_to_string(file_path).map_err(|e| ScoreLoadError::FileRead {
        path: file_path.display().to_string(),
        source: e,
    })?;

    // Parse YAML
    let score: ScoreDefinition =
        serde_yaml::from_str(&contents).map_err(|e| ScoreLoadError::YamlParse {
            path: file_path.display().to_string(),
            source: e,
        })?;

    // Validate the score
    validate_score(&score, file_path)?;

    Ok(score)
}

/// Validate a score definition
fn validate_score(score: &ScoreDefinition, file_path: &Path) -> Result<(), ScoreLoadError> {
    let path = file_path.display().to_string();

    // Check required fields
    if score.name.is_empty() {
        return Err(ScoreLoadError::InvalidScore {
            path,
            reason: "Score name is empty".to_string(),
        });
    }

    if score.inputs.is_empty() {
        return Err(ScoreLoadError::InvalidScore {
            path,
            reason: "Score must have at least one input field".to_string(),
        });
    }

    if score.interpretation.is_empty() {
        return Err(ScoreLoadError::InvalidScore {
            path,
            reason: "Score must have at least one interpretation rule".to_string(),
        });
    }

    // Validate input fields
    for (i, input) in score.inputs.iter().enumerate() {
        if input.field.is_empty() {
            return Err(ScoreLoadError::InvalidScore {
                path,
                reason: format!("Input field {} has empty field name", i),
            });
        }

        if input.label.is_empty() {
            return Err(ScoreLoadError::InvalidScore {
                path,
                reason: format!("Input field '{}' has empty label", input.field),
            });
        }

        // Check for duplicate field names
        let duplicate_count = score
            .inputs
            .iter()
            .filter(|f| f.field == input.field)
            .count();
        if duplicate_count > 1 {
            return Err(ScoreLoadError::InvalidScore {
                path,
                reason: format!("Duplicate field name: '{}'", input.field),
            });
        }
    }

    Ok(())
}

/// Recursively find all .yaml files in a directory
fn find_yaml_files<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>, ScoreLoadError> {
    let dir = dir.as_ref();
    let mut yaml_files = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| ScoreLoadError::DirectoryRead {
        path: dir.display().to_string(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ScoreLoadError::DirectoryRead {
            path: dir.display().to_string(),
            source: e,
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Recursively search subdirectories
            let mut sub_files = find_yaml_files(&path)?;
            yaml_files.append(&mut sub_files);
        } else if path.is_file() {
            // Check if it's a .yaml or .yml file
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    yaml_files.push(path);
                }
            }
        }
    }

    Ok(yaml_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_find_yaml_files() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test directory structure
        fs::create_dir_all(base_path.join("cardiology")).unwrap();
        fs::create_dir_all(base_path.join("nephrology")).unwrap();

        // Create some YAML files
        fs::File::create(base_path.join("cardiology/score1.yaml")).unwrap();
        fs::File::create(base_path.join("cardiology/score2.yml")).unwrap();
        fs::File::create(base_path.join("nephrology/score3.yaml")).unwrap();
        fs::File::create(base_path.join("readme.txt")).unwrap(); // Non-YAML file

        let yaml_files = find_yaml_files(base_path).unwrap();

        assert_eq!(yaml_files.len(), 3);
        assert!(yaml_files
            .iter()
            .any(|p| p.ends_with("cardiology/score1.yaml")));
        assert!(yaml_files
            .iter()
            .any(|p| p.ends_with("cardiology/score2.yml")));
        assert!(yaml_files
            .iter()
            .any(|p| p.ends_with("nephrology/score3.yaml")));
    }

    #[test]
    fn test_load_score_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let score_file = temp_dir.path().join("test_score.yaml");

        let yaml_content = r#"
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
    points: 1
interpretation:
  - score: 0
    risk: "Low"
    risk_de: "Niedrig"
    risk_level: Low
    recommendation: "Test"
    recommendation_de: "Test"
"#;

        let mut file = fs::File::create(&score_file).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        let score = load_score_from_file(&score_file).unwrap();
        assert_eq!(score.name, "Test Score");
        assert_eq!(score.specialty, Specialty::Cardiology);
        assert_eq!(score.inputs.len(), 1);
    }

    #[test]
    fn test_validate_score_empty_name() {
        let score = ScoreDefinition {
            name: String::new(), // Invalid: empty name
            name_de: "Test".to_string(),
            specialty: Specialty::Cardiology,
            specialty_de: "Kardiologie".to_string(),
            version: "1.0".to_string(),
            guideline_source: "Test".to_string(),
            reference: "Test".to_string(),
            validation_status: "draft".to_string(),
            description: String::new(),
            description_de: String::new(),
            inputs: vec![],
            interpretation: vec![],
            metadata: HashMap::new(),
        };

        let result = validate_score(&score, Path::new("test.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_all_scores() {
        // Test loading from the actual scores directory
        let scores_dir = Path::new("scores");

        // Only run this test if the scores directory exists
        if scores_dir.exists() {
            let library = load_all_scores(scores_dir).unwrap();

            // Should have loaded at least the example score
            assert!(library.count() > 0);

            // Should have Cardiology specialty
            assert!(library.by_specialty.contains_key(&Specialty::Cardiology));
        }
    }

    #[test]
    fn test_score_library_methods() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        fs::create_dir_all(base_path.join("cardiology")).unwrap();

        let yaml_content = r#"
name: "Test Score"
name_de: "Test-Score"
specialty: Cardiology
specialty_de: "Kardiologie"
version: "1.0"
guideline_source: "Test"
reference: "Test"
validation_status: "draft"
inputs:
  - field: "test"
    type: "boolean"
    label: "Test"
    label_de: "Test"
    points: 1
interpretation:
  - score: 0
    risk: "Low"
    risk_de: "Niedrig"
    risk_level: Low
    recommendation: "Test"
    recommendation_de: "Test"
"#;

        let mut file = fs::File::create(base_path.join("cardiology/test.yaml")).unwrap();
        file.write_all(yaml_content.as_bytes()).unwrap();

        let library = load_all_scores(base_path).unwrap();

        // Test get_score
        assert!(library.get_score("test").is_some());
        assert!(library.get_score("nonexistent").is_none());

        // Test get_scores_for_specialty
        let cardio_scores = library.get_scores_for_specialty(Specialty::Cardiology);
        assert_eq!(cardio_scores.len(), 1);

        // Test get_specialties
        let specialties = library.get_specialties();
        assert!(specialties.contains(&Specialty::Cardiology));

        // Test count
        assert_eq!(library.count(), 1);
    }
}
