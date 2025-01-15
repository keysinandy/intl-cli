use std::env;

use clap::{command, Parser, Subcommand};

use crate::{
    intl::run::run_extract,
    translate::{tencent::TencentPayload, translate::Translate},
};

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
        #[arg(short, long, help = "Exclude files glob patterns", default_values = ["**/node_modules/**", "**/.git/**"])]
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

    TencentTranslate {
        #[arg(short, long, help = "Input file path", default_value = "output.json")]
        input: String,
        #[arg(short, long, help = "Output file path")]
        output: String,
        #[arg(short, long, help = "source language", default_value = "zh")]
        source: Option<String>,
        #[arg(short, long, help = "target language", default_value = "en")]
        target: Option<String>,
        #[arg(short, long, help = "project id", default_value = "0")]
        project_id: Option<u32>,
        #[arg(short('d'), long, help = "secret_id")]
        secret_id: String,
        #[arg(short('k'), long, help = "secret_key")]
        secret_key: String,
        #[arg(short, long, help = "Translate and write all from input to output")]
        write_all: bool,
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

        Some(Commands::TencentTranslate {
            input,
            output,
            source,
            target,
            project_id,
            secret_id,
            secret_key,
            write_all,
        }) => {
            let input_dir = env::current_dir().unwrap().join(input);
            let output_dir = env::current_dir().unwrap().join(output);

            let input_str = input_dir.to_str().unwrap();
            let output_str = output_dir.to_str().unwrap();

            let payload =
                TencentPayload::new(source.unwrap(), target.unwrap(), project_id.unwrap());
            let mut translate =
                Translate::new(input_str.to_string(), output_str.to_string(), payload);
            let _ = translate.from_tencent(secret_id.as_str(), secret_key.as_str(), write_all);
        }
        _ => (),
    }
}
