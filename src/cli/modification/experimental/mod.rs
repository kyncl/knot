use anyhow::Result;
use inquire::Confirm;

pub fn prompt_asychrnous_sync() -> Result<bool> {
    let choice = Confirm::new("Do you want to do asynchronous synchronization?")
        .with_default(false)
        .with_help_message(
            "You'll get faster synchronization with multiple Remote Knots for price of broken UI, BUT possible race conditioning.
Use at your own risk.",
        )
        .prompt()?;

    Ok(choice)
}
