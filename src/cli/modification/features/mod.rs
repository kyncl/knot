use anyhow::Result;
use inquire::Confirm;

pub fn prompt_allow_caching() -> Result<bool> {
    let choice = Confirm::new("Enable structure caching?")
        .with_default(true)
        .with_help_message("Stores directory structure to speed up subsequent executions")
        .prompt()?;

    Ok(choice)
}

pub fn prompt_allow_compression() -> Result<bool> {
    let choice = Confirm::new("Enable compression?")
        .with_default(false)
        .with_help_message(
            "Reduces payload size to speed up network transfer, but uses additional CPU",
        )
        .prompt()?;

    Ok(choice)
}

pub fn prompt_allow_gitignore() -> Result<bool> {
    let choice = Confirm::new("Respect .gitignore rules?")
        .with_default(true)
        .with_help_message(
            "Automatically skips ignored files and directories in your `.gitignore` file (e.g., node_modules, target)",
        )
        .prompt()?;

    Ok(choice)
}
