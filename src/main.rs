//! occonfig: named model profiles for your opencode configuration.

use clap::{Parser, Subcommand};

use occonfig::commands;

#[derive(Parser)]
#[command(
    name = "occonfig",
    version = occonfig::version::version(),
    about = "Named model profiles for your opencode configuration",
    long_about = "Save the set of model assignments your agents are using under a \
                  name, and switch between named profiles without editing JSON by \
                  hand.\n\n\
                  A profile carries model assignments only. It never rewrites \
                  provider, mcp, plugin, tools, or permission."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Snapshot the current model assignments under a name.
    Save {
        /// Name for the profile.
        name: String,
        /// Overwrite an existing profile of the same name.
        #[arg(long)]
        force: bool,
    },

    /// Apply a saved profile to your opencode config.
    Use {
        /// Name of the profile to apply.
        name: String,
        /// Create agents named in the profile that the config does not define.
        #[arg(long)]
        add_agents: bool,
        /// Show what would change without writing anything.
        #[arg(long)]
        dry_run: bool,
    },

    /// List saved profiles.
    List,

    /// Show the active top-level model and which agents differ from it.
    Current,

    /// Point every model assignment at a single provider/model reference.
    SetModel {
        /// Reference in the form provider/model.
        reference: String,
        /// Rewrite agents only, leaving the top-level model alone.
        #[arg(long)]
        agents_only: bool,
        /// Also pin agents that carry no model key and inherit the top-level one.
        #[arg(long)]
        add_agents: bool,
        /// Show what would change without writing anything.
        #[arg(long)]
        dry_run: bool,
    },

    /// Validate the config and report profiles with stale references.
    Doctor,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Save { name, force } => commands::save::run(&name, force),
        Command::Use {
            name,
            add_agents,
            dry_run,
        } => commands::use_profile::run(&name, add_agents, dry_run),
        Command::List => commands::list::run(),
        Command::Current => commands::current::run(),
        Command::SetModel {
            reference,
            agents_only,
            add_agents,
            dry_run,
        } => commands::set_model::run(&reference, agents_only, add_agents, dry_run),
        Command::Doctor => commands::doctor::run(),
    };

    if let Err(err) = result {
        // `{:#}` prints the whole anyhow context chain on one line, which is
        // the useful shape for a CLI: the user sees the cause, not a backtrace.
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
