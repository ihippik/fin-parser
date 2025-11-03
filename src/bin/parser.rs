use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use fin_parser::format::xml::XML;
use fin_parser::format::csv::CSV;
use fin_parser::format::mt940::Mt940;
use fin_parser::adapter::adapter::Adapter;
use fin_parser::adapter::errors::AdapterError;
use fin_parser::format::camt::CAMT;

#[derive(Debug, Clone,ValueEnum)]
enum Format {
    Csv,
    Mt940,
    Camt053,
    Xml,
}

#[derive(Parser, Debug)]
#[command(name="parser", version, about="сonversion of financial formats")]
struct Cli {
    #[arg(long="input")]
    input: Option<String>,

    #[arg(long="output")]
    output: Option<String>,

    #[arg(long="in-format", value_enum)]
    in_format: Format,

    #[arg(long="out-format", value_enum)]
    out_format: Format,
}

fn main() -> Result<(), AdapterError>{
    let cli = Cli::parse();
    let reader: Box<dyn Read> = match cli.input {
        Some(path) => Box::new(
            File::open(path).map_err(|e|AdapterError::ParseError(e.to_string()))?
        ),
        None => Box::new(io::stdin()),
    };
    let buf = BufReader::new(reader);

    let statement = match cli.in_format {
        Format::Csv => { CSV::read_from(buf)},
        Format::Mt940 => { Mt940::read_from(buf)},
        Format::Xml => { XML::read_from(buf)},
        Format::Camt053 => { CAMT::read_from(buf)},
    }?;


    let mut writer: Box<dyn Write> = match cli.output {
        Some(path) => Box::new(
            File::create(path).map_err(|e|AdapterError::WriteError(e.to_string()))?
        ),
        None => Box::new(io::stdout()),
    };

    match cli.out_format {
        Format::Csv => CSV::write_to(&mut writer, &statement),
        Format::Mt940 => Mt940::write_to(&mut writer, &statement),
        Format::Xml => XML::write_to(&mut writer, &statement),
        Format::Camt053 => CAMT::write_to(&mut writer, &statement),
    }?;

    writer.flush().map_err(|e|AdapterError::WriteError(e.to_string()))
}