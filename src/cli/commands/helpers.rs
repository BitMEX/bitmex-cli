/// Shared helper functions used across multiple command modules.
use crate::errors::{BitmexError, Result};

/// Build a URL query string from (key, Option<value>) pairs, skipping None values.
///
/// Values are NOT percent-encoded here; BitMEX query params are simple strings.
/// Callers should avoid characters that require encoding, or encode them manually.
pub(crate) fn build_query(params: &[(&str, Option<String>)]) -> String {
    params
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|val| format!("{k}={val}")))
        .collect::<Vec<_>>()
        .join("&")
}

/// Prompt user to confirm a destructive operation; abort on decline.
pub(crate) fn confirm_destructive(prompt: &str) -> Result<()> {
    let confirmed = inquire::Confirm::new(prompt)
        .with_default(false)
        .prompt()
        .map_err(|e| BitmexError::Config { message: format!("Prompt error: {e}") })?;

    if !confirmed {
        return Err(BitmexError::Validation {
            message: "Operation cancelled by user".into(),
        });
    }
    Ok(())
}
