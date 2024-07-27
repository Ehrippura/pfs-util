
mod pfs;

use clap::{Parser, Subcommand};
use pfs::{archive::PFSArchive, unpack::unpack};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Operator,
}

#[derive(Subcommand)]
enum Operator {

    /// Archive a file or entire folder.
    Archive,

    /// Unarchive selected pfs file.
    Unarchive {
        /// input file name
        input: String,
        /// output folder name (optional)
        output: Option<String>,
    }
}

fn main() {

    let cli = Cli::parse();

    match &cli.command {
        Operator::Archive => todo!(),
        Operator::Unarchive { input, output } => {
            let archive = PFSArchive::from_file(&input).unwrap();
            unpack(&archive, output.as_deref()).unwrap();
        },
    }
}

