use fin_parser::{convert, FormatType};
use std::io::{BufReader};

fn main() {
    let mut file = std::fs::File::open("data.mt940").unwrap();
    let reader = BufReader::new(&mut file);

    match convert(reader, FormatType::MT940, FormatType::CAMT053) {
        Ok(res) => {
            println!("{}", res);
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}