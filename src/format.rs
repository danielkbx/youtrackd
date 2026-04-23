use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Text,
    Raw,
    Md,
}

#[derive(Debug, Clone)]
pub struct OutputOptions {
    pub format: Format,
    pub no_meta: bool,
}

const META_FIELDS: &[&str] = &[
    "id",
    "idReadable",
    "created",
    "updated",
    "resolved",
    "reporter",
    "updatedBy",
    "author",
    "project",
];

impl OutputOptions {
    pub fn from_flags(flags: &HashMap<String, String>) -> Self {
        let format = match flags.get("format").map(|s| s.as_str()) {
            Some("raw") => Format::Raw,
            Some("md") => Format::Md,
            _ => Format::Text,
        };
        let no_meta = flags.get("no-meta").map(|v| v == "true").unwrap_or(false);
        Self { format, no_meta }
    }
}

pub fn print_value(value: &Value, opts: &OutputOptions) {
    match opts.format {
        Format::Raw => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_default()
            );
        }
        Format::Md => {
            print_md(value);
        }
        Format::Text => {
            print_text(value, opts);
        }
    }
}

pub fn print_items<T: serde::Serialize>(items: &[T], opts: &OutputOptions) {
    let value = serde_json::to_value(items).unwrap_or(Value::Array(vec![]));
    match opts.format {
        Format::Raw => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).unwrap_or_default()
            );
        }
        Format::Md => {
            if let Value::Array(arr) = &value {
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        println!("\n---\n");
                    }
                    print_md(item);
                }
            }
        }
        Format::Text => {
            if let Value::Array(arr) = &value {
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    print_text(item, opts);
                }
            }
        }
    }
}

pub fn print_single<T: serde::Serialize>(item: &T, opts: &OutputOptions) {
    let value = serde_json::to_value(item).unwrap_or(Value::Null);
    print_value(&value, opts);
}

fn print_md(value: &Value) {
    let obj = match value {
        Value::Object(map) => map,
        _ => {
            println!("{value}");
            return;
        }
    };

    // Title: summary (articles) or summary (tickets)
    let summary = obj.get("summary").and_then(|v| v.as_str()).unwrap_or("");
    if !summary.is_empty() {
        println!("# {summary}");
        println!();
    }

    // Body: content (articles) or description (tickets)
    let body = obj
        .get("content")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("description").and_then(|v| v.as_str()));
    if let Some(body) = body {
        println!("{body}");
    }

    // Comments section
    let comments = obj.get("comments").and_then(|v| v.as_array());
    if let Some(comments) = comments {
        if !comments.is_empty() {
            println!();
            println!("---");
            println!();
            println!("## Comments");
            for comment in comments {
                let author = comment
                    .get("author")
                    .and_then(|a| a.get("fullName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let text = comment.get("text").and_then(|v| v.as_str()).unwrap_or("");
                println!();
                println!("**{author}**:");
                println!("{text}");
            }
        }
    }
}

fn print_text(value: &Value, opts: &OutputOptions) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                if opts.no_meta && META_FIELDS.contains(&key.as_str()) {
                    continue;
                }
                if val.is_null() {
                    continue;
                }
                let formatted = format_value(key, val);
                if !formatted.is_empty() {
                    println!("{key}: {formatted}");
                }
            }
        }
        Value::String(s) => println!("{s}"),
        _ => println!("{value}"),
    }
}

fn format_value(key: &str, val: &Value) -> String {
    match val {
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "yes" } else { "no" }.to_string(),
        Value::Number(n) => {
            // Check if this looks like a timestamp (ms since epoch, > 2000-01-01)
            if let Some(ms) = n.as_u64() {
                if ms > 946_684_800_000 && is_time_field(key) {
                    return format_timestamp(ms);
                }
            }
            n.to_string()
        }
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            // Array of tags or simple objects
            let parts: Vec<String> = arr
                .iter()
                .filter_map(|v| {
                    if let Value::Object(m) = v {
                        m.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    } else if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .collect();
            parts.join(", ")
        }
        Value::Object(map) => {
            // User: "Full Name (login)"
            if let (Some(name), Some(login)) = (
                map.get("fullName").and_then(|v| v.as_str()),
                map.get("login").and_then(|v| v.as_str()),
            ) {
                return format!("{name} ({login})");
            }
            // Project: shortName or name
            if let Some(short) = map.get("shortName").and_then(|v| v.as_str()) {
                return short.to_string();
            }
            if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
                return name.to_string();
            }
            serde_json::to_string(val).unwrap_or_default()
        }
    }
}

fn is_time_field(key: &str) -> bool {
    matches!(
        key,
        "created" | "updated" | "resolved" | "date" | "start" | "finish" | "timestamp"
    )
}

fn format_timestamp(ms: u64) -> String {
    let secs = (ms / 1000) as i64;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    // Calculate date from days since epoch (1970-01-01)
    let (year, month, day) = days_to_date(days_since_epoch);
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

fn days_to_date(mut days: i64) -> (i64, i64, i64) {
    // Civil days to date algorithm
    days += 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_text_from_flags() {
        let flags = HashMap::new();
        let opts = OutputOptions::from_flags(&flags);
        assert_eq!(opts.format, Format::Text);
        assert!(!opts.no_meta);
    }

    #[test]
    fn format_raw_from_flags() {
        let mut flags = HashMap::new();
        flags.insert("format".into(), "raw".into());
        let opts = OutputOptions::from_flags(&flags);
        assert_eq!(opts.format, Format::Raw);
    }

    #[test]
    fn no_meta_from_flags() {
        let mut flags = HashMap::new();
        flags.insert("no-meta".into(), "true".into());
        let opts = OutputOptions::from_flags(&flags);
        assert!(opts.no_meta);
    }

    #[test]
    fn timestamp_formatting() {
        // 2024-01-15 12:00 UTC = 1705320000000 ms
        let s = format_timestamp(1_705_320_000_000);
        assert!(s.contains("2024"));
        assert!(s.ends_with("UTC"));
    }

    #[test]
    fn format_user_object() {
        let val = serde_json::json!({"fullName": "Alice Smith", "login": "asmith"});
        assert_eq!(format_value("reporter", &val), "Alice Smith (asmith)");
    }

    #[test]
    fn format_md_from_flags() {
        let mut flags = HashMap::new();
        flags.insert("format".into(), "md".into());
        let opts = OutputOptions::from_flags(&flags);
        assert_eq!(opts.format, Format::Md);
    }

    #[test]
    fn format_bool() {
        assert_eq!(format_value("archived", &Value::Bool(true)), "yes");
        assert_eq!(format_value("archived", &Value::Bool(false)), "no");
    }
}
