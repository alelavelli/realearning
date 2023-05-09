use crate::model::account::{Account, TransactionAccountName};
use crate::model::registry::Registry;
use crate::model::transaction::{TransactionCategory, TransactionEvent};
use calamine::{open_workbook, DataType, Range, Reader, Xlsx};
use chrono::NaiveDate;
use indicatif::{MultiProgress, ProgressBar, ProgressIterator, ProgressStyle};
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;

use super::compatibility_errors::ExtractionError;

/// Build a registry from a excel file composed of many sheets
///
/// # Arguments
///
/// * `path`: a string slice with the path of the excel file
/// * `worksheet_template`: the regular expression that defines valid worksheets
///
/// # Return
///
/// It returns a Tuple with two entries:
/// * `Registry`: the extracted registry
/// * `Vec<String>`: vector containing failed registry extractions
pub fn build_registry_batch(
    path: &str,
    worksheet_template: Regex,
) -> Result<(Registry, Vec<String>), Box<dyn std::error::Error>> {
    let workbook: Xlsx<_> = open_workbook(path)?;
    let mut sheet_names = workbook.sheet_names().to_vec();
    // We sort the sheet names to keep the registries ordered by time
    sheet_names.sort();

    let multi_progress = MultiProgress::new();
    let progress_bar = multi_progress.add(ProgressBar::new(sheet_names.len() as u64));

    // create the two resulting structures that will be filled during the for loop
    let mut failed_extractions: Vec<String> = Vec::new();
    let mut result_registry = Registry::new(None);

    // for loop that extract each registry at a time
    for worksheet in sheet_names.iter().progress_with(progress_bar) {
        if worksheet_template.is_match(worksheet) {
            result_registry = match build_registry(path, worksheet, &multi_progress) {
                Ok(new_registry) => result_registry + new_registry,
                Err(_) => {
                    failed_extractions.push(worksheet.clone());
                    result_registry
                }
            };
        }
    }
    Ok((result_registry, failed_extractions))
}

/// Build the Registry strut from the excel file.
///
/// First of all loads the excel, then extracts from the first row
/// the names and the indexes of the columns. Then iterates for each
/// row to create TransactionEvent by adding it to the Registry.
///
/// # Parameters
///
/// * `path`: path of the excel file
/// * `worksheet`: name of the worksheet file
/// * `multi_progress`: MultiProgress struct used to plot the progress bar
///
/// # Returns
///
/// * `Registry`: the extracted registry from the worksheet
pub fn build_registry(
    path: &str,
    worksheet: &str,
    multi_progress: &MultiProgress,
) -> Result<Registry, Box<dyn std::error::Error>> {
    let mut spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_secs(1));
    spinner.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner = multi_progress.add(spinner);

    spinner.set_message(format!("Extracting {worksheet}"));

    let mut workbook: Xlsx<_> = open_workbook(path).unwrap();
    let range = workbook.worksheet_range(worksheet).unwrap()?;

    let transactions = retrieve_transactions(&range)?;
    let accounts = retrieve_accounts(worksheet, &range)?;

    let mut registry = Registry::new(Some(accounts));
    registry.add_batch(transactions);

    spinner.finish_with_message(format!("{worksheet} done"));
    Ok(registry)
}

/// Retrieve transactions from the worksheet
///
/// The first row contains the columns and the iteration gets their positions.
/// Then, the second iteration retreive the data from each row and creates transaction
/// structs.
///
/// # Parameters
///
/// * `range`: calamine::Range that represents a set of rows in the worksheet
///
/// # Returns
///
/// * Vector of transaction events extracted from the worksheet
fn retrieve_transactions(
    range: &Range<DataType>,
) -> Result<Vec<TransactionEvent>, ExtractionError> {
    let mut transactions: Vec<TransactionEvent> = Vec::new();
    let mut columns_positions: HashMap<String, usize> = HashMap::new();

    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            // The first row is the header, then we extract the names of the columns
            let row_iterator = row.iter();

            for (col_index, cell) in row_iterator.enumerate() {
                if *cell == DataType::Empty {
                    break;
                }
                columns_positions.insert(cell.to_string(), col_index);
            }
        } else {
            let date = row
                .get(*columns_positions.get("Data").ok_or(ExtractionError)?)
                .ok_or(ExtractionError)?
                .as_date()
                .ok_or(ExtractionError)?;

            let amount = row
                .get(*columns_positions.get("Saldo").ok_or(ExtractionError)?)
                .ok_or(ExtractionError)?
                .get_float()
                .ok_or(ExtractionError)? as f32;

            let category = row
                .get(*columns_positions.get("Categoria").ok_or(ExtractionError)?)
                .ok_or(ExtractionError)?
                .get_string()
                .ok_or(ExtractionError)?;

            let description = row
                .get(*columns_positions.get("Nota").ok_or(ExtractionError)?)
                .ok_or(ExtractionError)?
                .get_string()
                .map(String::from);

            let account = row
                .get(*columns_positions.get("Conto").ok_or(ExtractionError)?)
                .ok_or(ExtractionError)?
                .get_string()
                .ok_or(ExtractionError)?;

            let transaction = TransactionEvent::new(
                date,
                amount,
                match TransactionCategory::from_str(category) {
                    Ok(c) => c,
                    Err(_) => return Err(ExtractionError),
                },
                description,
                match TransactionAccountName::from_str(account) {
                    Ok(d) => d,
                    Err(_) => return Err(ExtractionError),
                },
            );
            transactions.push(transaction);
        }
    }
    Ok(transactions)
}

/// Retrieve accounts from the worksheet
///
/// The accounts information are stored in a table near the the transaction one.
/// Therefore, the column definition step needs to go beyond the first set of non empty columns.
///
/// # Parameters
///
/// * `worksheet`: name of the worksheet
/// * `range`: calamine::Range with the rows in the worksheet
///
/// # Returns
///
/// * Vector with accounts
fn retrieve_accounts(
    worksheet: &str,
    range: &Range<DataType>,
) -> Result<Vec<Account>, ExtractionError> {
    let mut date_str = String::from(worksheet);
    date_str.push_str("-01");
    let date = match NaiveDate::from_str(&date_str) {
        Ok(d) => d,
        Err(_) => return Err(ExtractionError),
    };

    let mut accounts: Vec<Account> = Vec::new();

    // This variables encodes if during the retrival of columns we are in the first or second block of data
    let mut in_second_block = false;
    let mut columns_positions: HashMap<String, usize> = HashMap::new();

    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            // The first row is the header, then we extract the names of the columns
            let row_iterator = row.iter();

            for (col_index, cell) in row_iterator.enumerate() {
                let empty_cell = *cell == DataType::Empty;
                if empty_cell {
                    in_second_block = true;
                }

                if in_second_block & !empty_cell {
                    columns_positions.insert(cell.to_string(), col_index);
                }
            }
        } else {
            // If we get empty column corresponding to Conti corrente then we stop the iteration
            let cell = row
                .get(
                    *columns_positions
                        .get("Conti corrente")
                        .ok_or(ExtractionError)?,
                )
                .ok_or(ExtractionError)?;
            if *cell == DataType::Empty {
                break;
            }

            let account_name = match TransactionAccountName::from_str(
                &row.get(
                    *columns_positions
                        .get("Conti corrente")
                        .ok_or(ExtractionError)?,
                )
                .ok_or(ExtractionError)?
                .to_string(),
            ) {
                Ok(a) => a,
                Err(_) => return Err(ExtractionError),
            };

            let saldo_iniziale = row
                .get(
                    *columns_positions
                        .get("Saldo iniziale")
                        .ok_or(ExtractionError)?,
                )
                .ok_or(ExtractionError)?
                .get_float()
                .ok_or(ExtractionError)? as f32;

            let account = Account::new(account_name, saldo_iniziale, date);
            accounts.push(account);
        }
    }
    Ok(accounts)
}
