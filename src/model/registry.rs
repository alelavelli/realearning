use super::{account::Account, transaction::TransactionEvent};
use csv;
use polars::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt,
    fs::{File, OpenOptions},
    io::{self, Cursor},
    ops::Add,
};

/// Registry that contains a set of transactions
#[derive(Serialize)]
pub struct Registry {
    transactions: Vec<TransactionEvent>,
    accounts: HashMap<String, Account>,
}

impl Registry {
    pub fn new(accounts: Option<Vec<Account>>) -> Registry {
        let accounts = accounts.unwrap_or_default();

        let mut accounts_hm: HashMap<String, Account> = HashMap::new();
        for account in accounts {
            accounts_hm.insert(account.name.to_string(), account);
        }

        Registry {
            transactions: Vec::new(),
            accounts: accounts_hm,
        }
    }

    /// Add a transaction to the registry
    ///
    /// If the account of the transaction is not already present then it is added
    /// to the account list. If the account already exists then its value is updated
    pub fn add_single(&mut self, transaction: TransactionEvent) {
        if let std::collections::hash_map::Entry::Vacant(e) =
            self.accounts.entry(transaction.account.to_string())
        {
            e.insert(Account::new(
                transaction.account.clone(),
                transaction.amount,
                transaction.date,
            ));
        } else {
            let account = self
                .accounts
                .get_mut(&transaction.account.to_string())
                .unwrap();
            account.set_value(account.current_value + transaction.amount, transaction.date)
        }
        self.transactions.push(transaction);
    }

    /// Add a batch of transactions to the registry
    pub fn add_batch(&mut self, transactions: Vec<TransactionEvent>) {
        let mut transactions = transactions;
        transactions.sort_by_key(|t| t.date);
        for transaction in transactions {
            self.add_single(transaction);
        }
    }

    pub fn get_accounts(&self) -> Vec<String> {
        self.accounts.keys().map(|x| (*x).clone()).collect()
    }

    pub fn get_initial_account_values(&self, accounts: Option<&Vec<String>>) -> f32 {
        let mut value: f32 = 0.;
        let mut accounts_to_use = &self.get_accounts();
        if let Some(required_accounts) = accounts {
            accounts_to_use = required_accounts;
        }

        for account_name in accounts_to_use {
            value += self
                .accounts
                .get(account_name)
                .map_or_else(|| 0.0f32, |a| a.get_initial_value());
        }
        value
    }

    /// Export TranactionEvent to Polars DataFrame
    ///
    /// First, it serializes it as a JSON string, then
    /// it uses the Polars JsonReader to create the DataFrame
    pub fn to_dataframe(&self) -> Result<DataFrame, PolarsError> {
        let myschema = Schema::from(
            vec![
                Field::new("date", DataType::Float32),
                Field::new("amount", DataType::Float32),
                Field::new("category", DataType::Categorical(None)),
                Field::new("description", DataType::Utf8),
                Field::new("account", DataType::Categorical(None)),
            ]
            .into_iter(),
        );
        let df = JsonReader::new(Cursor::new(
            serde_json::to_string(&self.transactions)
                .expect("Transitions should be able to json serialize"),
        ))
        .with_schema(&myschema)
        .finish()?;
        df
            .lazy()
            .with_column(col("date").str().strptime(StrpTimeOptions {
                date_dtype: DataType::Date,
                fmt: Some("%Y-%m-%d".into()),
                strict: false,
                exact: true,
                cache: true,
                tz_aware: false,
                utc: false,
            }))
            .collect()
    }

    /// Build a regstry from a dumped csv
    pub fn from_csv(path: &str) -> Result<Registry, io::Error> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut registry = Registry::new(None);
        for result in rdr.deserialize() {
            let transaction: TransactionEvent = result?;
            registry.add_single(transaction);
        }
        Ok(registry)
    }

    /// Dumps the registry as csv
    pub fn to_csv(&self, path: &str) -> Result<(), io::Error> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .expect("Error in opening the file");

        let mut wtr = csv::Writer::from_writer(file);
        for transaction in &self.transactions {
            wtr.serialize(transaction)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

impl fmt::Display for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The registry has {} accounts:\n\n", self.accounts.len())?;
        for (name, account) in &self.accounts {
            writeln!(f, "\t> {}:\t{}€", name, account.current_value)?;
        }
        let transaction_len = self.transactions.len();

        if transaction_len > 0 {
            let num_last_transactions = match transaction_len {
                1..=5 => transaction_len,
                _ => transaction_len - 5,
            };
            write!(
                f,
                "\n\nThere are {} transactions in the registry:\n\n",
                transaction_len
            )?;
            for transaction in &self.transactions[num_last_transactions..] {
                writeln!(f, "\t- {}", transaction)?
            }
        }
        Ok(())
    }
}

impl Add for Registry {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut transactions: Vec<TransactionEvent> = Vec::new();
        transactions.extend(self.transactions);
        transactions.extend(other.transactions);

        // We use a hashmap to store the references of the accounts inside the
        // two registries. Then we use it to merge the accouts to get the final
        // version of the registry to return
        let mut accounts_to_insert: HashMap<String, Account> = HashMap::new();
        for (name, self_account) in self.accounts {
            accounts_to_insert.insert(name, self_account);
        }
        for (name, other_account) in other.accounts {
            let self_entry_ref: Option<Account> = accounts_to_insert.remove(&name);
            match self_entry_ref {
                Some(r) => accounts_to_insert.insert(name, r + other_account),
                None => accounts_to_insert.insert(name, other_account),
            };
        }

        Registry {
            accounts: accounts_to_insert,
            transactions,
        }
    }
}
