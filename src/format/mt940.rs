use std::io::{BufRead,BufReader, Read, Write};
use std::string::ToString;
use crate::adapter::adapter::{Adapter, Statement};
use crate::adapter::statement::Balance as StBalance;
use crate::adapter::statement::{DebitCredit, Entry};
use crate::adapter::errors::AdapterError;

#[derive(Debug)]
struct MT940Statement {
    reference: String,        // :20:
    account_id: String,       // :25:
    opening_balance: Balance, // :60F:
    transactions: Vec<Transaction>, // :61: + :86:
    closing_balance: Balance, // :62F:
}

#[derive(Debug)]
struct Balance {
    credit: bool,
    date_yyymmdd: String,
    currency: String,
    amount: String,
}

#[derive(Debug)]
struct Transaction {
    date_yyymmdd: String,
    entry_mmdd: String,
    is_credit: bool,
    amount: String,
    type_code: String,
    reference: String,
    description: String,
}

const PREFIX_TX: &str = ":61:";
const PREFIX_TX_ID: &str = ":20:";
const PREFIX_ACCOUNT_ID: &str = ":25:";
const PREFIX_OPN_BALANCE: &str = ":60F:";
const PREFIX_CLS_BALANCE: &str = ":62F:";
const PREFIX_TX_DESC: &str = ":86:";

pub struct FormatMt940;

impl Adapter for FormatMt940 {
    fn read_from<R: BufRead>(r: R) -> Result<Statement, AdapterError> {
        let reader = BufReader::new(r);
        let mt_st = parse_mt940_min(reader).map_err(|e| AdapterError::ParseError(e))?;
        Ok(mt_st.into())
    }


    fn write_to<W: Write>(mut writer: W, st: &Statement) -> Result<(), AdapterError> {
        writeln!(writer,":20:{}",st.id).map_err(|e|AdapterError::ParseError(e.to_string()))?;
        writeln!(writer,":25:{}",st.account_id).
            map_err(|e|AdapterError::ParseError(e.to_string()))?;

        if st.opening_balance.is_some(){
            writeln!(writer,":60F:{}",balance_to_str(st.opening_balance.as_ref().unwrap())).
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
        }

        for entry in &st.entries {
            writeln!(writer,":61:{}",entry.booking_date).map_err(|e|AdapterError::ParseError(e.to_string()))?;
            writeln!(writer,":86:{}",entry.description).map_err(|e|AdapterError::ParseError(e.to_string()))?;
        }

        if st.closing_balance.is_some(){
            writeln!(writer,":62F:{}",balance_to_str(&st.closing_balance.as_ref().unwrap())).
                map_err(|e|AdapterError::ParseError(e.to_string()))?;
        }

        Ok(())
    }
}

fn balance_to_str(balance: &StBalance) -> String {
    let sign = match balance.kind {
        DebitCredit::Credit => 'C',
        DebitCredit::Debit => 'D',
    };

    let date_short = if balance.date_yyymmdd.len() == 8 {
        &balance.date_yyymmdd[2..]
    } else {
        &balance.date_yyymmdd
    };

    let amount_str = format!("{:.2}", balance.amount).replace('.', ",");

    format!("{}{}{}{}", sign, date_short, balance.currency, amount_str)
}


fn parse_mt940_min(input: BufReader<impl Read>) -> Result<MT940Statement, String> {
    let mut reference = String::new();
    let mut account_id = String::new();
    let mut opening_balance: Option<Balance> = None;
    let mut closing_balance: Option<Balance> = None;
    let mut transactions: Vec<Transaction> = Vec::new();
    let mut last_tx_needs_86 = false;

    for raw_line in input.lines() {
        let line = raw_line.unwrap();
        if line.is_empty() { continue; }

        if let Some(rest) = line.strip_prefix(PREFIX_TX_ID) {
            reference = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix(PREFIX_ACCOUNT_ID) {
            account_id = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix(PREFIX_OPN_BALANCE) {
            opening_balance = Some(parse_balance_field(rest.trim())?);
        } else if let Some(rest) = line.strip_prefix(PREFIX_TX) {
            let (tx, _) = parse_transaction_61(rest.trim())?;
            transactions.push(tx);
            last_tx_needs_86 = true;
        } else if let Some(rest) = line.strip_prefix(PREFIX_TX_DESC) {
            if last_tx_needs_86 {
                if let Some(last) = transactions.last_mut() {
                    if !last.description.is_empty() {
                        last.description.push(' ');
                    }
                    last.description.push_str(rest.trim());
                }
                last_tx_needs_86 = false;
            }
        } else if let Some(rest) = line.strip_prefix(PREFIX_CLS_BALANCE) {
            closing_balance = Some(parse_balance_field(rest.trim())?);
        }
    }

    let opening_balance = opening_balance.ok_or("missing :60F: opening balance")?;
    let closing_balance = closing_balance.ok_or("missing :62F: closing balance")?;
    if reference.is_empty() { return Err("missing :20: reference".into()); }
    if account_id.is_empty() { return Err("missing :25: account id".into()); }

    Ok(MT940Statement {
        reference,
        account_id,
        opening_balance,
        transactions,
        closing_balance,
    })
}

fn parse_balance_field(s: &str) -> Result<Balance, String> {
    if s.len() < 1 + 6 + 3 {
        return Err(format!("balance too short: `{s}`"));
    }
    let sign = &s[0..1];
    let credit = match sign {
        "C" => true,
        "D" => false,
        _ => return Err(format!("unknown balance sign `{sign}`")),
    };

    let date = &s[1..1 + 6];
    let currency = &s[1 + 6..1 + 6 + 3];
    let amount_str = &s[1 + 6 + 3..];

    Ok(Balance {
        credit,
        date_yyymmdd: date.to_string(),
        currency: currency.to_string(),
        amount: amount_str.to_string(),
    })
}

fn parse_transaction_61(s: &str) -> Result<(Transaction, ()), String> {
    if s.len() < 6 + 4 + 1 + 1 {
        return Err(format!(":61: too short: `{s}`"));
    }

    let date = &s[0..6];
    let entry = &s[6..10];

    let sign = &s[10..11];
    let is_credit = match sign {
        "C" => true,
        "D" => false,
        _ => return Err(format!(":61: bad sign `{sign}`")),
    };

    let mut i = 11;
    let bytes = s.as_bytes();
    while i < s.len() && (bytes[i].is_ascii_digit() || bytes[i] == b',' || bytes[i] == b'.') {
        i += 1;
    }

    if i == 11 {
        return Err(format!(":61: amount missing in `{s}`"));
    }

    let amount_str = &s[11..i];
    let start_code = i;

    while i < s.len() && bytes[i].is_ascii_alphabetic() {
        i += 1;
    }

    if i == start_code {
        return Err(format!(":61: type code missing in `{s}`"));
    }

    let type_code = &s[start_code..i];
    let reference = s[i..].to_string();

    Ok((
        Transaction {
            date_yyymmdd: date.to_string(),
            entry_mmdd: entry.to_string(),
            is_credit,
            amount:amount_str.to_string(),
            type_code: type_code.to_string(),
            reference,
            description: String::new(),
        },
        (),
    ))
}


impl From<bool> for DebitCredit {
    fn from(is_credit: bool) -> Self {
        if is_credit { DebitCredit::Credit } else { DebitCredit::Debit }
    }
}

impl From<(&Transaction, &str)> for Entry {
    fn from((tx, currency): (&Transaction, &str)) -> Self {
        Entry {
            booking_date: compose_booking_date(&tx.date_yyymmdd, &tx.entry_mmdd),
            value_date: tx.date_yyymmdd.clone(),
            amount: format!("{:.2}", tx.amount),
            currency: currency.to_string(),
            kind: DebitCredit::from(tx.is_credit),
            description: tx.description.clone(),
            reference: Some(tx.reference.clone()),
        }
    }
}

impl From<&MT940Statement> for Statement {
    fn from(s: &MT940Statement) -> Self {
        let currency = s.opening_balance.currency.clone();
        let entries = s
            .transactions
            .iter()
            .map(|tx| Entry::from((tx, currency.as_str())))
            .collect();

        Statement {
            id: s.reference.clone(),
            account_id: s.account_id.clone(),
            opening_balance: Some(StBalance{
                kind: DebitCredit::Debit,
                date_yyymmdd: s.opening_balance.date_yyymmdd.clone(),
                currency: s.opening_balance.currency.clone(),
                amount: s.opening_balance.amount.clone(),
            }),
            closing_balance: Some(StBalance{
                kind: DebitCredit::Debit,
                date_yyymmdd: s.closing_balance.date_yyymmdd.clone(),
                currency: s.closing_balance.currency.clone(),
                amount: s.closing_balance.amount.clone(),
            }),
            entries,
        }
    }
}

impl From<MT940Statement> for Statement {
    fn from(s: MT940Statement) -> Self {
        Statement::from(&s)
    }
}

fn compose_booking_date(value_date_yyyymmdd: &str, entry_mmdd: &str) -> String {
    let year = &value_date_yyyymmdd[0..4];
    format!("{year}{entry_mmdd}")
}