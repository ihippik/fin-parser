use std::io::BufRead;
use crate::parser::errors::IoError;
use crate::finance::record::FinanceRecord;

pub trait Parser {
    fn parse<R: BufRead>(&self, reader: R) -> Result<Vec<FinanceRecord>, IoError>;
}