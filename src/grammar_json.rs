use std::fs;
use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::Error;

pub fn read_from_file(path: &Path) -> Result<Root, Error> {
    read(&fs::read_to_string(path).map_err(|err| Error::GrammarJson(err.to_string()))?)
}

pub fn read(contents: &str) -> Result<Root, Error> {
    serde_json::from_str(contents).map_err(|err| Error::GrammarJson(err.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct Root {
    pub rules: IndexMap<String, Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rule {
    Prec(Prec),
    Seq(Seq),
    String(String_),
    Blank,
    Choice(Choice),
    Repeat(Repeat),
    Symbol(Symbol),
    Field(Field),
    Pattern(Pattern),
    Alias(Alias),
    PrecRight(PrecRight),
    Repeat1(Repeat1),
    PrecLeft(PrecLeft),
    Token(Token),
    ImmediateToken(ImmediateToken),
}

impl Rule {
    pub fn is_blank(&self) -> bool {
        matches!(self, Self::Blank)
    }

    pub fn as_field(&self) -> &Field {
        match self {
            Self::Field(field) => field,
            _ => panic!("expected field"),
        }
    }
}

pub type Precision = i32;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prec {
    pub value: Precision,
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seq {
    pub members: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub members: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeat {
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Symbol {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct String_ {
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    pub content: Box<Rule>,
    pub named: bool,
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrecRight {
    pub value: Precision,
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repeat1 {
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrecLeft {
    pub value: Precision,
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Token {
    pub content: Box<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmediateToken {
    pub content: Box<Rule>,
}
