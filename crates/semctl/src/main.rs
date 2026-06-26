use clap::{Parser, Subcommand};
use sem_core::ipc::{send_set, send_status};
use sem_core::state::LightState;
use semctl::doctor;
use semctl::install;

#[derive(Parser)]
#[command(name = "semctl", about = "Control Semaphore from hooks and scripts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set light state (green, yellow, red)
    Set {
        state: String,
        #[arg(long, default_value = "default")]
        session: String,
        #[arg(long, default_value = "hook")]
        source: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    Green {
        #[arg(long, default_value = "default")]
        session: String,
        #[arg(long, default_value = "hook")]
        source: String,
        #[arg(long, default_value = "idle")]
        reason: String,
    },
    Yellow {
        #[arg(long, default_value = "default")]
        session: String,
        #[arg(long, default_value = "hook")]
        source: String,
        #[arg(long, default_value = "thinking")]
        reason: String,
    },
    Red {
        #[arg(long, default_value = "default")]
        session: String,
        #[arg(long, default_value = "hook")]
        source: String,
        #[arg(long, default_value = "writing")]
        reason: String,
    },
    /// Query aggregated state
    Status,
    /// Install hook adapters
    Install {
        #[arg(long)]
        all: bool,
        tool: Option<String>,
    },
    /// Remove Semaphore hook entries
    Uninstall {
        #[arg(long)]
        all: bool,
        tool: Option<String>,
    },
    /// Diagnose installation
    Doctor,
    /// Launch the Semaphore desktop app if not already running
    Launch,
    /// Cursor beforeSubmitPrompt: cache composer mode and set yellow
    CursorPrompt,
    /// Cursor stop: idle for Ask, awaiting-input for Agent
    CursorStop,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let code = match run(cli).await {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("semctl: {err}");
            0
        }
    };
    std::process::exit(code);
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Set {
            state,
            session,
            source,
            reason,
        } => {
            let parsed = LightState::parse(&state)
                .ok_or_else(|| format!("unknown state: {state}"))?;
            let _ = send_set(parsed, &session, &source, &reason).await?;
        }
        Commands::Green {
            session,
            source,
            reason,
        } => {
            let _ = send_set(LightState::Green, &session, &source, &reason).await?;
        }
        Commands::Yellow {
            session,
            source,
            reason,
        } => {
            let _ = send_set(LightState::Yellow, &session, &source, &reason).await?;
        }
        Commands::Red {
            session,
            source,
            reason,
        } => {
            let _ = send_set(LightState::Red, &session, &source, &reason).await?;
        }
        Commands::Status => {
            let response = send_status().await?;
            println!("{}", response.state);
        }
        Commands::Install { all, tool } => {
            install::run_install(all, tool.as_deref())?;
        }
        Commands::Uninstall { all, tool } => {
            install::run_uninstall(all, tool.as_deref())?;
        }
        Commands::Doctor => {
            doctor::run()?;
        }
        Commands::Launch => {
            semctl::launch::launch_app()?;
        }
        Commands::CursorPrompt => {
            let input = read_stdin()?;
            semctl::cursor_hooks::handle_cursor_prompt(&input).await?;
        }
        Commands::CursorStop => {
            let input = read_stdin()?;
            semctl::cursor_hooks::handle_cursor_stop(&input).await?;
        }
    }
    Ok(())
}

fn read_stdin() -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(input)
}
