use std::io::{BufRead, BufReader};
use crate::format::csv::FormatCsv;
use crate::adapter::errors::AdapterError;
use crate::adapter::adapter::DataAdapter;
mod format;
mod adapter;

pub enum FormatType {
    CSV,
    MT940,
    CAMT053,
}

pub fn convert<R: std::io::Read>(
    reader: R,
    input_format: FormatType,
    // output_format: FormatType,
) -> Result<String, AdapterError> {
    match input_format {
        FormatType::CSV => {
            let csv = FormatCsv::new();

            let recs=csv.import(BufReader::new(reader))?;

            for rec in &recs {
                println!("{:?}", rec.bank_name);
            }

            Ok("success".to_string())
        }
        _ => Err(AdapterError::UnknownFormat)
    }
}

#[cfg(test)]
mod tests {
}
