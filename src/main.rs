use std::{
    fs::File,
    io::{Read, Write},
};

use clap::{Parser, Subcommand};
use executor::{execute, Environment};
use file::pack;

use crate::executor::Value;

mod executor;
mod file;
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
        input: String,
        #[clap(value_parser)]
        output: String,
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
        Commands::Pack { input, output } => {
            let mut file = File::open(input).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let tokenizer = translate::Tokenizer::from_string(contents);
            let mut tokens = tokenizer.tokenize();

            let ast = translate::parse(&mut tokens);
            let new_file_contents = pack(&ast);

            // Write file to output
            let mut file = File::create(output).unwrap();
            file.write_all(new_file_contents.as_bytes()).unwrap();
        }
        Commands::Unpack { file: _ } => todo!(),
        Commands::Run { file } => {
            let mut file = File::open(file).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let tokenizer = translate::Tokenizer::from_string(contents);
            let mut tokens = tokenizer.tokenize();

            let ast = translate::parse(&mut tokens);
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