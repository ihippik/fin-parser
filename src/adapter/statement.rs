use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DebitCredit {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub booking_date: String,
    pub value_date: String,
    pub amount: String,
    pub currency: String,
    pub kind: DebitCredit,
    pub description: String,
    pub reference: Option<String>,
}

#[derive(Debug,Clone,PartialEq)]
pub struct Balance {
    pub kind: DebitCredit,
    pub date_yyymmdd: String,
    pub currency: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub id: String,
    pub account_id: String,
    pub opening_balance: Option<Balance>,
    pub entries: Vec<Entry>,
    pub closing_balance: Option<Balance>,
}