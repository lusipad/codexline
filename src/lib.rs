mod cli;
mod collect;
mod config;
mod context;
mod patch_diagnose;
mod profiles;
mod render;
mod segments;
mod themes;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, EnhancementKind, InspectSource};
use profiles::Enhancement;
use serde::Serialize;
use std::collections::HashSet;
use std::io::IsTerminal;

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.init {
        let result = config::init()?;
        let path = config::config_path();
        match result {
            config::InitResult::Created => println!("created config: {}", path.display()),
            config::InitResult::AlreadyExists => {
                println!("config already exists: {}", path.display())
            }
        }
        return Ok(());
    }

    let mut cfg = config::load()?;

    if cli.quick_config || !cli.enhance.is_empty() {
        if cli.quick_config {
            profiles::apply_quick_config(&mut cfg);
        }

        let mut seen = HashSet::new();
        let mut applied = Vec::new();
        for capability in &cli.enhance {
            if !seen.insert(*capability) {
                continue;
            }
            let enhancement = match capability {
                EnhancementKind::Git => Enhancement::Git,
                EnhancementKind::Observability => Enhancement::Observability,
            };
            profiles::apply_enhancement(&mut cfg, enhancement);
            applied.push(*capability);
        }

        config::save(&cfg)?;
        println!("saved config: {}", config::config_path().display());
        if cli.quick_config {
            println!("- quick profile applied");
        }
        for capability in applied {
            match capability {
                EnhancementKind::Git => {
                    println!("- enhancement applied: git (detailed git metrics)")
                }
                EnhancementKind::Observability => {
                    println!("- enhancement applied: observability (usage/limits/session/version)")
                }
            }
        }
        return Ok(());
    }

    if let Some(theme) = cli.theme.as_deref() {
        cfg = themes::apply_theme(&cfg, theme, &config::themes_dir())?;
    } else {
        cfg = themes::apply_theme(&cfg, &cfg.theme, &config::themes_dir()).unwrap_or(cfg);
    }

    if cli.print {
        println!("{}", toml::to_string_pretty(&cfg)?);
        return Ok(());
    }

    if cli.check {
        cfg.validate()?;
        println!("configuration valid");
        return Ok(());
    }

    if cli.config {
        let result = ui::run_configurator(&cfg)?;
        if result.is_some() {
            println!("configuration saved");
        } else {
            println!("configuration not changed");
        }
        return Ok(());
    }

    if cli.doctor {
        run_doctor(&cfg, cli.json)?;
        return Ok(());
    }

    if let Some(source) = cli.inspect {
        run_inspect(&cfg, source)?;
        return Ok(());
    }

    if cli.patch {
        run_patch_diagnose(&cfg, cli.json)?;
        return Ok(());
    }

    if cli.menu || should_open_menu(&cli) {
        let action = ui::run_main_menu()?;
        match action {
            ui::MainMenuAction::Render => {}
            ui::MainMenuAction::Configure => {
                let result = ui::run_configurator(&cfg)?;
                if result.is_some() {
                    println!("configuration saved");
                } else {
                    println!("configuration not changed");
                }
                return Ok(());
            }
            ui::MainMenuAction::Init => {
                let result = config::init()?;
                let path = config::config_path();
                match result {
                    config::InitResult::Created => println!("created config: {}", path.display()),
                    config::InitResult::AlreadyExists => {
                        println!("config already exists: {}", path.display())
                    }
                }
                return Ok(());
            }
            ui::MainMenuAction::Check => {
                cfg.validate()?;
                println!("configuration valid");
                return Ok(());
            }
            ui::MainMenuAction::Patch => {
                run_patch_diagnose(&cfg, false)?;
                return Ok(());
            }
            ui::MainMenuAction::Exit => return Ok(()),
        }
    }

    run_statusline(&cfg, cli.plain, cli.json)
}

fn should_open_menu(cli: &Cli) -> bool {
    !cli.has_explicit_action() && std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

#[derive(Serialize)]
struct DoctorReport {
    config_path: String,
    config_exists: bool,
    theme: String,
    style_mode: String,
    separator: String,
    codex_home: String,
    sessions_dir: String,
    sessions_exists: bool,
    latest_rollout: Option<String>,
    git: Option<context::GitStatus>,
    warnings: Vec<String>,
}

fn run_doctor(cfg: &config::Config, as_json: bool) -> Result<()> {
    let collection = collect::collect(cfg)?;

    let config_path = config::config_path();
    let config_exists = config_path.exists();
    let sessions_exists = collection.sessions_dir.exists();
    let latest_rollout = collection
        .latest_rollout
        .as_ref()
        .map(|path| path.display().to_string());

    let mut warnings = Vec::new();
    if !config_exists {
        warnings.push("config file missing, run codexline --init".to_string());
    }
    if !sessions_exists {
        warnings.push("sessions directory missing, run Codex once to initialize".to_string());
    }
    if latest_rollout.is_none() {
        warnings.push("no rollout data found in sessions directory".to_string());
    }
    if collection.context.git.is_none() {
        warnings.push("current directory is not a git repository".to_string());
    }

    let report = DoctorReport {
        config_path: config_path.display().to_string(),
        config_exists,
        theme: cfg.theme.clone(),
        style_mode: format!("{:?}", cfg.style.mode),
        separator: cfg.style.separator.clone(),
        codex_home: collection.codex_home.display().to_string(),
        sessions_dir: collection.sessions_dir.display().to_string(),
        sessions_exists,
        latest_rollout,
        git: collection.context.git,
        warnings,
    };

    if as_json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("config: {}", report.config_path);
    println!("config_exists: {}", report.config_exists);
    println!("theme: {}", report.theme);
    println!("style_mode: {}", report.style_mode);
    println!("separator: {}", report.separator);
    println!("codex_home: {}", report.codex_home);
    println!("sessions_dir: {}", report.sessions_dir);
    println!("sessions_exists: {}", report.sessions_exists);

    if let Some(path) = &report.latest_rollout {
        println!("latest_rollout: {}", path);
    } else {
        println!("latest_rollout: <none>");
    }

    if let Some(git) = &report.git {
        println!(
            "git: branch={} dirty={} staged={} unstaged={} untracked={} conflicted={}",
            git.branch, git.dirty, git.staged, git.unstaged, git.untracked, git.conflicted
        );
    } else {
        println!("git: <not-a-repo>");
    }

    if !report.warnings.is_empty() {
        println!("warnings:");
        for warning in report.warnings {
            println!("- {}", warning);
        }
    }

    Ok(())
}

fn run_inspect(cfg: &config::Config, source: InspectSource) -> Result<()> {
    let collection = collect::collect(cfg)?;

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

    let payload = InspectOutput {
        source: source_name.to_string(),
        codex_home: collection.codex_home.display().to_string(),
        sessions_dir: collection.sessions_dir.display().to_string(),
        latest_rollout: collection
            .latest_rollout
            .as_ref()
            .map(|path| path.display().to_string()),
        model,
        git,
        usage,
        limits,
        session,
    };

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn run_patch_diagnose(cfg: &config::Config, as_json: bool) -> Result<()> {
    let collection = collect::collect(cfg)?;
    let report = patch_diagnose::run_patch_diagnostics(cfg, &collection);
    if as_json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", patch_diagnose::render_text(&report));
    }
    Ok(())
}

fn run_statusline(cfg: &config::Config, plain: bool, as_json: bool) -> Result<()> {
    let collection = collect::collect(cfg)?;
    let segment_list = segments::build_segments(cfg, &collection.context);

    if as_json {
        #[derive(Serialize)]
        struct JsonOut {
            line: String,
            segments: Vec<segments::SegmentPiece>,
            context: context::StatusContext,
        }

        let line = render::render_line(cfg, &segment_list, true);
        let payload = JsonOut {
            line,
            segments: segment_list,
            context: collection.context,
        };
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let line = render::render_line(cfg, &segment_list, plain);
    println!("{}", line);
    Ok(())
}
