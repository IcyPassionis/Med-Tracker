use chrono::DateTime;

use super::{dosetype::DoseType, occurrencestatus::OccurrenceStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OneTimeData {
    pub name: String,
    pub dose: f32,
    pub dose_type: DoseType,
}

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub medication_id: String,
    pub schedule_id: String,
    pub time: DateTime<chrono::Utc>,
    pub occurrence_status: OccurrenceStatus,
    #[serde(default)]
    pub rescheduled: bool,
    #[serde(skip)]
    pub is_muted: bool,
    #[serde(default)]
    pub pills_deducted: f32,
    #[serde(default)]
    pub one_time: Option<OneTimeData>,
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
            is_muted: false,
            pills_deducted: 0.0,
            one_time: None,
        }
    }

    pub fn new_one_time(
        name: String,
        dose: f32,
        dose_type: DoseType,
        time: DateTime<chrono::Utc>,
    ) -> Self {
        Record {
            id: uuid::Uuid::new_v4().to_string(),
            medication_id: String::new(),
            schedule_id: String::new(),
            time,
            occurrence_status: OccurrenceStatus::Pending,
            rescheduled: false,
            is_muted: false,
            pills_deducted: 0.0,
            one_time: Some(OneTimeData {
                name,
                dose,
                dose_type,
            }),
        }
    }

    pub fn is_one_time(&self) -> bool {
        self.one_time.is_some()
    }

    pub fn empty_new() -> Self {
        Record {
            id: uuid::Uuid::new_v4().to_string(),
            medication_id: String::new(),
            schedule_id: String::new(),
            time: chrono::Utc::now(),
            occurrence_status: OccurrenceStatus::Pending,
            rescheduled: false,
            is_muted: false,
            pills_deducted: 0.0,
            one_time: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn old_record_json_defaults_one_time_payload_to_none() {
        let json = r#"
            {
                "id": "record-id",
                "medication_id": "medication-id",
                "schedule_id": "schedule-id",
                "time": "2026-08-26T10:00:00Z",
                "occurrence_status": "Pending",
                "rescheduled": false,
                "pills_deducted": 0.0
            }
        "#;

        let record: Record = serde_json::from_str(json).unwrap();

        assert!(record.one_time.is_none());
        assert_eq!(record.medication_id, "medication-id");
    }

    #[test]
    fn one_time_constructor_has_no_medication_or_schedule_association() {
        let record =
            Record::new_one_time("As-needed medicine".into(), 2.5, DoseType::Ml, Utc::now());

        assert!(record.is_one_time());
        assert!(record.medication_id.is_empty());
        assert!(record.schedule_id.is_empty());
        assert_eq!(
            record.one_time,
            Some(OneTimeData {
                name: "As-needed medicine".into(),
                dose: 2.5,
                dose_type: DoseType::Ml,
            })
        );
    }

    #[test]
    fn mute_state_is_not_persisted() {
        let mut record = Record::new("medication-id".into(), "schedule-id".into(), Utc::now());
        record.is_muted = true;

        let json = serde_json::to_string(&record).unwrap();
        let restored: Record = serde_json::from_str(&json).unwrap();

        assert!(!json.contains("is_muted"));
        assert!(!restored.is_muted);
    }
}
