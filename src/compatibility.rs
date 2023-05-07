//! # Compatibility
//!
//! The `compatibility` library contains code that converts
//! registries in different format and with different schema
//! to the actual `realearning` structure.
//!
//! # Modules
//!
//! * `registro_ale`: this module converts from the registro of Ale

pub mod registro_ale;

mod compatibility_errors {
    use std::{error, fmt};

    #[derive(Debug, Clone)]
    pub struct ExtractionError;

    impl fmt::Display for ExtractionError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "invalid first item to double")
        }
    }

    impl error::Error for ExtractionError {}
}
