use std::io::BufRead;
use crate::adapter::errors::AdapterError;

pub trait DataAdapter<T> {
    fn import<R: BufRead>(&self, reader: R) -> Result<Vec<T>, AdapterError>;
}