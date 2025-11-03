#[derive(Debug)]
pub enum AdapterError {
    ParseError(String),
    WriteError(String),
    UnknownFormat,
}

pub fn map_parse_err<E: std::fmt::Display>(e: E) -> AdapterError {
    AdapterError::ParseError(e.to_string())
}

pub fn map_write_err<E: std::fmt::Display>(e: E) -> AdapterError {
    AdapterError::WriteError(e.to_string())
}