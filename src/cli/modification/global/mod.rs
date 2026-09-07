use anyhow::Result;
use inquire::Text;

pub fn prompt_ignore_patterns() -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    loop {
        let input = Text::new("Enter ignore pattern (or press Enter to finish):")
            .with_placeholder("e.g., *.log, temp/*, .DS_Store")
            .with_help_message("Comma-separated values are allowed")
            .prompt()?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            break;
        }

        // Allow comma-separated entries in a single prompt line
        for pattern in trimmed.split(',') {
            let pat = pattern.trim();
            if !pat.is_empty() && !patterns.contains(&pat.to_string()) {
                patterns.push(pat.to_string());
            }
        }

        eprintln!("Current patterns: {:?}", patterns);
    }

    Ok(patterns)
}
