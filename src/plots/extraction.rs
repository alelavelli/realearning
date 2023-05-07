//! # Extraction
//!
//! `extraction` is a colletion of utilities to extract information from a registry to make report plots
//!
use crate::model::registry::Registry;
use chrono::NaiveDate;
use itertools::Itertools;
use polars::lazy::dsl::col;
use polars::prelude::*;
use std::{cmp::Ordering::Equal, collections::HashMap};

pub struct DailyTransactions {
    pub days: Vec<NaiveDate>,
    pub amounts: Vec<f32>,
    pub cumsum_amounts: Vec<f32>,
    pub days_idx: Vec<f32>,
    pub days_idx_range: (f32, f32),
    pub amounts_range: (f32, f32),
    pub cumsum_amounts_range: (f32, f32),
    pub amounts_pairs: Vec<(f32, f32)>,
    pub amount_cumulative_pairs: Vec<(f32, f32)>,
}

pub struct CategoriesSplit {
    pub income_categories: Vec<String>,
    pub income_percentages: Vec<f64>,
    pub income_amounts: Vec<f64>,
    pub expense_categories: Vec<String>,
    pub expense_percentages: Vec<f64>,
    pub expense_amounts: Vec<f64>,
}

pub struct MonthlyTransactions {
    pub months: Vec<NaiveDate>,
    pub net_income: Vec<f32>,
    pub months_idx: Vec<f32>,
    pub months_idx_range: (f32, f32),
    pub net_income_range: (f32, f32),
    pub net_income_pairs: Vec<(f32, f32)>,
    pub categories: Vec<String>,
    pub categories_amounts: Vec<Vec<f32>>,
    pub categories_months: Vec<Vec<NaiveDate>>,
    pub categories_months_idx: Vec<Vec<f32>>,
    pub categories_amounts_range: (f32, f32),
    pub categories_months_idx_range: (f32, f32),
    pub categories_pairs: Vec<Vec<(f32, f32)>>,
    pub categories_amounts_perc: Vec<Vec<f64>>,
    pub categories_amounts_perc_value: Vec<Vec<f64>>,
    pub categories_amounts_perc_months: Vec<String>,
    pub categories_amounts_perc_names: Vec<Vec<String>>,
}

/// filter_registry returns registry as dataframe with applied filters
///
/// ## Parameters
///
/// `registry`: Registry struct
/// `accounts`: Optional parameter with a filter of the accounts to consider
/// `date_range`: Optional parameter with a filter over the dates to consider
fn filter_registry_df(
    registry: &Registry,
    accounts: Option<&Vec<String>>,
    date_range: Option<(&NaiveDate, &NaiveDate)>,
) -> Result<DataFrame, PolarsError> {
    let mut df = registry.to_dataframe()?.lazy();

    if let Some(vector) = accounts {
        let accounts = Series::new("account_list", vector);
        df = df.filter(col("account").is_in(lit(accounts)));
    }

    if let Some((from, to)) = date_range {
        df = df.filter(
            col("date")
                .dt()
                .strftime("%Y-%m-%d")
                .gt_eq(lit(&from.to_string()[..])),
        );
        df = df.filter(
            col("date")
                .dt()
                .strftime("%Y-%m-%d")
                .lt_eq(lit(&to.to_string()[..])),
        );
    }
    let df = df.collect()?;

    Ok(df)
}

/// extract_daily_transaction returns a tuple with two elements: a vector of dates
/// and a vector of floats representing the amount
///
/// ## Parameters
///
/// `registry`: Registry struct
/// `accounts`: Optional parameter with a filter of the accounts to consider
/// `date_range`: Optional parameter with a filter over the dates to consider
/// `with_initial_total_value`: bool, if true the initial value of the accouts
/// in the registry cumulative amounts is added to the cumulative sum accounts
pub fn extract_daily_transactions(
    registry: &Registry,
    accounts: Option<&Vec<String>>,
    date_range: Option<(&NaiveDate, &NaiveDate)>,
    with_initial_total_value: bool,
) -> Result<DailyTransactions, Box<dyn std::error::Error>> {
    let mut initial_total_value: f32 = 0.0;
    if with_initial_total_value {
        initial_total_value = registry.get_initial_account_values(accounts);
    }

    let df = filter_registry_df(registry, accounts, date_range)?;
    let df = df
        .lazy()
        .groupby(["date"])
        .agg([col("amount").sum()])
        .sort(
            "date",
            SortOptions {
                descending: false,
                nulls_last: true,
                multithreaded: true,
            },
        )
        .with_column(col("amount").cumsum(false).alias("amount_cumsum"))
        .collect()?;

    let days: Vec<NaiveDate> = df
        .column("date")
        .unwrap()
        .date()
        .unwrap()
        .as_date_iter()
        .map(|x| x.unwrap())
        .collect();
    let amounts: Vec<f32> = df
        .column("amount")
        .unwrap()
        .f64()
        .unwrap()
        .to_vec()
        .iter()
        .map(|x| x.unwrap() as f32)
        .collect();
    let cumsum_amounts: Vec<f32> = df
        .column("amount_cumsum")
        .unwrap()
        .f64()
        .unwrap()
        .to_vec()
        .iter()
        .map(|x| (x.unwrap() as f32) + initial_total_value)
        .collect();

    let days_idx: Vec<f32> = (0u8..=days.len() as u8).map(f32::from).collect();

    // We take min and max to create plot boundaries
    let x_min = *days_idx
        .iter()
        .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
        .unwrap();
    let x_max = *days_idx
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
        .unwrap();
    let y_min = *amounts
        .iter()
        .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
        .unwrap();
    let y_max = *amounts
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
        .unwrap();
    let cumulative_y_min = *cumsum_amounts
        .iter()
        .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
        .unwrap();
    let cumulative_y_max = *cumsum_amounts
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
        .unwrap();

    let amounts_pairs: Vec<(f32, f32)> =
        days_idx.clone().into_iter().zip(amounts.clone()).collect();
    let amount_cumulative_pairs: Vec<(f32, f32)> = days_idx
        .clone()
        .into_iter()
        .zip(cumsum_amounts.clone())
        .collect();

    Ok(DailyTransactions {
        days,
        amounts,
        cumsum_amounts,
        days_idx,
        days_idx_range: (x_min, x_max),
        amounts_range: (y_min, y_max),
        cumsum_amounts_range: (cumulative_y_min, cumulative_y_max),
        amounts_pairs,
        amount_cumulative_pairs,
    })
}

pub fn extract_categories_split(
    registry: &Registry,
    accounts: Option<&Vec<String>>,
    date_range: Option<(&NaiveDate, &NaiveDate)>,
    max_categories: Option<usize>,
) -> Result<CategoriesSplit, Box<dyn std::error::Error>> {
    let df = filter_registry_df(registry, accounts, date_range)?;

    let mut incomes = df
        .clone()
        .lazy()
        .filter(col("amount").gt(0.0))
        .groupby(["category"])
        .agg([col("amount").sum()])
        .sort(
            "amount",
            SortOptions {
                descending: false,
                nulls_last: true,
                multithreaded: true,
            },
        )
        .with_column((col("amount") / col("amount").sum() * lit(100.0)).alias("amount_perc"))
        .collect()?;

    let mut expenses = df
        .lazy()
        .filter(col("amount").lt(0.0))
        .groupby(["category"])
        .agg([col("amount").sum()])
        .sort(
            "amount",
            SortOptions {
                descending: false,
                nulls_last: true,
                multithreaded: true,
            },
        )
        .with_column((col("amount") / col("amount").sum() * lit(100.0)).alias("amount_perc"))
        .collect()?;

    if let Some(num) = max_categories {
        incomes = incomes.head(Some(num));
        expenses = expenses.head(Some(num))
    }

    Ok(CategoriesSplit {
        income_categories: incomes
            .column("category")
            .unwrap()
            .iter()
            .map(|x| x.to_owned().to_string().replace('\"', ""))
            .collect(),
        income_percentages: incomes
            .column("amount_perc")
            .unwrap()
            .f64()
            .unwrap()
            .to_vec()
            .iter()
            .map(|x| x.unwrap())
            .collect(),
        income_amounts: incomes
            .column("amount")
            .unwrap()
            .f64()
            .unwrap()
            .to_vec()
            .iter()
            .map(|x| x.unwrap())
            .collect(),
        expense_categories: expenses
            .column("category")
            .unwrap()
            .iter()
            .map(|x| x.to_owned().to_string().replace('\"', ""))
            .collect(),
        expense_percentages: expenses
            .column("amount_perc")
            .unwrap()
            .f64()
            .unwrap()
            .to_vec()
            .iter()
            .map(|x| x.unwrap())
            .collect(),
        expense_amounts: expenses
            .column("amount")
            .unwrap()
            .f64()
            .unwrap()
            .to_vec()
            .iter()
            .map(|x| x.unwrap())
            .collect(),
    })
}

pub fn monthy_extraction(
    registry: &Registry,
    accounts: Option<&Vec<String>>,
    date_range: Option<(&NaiveDate, &NaiveDate)>,
    max_categories: Option<usize>,
) -> Result<MonthlyTransactions, Box<dyn std::error::Error>> {
    let df = filter_registry_df(registry, accounts, date_range)?;

    let monthy_net_income = df
        .clone()
        .lazy()
        .with_column(col("date").alias("year-month").dt().truncate("1mo", "1"))
        .groupby(["year-month"])
        .agg([col("amount").sum()])
        //.with_column((col("amount") / col("amount").sum() * lit(100.0)).alias("amount_perc"))
        .sort(
            "year-month",
            SortOptions {
                descending: false,
                nulls_last: true,
                multithreaded: true,
            },
        )
        .collect()
        .unwrap();

    let months: Vec<NaiveDate> = monthy_net_income
        .column("year-month")
        .unwrap()
        .date()
        .unwrap()
        .as_date_iter()
        .map(|x| x.unwrap())
        .collect();
    let months_idx: Vec<f32> = (0u8..months.len() as u8).map(f32::from).collect();
    let months_idx_range = (
        *months_idx
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
            .unwrap(),
        *months_idx
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
            .unwrap(),
    );

    let net_income: Vec<f32> = monthy_net_income
        .column("amount")
        .unwrap()
        .f64()
        .unwrap()
        .to_vec()
        .iter()
        .map(|x| x.unwrap() as f32)
        .collect();
    let net_income_range = (
        *net_income
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
            .unwrap(),
        *net_income
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
            .unwrap(),
    );
    let net_income_pairs: Vec<(f32, f32)> = months_idx
        .clone()
        .into_iter()
        .zip(net_income.clone())
        .collect();

    let expenses_per_category = df
        .lazy()
        .filter(col("amount").lt(0.0))
        .with_column(col("date").alias("year-month").dt().truncate("1mo", "1"))
        .groupby(["year-month", "category"])
        .agg([col("amount").sum()])
        .with_column((col("amount") / col("amount").sum() * lit(100.0)).alias("amount_perc"))
        .sort(
            "year-month",
            SortOptions {
                descending: false,
                nulls_last: true,
                multithreaded: true,
            },
        )
        .collect()
        .unwrap();

    let mut categories: Vec<String> = Vec::new();
    let mut categories_months: Vec<Vec<NaiveDate>> = Vec::new();
    let mut categories_months_idx: Vec<Vec<f32>> = Vec::new();
    let mut categories_amounts: Vec<Vec<f32>> = Vec::new();
    let mut categories_amounts_min: Option<f32> = None;
    let mut categories_amounts_max: Option<f32> = None;
    let mut categories_pairs: Vec<Vec<(f32, f32)>> = Vec::new();
    let categories_months_idx_min: f32 = 0.0;
    let categories_months_idx_max: f32 = months_idx_range.1;
    let mut months_idx_mapping: HashMap<&NaiveDate, f32> = HashMap::new();
    for (i, month) in months.iter().enumerate() {
        months_idx_mapping.insert(month, i as f32);
    }

    for category in expenses_per_category
        .column("category")
        .unwrap()
        .utf8()
        .unwrap()
        .unique()
        .unwrap()
        .into_iter()
        .map(|f| String::from(f.unwrap()))
    {
        let cat_df = expenses_per_category
            .clone()
            .lazy()
            .filter(col("category").eq(lit(&category[..])))
            .collect()
            .unwrap();
        let xs: Vec<NaiveDate> = cat_df
            .column("year-month")
            .unwrap()
            .date()
            .unwrap()
            .as_date_iter()
            .map(|x| x.unwrap())
            .collect();
        let ys: Vec<f32> = cat_df
            .column("amount")
            .unwrap()
            .f64()
            .unwrap()
            .to_vec()
            .iter()
            .map(|x| x.unwrap() as f32)
            .collect();

        categories_amounts_min = match categories_amounts_min {
            Some(v) => {
                let m = ys
                    .iter()
                    .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
                    .unwrap();
                if v > *m {
                    Some(*m)
                } else {
                    Some(v)
                }
            }
            None => Some(
                *ys.iter()
                    .min_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
                    .unwrap(),
            ),
        };
        categories_amounts_max = match categories_amounts_max {
            Some(v) => {
                let m = ys
                    .iter()
                    .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
                    .unwrap();
                if v < *m {
                    Some(*m)
                } else {
                    Some(v)
                }
            }
            None => Some(
                *ys.iter()
                    .max_by(|x, y| x.partial_cmp(y).unwrap_or(Equal))
                    .unwrap(),
            ),
        };
        let xs_idx_local: Vec<f32> = xs
            .clone()
            .iter()
            .map(|x| *months_idx_mapping.get(x).unwrap())
            .collect();
        categories.push(category);
        categories_months_idx.push(xs_idx_local.clone());
        categories_months.push(xs);
        categories_amounts.push(ys.clone());
        categories_pairs.push(xs_idx_local.into_iter().zip(ys.clone()).collect());
    }

    let mut categories_amounts_perc: Vec<Vec<f64>> = Vec::new();
    let mut categories_amounts_perc_value: Vec<Vec<f64>> = Vec::new();
    let mut categories_amounts_perc_months: Vec<String> = Vec::new();
    let mut categories_amounts_perc_names: Vec<Vec<String>> = Vec::new();

    for month in months.clone().into_iter().unique() {
        //expenses_per_category.column("year-month").unwrap().date().unwrap().unique().unwrap().cast(&DataType::Utf8).unwrap().utf8().unwrap().into_iter().map(|x| x.unwrap()) {
        let mut month_df = expenses_per_category
            .clone()
            .lazy()
            //.filter(col("category").is_in(lit(Series::new("categories", categories.clone()))))
            .filter(
                col("year-month")
                    .dt()
                    .strftime("%Y-%m-%d")
                    .eq(lit(&month.to_string()[..])),
            )
            .sort(
                "amount_perc",
                SortOptions {
                    descending: true,
                    nulls_last: true,
                    multithreaded: true,
                },
            )
            .collect()
            .unwrap();
        if max_categories.is_some() {
            month_df = month_df.head(max_categories);
        }

        let percs: Vec<f64> = month_df
            .column("amount_perc")
            .unwrap()
            .f64()
            .unwrap()
            //.to_vec().iter().map(|x| x.unwrap().abs().log(10.0) as f32) logarithmic
            .to_vec()
            .iter()
            .map(|x| x.unwrap())
            .collect();
        let amounts: Vec<f64> = month_df
            .column("amount")
            .unwrap()
            .f64()
            .unwrap()
            //.to_vec().iter().map(|x| x.unwrap().abs().log(10.0) as f32) logarithmic
            .to_vec()
            .iter()
            .map(|x| x.unwrap())
            .collect();
        let cats: Vec<String> = month_df
            .column("category")
            .unwrap()
            .utf8()
            .unwrap()
            .into_iter()
            .map(|f| String::from(f.unwrap()))
            .collect();
        //.unique().unwrap().into_iter().map(|f| String::from(f.unwrap())).collect();
        categories_amounts_perc.push(percs);
        categories_amounts_perc_value.push(amounts);
        categories_amounts_perc_months.push(month.to_string());
        categories_amounts_perc_names.push(cats);
    }

    let categories_amounts_min = categories_amounts_min.unwrap();
    let categories_amounts_max = categories_amounts_max.unwrap();

    Ok(MonthlyTransactions {
        months,
        net_income,
        months_idx,
        months_idx_range,
        net_income_range,
        net_income_pairs,
        categories,
        categories_amounts,
        categories_months,
        categories_months_idx,
        categories_amounts_range: (categories_amounts_min, categories_amounts_max),
        categories_months_idx_range: (categories_months_idx_min, categories_months_idx_max),
        categories_pairs,
        categories_amounts_perc_months,
        categories_amounts_perc,
        categories_amounts_perc_value,
        categories_amounts_perc_names,
    })
}
