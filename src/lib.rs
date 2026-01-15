mod error;
mod grammar_json;

pub use error::Error;
pub use grammar_json::{read, read_from_file};
