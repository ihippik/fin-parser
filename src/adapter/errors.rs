use std::io::Error;
#[derive(Debug)]
pub enum AdapterError {
    CsvParseError(String),
    UnknownFormat,
}