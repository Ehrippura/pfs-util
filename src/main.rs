
mod pfs;

use clap::{Parser, Subcommand};
use pfs::{archive::PFSArchive, pack::pack, unpack::unpack};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Operator,
}

#[derive(Subcommand)]
enum Operator {

    /// Archive a file or entire folder.
    Archive {
        /// input file name
        input: String,
        /// output file name
        output: String,
    },

    /// Unarchive selected pfs file.
    Unarchive {
        /// Input file name
        input: String,
        /// Output folder name (optional)
        output: Option<String>,
        /// Dry run
        #[arg(long, short)]
        dry: bool
    }
}

fn main() {

    let cli = Cli::parse();

    match &cli.command {
        Operator::Archive { input, output } => {
            pack(input, output).unwrap();
        },
        Operator::Unarchive { input, output, dry } => {
            let archive = PFSArchive::from_file(&input).unwrap();
            unpack(&archive, output.as_deref(), *dry).unwrap();
        },
    }
}

