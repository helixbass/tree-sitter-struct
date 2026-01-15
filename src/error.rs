use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("grammar.json: {0}")]
    GrammarJson(String),
}
