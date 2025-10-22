use std::io::Error;
#[derive(Debug)]
pub enum IoError {
    Io(Error),
    CsvParseError(String),
    UnknownFormat,
}