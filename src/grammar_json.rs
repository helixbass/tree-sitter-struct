use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::Error;

pub fn read_from_file(path: &Path) -> Result<Root, Error> {
    read(&fs::read_to_string(path).map_err(|err| Error::GrammarJson(err.to_string()))?)
}

pub fn read(contents: &str) -> Result<Root, Error> {
    serde_json::from_str(contents).map_err(|err| Error::GrammarJson(err.to_string()))
}

#[derive(Deserialize)]
pub struct Root {
    pub rules: HashMap<String, Rule>,
}

#[derive(Deserialize)]
pub enum Rule {
    Prec(Prec),
    Seq(Seq),
    String(String),
    Blank,
    Choice(Choice),
    Repeat(Repeat),
    Symbol(String),
}

#[derive(Deserialize)]
pub struct Prec {
    pub value: u32,
    pub content: Box<Rule>,
}

#[derive(Deserialize)]
pub struct Seq {
    pub members: Vec<Rule>,
}

#[derive(Deserialize)]
pub struct Choice {
    pub members: Vec<Rule>,
}

#[derive(Deserialize)]
pub struct Repeat {
    pub content: Box<Rule>,
}
