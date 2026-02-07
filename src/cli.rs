use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "codexline", version, about = "Codex statusline toolkit")]
pub struct Cli {
    #[arg(long, help = "Enter interactive configuration TUI")]
    pub config: bool,

    #[arg(long, help = "Open interactive main menu")]
    pub menu: bool,

    #[arg(long, help = "Override theme for current execution")]
    pub theme: Option<String>,

    #[arg(long, help = "Print current config as TOML")]
    pub print: bool,

    #[arg(long, help = "Initialize config and themes")]
    pub init: bool,

    #[arg(long, help = "Check config validity")]
    pub check: bool,

    #[arg(long, help = "Run environment diagnostics")]
    pub doctor: bool,

    #[arg(
        long,
        help = "Run patch compatibility diagnostics (no file modification)"
    )]
    pub patch: bool,

    #[arg(
        long,
        value_enum,
        num_args = 0..=1,
        default_missing_value = "all",
        help = "Inspect data sources as JSON"
    )]
    pub inspect: Option<InspectSource>,

    #[arg(long, help = "Output without ANSI colors")]
    pub plain: bool,

    #[arg(long, help = "Output structured JSON")]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InspectSource {
    Rollout,
    Git,
    All,
}

impl Cli {
    pub fn has_explicit_action(&self) -> bool {
        self.config
            || self.menu
            || self.theme.is_some()
            || self.print
            || self.init
            || self.check
            || self.doctor
            || self.patch
            || self.inspect.is_some()
            || self.plain
            || self.json
    }
}
