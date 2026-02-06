mod cli;
mod collect;
mod config;
mod context;
mod render;
mod segments;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command, InspectSource};
use serde::Serialize;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Init) => cmd_init(),
        Some(Command::PrintConfig) => cmd_print_config(),
        Some(Command::CheckConfig) => cmd_check_config(),
        Some(Command::Doctor) => cmd_doctor(),
        Some(Command::Inspect { source }) => cmd_inspect(source),
        None => cmd_statusline(cli.plain, cli.json),
    }
}

fn cmd_init() -> Result<()> {
    let result = config::init()?;
    let path = config::config_path();
    match result {
        config::InitResult::Created => {
            println!("created config: {}", path.display());
        }
        config::InitResult::AlreadyExists => {
            println!("config already exists: {}", path.display());
        }
    }
    Ok(())
}

fn cmd_print_config() -> Result<()> {
    let cfg = config::load()?;
    println!("{}", toml::to_string_pretty(&cfg)?);
    Ok(())
}

fn cmd_check_config() -> Result<()> {
    let cfg = config::load()?;
    cfg.validate()?;
    println!("config is valid");
    Ok(())
}

fn cmd_doctor() -> Result<()> {
    let cfg = config::load()?;
    let collection = collect::collect(&cfg)?;

    let config_path = config::config_path();
    println!("config: {}", config_path.display());
    println!("config_exists: {}", config_path.exists());
    println!("codex_home: {}", collection.codex_home.display());
    println!("sessions_dir: {}", collection.sessions_dir.display());
    println!("sessions_dir_exists: {}", collection.sessions_dir.exists());

    match collection.latest_rollout {
        Some(path) => println!("latest_rollout: {}", path.display()),
        None => println!("latest_rollout: <none>"),
    }

    if let Some(git) = collection.context.git {
        println!("git: branch={} dirty={}", git.branch, git.dirty);
    } else {
        println!("git: <not-a-repo>");
    }

    Ok(())
}

fn cmd_inspect(source: InspectSource) -> Result<()> {
    let cfg = config::load()?;
    let collection = collect::collect(&cfg)?;

    #[derive(Serialize)]
    struct InspectOutput {
        source: String,
        codex_home: String,
        sessions_dir: String,
        latest_rollout: Option<String>,
        model: Option<String>,
        git: Option<context::GitStatus>,
        usage: Option<context::TokenUsageSnapshot>,
        limits: Option<context::RateLimitSnapshot>,
        session: Option<context::SessionMetaSnapshot>,
    }

    let (model, git, usage, limits, session, source_name) = match source {
        InspectSource::Rollout => (
            collection.context.model,
            None,
            collection.context.usage,
            collection.context.limits,
            collection.context.session,
            "rollout",
        ),
        InspectSource::Git => (None, collection.context.git, None, None, None, "git"),
        InspectSource::All => (
            collection.context.model,
            collection.context.git,
            collection.context.usage,
            collection.context.limits,
            collection.context.session,
            "all",
        ),
    };

    let out = InspectOutput {
        source: source_name.to_string(),
        codex_home: collection.codex_home.display().to_string(),
        sessions_dir: collection.sessions_dir.display().to_string(),
        latest_rollout: collection
            .latest_rollout
            .map(|path| path.display().to_string()),
        model,
        git,
        usage,
        limits,
        session,
    };

    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn cmd_statusline(plain: bool, json: bool) -> Result<()> {
    let cfg = config::load()?;
    let collection = collect::collect(&cfg)?;
    let segment_list = segments::build_segments(&cfg, &collection.context);

    if json {
        #[derive(Serialize)]
        struct JsonOut {
            line: String,
            segments: Vec<segments::SegmentPiece>,
            context: context::StatusContext,
        }

        let line = render::render_line(&segment_list, &cfg.separator, true);
        let payload = JsonOut {
            line,
            segments: segment_list,
            context: collection.context,
        };
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let line = render::render_line(&segment_list, &cfg.separator, plain);
    println!("{line}");
    Ok(())
}
