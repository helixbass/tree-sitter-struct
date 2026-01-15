use std::path::PathBuf;

use clap::Parser;

use tree_sitter_struct::read_from_file;

fn main() {
    let args = Args::parse();

    let grammar_json = read_from_file(&args.grammar_json_filename).unwrap();
    println!("grammar_json: {grammar_json:#?}");
}

#[derive(Parser)]
struct Args {
    pub grammar_json_filename: PathBuf,
}
