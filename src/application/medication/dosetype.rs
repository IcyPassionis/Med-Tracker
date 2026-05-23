use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default)]
pub enum DoseType {
    #[default]
    Mg,
    Mcg,
    Ml,
    Unit,
}

impl fmt::Display for DoseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DoseType::Mg => write!(f, "mg"),
            DoseType::Mcg => write!(f, "mcg"),
            DoseType::Ml => write!(f, "ml"),
            DoseType::Unit => write!(f, "unit"),
        }
    }
}