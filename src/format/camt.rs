use std::io::{BufRead, Write};
use crate::adapter::adapter::{Adapter, Statement};
use crate::adapter::errors::AdapterError;
use serde::{Deserialize, Serialize};
use crate::adapter::statement::{DebitCredit,Balance,Entry};
use quick_xml::{de::from_reader, se::Serializer};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "entry")]
struct XmlEntry {
    booking_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_date: Option<String>,
    amount: String,
    currency: String,
    dc: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct XmlBalance {
    date: String,
    amount: String,
    currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Entries {
    #[serde(rename = "entry")]
    items: Vec<XmlEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct XmlStatement {
    #[serde(skip_serializing_if = "Option::is_none")]
    statement_id: Option<String>,
    account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    opening_balance: Option<XmlBalance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closing_balance: Option<XmlBalance>,
    #[serde(rename = "entries")]
    entries: Entries,
}

pub struct FormatXML;

impl Adapter for FormatXML {
    fn read_from<R: BufRead>(reader: R) -> Result<Statement, AdapterError> {
        let x: XmlStatement = from_reader(reader).map_err(|e| AdapterError::ParseError(e.to_string()))?;
        let opening: Option<Balance> = x.opening_balance.map(parse_xml_balance).transpose()?;
        let closing: Option<Balance> = x.closing_balance.map(parse_xml_balance).transpose()?;
        let mut entries = Vec::with_capacity(x.entries.items.len());

        for e in x.entries.items {
            let booking_date = e.booking_date;
            let value_date = e.value_date;
            let amount = e.amount;

            entries.push(Entry {
                booking_date,
                value_date: value_date.unwrap_or("undefined".to_string()),
                amount,
                currency: e.currency,
                description: e.description,
                reference: e.reference,
                kind: DebitCredit::Debit,
            });
        }

        Ok(Statement {
            id: x.statement_id.unwrap(),
            account_id: x.account_id,
            opening_balance: opening,
            closing_balance: closing,
            entries,
        })
    }

    fn write_to<W: Write>(mut writer: W, st: &Statement) -> Result<(), AdapterError> {
        let opening = st.opening_balance.as_ref().map(|b| XmlBalance {
            date: b.date_yyymmdd.clone(),
            amount: b.amount.clone(),
            currency: b.currency.clone(),
        });

        let closing = st.closing_balance.as_ref().map(|b| XmlBalance {
            date: b.date_yyymmdd.clone(),
            amount: b.amount.clone(),
            currency: b.currency.clone(),
        });

        let items: Vec<XmlEntry> = st.entries.iter().map(|e| XmlEntry {
            booking_date: e.booking_date.clone(),
            value_date: Some(e.value_date.clone()),
            amount: e.amount.clone(),
            currency: e.currency.clone(),
            dc: match e.kind { DebitCredit::Debit => "D".into(), DebitCredit::Credit => "C".into() },
            description: e.description.clone(),
            reference: e.reference.clone(),
        }).collect();

        let x = XmlStatement {
            statement_id: Some(st.id.clone()),
            account_id: st.account_id.clone(),
            opening_balance: opening,
            closing_balance: closing,
            entries: Entries { items },
        };

        let raw = to_pretty_xml(&x).map_err(|e| AdapterError::ParseError(format!("{e}")))?;
        writer.write_all(raw.as_bytes()).map_err(|e|AdapterError::WriteError(e.to_string()))
    }
}

fn to_pretty_xml<T: Serialize>(value: &T) -> Result<String, quick_xml::Error> {
    let mut out = String::new();
    let mut ser = Serializer::new(&mut out);
    ser.indent(' ', 4);
    value.serialize(ser).unwrap();
    Ok(out)
}

fn parse_dc(s: &str) -> Result<DebitCredit,AdapterError> {
    match s {
        "D" => Ok(DebitCredit::Debit),
        "C" => Ok(DebitCredit::Credit),
        _ => Err(AdapterError::ParseError(format!("wrong payment type: {s}"))),
    }
}

fn parse_xml_balance(b: XmlBalance) -> Result<Balance, AdapterError> {
    Ok(Balance {
        kind: DebitCredit::Debit,
        date_yyymmdd: b.date,
        amount: b.amount,
        currency: b.currency,
    })
}

