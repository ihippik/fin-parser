use std::io::{BufRead};
use crate::adapter::adapter::Adapter;
use crate::adapter::errors::AdapterError;
use crate::adapter::statement::Statement;
use crate::format::csv::CSV;
use crate::format::mt940::Mt940;
use crate::format::xml::XML;
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
            statement = CSV::read_from(reader)?;
        }
        FormatType::MT940 =>{
            statement = Mt940::read_from(reader)?;
        }
        FormatType::CAMT053 =>{
            statement = XML::read_from(reader)?;
        }
    }

    match output_format{
        FormatType::CSV => {
            let file = File::create("output.csv").
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
            CSV::write_to(file, &statement).unwrap();
            Ok("csv file was created.".to_string())
        }
        FormatType::MT940 => {
            let file = File::create("output.mt940").
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
            Mt940::write_to(file, &statement).unwrap();
            Ok("mt940 was created.".to_string())
        }
        FormatType::CAMT053 => {
            let file = File::create("output.camt053").
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
            XML::write_to(file, &statement).unwrap();
            Ok("camt053 was created.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
}
