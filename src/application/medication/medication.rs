use chrono::{DateTime, Local, Utc};

use crate::application::medication::dosetype::DoseType;
use crate::application::medication::schedule::Schedule;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Medication {
    pub id: String,
    pub name: String,
    pub stock: f32,
    #[serde(default = "default_pill_dose")]
    pub pill_dose: f32,
    #[serde(default)]
    pub dose_type: DoseType,
    pub created_at: DateTime<Utc>,
    pub schedules: Vec<Schedule>,
}

fn default_pill_dose() -> f32 {
    1.0
}

impl Medication {
    pub fn new(name: String, stock: f32) -> Self {
        Medication {
            name,
            id: uuid::Uuid::new_v4().to_string(),
            stock,
            pill_dose: 1.0,
            dose_type: DoseType::Mg,
            created_at: Local::now().to_utc(),
            schedules: Vec::new(),
        }
    }
}
