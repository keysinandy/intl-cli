use clap::{command, Parser, Subcommand};

use crate::intl::run::run_extract;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]

pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// extract i18n text from files
    Extract {
        #[arg(short, long, help = "Output file path",default_values = ["output.json"])]
        output: Option<String>,
        #[arg(short, long, help = "Exclude files glob patterns", default_values = ["/node_modules/**"])]
        excludes: Option<Vec<String>>,
        #[arg(short, long, help = "Include files glob patterns", default_values = ["*.{ts,tsx}"])]
        includes: Option<Vec<String>>,
        #[arg(
            short,
            long,
            help = "Use -d to Delate unreached key and value pairs in output"
        )]
        delete_unreached: bool,
    },
}

pub fn run_cli() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Extract {
            output,
            excludes,
            includes,
            delete_unreached,
        }) => {
            run_extract(output, excludes, includes, delete_unreached);
        }
        _ => (),
    }
}
