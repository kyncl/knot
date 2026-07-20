use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_FULL_CONDENSED,
};

use crate::{
    knot::{file::KnotFile, file_diffs::FileDiffs},
    utils::formatting::{format_hash, format_relative_time},
};
const MAX_ROWS_PER_TABLE: usize = 25;

pub fn file_diff_visualization_table(diffs: &FileDiffs) {
    let term_width = terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), _)| w as u16)
        .unwrap_or(80);
    println!("\n{}", "=== SYNCHRONIZATION REPORT ===".bold().cyan());
    println!(
        "{} {} | {} {} | {} {} | {} {}",
        "++ Source Unique:".green(),
        diffs.source_unique.len().to_string().bold(),
        "-- Remote Unique:".blue(),
        diffs.remote_unique.len().to_string().bold(),
        "!! Conflicts:".red(),
        diffs.conflicts.len().to_string().bold(),
        "== Archived:".yellow(),
        diffs.archived.len().to_string().bold(),
    );
    println!("{}", "─".repeat(term_width as usize).bright_black());
    if !diffs.conflicts.is_empty() {
        println!(
            "\n {}",
            "!!  CONFLICTS (Modified on both devices)".bold().red()
        );
        render_conflicts_table(diffs, term_width);
    }

    if !diffs.source_unique.is_empty() {
        println!(
            "\n {}",
            "++ SOURCE UNIQUE (Missing on Remote)".bold().green()
        );
        render_standard_table(&diffs.source_unique, term_width, TableColor::Green);
    }

    if !diffs.remote_unique.is_empty() {
        println!(
            "\n {}",
            "-- REMOTE UNIQUE (Missing on Source)".bold().blue()
        );
        render_standard_table(&diffs.remote_unique, term_width, TableColor::Blue);
    }

    if !diffs.archived.is_empty() {
        println!("\n {}", "== ARCHIVED FILES".bold().yellow());
        render_standard_table(&diffs.archived, term_width, TableColor::Yellow);
    }

    println!();
}

fn render_standard_table(files: &[KnotFile], width: u16, accent_color: TableColor) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(width);

    table.set_header(vec![
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Path").add_attribute(Attribute::Bold),
        Cell::new("Modified").add_attribute(Attribute::Bold),
        Cell::new("Hash").add_attribute(Attribute::Bold),
    ]);

    for file in files.iter().take(MAX_ROWS_PER_TABLE) {
        let file_type = if file.is_dir { "DIR" } else { "FILE" };
        table.add_row(vec![
            Cell::new(file_type).fg(accent_color),
            Cell::new(file.path.display().to_string()),
            Cell::new(format_relative_time(file.mtime)),
            Cell::new(format_hash(file.content_hash)).fg(TableColor::DarkGrey),
        ]);
    }

    if files.len() > MAX_ROWS_PER_TABLE {
        let remaining = files.len() - MAX_ROWS_PER_TABLE;
        table.add_row(vec![
            Cell::new("...").fg(accent_color),
            Cell::new(format!("and {} more items not shown", remaining))
                .add_attribute(Attribute::Italic),
            Cell::new("-"),
            Cell::new("-"),
        ]);
    }

    println!("{table}");
}

fn render_conflicts_table(diffs: &FileDiffs, width: u16) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(width);

    table.set_header(vec![
        Cell::new("File Path").add_attribute(Attribute::Bold),
        Cell::new("Source Modified").add_attribute(Attribute::Bold),
        Cell::new("Remote Modified").add_attribute(Attribute::Bold),
        Cell::new("Source Hash").add_attribute(Attribute::Bold),
        Cell::new("Remote Hash").add_attribute(Attribute::Bold),
    ]);

    for (source, remote) in diffs.conflicts.iter().take(MAX_ROWS_PER_TABLE) {
        table.add_row(vec![
            Cell::new(source.path.display().to_string()).fg(TableColor::Red),
            Cell::new(format_relative_time(source.mtime)),
            Cell::new(format_relative_time(remote.mtime)),
            Cell::new(format_hash(source.content_hash)).fg(TableColor::Green),
            Cell::new(format_hash(remote.content_hash)).fg(TableColor::Blue),
        ]);
    }

    if diffs.conflicts.len() > MAX_ROWS_PER_TABLE {
        let remaining = diffs.conflicts.len() - MAX_ROWS_PER_TABLE;
        table.add_row(vec![
            Cell::new(format!("... and {} more conflicting files", remaining))
                .add_attribute(Attribute::Italic)
                .fg(TableColor::Red),
            Cell::new("-"),
            Cell::new("-"),
            Cell::new("-"),
            Cell::new("-"),
        ]);
    }

    println!("{table}");
}
