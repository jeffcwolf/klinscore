// Example: Load and display all available scores
use klinscore::scores::load_all_scores;

fn main() {
    println!("KlinScore - Score Library Loader Demo\n");

    // Load all scores from the scores/ directory
    match load_all_scores("scores/") {
        Ok(library) => {
            println!("✓ Successfully loaded {} scores\n", library.count());

            // Display scores by specialty
            for specialty in library.get_specialties() {
                println!("━━━ {} ━━━", specialty.to_english());
                let scores = library.get_scores_for_specialty(specialty);

                for score in scores {
                    println!("  • {} ({})", score.name, score.name_de);
                    println!("    Source: {}", score.guideline_source);
                    println!("    Inputs: {}", score.inputs.len());
                    println!(
                        "    Risk categories: {}",
                        score.interpretation.len()
                    );
                    println!();
                }
            }

            // Demo: Get a specific score
            if let Some(score) = library.get_score("cha2ds2_va_example") {
                println!("\n━━━ Example Score Details ━━━");
                println!("Name: {}", score.name);
                println!("Description: {}", score.description);
                println!("\nInput fields:");
                for input in &score.inputs {
                    println!("  - {} ({})", input.label, input.field);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Error loading scores: {}", e);
            std::process::exit(1);
        }
    }
}
