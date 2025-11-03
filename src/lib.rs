use std::io::{BufRead};
use crate::adapter::adapter::Adapter;
use crate::adapter::errors::AdapterError;
use crate::adapter::statement::Statement;
use crate::format::csv::FormatCsv;
use crate::format::mt940::FormatMt940;
use crate::format::camt::FormatXML;
use std::fs::File;

pub mod format;
pub mod adapter;

pub enum FormatType {
    CSV,
    MT940,
    CAMT053,
}

pub fn convert<R: BufRead>(
    reader: R,
    input_format: FormatType,
    output_format: FormatType,
) -> Result<String, AdapterError> {
    let statement: Statement;

    match input_format {
        FormatType::CSV => {
            statement = FormatCsv::read_from(reader)?;
        }
        FormatType::MT940 =>{
            statement = FormatMt940::read_from(reader)?;
        }
        FormatType::CAMT053 =>{
            statement = FormatXML::read_from(reader)?;
        }
    }

    match output_format{
        FormatType::CSV => {
            let file = File::create("output.csv").
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
            FormatCsv::write_to(file, &statement).unwrap();
            Ok("csv file was created.".to_string())
        }
        FormatType::MT940 => {
            let file = File::create("output.mt940").
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
            FormatMt940::write_to(file, &statement).unwrap();
            Ok("mt940 was created.".to_string())
        }
        FormatType::CAMT053 => {
            let file = File::create("output.camt053").
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
            FormatXML::write_to(file, &statement).unwrap();
            Ok("camt053 was created.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
}
