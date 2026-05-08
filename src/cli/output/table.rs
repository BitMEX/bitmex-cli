/// Human-readable table output renderer using comfy-table.
use comfy_table::{presets::UTF8_FULL_CONDENSED, ContentArrangement, Table};

use super::CommandOutput;

/// Render a `CommandOutput` as a human-readable table to stdout.
pub(crate) fn render(output: &CommandOutput) {
    if output.rows.is_empty() {
        println!("No results.");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(&output.headers);

    for row in &output.rows {
        table.add_row(row);
    }

    println!("{table}");
}
