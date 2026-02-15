use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use log::error;

use crate::api::fetch_latest_version;
use crate::artifact::{Artifact, ArtifactParser, write_as_csv};
use crate::gradle::GradleArtifactParser;
use crate::logging::init_logger;
use crate::maven::MavenArtifactParser;

mod api;
mod artifact;
mod gradle;
mod logging;
mod maven;

#[derive(Debug, Clone, ValueEnum)]
enum Builder {
    Maven,
    Gradle,
}

#[derive(Parser)]
#[command(version, about)]
struct Command {
    input_file: PathBuf,
    /// Stop fetching metadata from network
    #[arg(long)]
    offline: bool,
    /// Builder that generated the input file
    #[arg(short, long)]
    builder: Builder,
}

fn main() -> ExitCode {
    let command = Command::parse();

    init_logger();

    command.run()
}

impl Command {
    fn run(&self) -> ExitCode {
        self.process_file(&self.input_file)
    }

    fn process_file(&self, path: &PathBuf) -> ExitCode {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to open the file {:?}: {}", self.input_file, err);
                return ExitCode::FAILURE;
            }
        };
        self.process_reader(file)
    }

    fn process_reader<R: Read>(&self, reader: R) -> ExitCode {
        let mut parser = self.builder.new_parser(reader);
        let result = parser.parse();
        let root = match result {
            Ok(root) => root,
            Err(err) => {
                error!("Failed to parse input file: {}", err);
                return ExitCode::FAILURE;
            }
        };
        self.process_tree(root)
    }

    fn process_tree(&self, root: Artifact) -> ExitCode {
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

impl Builder {
    fn new_parser<'a, R: Read + 'a>(&self, reader: R) -> Box<dyn ArtifactParser + 'a> {
        match self {
            Self::Maven => Box::new(MavenArtifactParser::new(reader)),
            Self::Gradle => Box::new(GradleArtifactParser::new(reader)),
        }
    }
}
