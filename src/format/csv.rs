use std::io::{Read,Write};
use regex::Regex;
use csv::{StringRecord, WriterBuilder};
use crate::adapter::errors::AdapterError;
use crate::adapter::adapter::Adapter;
use crate::adapter::adapter::Statement;
use crate::adapter::statement::{DebitCredit, Entry};
use serde::{Serialize};

/// CSV adapter implementing the `Adapter` trait.
///
/// Converts between CSV and internal `Statement` representation.
pub struct CSV;

impl CSV {
    fn undefined() -> String {
        "undefined".to_string()
    }
}


/// A structure representing a single item in CSV format for financial transaction records.
///
/// This structure defines various fields that store transaction details in string format.
/// It derives the `Serialize` trait to facilitate conversion of the struct into formats like JSON or CSV.
#[derive(Serialize)]
pub struct ItemCsv {
    tx_data:String,
    tx_number: String,
    tx_description: String,
    debit_account_number:String,
    debit_inn:String,
    debit_account_name:String,
    debit_amount: String,
    credit_account_number:String,
    credit_inn:String,
    credit_account_name:String,
    credit_amount: String,
    bank_bik: String,
    bank_name: String,
}

impl Adapter for CSV {
    fn read_from<R: Read>(reader: R) -> Result<Statement,AdapterError>{
        let mut entries = Vec::new();

        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)   // первая строка — имена полей
            .from_reader(reader);

        for (i,row) in csv_reader.records().enumerate() {
            let rec = match row {
                Ok(r) => r,
                Err(e) => return Err(AdapterError::ParseError(format!("строка {}: {e}", i + 1))),
            };

            match parse_row(&rec)? {
                Some(tx) => entries.push(Entry{
                    booking_date: tx.tx_data.clone(),
                    value_date: tx.tx_data.clone(),
                    amount: if tx.credit_amount.is_empty() {tx.debit_amount} else {tx.credit_amount.clone()},
                    currency: "RUB".to_string(),
                    kind: if tx.credit_amount.is_empty() { DebitCredit::Debit } else { DebitCredit::Credit },
                    description: tx.tx_description,
                    reference: Some(tx.tx_number),
                }),
                None => continue,
            }

        }

        Ok(Statement{
            id: Self::undefined(),
            opening_balance: None,
            closing_balance:None,
            account_id: Self::undefined(),
            entries
        })
    }


    fn write_to<W: Write>(mut writer: W, st: &Statement) -> Result<(), AdapterError>{
        let mut builder = WriterBuilder::new().from_writer(&mut writer);
        for entry in &st.entries {
            let mut debit_amount = "";
            let mut credit_amount ="";

            if entry.kind == DebitCredit::Credit {
                credit_amount = &entry.amount
            }else{
                debit_amount = &entry.amount
            }

            let raw = ItemCsv{
                tx_data: entry.booking_date.clone(),
                tx_number: entry.reference.clone().unwrap_or_else(|| "none".to_string()),
                tx_description: entry.description.clone(),
                debit_account_number: entry.reference.clone().unwrap_or_else(|| "none".to_string()),
                debit_inn: Self::undefined(),
                debit_account_name: Self::undefined(),
                debit_amount: debit_amount.to_string(),
                credit_account_number: Self::undefined(),
                credit_inn: Self::undefined(),
                credit_account_name: "".to_string(),
                credit_amount: credit_amount.to_string(),
                bank_bik: Self::undefined(),
                bank_name: Self::undefined(),
            };
            builder.serialize(raw).map_err(|e| AdapterError::ParseError(format!("{e}")))?;
        }

        builder.flush().map_err(|e| AdapterError::ParseError(format!("{e}")))?;
        Ok(())
    }
}

fn parse_row(row: &StringRecord) -> Result<Option<ItemCsv>, AdapterError>{
    let tx_data    = get(row, 1).unwrap_or("").to_string();
    let tx_number    = get(row, 14).unwrap_or("").to_string();
    let tx_description    = get(row, 20).unwrap_or("").to_string();


    let debit    = get(row, 4).unwrap_or("").to_string();
    let (account, inn, name) = parse_counterparty(&debit);

    let debit_account_number    = account.unwrap_or("".to_string());
    let debit_inn    = inn.unwrap_or("".to_string());
    let debit_account_name    = name.unwrap_or("".to_string());
    let debit_amount    = get(row, 9).unwrap_or("").to_string();

    let credit    = get(row, 8).unwrap_or("").to_string();
    let (account, inn, name) = parse_counterparty(&credit);

    let credit_account_number  = account.unwrap_or("".to_string());
    let credit_inn    = inn.unwrap_or("".to_string());
    let credit_account_name    = name.unwrap_or("".to_string());
    let credit_amount    = get(row, 13).unwrap_or("").to_string();

    let bank    = get(row, 17).unwrap_or("").to_string();

    let mut bank_bik = "".to_string();
    let mut bank_name = "".to_string();

    if let Some((bik, bank)) = parse_bik_and_bank(&bank) {
        bank_bik = bik;
        bank_name = bank;
    }

    if tx_data.is_empty() || (debit_amount.is_empty() && credit_amount.is_empty()) {
        return Ok(None)
    }

    Ok(Some(ItemCsv {
        tx_data,
        tx_number,
        tx_description,
        debit_account_number,
        debit_inn,
        debit_account_name,
        debit_amount,
        credit_account_number,
        credit_inn,
        credit_account_name,
        credit_amount,
        bank_bik,
        bank_name,
    }))
}
fn get<'a>(rec: &'a StringRecord, idx: usize) -> Option<&'a str> {
    rec.get(idx).map(|s| s.trim()).filter(|s| !s.is_empty())
}

fn parse_counterparty(block: &str) -> (Option<String>, Option<String>, Option<String>) {
    let lines: Vec<_> = block
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let account = lines.get(0).map(|s| s.to_string());
    let inn     = lines.get(1).map(|s| s.to_string());
    let name    = lines.get(2).map(|s| s.to_string());

    (account, inn, name)
}

fn parse_bik_and_bank(line: &str) -> Option<(String, String)> {
    let re = Regex::new(r"БИК\s+(\d{9})\s+(.+)").
        expect("checked by unit-test, should not fail");

    re.captures(line).map(|cap| {
        let bik = cap[1].to_string();
        let bank = cap[2].trim().to_string();
        (bik, bank)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bik_and_bank_basic() {
        let input = "БИК 042202603 ВОЛГО-ВЯТСКИЙ БАНК ПАО СБЕРБАНК, г.Нижний Новгород";
        let result = parse_bik_and_bank(input);
        assert!(result.is_some());

        let (bik, bank) = result.unwrap();
        assert_eq!(bik, "042202603");
        assert_eq!(bank, "ВОЛГО-ВЯТСКИЙ БАНК ПАО СБЕРБАНК, г.Нижний Новгород");
    }

    #[test]
    fn test_parse_counterparty() {
        let input = "40702810440000030888
7735602068
ООО РОМАШКА
";
        let result = parse_counterparty(input);

        assert!(result.0.is_some());
        assert!(result.1.is_some());
        assert!(result.2.is_some());

        assert_eq!(result.0.unwrap(), "40702810440000030888");
        assert_eq!(result.1.unwrap(), "7735602068");
        assert_eq!(result.2.unwrap(), "ООО РОМАШКА");
    }
}