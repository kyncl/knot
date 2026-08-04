use anyhow::Result;
use inquire::{Confirm, Text, validator::Validation};
use parse_size::parse_size;

pub fn prompt_allow_size_limit() -> Result<bool> {
    let choice = Confirm::new("Would you like to set a maximum file size limit?")
        .with_default(false)
        .with_help_message("If enabled, files exceeding this size will be skipped")
        .prompt()?;

    Ok(choice)
}

pub fn prompt_size_limit() -> Result<u64> {
    let validator = |input: &str| match parse_size(input) {
        Ok(_) => Ok(Validation::Valid),
        Err(_) => Ok(Validation::Invalid(
            "Invalid size format. Please use formats like '10MB', '1GiB', or '500KB'".into(),
        )),
    };

    let input = Text::new("Enter maximum file size:")
        .with_placeholder("e.g., 10MB, 500KB, 1GB")
        .with_help_message("Supports standard units like KB, MB, GB")
        .with_validator(validator)
        .prompt()?;
    let parsed_size = parse_size(&input)?;
    Ok(parsed_size)
}
