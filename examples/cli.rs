use fin_parser::{convert, FormatType};

fn main() {
    let file = std::fs::File::open("sber.csv").unwrap();

    match convert(file, FormatType::CSV){
        Ok(res) => {
            println!("{:#?}", res);
        }
        Err(e) => {
            println!("{:#?}", e);
        }
    }
}