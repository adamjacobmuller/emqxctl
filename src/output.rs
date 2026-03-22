use colored::Colorize;
use serde_json::Value;
use tabled::settings::Style;
use tabled::Table;

use crate::client::PaginationMeta;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Wide,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub header: &'static str,
    pub json_path: &'static str,
    pub max_width: Option<usize>,
}

pub struct OutputFormatter {
    pub format: OutputFormat,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Extract a value from a JSON object using a dot-separated path.
    fn extract(value: &Value, path: &str) -> String {
        let mut current = value;
        for part in path.split('.') {
            match current {
                Value::Object(map) => {
                    current = match map.get(part) {
                        Some(v) => v,
                        None => return String::new(),
                    };
                }
                Value::Array(arr) => {
                    if let Ok(idx) = part.parse::<usize>() {
                        current = match arr.get(idx) {
                            Some(v) => v,
                            None => return String::new(),
                        };
                    } else {
                        return String::new();
                    }
                }
                _ => return String::new(),
            }
        }
        Self::value_to_string(current)
    }

    fn value_to_string(value: &Value) -> String {
        match value {
            Value::Null => String::new(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::value_to_string).collect();
                items.join(", ")
            }
            Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
        }
    }

    fn truncate(s: &str, max: usize) -> String {
        if s.len() <= max {
            s.to_string()
        } else if max > 3 {
            format!("{}...", &s[..max - 3])
        } else {
            s[..max].to_string()
        }
    }

    pub fn print_list(
        &self,
        items: &[Value],
        columns: &[Column],
        wide_columns: Option<&[Column]>,
        pagination: Option<&PaginationMeta>,
    ) {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&items).unwrap());
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&items).unwrap());
            }
            OutputFormat::Table | OutputFormat::Wide => {
                let use_wide = matches!(self.format, OutputFormat::Wide);
                let mut all_columns: Vec<&Column> = columns.iter().collect();
                if use_wide {
                    if let Some(extra) = wide_columns {
                        all_columns.extend(extra.iter());
                    }
                }

                if items.is_empty() {
                    println!("No resources found.");
                    return;
                }

                // Build table data
                let headers: Vec<String> =
                    all_columns.iter().map(|c| c.header.to_string()).collect();

                let mut rows: Vec<Vec<String>> = Vec::new();
                for item in items {
                    let row: Vec<String> = all_columns
                        .iter()
                        .map(|col| {
                            let val = Self::extract(item, col.json_path);
                            if !use_wide {
                                if let Some(max) = col.max_width {
                                    return Self::truncate(&val, max);
                                }
                            }
                            val
                        })
                        .collect();
                    rows.push(row);
                }

                let mut all_rows = vec![headers];
                all_rows.extend(rows);

                let table = Table::from_iter(all_rows).with(Style::blank()).to_string();

                println!("{}", table);

                if let Some(meta) = pagination {
                    if let (Some(page), Some(count)) = (meta.page, meta.count) {
                        let limit = meta.limit.unwrap_or(100);
                        let total_pages = if count == 0 {
                            1
                        } else {
                            count.div_ceil(limit)
                        };
                        eprintln!(
                            "{}",
                            format!("--- Page {}/{} ({} total) ---", page, total_pages, count)
                                .dimmed()
                        );
                    }
                }
            }
        }
    }

    pub fn print_item(&self, item: &Value, columns: &[Column]) {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(item).unwrap());
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(item).unwrap());
            }
            OutputFormat::Table | OutputFormat::Wide => {
                for col in columns {
                    let val = Self::extract(item, col.json_path);
                    println!("{:<20} {}", format!("{}:", col.header), val);
                }
            }
        }
    }

    pub fn print_value(&self, value: &Value) {
        // If the value is a plain string (e.g. HOCON from /configs), print it raw
        if let Value::String(s) = value {
            println!("{}", s);
            return;
        }
        match self.format {
            OutputFormat::Json | OutputFormat::Table | OutputFormat::Wide => {
                println!("{}", serde_json::to_string_pretty(value).unwrap());
            }
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(value).unwrap());
            }
        }
    }

    pub fn print_success(&self, msg: &str) {
        match self.format {
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({"status": "ok", "message": msg})
                    )
                    .unwrap()
                );
            }
            OutputFormat::Yaml => {
                println!(
                    "{}",
                    serde_yaml::to_string(&serde_json::json!({"status": "ok", "message": msg}))
                        .unwrap()
                );
            }
            OutputFormat::Table | OutputFormat::Wide => {
                println!("{}", msg.green());
            }
        }
    }
}
