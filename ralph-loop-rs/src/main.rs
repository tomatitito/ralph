use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use colored::Colorize;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use ralph_loop::agent::ClaudeAgent;
use ralph_loop::config::Config;
use ralph_loop::error::RalphError;
use ralph_loop::loop_controller::{LoopController, LoopResult};
use ralph_loop::tmux;

/// Ralph Loop: Run Claude Code in a loop until a promise is fulfilled
#[derive(Parser, Debug)]
#[command(name = "ralph-loop")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Prompt file path
    #[arg(short = 'f', long = "prompt-file")]
    prompt_file: Option<PathBuf>,

    /// Prompt text (alternative to prompt file)
    #[arg(short = 'p', long = "prompt")]
    prompt: Option<String>,

    /// Maximum number of iterations (omit for infinite loop)
    #[arg(short = 'm', long = "max-iterations")]
    max_iterations: Option<u32>,

    /// Promise text to detect completion (default: "TASK COMPLETE")
    #[arg(short = 'c', long = "completion-promise")]
    completion_promise: Option<String>,

    /// Output directory (default: .ralph-loop-output)
    #[arg(short = 'o', long = "output-dir")]
    output_dir: Option<PathBuf>,

    /// Token limit before restarting (default: 180000)
    #[arg(long = "context-limit")]
    context_limit: Option<usize>,

    /// Config file (TOML format)
    #[arg(long = "config")]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Start ralph-loop in a new tmux session
    #[arg(long = "tmux")]
    tmux: bool,

    /// Tmux session name (default: "ralph")
    #[arg(long = "tmux-session", default_value = "ralph")]
    tmux_session: String,

    /// Also start the viewer in a separate tmux window
    #[arg(long = "with-viewer")]
    with_viewer: bool,

    /// Attach to the tmux session after starting (only with --tmux)
    #[arg(long = "attach")]
    attach: bool,
}

fn setup_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("ralph_loop=debug,info")
    } else {
        EnvFilter::new("ralph_loop=info,warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn load_config(cli: &Cli) -> Result<Config, RalphError> {
    // Start with default config or load from file
    let mut config = if let Some(ref config_path) = cli.config {
        Config::from_file(config_path)?
    } else {
        Config::default()
    };

    // Load prompt from file if specified
    let prompt = if let Some(ref prompt_file) = cli.prompt_file {
        Some(std::fs::read_to_string(prompt_file).map_err(RalphError::PromptFileError)?)
    } else {
        cli.prompt.clone()
    };

    // Merge CLI arguments
    config.merge_cli_args(
        prompt,
        cli.max_iterations,
        cli.completion_promise.clone(),
        cli.output_dir.clone(),
        cli.context_limit,
    );

    // Validate that we have a prompt
    if config.prompt.is_empty() {
        return Err(RalphError::NoPromptProvided);
    }

    Ok(config)
}

async fn run(
    config: Config,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<LoopResult, RalphError> {
    // Create output directory
    std::fs::create_dir_all(&config.output_dir).map_err(RalphError::OutputDirError)?;

    info!(
        "Starting ralph-loop with completion promise: {}",
        config.completion_promise.cyan()
    );
    if let Some(max) = config.max_iterations {
        info!("Max iterations: {}", max);
    } else {
        info!("Running in infinite loop mode (until promise found or Ctrl+C)");
    }
    info!("Context limit: {} tokens", config.context_limit.max_tokens);

    // Get current working directory as project path
    let project_path = std::env::current_dir().map_err(RalphError::OutputDirError)?;

    // Create the agent and controller with transcript writer
    let agent = ClaudeAgent::new(Arc::new(config.clone()));
    let controller = LoopController::with_transcript_writer(config, agent, &project_path)?;

    info!(
        "Transcripts will be read from: {}/.claude/projects/{}",
        dirs::home_dir().unwrap_or_default().display(),
        project_path
            .to_string_lossy()
            .replace('/', "-")
            .strip_prefix('-')
            .unwrap_or(&project_path.to_string_lossy().replace('/', "-"))
    );

    // Run the loop with shutdown handling
    tokio::select! {
        result = controller.run() => {
            result
        }
        _ = shutdown_rx.recv() => {
            warn!("Shutdown signal received");
            Err(RalphError::ShutdownRequested)
        }
    }
}

/// Build the args to pass to ralph-loop when running inside tmux
fn build_ralph_loop_args(cli: &Cli) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(ref prompt_file) = cli.prompt_file {
        args.push("-f".to_string());
        args.push(prompt_file.display().to_string());
    }

    if let Some(ref prompt) = cli.prompt {
        args.push("-p".to_string());
        args.push(prompt.clone());
    }

    if let Some(max) = cli.max_iterations {
        args.push("-m".to_string());
        args.push(max.to_string());
    }

    if let Some(ref promise) = cli.completion_promise {
        args.push("-c".to_string());
        args.push(promise.clone());
    }

    if let Some(ref output_dir) = cli.output_dir {
        args.push("-o".to_string());
        args.push(output_dir.display().to_string());
    }

    if let Some(limit) = cli.context_limit {
        args.push("--context-limit".to_string());
        args.push(limit.to_string());
    }

    if let Some(ref config) = cli.config {
        args.push("--config".to_string());
        args.push(config.display().to_string());
    }

    if cli.verbose {
        args.push("-v".to_string());
    }

    args
}

/// Handle tmux mode: start ralph-loop in a new tmux session
fn handle_tmux_mode(cli: &Cli) {
    if !tmux::is_tmux_available() {
        eprintln!(
            "{} tmux is not available on this system",
            "ERROR:".red().bold()
        );
        std::process::exit(1);
    }

    // Build args for the ralph-loop process inside tmux
    let args = build_ralph_loop_args(cli);

    // Get output directory for the viewer
    let output_dir = cli
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(".ralph-loop-output"));

    // Find viewer if requested
    let viewer_path = if cli.with_viewer {
        tmux::find_viewer()
    } else {
        None
    };

    if cli.with_viewer && viewer_path.is_none() {
        eprintln!(
            "{} Could not find ralph-viewer. Make sure it's in the same directory as ralph-loop or in PATH.",
            "WARNING:".yellow().bold()
        );
    }

    // Start in tmux
    match tmux::start_in_tmux_session(
        &cli.tmux_session,
        &args,
        viewer_path.as_deref(),
        &output_dir,
    ) {
        Ok(()) => {
            println!(
                "{} Started ralph-loop in tmux session '{}'",
                "SUCCESS:".green().bold(),
                cli.tmux_session.cyan()
            );

            if cli.with_viewer && viewer_path.is_some() {
                println!("  Viewer is running in the 'viewer' window");
            }

            if cli.attach {
                println!("Attaching to session...");
                if let Err(e) = tmux::attach_to_session(&cli.tmux_session) {
                    eprintln!("{} Failed to attach: {}", "ERROR:".red().bold(), e);
                    std::process::exit(1);
                }
            } else {
                println!(
                    "  Attach with: {}",
                    format!("tmux attach -t {}", cli.tmux_session).cyan()
                );
            }
        }
        Err(e) => {
            eprintln!("{} {}", "ERROR:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle tmux mode
    if cli.tmux {
        handle_tmux_mode(&cli);
        return;
    }

    setup_logging(cli.verbose);

    // Setup shutdown signal handling
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    // Spawn signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        info!("Received Ctrl+C, shutting down...");
        let _ = shutdown_tx_clone.send(());
    });

    // Load configuration
    let config = match load_config(&cli) {
        Ok(c) => c,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };

    // Run the main loop
    match run(config, shutdown_rx).await {
        Ok(LoopResult::PromiseFulfilled {
            iterations,
            promise,
        }) => {
            println!(
                "\n{} Promise '{}' fulfilled after {} iteration(s)",
                "SUCCESS:".green().bold(),
                promise.cyan(),
                iterations
            );
            std::process::exit(0);
        }
        Ok(LoopResult::Shutdown { iterations }) => {
            println!(
                "\n{} Shutdown after {} iteration(s)",
                "INTERRUPTED:".yellow().bold(),
                iterations
            );
            std::process::exit(130); // Standard exit code for Ctrl+C
        }
        Err(RalphError::MaxIterationsExceeded(max)) => {
            println!(
                "\n{} Max iterations ({}) exceeded without finding promise",
                "FAILED:".red().bold(),
                max
            );
            std::process::exit(1);
        }
        Err(RalphError::ShutdownRequested) => {
            println!("\n{} Shutdown requested", "INTERRUPTED:".yellow().bold());
            std::process::exit(130);
        }
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    }
}
