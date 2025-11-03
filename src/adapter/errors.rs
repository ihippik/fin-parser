#[derive(Debug)]
pub enum AdapterError {
    ParseError(String),
    WriteError(String),
    UnknownFormat,
}