use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use strum_macros::{Display, EnumString};

/// TransactionSource enum with possible account of transactions.
#[derive(EnumString, Display, Serialize, Deserialize, PartialEq, Clone)]
pub enum TransactionAccountName {
    #[strum(ascii_case_insensitive)]
    Ale,
    #[strum(serialize = "buono pasto", ascii_case_insensitive)]
    BuonoPasto,
    #[strum(serialize = "carta ale", ascii_case_insensitive)]
    CartaAle,
    #[strum(serialize = "carta giulia", ascii_case_insensitive)]
    CartaGiulia,
    #[strum(ascii_case_insensitive)]
    Contante,
    #[strum(ascii_case_insensitive)]
    Giulia,
}

#[derive(Serialize)]
pub struct Account {
    pub name: TransactionAccountName,
    pub current_value: f32,
    history: Vec<(NaiveDate, f32)>,
}

impl Account {
    pub fn new(name: TransactionAccountName, value: f32, date: NaiveDate) -> Account {
        Account {
            name,
            current_value: value,
            history: vec![(date, value)],
        }
    }

    pub fn set_value(&mut self, new_value: f32, date: NaiveDate) {
        self.history.push((date, new_value));
        self.current_value = new_value;
    }

    pub fn get_initial_value(&self) -> f32 {
        self.history.iter().min_by_key(|&(date, _)| date).unwrap().1
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Add for Account {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.name != other.name {
            panic!("The accounts must have the same name!");
        } else {
            // Merge the history of the two accounts and take as current value the newest one
            let mut new_history: Vec<(NaiveDate, f32)> = Vec::new();
            new_history.extend(self.history);
            new_history.extend(other.history);

            let current_value = new_history.iter().max_by_key(|&(date, _)| date).unwrap().1;
            Account {
                name: self.name,
                current_value,
                history: new_history,
            }
        }
    }
}
