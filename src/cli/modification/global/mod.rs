use anyhow::Result;
use inquire::{Confirm, Text};

pub fn prompt_ignore_patterns() -> Result<Vec<String>> {
    let add_patterns = Confirm::new("Do you want to add custom ignore patterns?")
        .with_default(false)
        .with_help_message("Specify extra glob patterns to exclude during the directory crawl")
        .prompt()?;

    if !add_patterns {
        return Ok(Vec::new());
    }

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

        println!("Current patterns: {:?}", patterns);
    }

    Ok(patterns)
}
