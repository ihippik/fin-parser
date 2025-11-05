use serde::{Deserialize, Serialize};

/// Indicates the type of transaction: debit (outflow) or credit (inflow).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DebitCredit {
    /// A debit transaction (money out).
    Debit,
    /// A credit transaction (money in).
    Credit,
}

/// Represents a single transaction entry within a financial statement.
///
/// Each entry includes booking and value dates, amount, currency, and
/// whether it is a debit or credit. It may also include an optional
/// reference and a human-readable description.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    /// Date when the transaction was booked (format: YYYY-MM-DD).
    pub booking_date: String,
    /// Date when the transaction value takes effect (format: YYYY-MM-DD).
    pub value_date: String,
    /// Transaction amount as a string (to preserve exact formatting).
    pub amount: String,
    /// Currency code (e.g. "EUR", "USD").
    pub currency: String,
    /// Whether this is a debit or credit transaction.
    pub kind: DebitCredit,
    /// Description or purpose of the transaction.
    pub description: String,
    /// Optional reference or identifier provided by the bank.
    pub reference: Option<String>,
}

/// Represents an account balance at a specific date.
///
/// Used for both opening and closing balances in a statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Balance {
    /// Indicates whether the balance is debit or credit.
    pub kind: DebitCredit,
    /// Balance date in YYYY-MM-DD format.
    pub date_yyymmdd: String,
    /// Currency code (e.g. "EUR", "USD").
    pub currency: String,
    /// Account balance amount as a string.
    pub amount: String,
}

/// Represents a full financial statement (e.g. one MT940 message).
///
/// Contains metadata such as statement ID and account ID, as well as
/// the list of entries and optional opening and closing balances.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// Unique identifier of the statement (e.g. `:20:` field in MT940).
    pub id: String,
    /// Account identifier (e.g. IBAN or account number).
    pub account_id: String,
    /// Opening balance (e.g. MT940 `:60F:` field).
    pub opening_balance: Option<Balance>,
    /// List of transaction entries in this statement.
    pub entries: Vec<Entry>,
    /// Closing balance (e.g. MT940 `:62F:` field).
    pub closing_balance: Option<Balance>,
}
