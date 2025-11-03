use std::io::{BufRead, Write};
use crate::adapter::errors::AdapterError;
pub(crate) use crate::adapter::statement::Statement;

pub trait Adapter {
    fn read_from<R: BufRead>(reader: R) -> Result<Statement, AdapterError>;
    fn write_to<W: Write>(writer: W, st: &Statement) -> Result<(), AdapterError>;
}