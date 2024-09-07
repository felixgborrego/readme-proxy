use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    ArgsMissing(String),
    UnexpectedHeader(String),

    // Externals
    #[from]
    IO(std::io::Error),
    #[from]
    TargetIO(reqwest::Error),
}
