use std::process;

use clap::Parser;
use log::{error, info, warn};
use realearning::{
    compatibility::{registro_ale::build_registry_batch, CompatibilityEnum},
    io::app_io::CliArgs,
    plots::{
        plot_registry::*,
        plot_utils::{palettes::RED_PALETTE, resolution::R720},
    },
};
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("{:?}", args.input_file);
    info!("{:?}", args.plot_folder);
    info!("{:?}", args.compatibility);

    let re = Regex::new(r"^\d{4}-\d{2}$").unwrap();

    match args.compatibility {
        CompatibilityEnum::Ale => {
            let (loaded_registry, failed_extractions) = build_registry_batch(&args.input_file, re)
                .map_err(|e| {
                    error!(
                        "{}",
                        format!(
                            "Failed to extract registry from {} with error \"{}\"",
                            args.input_file, e
                        )
                    );
                    process::exit(1)
                })
                .unwrap();

            if !failed_extractions.is_empty() {
                warn!("Failed Extractions {:?}", failed_extractions);
            }
            let df = loaded_registry
                .to_dataframe()
                .map_err(|e| {
                    error!(
                        "{}",
                        format!(
                            "Failed to transform the registry to dataframe with error \"{}\"",
                            e
                        )
                    )
                })
                .unwrap();
            info!("The registry has shape {:?}", df.shape());

            plot_daily_transactions(&loaded_registry, R720, &args.plot_folder, &RED_PALETTE)
                .unwrap();
            plot_category_pie(&loaded_registry, R720, 7, &args.plot_folder, &RED_PALETTE).unwrap();
            plot_monthly_report(
                &loaded_registry,
                R720,
                Some(10),
                &args.plot_folder,
                &RED_PALETTE,
            )
            .unwrap();
        }
        _ => {
            error!("Only implemented compatibility is Ale");
        }
    };

    Ok(())
}
