use chrono::NaiveDate;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self},
    io::Cursor,
};
use strum_macros::{Display, EnumString};

use super::account::TransactionAccountName;

/// TransactionCategory enumeration contains
/// the categories a transaction event can belong to.
#[derive(EnumString, Display, Serialize, Deserialize)]
pub enum TransactionCategory {
    #[strum(ascii_case_insensitive)]
    Affitto,
    #[strum(ascii_case_insensitive)]
    Auto,
    #[strum(ascii_case_insensitive)]
    Banca,
    #[strum(ascii_case_insensitive)]
    Bolletta,
    #[strum(serialize = "carta di credito", ascii_case_insensitive)]
    CartaDiCredito,
    #[strum(ascii_case_insensitive)]
    Pasto,
    #[strum(serialize = "pranzo lavoro", ascii_case_insensitive)]
    PranzoLavoro,
    #[strum(serialize = "rata auto", ascii_case_insensitive)]
    RataAuto,
    #[strum(ascii_case_insensitive)]
    Regalo,
    #[strum(serialize = "ritiro bancomat", ascii_case_insensitive)]
    RitiroBancomat,
    #[strum(serialize = "sanità", ascii_case_insensitive)]
    Sanita,
    #[strum(ascii_case_insensitive)]
    Scarpe,
    #[strum(ascii_case_insensitive)]
    Spesa,
    #[strum(ascii_case_insensitive)]
    Stipendio,
    #[strum(ascii_case_insensitive)]
    Telefono,
    #[strum(ascii_case_insensitive)]
    Treno,
    #[strum(ascii_case_insensitive)]
    Uscite,
    #[strum(ascii_case_insensitive)]
    Varie,
    #[strum(ascii_case_insensitive)]
    Vestiti,
    #[strum(ascii_case_insensitive)]
    Vista,
    #[strum(ascii_case_insensitive)]
    Vacanza,
}

/// TransactionEvent struct that define a transaction.
///
/// A transaction is composed of:
/// - **date**: when the transaction occurred
/// - **amount**: quantity in euros of the transaction. It can be either positive or negative
/// - **category**: type of transaction
/// - **description**: optional description of the transaction
/// - **source**: source of the transaction
#[derive(Serialize, Deserialize)]
pub struct TransactionEvent {
    pub date: NaiveDate,
    pub amount: f32,
    pub category: TransactionCategory,
    pub description: Option<String>,
    pub account: TransactionAccountName,
}

impl TransactionEvent {
    pub fn new(
        date: NaiveDate,
        amount: f32,
        category: TransactionCategory,
        description: Option<String>,
        account: TransactionAccountName,
    ) -> TransactionEvent {
        TransactionEvent {
            date,
            amount,
            category,
            description,
            account,
        }
    }

    /// Export TranactionEvent to Polars DataFrame
    ///
    /// First, it serializes it as a JSON string, then
    /// it uses the Polars JsonReader to create the DataFrame
    pub fn to_dataframe(&self) -> DataFrame {
        JsonReader::new(Cursor::new(
            serde_json::to_string(&[&self]).expect("Transition should be able to json serialize"),
        ))
        .finish()
        .expect("Failed to export TransactionEvent to DataFrame")
    }
}

impl fmt::Display for TransactionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction on date {} of category {}, amount: {}€, account: {}, description: {}",
            self.date,
            self.category,
            self.amount,
            self.account,
            match &self.description {
                Some(s) => s,
                None => "missing",
            }
        )
    }
}
