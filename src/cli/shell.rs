/// Interactive REPL shell using reedline.
///
/// Features over the rustyline predecessor:
/// - Tab-completion of commands and flags (from the clap command tree)
/// - Fish-style inline history hints
/// - Syntax match highlighting
/// - Multi-line editing and configurable keybindings
use std::borrow::Cow;
use std::path::PathBuf;

use clap::Parser;
use reedline::{
    DefaultCompleter, DefaultHinter, FileBackedHistory, Highlighter, History, HistoryItem,
    HistoryItemId, HistorySessionId, Prompt, PromptEditMode, PromptHistorySearch, Reedline,
    SearchQuery, Signal, StyledText,
};

use crate::errors::{BitmexError, Result};
use crate::{dispatch, AppContext};

const HISTORY_CAPACITY: usize = 1_000;

/// Run the interactive shell session.
pub(crate) async fn run(ctx: &AppContext) -> Result<()> {
    let history_path = history_file()?;
    secure_history_file(&history_path)?;
    let history = FileBackedHistory::with_file(HISTORY_CAPACITY, history_path)
        .map_err(|e| BitmexError::Config {
            message: format!("Failed to open shell history: {e}"),
        })?;
    let history = RedactingHistory::new(history);

    let mut rl = Reedline::create()
        .with_history(Box::new(history))
        .with_completer(Box::new(build_completer()))
        .with_hinter(Box::new(DefaultHinter::default()))
        .with_highlighter(Box::new(BitmexHighlighter));

    let prompt = BitmexPrompt;

    println!("BitMEX CLI interactive shell. Type 'help' or 'exit'.");
    println!();

    loop {
        match rl.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match line {
                    "exit" | "quit" => break,
                    "help" => {
                        print_shell_help();
                        continue;
                    }
                    _ => {}
                }

                // Parse as CLI args, prefixing "bitmex" so clap sees a full invocation.
                let args: Vec<String> = std::iter::once("bitmex".to_string())
                    .chain(shell_words(line))
                    .collect();

                match crate::Cli::try_parse_from(&args) {
                    Ok(cli) => {
                        let fmt = cli.output.unwrap_or(ctx.format);
                        let api_url = match cli.api_url {
                            Some(ref url) => match crate::exchange::client::resolve_url_override(
                                Some(url.as_str()),
                                None,
                            ) {
                                Ok(u) => u,
                                Err(e) => {
                                    crate::cli::output::render_error(fmt, &e);
                                    continue;
                                }
                            },
                            None => ctx.api_url.clone(),
                        };
                        let shell_ctx = AppContext {
                            format: fmt,
                            verbose: cli.verbose || ctx.verbose,
                            testnet: cli.testnet || ctx.testnet,
                            api_url,
                            ws_url: ctx.ws_url.clone(),
                            api_key: cli.api_key.or_else(|| ctx.api_key.clone()),
                            api_secret: cli.api_secret.or_else(|| ctx.api_secret.clone()),
                            force: cli.yes || ctx.force,
                            secret_from_flag: false,
                            mcp_mode: false,
                            profile: cli.profile.or_else(|| ctx.profile.clone()),
                            no_keychain: cli.no_keychain || ctx.no_keychain,
                            timing: cli.timing || ctx.timing,
                        };
                        if let Some(command) = cli.command {
                            if let Err(e) = Box::pin(dispatch(&shell_ctx, command)).await {
                                crate::cli::output::render_error(shell_ctx.format, &e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                }
            }
            Ok(Signal::CtrlD | Signal::CtrlC) => break,
            Err(e) => {
                eprintln!("Shell error: {e}");
                break;
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Prompt
// ---------------------------------------------------------------------------

struct BitmexPrompt;

impl Prompt for BitmexPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        "bitmex> ".into()
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        "".into()
    }

    fn render_prompt_indicator(&self, _mode: PromptEditMode) -> Cow<'_, str> {
        "".into()
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        "... ".into()
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        "(history-search) ".into()
    }
}

// ---------------------------------------------------------------------------
// Tab-completion: collect all command/subcommand names from the clap tree
// ---------------------------------------------------------------------------

fn build_completer() -> DefaultCompleter {
    use clap::CommandFactory;
    let mut words: Vec<String> = Vec::new();
    collect_command_words(&crate::Cli::command(), &mut words);
    // Common flags and output values
    words.extend_from_slice(&[
        "--symbol".into(),
        "--count".into(),
        "--depth".into(),
        "--reverse".into(),
        "--testnet".into(),
        "-o".into(),
        "json".into(),
        "table".into(),
    ]);
    DefaultCompleter::new_with_wordlen(words, 1)
}

fn collect_command_words(cmd: &clap::Command, words: &mut Vec<String>) {
    let name = cmd.get_name().to_string();
    if !name.is_empty() && name != "bitmex" {
        words.push(name);
    }
    for sub in cmd.get_subcommands() {
        collect_command_words(sub, words);
    }
}

// ---------------------------------------------------------------------------
// Syntax highlighting: bold-green for known commands, dim-cyan for flags
// ---------------------------------------------------------------------------

struct BitmexHighlighter;

impl Highlighter for BitmexHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        use nu_ansi_term::Style;

        let mut styled = StyledText::new();
        if line.is_empty() {
            return styled;
        }

        // Tokenise on whitespace, preserving spaces, and colour each token.
        let mut token_start = 0usize;
        let mut first_token = true;
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i <= len {
            let at_boundary = i == len || bytes[i] == b' ';
            if at_boundary && i > token_start {
                let token = &line[token_start..i];
                let style = token_style(token, first_token);
                styled.push((style, token.to_string()));
                first_token = false;
                token_start = i;
            }
            if i < len && bytes[i] == b' ' {
                // Emit the space(s) unstyled
                while i < len && bytes[i] == b' ' {
                    styled.push((Style::default(), " ".to_string()));
                    i += 1;
                }
                token_start = i;
            } else {
                i += 1;
            }
        }

        styled
    }
}

fn token_style(token: &str, is_first: bool) -> nu_ansi_term::Style {
    use nu_ansi_term::{Color, Style};
    if token.starts_with("--") || (token.starts_with('-') && token.len() == 2) {
        Style::new().fg(Color::Cyan).dimmed()
    } else if is_first || KNOWN_COMMANDS.contains(&token) {
        Style::new().fg(Color::Green).bold()
    } else {
        Style::default()
    }
}

const KNOWN_COMMANDS: &[&str] = &[
    "market", "order", "position", "execution", "wallet", "staking",
    "account", "subaccount", "apikey", "chat", "guild", "bots",
    "notifications", "address", "referral", "porl", "announce",
    "ws", "auth", "setup", "shell", "mcp",
    "instrument", "quote", "trades", "orderbook", "funding", "stats",
    "liquidation", "settlement", "insurance", "leaderboard",
    "buy", "sell", "cancel", "cancel-all", "list", "update",
    "balance", "history", "withdraw", "transfer",
    "me", "margin", "preferences",
    "set", "show", "reset",
    "read", "send", "channels",
];

// ---------------------------------------------------------------------------
// Shell help and word splitting
// ---------------------------------------------------------------------------

fn history_file() -> Result<PathBuf> {
    let dir = crate::config::config_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("history"))
}

/// Ensure the shell history file exists with owner-only (0600) permissions.
///
/// `reedline`'s `FileBackedHistory` opens the file via a plain `OpenOptions`
/// with no mode set, so a freshly created file would inherit the process
/// umask (commonly 0644). We create it ourselves first with `mode(0o600)`:
/// on Unix that mode is applied atomically by the `open(2)` syscall at the
/// moment of creation, so the file is never briefly world/group-readable.
/// `reedline` then just opens the already-existing, correctly-permissioned
/// file. A pre-existing file (e.g. from before this fix) is chmod'd, which
/// only tightens permissions it already had — no new exposure window.
#[cfg(unix)]
fn secure_history_file(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    if path.exists() {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    } else {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
    }
    Ok(())
}

/// No-op on non-Unix platforms: we don't set an explicit ACL here.
///
/// On Windows this relies on `%APPDATA%\bitmex` inheriting the user
/// profile's default NTFS permissions (owner + SYSTEM + Administrators,
/// not `Everyone`). That's an environmental default, not a guarantee this
/// function enforces — it can differ on domain-joined or shared machines.
#[cfg(not(unix))]
fn secure_history_file(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

/// Wraps `FileBackedHistory`, keeping lines that carry exchange credentials
/// out of the persisted history file entirely.
///
/// `FileBackedHistory::save` only ever looks at `command_line` (session id,
/// exit status, etc. are discarded), so filtering on that field is enough.
/// Excluded entries are handed back to reedline unchanged instead of going
/// through `self.inner` — they never reach `self.inner`'s in-memory buffer,
/// so they can't be written to disk on sync/drop, and they also don't show
/// up in this session's up-arrow recall.
struct RedactingHistory {
    inner: FileBackedHistory,
}

impl RedactingHistory {
    fn new(inner: FileBackedHistory) -> Self {
        Self { inner }
    }
}

/// A line is sensitive if it invokes the `auth` command group, or passes
/// `--api-key`/`--api-secret` directly — both are global clap flags, so
/// they can appear on any command line, not just `auth set`.
fn is_sensitive(command_line: &str) -> bool {
    let words = shell_words(command_line);
    words.first().map(String::as_str) == Some("auth")
        || words
            .iter()
            .any(|w| w == "--api-key" || w == "--api-secret")
}

impl History for RedactingHistory {
    fn save(&mut self, entry: HistoryItem) -> reedline::Result<HistoryItem> {
        if is_sensitive(&entry.command_line) {
            return Ok(entry);
        }
        self.inner.save(entry)
    }

    fn load(&self, id: HistoryItemId) -> reedline::Result<HistoryItem> {
        self.inner.load(id)
    }

    fn count(&self, query: SearchQuery) -> reedline::Result<i64> {
        self.inner.count(query)
    }

    fn search(&self, query: SearchQuery) -> reedline::Result<Vec<HistoryItem>> {
        self.inner.search(query)
    }

    fn update(
        &mut self,
        id: HistoryItemId,
        updater: &dyn Fn(HistoryItem) -> HistoryItem,
    ) -> reedline::Result<()> {
        self.inner.update(id, updater)
    }

    fn clear(&mut self) -> reedline::Result<()> {
        self.inner.clear()
    }

    fn delete(&mut self, id: HistoryItemId) -> reedline::Result<()> {
        self.inner.delete(id)
    }

    fn sync(&mut self) -> std::io::Result<()> {
        self.inner.sync()
    }

    fn session(&self) -> Option<HistorySessionId> {
        self.inner.session()
    }
}

fn print_shell_help() {
    println!("Available command groups:");
    println!("  market, order, position, execution, wallet, staking, account");
    println!("  subaccount, apikey, chat, guild, bots, notifications, address");
    println!("  referral, porl, announce, ws, auth, setup");
    println!("  exit / quit — Exit the shell");
    println!();
    println!("Tab-complete commands and flags. Up/Down for history. Ctrl-R to search.");
    println!("Use --help on any command for details.");
}

/// Split a shell line into words, handling simple single and double quoting.
fn shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape_next = false;

    for ch in line.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if !in_single => escape_next = true,
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            ' ' | '\t' if !in_single && !in_double => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_words_basic() {
        assert_eq!(shell_words("foo bar baz"), vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn shell_words_quoted() {
        assert_eq!(
            shell_words("ticker \"XBT USD\" --verbose"),
            vec!["ticker", "XBT USD", "--verbose"]
        );
    }

    #[test]
    fn shell_words_single_quoted() {
        assert_eq!(shell_words("ticker 'XBT USD'"), vec!["ticker", "XBT USD"]);
    }

    #[test]
    fn is_sensitive_detects_auth_group() {
        assert!(is_sensitive("auth set --api-key foo --api-secret bar"));
        assert!(is_sensitive("auth show"));
    }

    #[test]
    fn is_sensitive_detects_inline_secret_flags_on_any_command() {
        assert!(is_sensitive(
            "market instrument --symbol XBTUSD --api-secret bar"
        ));
        assert!(is_sensitive(
            "order buy XBTUSD 100 --price 50000 --api-key foo"
        ));
    }

    #[test]
    fn is_sensitive_leaves_ordinary_commands_alone() {
        assert!(!is_sensitive("market instrument --symbol XBTUSD"));
        assert!(!is_sensitive("account me"));
    }

    #[test]
    fn redacting_history_does_not_persist_sensitive_lines() {
        let dir =
            std::env::temp_dir().join(format!("bitmex-cli-test-history-{}", std::process::id()));
        let path = dir.join("history");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::remove_file(&path);

        let inner = FileBackedHistory::with_file(100, path.clone()).unwrap();
        let mut history = RedactingHistory::new(inner);

        history
            .save(HistoryItem::from_command_line(
                "auth set --api-key foo --api-secret bar",
            ))
            .unwrap();
        history
            .save(HistoryItem::from_command_line(
                "market instrument --symbol XBTUSD",
            ))
            .unwrap();
        history.sync().unwrap();
        drop(history);

        let contents = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(!contents.contains("api-secret"));
        assert!(contents.contains("market instrument"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
