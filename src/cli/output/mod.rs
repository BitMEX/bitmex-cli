/// Unified output rendering layer.
///
/// Every command returns a `CommandResult` which is rendered through this
/// module based on the chosen output format (table or json). Diagnostics
/// and verbose output always go to stderr, keeping stdout clean for
/// machine consumption in JSON mode.
pub(crate) mod json;
pub(crate) mod table;

use crate::errors::BitmexError;

/// Output format selection for the CLI.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

/// A uniform result from any command, ready for rendering.
#[derive(Debug)]
pub(crate) struct CommandOutput {
    /// JSON value representing the successful payload.
    pub(crate) data: serde_json::Value,
    /// Column headers for table rendering (in display order).
    pub(crate) headers: Vec<String>,
    /// Row data for table rendering. Each row is a vec of cell strings.
    pub(crate) rows: Vec<Vec<String>>,
}

impl CommandOutput {
    /// Return a fluent builder for constructing `CommandOutput`.
    ///
    /// ```ignore
    /// // Simple — auto-detect headers from JSON structure:
    /// CommandOutput::builder().data(val).build()
    ///
    /// // Explicit columns (replaces from_json_cols):
    /// CommandOutput::builder().data(val).columns(&["price", "size"]).build()
    ///
    /// // Fixed message:
    /// CommandOutput::builder().message("Done").build()
    /// ```
    pub(crate) fn builder() -> CommandOutputBuilder {
        CommandOutputBuilder::default()
    }

    /// Create output from a JSON value with explicit table structure.
    pub(crate) fn new(
        data: serde_json::Value,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> Self {
        Self {
            data,
            headers,
            rows,
        }
    }

    /// Create a simple key-value output (rendered as two-column table).
    pub(crate) fn key_value(pairs: Vec<(String, String)>, json_data: serde_json::Value) -> Self {
        let headers = vec!["Field".to_string(), "Value".to_string()];
        let rows: Vec<Vec<String>> = pairs.into_iter().map(|(k, v)| vec![k, v]).collect();
        Self {
            data: json_data,
            headers,
            rows,
        }
    }

    /// Create output from a raw JSON value, auto-populating table rows from the data.
    ///
    /// - If the value is an array of objects, headers come from the first object's keys.
    /// - If the value is a single object, renders as key-value pairs.
    /// - Otherwise renders as a single "Result" column.
    pub(crate) fn from_json(data: serde_json::Value) -> Self {
        use serde_json::Value;
        match &data {
            Value::Array(arr) if !arr.is_empty() => {
                if let Some(Value::Object(first)) = arr.first() {
                    let headers: Vec<String> = first.keys().cloned().collect();
                    let rows: Vec<Vec<String>> = arr
                        .iter()
                        .map(|row| {
                            headers
                                .iter()
                                .map(|h| {
                                    row.get(h)
                                        .map(|v| match v {
                                            Value::String(s) => s.clone(),
                                            other => other.to_string(),
                                        })
                                        .unwrap_or_else(|| "-".to_string())
                                })
                                .collect()
                        })
                        .collect();
                    return Self { data, headers, rows };
                }
                Self {
                    headers: vec!["Value".to_string()],
                    rows: arr.iter().map(|v| vec![v.to_string()]).collect(),
                    data,
                }
            }
            Value::Object(obj) => {
                let headers = vec!["Field".to_string(), "Value".to_string()];
                let rows = obj
                    .iter()
                    .map(|(k, v)| {
                        vec![
                            k.clone(),
                            match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            },
                        ]
                    })
                    .collect();
                Self { data, headers, rows }
            }
            other => Self {
                headers: vec!["Result".to_string()],
                rows: vec![vec![other.to_string()]],
                data,
            },
        }
    }

    /// Create output from a JSON value, showing only the specified columns in table view.
    /// JSON output (`-o json`) still contains the full data.
    pub(crate) fn from_json_cols(data: serde_json::Value, cols: &[&str]) -> Self {
        use serde_json::Value;
        let headers: Vec<String> = cols.iter().map(|c| c.to_string()).collect();
        let rows = match &data {
            Value::Array(arr) => arr
                .iter()
                .map(|row| {
                    headers
                        .iter()
                        .map(|h| {
                            row.get(h.as_str())
                                .map(|v| match v {
                                    Value::String(s) => s.clone(),
                                    Value::Null => "-".to_string(),
                                    other => other.to_string(),
                                })
                                .unwrap_or_else(|| "-".to_string())
                        })
                        .collect()
                })
                .collect(),
            Value::Object(_obj) => {
                vec![headers
                    .iter()
                    .map(|h| {
                        data.get(h.as_str())
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                Value::Null => "-".to_string(),
                                other => other.to_string(),
                            })
                            .unwrap_or_else(|| "-".to_string())
                    })
                    .collect()]
            }
            _ => vec![],
        };
        Self { data, headers, rows }
    }

    /// Create an empty output (used for commands with no meaningful return value).
    pub(crate) fn empty() -> Self {
        Self {
            data: serde_json::json!({}),
            headers: vec![],
            rows: vec![],
        }
    }

    /// Create output with a single message.
    pub(crate) fn message(msg: &str) -> Self {
        Self {
            data: serde_json::json!({ "message": msg }),
            headers: vec!["Message".to_string()],
            rows: vec![vec![msg.to_string()]],
        }
    }
}

/// Fluent builder for `CommandOutput`.
///
/// Obtain via [`CommandOutput::builder()`].
#[derive(Default)]
pub(crate) struct CommandOutputBuilder {
    data: Option<serde_json::Value>,
    columns: Option<Vec<String>>,
    message: Option<String>,
}

impl CommandOutputBuilder {
    /// Set the JSON payload. Required unless `.message()` is used.
    pub(crate) fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Restrict table view to specific columns (full JSON is always emitted).
    pub(crate) fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = Some(cols.iter().map(|c| c.to_string()).collect());
        self
    }

    /// Produce a single-message output (sets data to `{"message": msg}`).
    pub(crate) fn message(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Consume the builder and produce a `CommandOutput`.
    pub(crate) fn build(self) -> CommandOutput {
        if let Some(msg) = self.message {
            return CommandOutput::message(&msg);
        }
        match (self.data, self.columns) {
            (Some(data), Some(cols)) => {
                let col_refs: Vec<&str> = cols.iter().map(|s| s.as_str()).collect();
                CommandOutput::from_json_cols(data, &col_refs)
            }
            (Some(data), None) => CommandOutput::from_json(data),
            (None, _) => CommandOutput::empty(),
        }
    }
}

/// Render a successful command result to the appropriate output stream.
pub(crate) fn render(format: OutputFormat, output: &CommandOutput) {
    match format {
        OutputFormat::Table => table::render(output),
        OutputFormat::Json => json::render_success(&output.data),
    }
}

/// Render an error to the appropriate output stream.
pub fn render_error(format: OutputFormat, err: &BitmexError) {
    match format {
        OutputFormat::Table => {
            eprintln!("Error: {err}");
        }
        OutputFormat::Json => json::render_error(err),
    }
}

/// Write a verbose diagnostic message to stderr (never contaminates stdout).
pub(crate) fn verbose(msg: &str) {
    eprintln!("[verbose] {msg}");
}

/// Write a warning to stderr.
pub fn warn(msg: &str) {
    eprintln!("Warning: {msg}");
}
