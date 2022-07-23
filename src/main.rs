use std::{fs::File, io::Read};

use clap::{Parser, Subcommand};
use executor::{execute, Environment};

use crate::executor::Value;

mod executor;
mod translate;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Pack {
        #[clap(value_parser)]
        file: String,
    },
    Unpack {
        #[clap(value_parser)]
        file: String,
    },
    Run {
        #[clap(value_parser)]
        file: String,
    },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Pack { file } => {
            let mut file = File::open(file).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let tokenizer = translate::Tokenizer::from_string(contents);
            let tokens = tokenizer.tokenize();

            let ast = translate::parse(tokens);
            println!("{:#?}", ast);
        }
        Commands::Unpack { file } => todo!(),
        Commands::Run { file } => {
            let mut file = File::open(file).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let tokenizer = translate::Tokenizer::from_string(contents);
            let tokens = tokenizer.tokenize();

            let ast = translate::parse(tokens);
            let mut env = Environment::new();

            env.add_rust_function("print", vec![String::from("value")], |args, _env| {
                println!("{}", args[0]);
                Value::Option(None)
            });

            let env = env.contain();

            execute(&ast, env);
            // println!("{:#?}", ast);
        }
    }
}
