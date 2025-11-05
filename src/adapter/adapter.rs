use std::io::{BufRead, Write};
use crate::adapter::errors::AdapterError;
pub(crate) use crate::adapter::statement::Statement;

/// Defines a common interface for reading and writing financial statements
/// in different data formats.
pub trait Adapter {
    /// Reads a [`Statement`] from the given buffered input source.
    fn read_from<R: BufRead>(reader: R) -> Result<Statement, AdapterError>;

    /// Writes the provided [`Statement`] to the given output stream.
    fn write_to<W: Write>(writer: W, st: &Statement) -> Result<(), AdapterError>;
}
