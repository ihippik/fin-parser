use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct FinanceRecord {
    pub posting_date: String,     // [01]
    pub account_debit: String,       // [04]
    pub account_credit: String,      // [08]
    pub debit_amount: Option<f64>,   // [09]
    pub credit_amount: Option<f64>,  // [13]
    pub doc_number: Option<String>,  // [14]
    pub operation_code: Option<String>, // [16] (ВО)
    pub bank_info: Option<String>,   // [17]
    pub purpose: Option<String>,     // [20]
}