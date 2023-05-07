use realearning::{
    compatibility::registro_ale::build_registry_batch,
    plots::{
        plot_registry::*,
        plot_utils::{palettes::RED_PALETTE, resolution::R720},
    },
};
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let re = Regex::new(r"^\d{4}-\d{2}$").unwrap();

    let (loaded_registry, failed_extractions) = build_registry_batch("data/registro.xlsx", re)
        .expect("Failed to extract registry from excel!");

    if !failed_extractions.is_empty(){
        println!("Failed Extractions {:?}", failed_extractions);
    }
    let df = loaded_registry
        .to_dataframe()
        .expect("Failed to transform to dataframe!");
    println!("The registry has shape {:?}", df.shape());

    let root_path = "plots";
    plot_daily_transactions(&loaded_registry, R720, root_path, &RED_PALETTE).unwrap();
    plot_category_pie(&loaded_registry, R720, 7, root_path, &RED_PALETTE).unwrap();
    plot_monthly_report(&loaded_registry, R720, Some(10), root_path, &RED_PALETTE).unwrap();
    Ok(())
}
