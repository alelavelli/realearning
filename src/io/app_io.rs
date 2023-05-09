use clap::Parser;
use clap_verbosity_flag::Verbosity;

use crate::compatibility::CompatibilityEnum;

/// Arguments to pass to clit application
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// The file of the raw file
    #[arg(short, long)]
    pub input_file: String,
    // Type of compatibility for the input raw file
    #[arg(short, long, default_value_t=CompatibilityEnum::Base)]
    pub compatibility: CompatibilityEnum,
    /// The folder where to put plots
    #[arg(short, long)]
    pub plot_folder: String,
    /// Set verbosity level of the application
    ///
    /// -q silences output
    /// -v show warnings
    /// -vv show info
    /// -vvv show debug
    /// -vvvv show trace
    #[command(flatten)]
    pub verbose: Verbosity,
}
