use std::io::{BufRead, BufReader};
use crate::format::csv::EngineCsv;
use crate::parser::errors::IoError;
use crate::parser::parser::Parser;

mod parser;
mod finance;
mod format;

pub enum FormatType {
    CSV,
    MT940,
    CAMT053,
}

pub fn convert<R: std::io::Read>(
    reader: R,
    input_format: FormatType,
    // output_format: FormatType,
) -> Result<String, IoError> {
    match input_format {
        FormatType::CSV => {
            let csv = EngineCsv::new();

            let recs=csv.parse(BufReader::new(reader))?;

            for rec in &recs {
                println!("{:?}", rec.account_credit);
            }

            println!("size {}", recs.len());

            Ok("success".to_string())
        }
        _ => Err(IoError::UnknownFormat)
    }
}

#[cfg(test)]
mod tests {
}
