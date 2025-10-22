use std::io::BufRead;
use crate::finance::record::FinanceRecord;
use crate::parser::errors::IoError;
use crate::parser::parser::Parser;
use csv::{StringRecord};
pub struct EngineCsv {
}

impl EngineCsv {
    pub fn new() -> EngineCsv {
        EngineCsv {}
    }
}

impl Parser for EngineCsv {
    fn parse<R: BufRead>(&self, reader: R) -> Result<Vec<FinanceRecord>, IoError> {
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(false)   // первая строка — имена полей
            .from_reader(reader);

        let mut out = Vec::new();
        let start_line = 12;

        for (i,row) in csv_reader.records().enumerate() {
            if i + 1 < start_line {
                continue;
            }

            let rec = match row {
                Ok(r) => r,
                Err(e) => return Err(IoError::CsvParseError(format!("строка {}: {e}", i + 1))),
            };

            match parse_row(&rec)? {
                Some(tx) => out.push(tx),
                None => continue,
            }

        }

        Ok(out)
    }
}

fn parse_amount(raw: &str) -> Result<f64, IoError> {
    let cleaned = raw.
        replace(' ', "").
        replace('\u{00A0}', "").
        replace(',', ".");

    cleaned.parse::<f64>()
        .map_err(|e| IoError::CsvParseError(format!("некорректная сумма '{raw}': {e}")))
}

fn parse_row(row: &StringRecord) -> Result<Option<FinanceRecord>, IoError>{
    let posting_date    = get(row, 1).unwrap_or("").to_string();
    let account_debit   = get(row, 4).unwrap_or("").to_string();
    let account_credit  = get(row, 8).unwrap_or("").to_string();

    let debit_amount    = get(row, 9).map(parse_amount).transpose()?;
    let credit_amount   = get(row, 13).map(parse_amount).transpose()?;

    let doc_number      = get(row, 14).map(|s| s.to_string());
    let operation_code  = get(row, 16).map(|s| s.to_string());
    let bank_info       = get(row, 17).map(|s| s.to_string());
    let purpose         = get(row, 20).map(|s| s.to_string());

    if posting_date.is_empty() {
        return Ok(None)
    }

    Ok(Some(FinanceRecord {
        posting_date,
        account_debit,
        account_credit,
        debit_amount,
        credit_amount,
        doc_number,
        operation_code,
        bank_info,
        purpose,
    }))
}

fn get<'a>(rec: &'a StringRecord, idx: usize) -> Option<&'a str> {
    rec.get(idx).map(|s| s.trim()).filter(|s| !s.is_empty())
}