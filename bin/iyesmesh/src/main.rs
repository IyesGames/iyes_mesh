use iyes_mesh::{read::IyesMeshReaderSettings, write::IyesMeshWriterSettings};

use crate::prelude::*;

#[allow(unused_imports)]
mod prelude {
    pub use std::path::{Path, PathBuf};

    pub use anyhow::{Context, Result as AnyResult, bail};
}

mod cmd {
    pub mod edit;
    pub mod extract_user_data;
    pub mod info;
    pub mod verify;
    pub mod merge;
    #[cfg(feature = "obj")]
    pub mod from_obj;
}

mod util;

#[derive(clap::Parser, Debug)]
#[command(about = "Tool for working with MineWars data files.")]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    /// Operation to perform
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(clap::Args, Debug)]
struct CommonArgs {
    /// Print extra info about what the tool is doing
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::Args, Debug)]
struct WriteArgs {
    /// Zstd compression level (default: max)
    #[arg(short, long)]
    level: Option<i32>,
    /// Do not write data checksum into file (faster)
    #[arg(long)]
    no_data_checksum: bool,
    /// Convert index data from U16 to U32 if needed
    #[arg(long)]
    upconvert_indices: bool,
}

#[derive(clap::Args, Debug)]
struct ReadArgs {
    /// Try to process files even if checksums are wrong
    #[arg(long)]
    ignore_checksums: bool,
}

#[derive(clap::Args, Debug)]
struct OutputArgs {
    /// Overwrite output file if it exists
    #[arg(short, long)]
    overwrite: bool,
}

#[derive(clap::Args, Debug)]
struct InputPath {
    /// Path to the input file
    in_file: PathBuf,
}

#[derive(clap::Args, Debug)]
struct InputPaths {
    /// Path to the input files
    in_files: Vec<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct OutputPath {
    /// Path where to save the output file
    out_file: PathBuf,
}

#[derive(clap::Args, Debug)]
struct InOutPaths {
    /// Path to the input file
    in_file: PathBuf,
    /// Path to the output file (if unspecified, overwrite the input)
    out_file: Option<PathBuf>,
}

#[derive(clap::Args, Debug)]
struct OptOutputPath {
    /// Path where to save the output file (stdout if unspecified)
    out_file: Option<PathBuf>,
}

#[derive(clap::Subcommand, Debug)]
enum CliCommand {
    /// Print information about the tool
    Version,
    /// Show general info about the file
    Info(cmd::info::InfoArgs),
    /// Try decoding the file to check for errors
    Verify(cmd::verify::VerifyArgs),
    /// Load a file, make some changes, save the changes
    Edit(cmd::edit::EditArgs),
    /// Decode the user data from a file
    ExtractUserData(cmd::extract_user_data::ExtractUserDataArgs),
    /// Load several files, save a file with their combined meshes
    Merge(cmd::merge::MergeArgs),
    /// Import from OBJ format
    #[cfg(feature = "obj")]
    FromObj(cmd::from_obj::FromObjArgs),
}

impl From<&ReadArgs> for IyesMeshReaderSettings {
    fn from(args: &ReadArgs) -> Self {
        Self {
            verify_metadata_checksum: !args.ignore_checksums,
            verify_data_checksum: !args.ignore_checksums,
        }
    }
}

impl From<&WriteArgs> for IyesMeshWriterSettings {
    fn from(args: &WriteArgs) -> Self {
        let default = Self::default();
        Self {
            upconvert_indices: args.upconvert_indices,
            write_data_checksum: !args.no_data_checksum,
            compression_level: args.level.unwrap_or(default.compression_level),
        }
    }
}

fn run_command(cli: &Cli) -> AnyResult<()> {
    match &cli.command {
        CliCommand::Version => {
            // Verbose always prints version anyway
            if !cli.common.verbose {
                print_version();
            }
            Ok(())
        }
        CliCommand::Info(args) => cmd::info::run(&cli.common, args),
        CliCommand::Verify(args) => cmd::verify::run(&cli.common, args),
        CliCommand::ExtractUserData(args) => {
            cmd::extract_user_data::run(&cli.common, args)
        }
        CliCommand::Edit(args) => cmd::edit::run(&cli.common, args),
        CliCommand::Merge(args) => cmd::merge::run(&cli.common, args),
        #[cfg(feature = "obj")]
        CliCommand::FromObj(args) => cmd::from_obj::run(&cli.common, args),
    }
}

fn print_version() {
    eprintln!(
        "{} version {}. Works with file format version {}.",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        iyes_mesh::FORMAT_VERSION,
    );
    eprintln!();
}

fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    if cli.common.verbose {
        print_version();
    }

    if let Err(e) = run_command(&cli) {
        eprintln!("Error: {:#}", e);
        std::process::exit(2);
    }
}
