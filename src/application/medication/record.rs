use chrono::DateTime;

use super::occurrencestatus::OccurrenceStatus;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub medication_id: String,
    pub schedule_id: String,
    pub time: DateTime<chrono::Utc>,
    pub occurrence_status: OccurrenceStatus,
    #[serde(default)]
    pub rescheduled: bool,
    #[serde(default)]
    pub pills_deducted: f32,
}
impl Record {
    pub fn new(medication_id: String, schedule_id: String, time: DateTime<chrono::Utc>) -> Self {
        Record {
            id: uuid::Uuid::new_v4().to_string(),
            medication_id,
            schedule_id,
            time,
            occurrence_status: OccurrenceStatus::Pending,
            rescheduled: false,
            pills_deducted: 0.0,
        }
    }
    pub fn empty_new() -> Self {
        Record {
            id: uuid::Uuid::new_v4().to_string(),
            medication_id: String::new(),
            schedule_id: String::new(),
            time: chrono::Utc::now(),
            occurrence_status: OccurrenceStatus::Pending,
            rescheduled: false,
            pills_deducted: 0.0,
        }
    }
}
