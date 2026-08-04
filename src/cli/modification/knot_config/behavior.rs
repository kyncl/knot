use anyhow::Result;
use inquire::Select;

use crate::utils::behavior::{ConflictBehavior, UniqueBehavior};

struct MenuOption<T> {
    label: &'static str,
    value: T,
}

impl<T> std::fmt::Display for MenuOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub fn prompt_unique_behavior() -> Result<UniqueBehavior> {
    let unique_options = vec![
        MenuOption {
            label: "Archive them (Safe & Default)",
            value: UniqueBehavior::Archive,
        },
        MenuOption {
            label: "Only Add (Make files exist everywhere, never delete)",
            value: UniqueBehavior::OnlyAdd,
        },
        MenuOption {
            label: "Mirror Source (Deletes files on Remote that aren't on Source!)",
            value: UniqueBehavior::MirrorSource,
        },
        MenuOption {
            label: "Mirror Remote (Deletes files on Source that aren't on Remote!)",
            value: UniqueBehavior::MirrorRemote,
        },
        MenuOption {
            label: "Ask me what to do for each unique file",
            value: UniqueBehavior::Ask,
        },
        MenuOption {
            label: "Skip unique files entirely",
            value: UniqueBehavior::Skip,
        },
    ];

    Ok(Select::new(
        "What should we do with 'unique' files (files that only exist on one side)?",
        unique_options,
    )
    .with_help_message("Be careful with Mirror options as they can delete files")
    .prompt()?
    .value)
}

pub fn prompt_conflict_behavior() -> Result<ConflictBehavior> {
    let conflict_options = vec![
        MenuOption {
            label: "Keep the Newest file (Recommended)",
            value: ConflictBehavior::Newer,
        },
        MenuOption {
            label: "Always trust the Source",
            value: ConflictBehavior::Source,
        },
        MenuOption {
            label: "Always trust the Remote",
            value: ConflictBehavior::Remote,
        },
        MenuOption {
            label: "Keep the Oldest file",
            value: ConflictBehavior::Older,
        },
        MenuOption {
            label: "Ask me every time a conflict happens",
            value: ConflictBehavior::Ask,
        },
        MenuOption {
            label: "Skip all conflicting files",
            value: ConflictBehavior::Skip,
        },
    ];

    Ok(Select::new(
        "When a file has been modified in both places, how should we handle it?",
        conflict_options,
    )
    .with_help_message("Use arrow keys to navigate and press Enter to select")
    .prompt()?
    .value)
}
