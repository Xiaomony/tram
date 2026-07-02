use std::{cell::RefCell, rc::Rc};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use ratatui::{DefaultTerminal, Frame};

use crate::{
    core::{btrfs_manager::BtrfsManager, error::CResult},
    tui::app_tui::AppTUI,
};

#[derive(Parser, Debug)]
#[command(
    name = "tram_btrfs",
    author,
    version,
    about = "A TUI Btrfs snapshot manager"
)]
pub struct Cli {
    /// Run scheduled snapshot checks without launching the TUI.
    /// Does not create a boot snapshot.
    #[arg(long)]
    no_tui: bool,

    /// Run scheduled snapshot checks without launching the TUI.
    /// Create a boot snapshot.
    #[arg(long)]
    boot: bool,

    /// Specify a Btrfs device under "/dev/"
    #[arg(long, value_name = "DEVICE")]
    device: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate and print shell completion
    Completion {
        /// The shell you want to generate completion
        shell: Shell,
    },
}

impl Cli {
    pub fn process_args() -> CResult<()> {
        let cli = Cli::parse();
        // handle shell completion
        if let Some(Commands::Completion { shell }) = cli.command {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();

            clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
            return Ok(());
        }

        let mut btrfs_manager = if let Some(device) = cli.device {
            BtrfsManager::new(device)?
        } else {
            BtrfsManager::new_default_partion()?
        };

        btrfs_manager.check_schedule(cli.boot)?;

        if cli.no_tui || cli.boot {
            Ok(())
        } else {
            let result = run(ratatui::init(), btrfs_manager);
            ratatui::restore();
            result
        }
    }
}

fn run(mut terminal: DefaultTerminal, btrfs_manager: BtrfsManager) -> CResult<()> {
    let mgr = Rc::new(RefCell::new(btrfs_manager));
    let mut tui = AppTUI::new(mgr.clone());

    loop {
        terminal.draw(|frame: &mut Frame| tui.render(frame))?;
        if tui.read_events()? {
            break Ok(());
        }
    }
}
