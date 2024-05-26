use clap::Parser;
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
    /// Minify file and exit. Will remove all whitespace.
    #[clap(name = "minify", visible_alias = "min")]
    Minify {
        /// Input JSON file
        #[clap(value_parser)]
        input: PathBuf,
    },
    /// Prettify file and exit. Will add whitespaces to make the file more readable.
    #[clap(name = "prettify", visible_alias = "fmt")]
    Prettify {
        /// How many spaces
        #[clap(long = "indent", default_value_t = 4)]
        indent: u32,
        /// Input JSON file
        #[clap(value_parser)]
        input: PathBuf,
    },
}

fn main() {
    let opt: Args = Args::parse_from(wild::args_os());

    if opt.command.is_some() {
        assert!(
            opt.input.is_none(),
            "Parse error, input file should be part of subcommand"
        );
    }

    match opt.command.unwrap_or_else(|| Command::Interactive {
        input: opt
            .input
            .unwrap_or_else(|| panic!("No input file provided")),
    }) {
        Command::Interactive { input: _ } => {
            unimplemented!("interactive mode not ready :(")
        }
        Command::Show { input, pointer } => {
            show(&input, pointer.as_deref());
        }
        Command::Minify { input } => {
            let json = fs::read_to_string(&input).unwrap();
            let size_before = json.len();
            let minified = jsdu::minify::minify(&json);
            let size_after = minified.len();
            fs::write(&input, minified).unwrap();
            println!("minified from {} to {} bytes", size_before, size_after);
        }
        Command::Prettify { input, indent } => {
            let json = fs::read_to_string(&input).unwrap();
            let size_before = json.len();
            let prettified = jsdu::prettify::prettify(&json, usize::try_from(indent).unwrap());
            let size_after = prettified.len();
            fs::write(&input, prettified).unwrap();
            println!("prettified from {} to {} bytes", size_before, size_after);
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
