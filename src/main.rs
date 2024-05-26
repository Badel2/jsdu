use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Debug, Clone, Copy, clap::ValueEnum)]
pub enum ByteFormat {
    Metric,
    Binary,
    Bytes,
    GB,
    Gib,
    MB,
    Mib,
}

fn dft_format() -> ByteFormat {
    if cfg!(target_vendor = "apple") {
        ByteFormat::Metric
    } else {
        ByteFormat::Binary
    }
}

/// JSON file size analyzer
#[derive(Debug, Parser)]
#[clap(name = "jsdu", version)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// The format with which to print byte counts.
    #[clap(
        short = 'f',
        long,
        value_enum,
        default_value_t = dft_format(),
        ignore_case = true,
    )]
    pub format: ByteFormat,

    /// Input JSON file (for when no subcommand is provided)
    #[clap(value_parser)]
    pub input: Option<PathBuf>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Launch the terminal user interface
    #[clap(name = "interactive", visible_alias = "i")]
    Interactive {
        /// Input JSON file
        #[clap(value_parser)]
        input: PathBuf,
    },
    /// Non interactive version, print the top level size usage and exit
    #[clap(name = "show", visible_alias = "s")]
    Show {
        /// JSON pointer to navigate to before printing size (RFC 6901)
        #[clap(long = "pointer")]
        pointer: Option<String>,
        /// Input JSON file
        #[clap(value_parser)]
        input: PathBuf,
    },
}

fn main() {
    let mut opt: Args = Args::parse_from(wild::args_os());

    match opt.command.unwrap_or_else(|| Command::Interactive {
        input: opt
            .input
            .unwrap_or_else(|| panic!("No input file provided")),
    }) {
        Command::Interactive { input } => {
            unimplemented!("interactive mode not ready :(")
        }
        Command::Show { input, pointer } => {
            show(&input, pointer.as_deref());
        }
    }
}

fn show(path: &Path, json_pointer: Option<&str>) {
    let json = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading from {}: {}", path.display(), e);
            std::process::exit(1);
        }
    };

    let mut js = &mut jsdu::size::JsonSize::new(&json);
    if let Some(json_pointer) = json_pointer {
        js = js
            .index_json_pointer(&json, json_pointer)
            .expect("invalid JSON pointer");
    }
    for l in js.display_list(&json) {
        println!("{}", l);
    }
}
