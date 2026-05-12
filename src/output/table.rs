use humansize::{format_size, BINARY};
use owo_colors::OwoColorize;
use tabled::settings::{object::Columns, Alignment, Modify, Style};
use tabled::{Table, Tabled};

use crate::cli::doctor::Finding;
use crate::model::Tool;

#[derive(Tabled)]
struct Row {
    name: String,
    version: String,
    source: String,
    size: String,
    path: String,
}

fn rows_from(tools: &[Tool]) -> Vec<Row> {
    tools
        .iter()
        .map(|t| Row {
            name: t.name.clone(),
            version: t.version.clone().unwrap_or_default(),
            source: t.source.label().to_string(),
            size: if t.size_bytes == 0 {
                String::new()
            } else {
                format_size(t.size_bytes, BINARY)
            },
            path: t.bin_path.display().to_string(),
        })
        .collect()
}

pub fn print_list(tools: &[Tool]) {
    if tools.is_empty() {
        println!("{}", "(no tools found)".dimmed());
        return;
    }
    let rows = rows_from(tools);
    let mut t = Table::new(rows);
    t.with(Style::psql())
        .with(Modify::new(Columns::single(3)).with(Alignment::right()));
    println!("{t}");
    println!(
        "{} tool{} listed",
        tools.len(),
        if tools.len() == 1 { "" } else { "s" }
    );
}

pub fn print_size(tools: &[Tool], total: u64) {
    let rows = rows_from(tools);
    let mut t = Table::new(rows);
    t.with(Style::psql())
        .with(Modify::new(Columns::single(3)).with(Alignment::right()));
    println!("{t}");
    println!("{}: {}", "total".bold(), format_size(total, BINARY).bold());
}

#[derive(Tabled)]
struct FindingRow {
    severity: String,
    kind: String,
    message: String,
}

pub fn print_doctor(findings: &[Finding]) {
    if findings.is_empty() {
        println!("{}", "everything looks healthy".green());
        return;
    }
    let rows: Vec<FindingRow> = findings
        .iter()
        .map(|f| FindingRow {
            severity: f.severity.to_string(),
            kind: f.kind.to_string(),
            message: f.message.clone(),
        })
        .collect();
    let mut t = Table::new(rows);
    t.with(Style::psql());
    println!("{t}");
}
