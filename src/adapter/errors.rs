/// Common error type for adapter operations such as parsing and writing.
#[derive(Debug)]
pub enum AdapterError {
    /// Error occurred while parsing input data.
    ParseError(String),
    /// Error occurred while writing or serializing output data.
    WriteError(String),
}

/// Maps any displayable error into an [`AdapterError::ParseError`].
pub fn map_parse_err<E: std::fmt::Display>(e: E) -> AdapterError {
    AdapterError::ParseError(e.to_string())
}

/// Maps any displayable error into an [`AdapterError::WriteError`].
pub fn map_write_err<E: std::fmt::Display>(e: E) -> AdapterError {
    AdapterError::WriteError(e.to_string())
}