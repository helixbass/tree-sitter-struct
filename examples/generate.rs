use std::path::PathBuf;

use clap::Parser;

use tree_sitter_struct::{generate, read_from_file};

fn main() {
    let args = Args::parse();

    let grammar_json = read_from_file(&args.grammar_json_filename).unwrap();
    // println!("grammar_json: {grammar_json:#?}");
    generate(&grammar_json, &args.language);
}

#[derive(Parser)]
struct Args {
    pub grammar_json_filename: PathBuf,
    pub language: String,
}
