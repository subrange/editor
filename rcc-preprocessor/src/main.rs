use anyhow::Result;
use clap::Parser;
use rcc_preprocessor::Preprocessor;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(name = "rcpp", version, about = "C preprocessor for Ripple C Compiler")]
struct Args {
    /// Input C source file
    input: PathBuf,

    /// Output preprocessed file (defaults to stdout)
    #[clap(short, long)]
    output: Option<PathBuf>,

    /// Include directories
    #[clap(short = 'I', long = "include", value_name = "DIR")]
    include_dirs: Vec<PathBuf>,

    /// Define macro
    #[clap(short = 'D', long = "define", value_name = "NAME[=VALUE]")]
    defines: Vec<String>,

    /// Undefine macro
    #[clap(short = 'U', long = "undefine", value_name = "NAME")]
    undefines: Vec<String>,

    /// Keep comments in output
    #[clap(long)]
    keep_comments: bool,

    /// Keep line directives
    #[clap(long)]
    keep_line_directives: bool,

    /// Verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        env_logger::init();
    }

    // Read input file
    let input = fs::read_to_string(&args.input)?;

    // Create preprocessor
    let mut preprocessor = Preprocessor::new();

    // Add include directories
    for dir in args.include_dirs {
        preprocessor.add_include_dir(dir);
    }

    // Process defines
    for define in args.defines {
        if let Some((name, value)) = define.split_once('=') {
            preprocessor.define(name.to_string(), Some(value.to_string()));
        } else {
            preprocessor.define(define, None);
        }
    }

    // Process undefines
    for undef in args.undefines {
        preprocessor.undefine(&undef);
    }

    // Configure options
    preprocessor.set_keep_comments(args.keep_comments);
    preprocessor.set_keep_line_directives(args.keep_line_directives);

    // Process the input
    let output = preprocessor.process(&input, args.input)?;

    // Write output
    if let Some(output_path) = args.output {
        fs::write(output_path, output)?;
    } else {
        print!("{}", output);
    }

    Ok(())
}