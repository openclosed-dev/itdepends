use std::fs::File;
use std::io::{Write, stderr};
use std::process::ExitCode;
use std::{env, io};

use log::error;

use crate::api::fetch_latest_version;
use crate::artifact::{read_tree, write_as_csv};
use crate::logging::init_logger;

mod api;
mod artifact;
mod logging;

fn print_usage() {
    let _ = writeln!(stderr(), "Usage: itdepends <json file>");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        print_usage();
        return ExitCode::from(2);
    }

    init_logger();

    analyze_deps(&args[1])
}

fn analyze_deps(path: &str) -> ExitCode {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to open the file {}: {}", path, err);
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

    let mut flattened = root.flatten();

    if let Err(err) = fetch_latest_version(&mut flattened) {
        error!("Failed to call remote API: {}", err);
        return ExitCode::FAILURE;
    }

    if let Err(err) = write_as_csv(io::stdout(), &flattened) {
        error!("Failed to output: {}", err);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
