use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "codexline",
    version,
    about = "Statusline tool for Codex workflows"
)]
pub struct Cli {
    #[arg(long, help = "Output without ANSI color codes")]
    pub plain: bool,

    #[arg(long, help = "Output as JSON")]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    PrintConfig,
    CheckConfig,
    Doctor,
    Inspect {
        #[arg(long, value_enum, default_value_t = InspectSource::All)]
        source: InspectSource,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InspectSource {
    Rollout,
    Git,
    All,
}
