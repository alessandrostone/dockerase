mod commands;
mod display;
mod docker;
mod resources;
mod system;

use clap::{Parser, Subcommand};
use display::print_error;
use std::process::ExitCode;

const BANNER: &str = r#"
 ___     ___      __  __  _    ___  ____    ____  _____   ___
|   \   /   \    /  ]|  l/ ]  /  _]|    \  /    T/ ___/  /  _]
|    \ Y     Y  /  / |  ' /  /  [_ |  D  )Y  o  (   \_  /  [_
|  D  Y|  O  | /  /  |    \ Y    _]|    / |     |\__  TY    _]
|     ||     |/   \_ |     Y|   [_ |    \ |  _  |/  \ ||   [_
|     |l     !\     ||  .  ||     T|  .  Y|  |  |\    ||     T
l_____j \___/  \____jl__j\_jl_____jl__j\_jl__j__j \___jl_____j
"#;

#[derive(Parser)]
#[command(
    name = "dockerase",
    about = "Docker cleaning utility CLI",
    version,
    author,
    before_help = BANNER
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Remove ALL Docker resources (containers, images, volumes, networks, build cache)
    #[arg(long)]
    nuclear: bool,

    /// Skip confirmation prompts
    #[arg(short, long)]
    force: bool,

    /// Show what would be removed without making changes
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Safely remove unused Docker resources (dangling images, stopped containers, unused volumes)
    Purge {
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,

        /// Show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Interactively select which resources to purge
    Select {
        /// Skip confirmation prompts (select all)
        #[arg(short, long)]
        force: bool,

        /// Show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage macOS system caches (Homebrew, npm, Xcode, etc.)
    System {
        #[command(subcommand)]
        action: Option<SystemAction>,

        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,

        /// Show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SystemAction {
    /// Purge all system caches
    Purge {
        /// Skip confirmation prompts
        #[arg(short, long)]
        force: bool,

        /// Show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    /// Interactively select which system caches to purge
    Select {
        /// Skip confirmation prompts (select all)
        #[arg(short, long)]
        force: bool,

        /// Show what would be removed without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = if cli.nuclear {
        commands::nuclear::run(cli.force, cli.dry_run)
    } else {
        match cli.command {
            Some(Commands::Purge { force, dry_run }) => {
                commands::purge::run(force || cli.force, dry_run || cli.dry_run)
            }
            Some(Commands::Select { force, dry_run }) => {
                commands::select::run(force || cli.force, dry_run || cli.dry_run)
            }
            Some(Commands::System {
                action,
                force,
                dry_run,
            }) => match action {
                Some(SystemAction::Purge {
                    force: purge_force,
                    dry_run: purge_dry_run,
                }) => commands::system::purge(
                    force || purge_force || cli.force,
                    dry_run || purge_dry_run || cli.dry_run,
                    false, // not interactive
                ),
                Some(SystemAction::Select {
                    force: select_force,
                    dry_run: select_dry_run,
                }) => commands::system::purge(
                    force || select_force || cli.force,
                    dry_run || select_dry_run || cli.dry_run,
                    true, // interactive
                ),
                None => commands::system::list(),
            },
            None => commands::list::run(),
        }
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            print_error(&e);
            ExitCode::FAILURE
        }
    }
}
