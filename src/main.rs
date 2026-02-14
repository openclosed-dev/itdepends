use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use log::error;

use crate::api::fetch_latest_version;
use crate::artifact::{read_tree, write_as_csv};
use crate::logging::init_logger;

mod api;
mod artifact;
mod logging;

#[derive(Parser)]
#[command(version, about)]
struct Command {
    input_file: PathBuf,
    /// Stop fetching metadata from network
    #[arg(long)]
    offline: bool,
}

fn main() -> ExitCode {
    let command = Command::parse();

    init_logger();

    command.run()
}

impl Command {
    fn run(&self) -> ExitCode {
        let file = match File::open(&self.input_file) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to open the file {:?}: {}", self.input_file, err);
                return ExitCode::FAILURE;
            }
        };

        let root = match read_tree(file) {
            Ok(root) => root,
            Err(err) => {
                error!("Failed to parse JSON: {}", err);
                return ExitCode::FAILURE;
            }
        };

        let root_group_id = root.group_id.clone();

        let mut flattened = root.flatten();

        flattened.retain(|a| !a.belongs_to(&root_group_id));
        flattened.retain(|a| a.is_runtime());

        if !self.offline
            && let Err(err) = fetch_latest_version(&mut flattened)
        {
            error!("Failed to call remote API: {}", err);
            return ExitCode::FAILURE;
        }

        if let Err(err) = write_as_csv(io::stdout(), &flattened) {
            error!("Failed to output: {}", err);
            return ExitCode::FAILURE;
        }

        ExitCode::SUCCESS
    }
}
