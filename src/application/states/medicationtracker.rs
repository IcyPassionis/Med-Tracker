use chrono::{DateTime, NaiveDate, Utc};

use crate::application::medication::{
    medication::Medication, occurrencestatus::OccurrenceStatus, record::Record,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MedicationTracker {
    pub records: Vec<Record>,
    pub medications: Vec<Medication>,
    pub last_generation_date: Option<NaiveDate>,
}
impl MedicationTracker {
    pub fn new() -> Self {
        MedicationTracker {
            records: Vec::new(),
            medications: Vec::new(),
            last_generation_date: None,
        }
    }
    pub fn generate_records(
        &mut self,
        start_time: DateTime<chrono::Utc>,
        end_time: DateTime<chrono::Utc>,
    ) {
        for medication in self.medications.iter() {
            for schedule in medication.schedules.iter() {}
        }
    }

    fn deduct_stock(&mut self, record_id: &str) {
        let record = match self.records.iter().position(|r| r.id == record_id) {
            Some(idx) => idx,
            None => return,
        };
        let medication_id = self.records[record].medication_id.clone();
        let schedule_id = self.records[record].schedule_id.clone();
        let medication = match self.medications.iter().position(|m| m.id == medication_id) {
            Some(idx) => idx,
            None => return,
        };
        let pill_dose = self.medications[medication].pill_dose;
        let schedule = match self.medications[medication]
            .schedules
            .iter()
            .find(|s| s.id == schedule_id)
        {
            Some(s) => s,
            None => return,
        };
        if pill_dose <= 0.0 {
            return;
        }
        let pills_deducted = schedule.dose / pill_dose;
        self.medications[medication].stock =
            (self.medications[medication].stock - pills_deducted).max(0.0);
        self.records[record].pills_deducted = pills_deducted;
    }

    fn restore_stock(&mut self, record_id: &str) {
        let record = match self.records.iter().position(|r| r.id == record_id) {
            Some(idx) => idx,
            None => return,
        };
        let pills_deducted = self.records[record].pills_deducted;
        if pills_deducted <= 0.0 {
            return;
        }
        let medication_id = self.records[record].medication_id.clone();
        let medication = match self.medications.iter().position(|m| m.id == medication_id) {
            Some(idx) => idx,
            None => return,
        };
        self.medications[medication].stock += pills_deducted;
        self.records[record].pills_deducted = 0.0;
    }

    pub fn mark_as_taken(&mut self, record_id: &str) {
        let is_taken = match self.records.iter().find(|r| r.id == record_id) {
            Some(r) => matches!(r.occurrence_status, OccurrenceStatus::Taken { .. }),
            None => return,
        };

        if is_taken {
            self.restore_stock(record_id);
            if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
                record.occurrence_status = OccurrenceStatus::Pending;
            }
        } else {
            if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
                record.occurrence_status = OccurrenceStatus::Taken {
                    taken_at: Utc::now(),
                };
            }
            self.deduct_stock(record_id);
        }
    }

    pub fn mark_as_taken_at(&mut self, record_id: &str, taken_at: DateTime<Utc>) {
        let is_taken = match self.records.iter().find(|r| r.id == record_id) {
            Some(r) => matches!(r.occurrence_status, OccurrenceStatus::Taken { .. }),
            None => return,
        };

        if is_taken {
            self.restore_stock(record_id);
            if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
                record.occurrence_status = OccurrenceStatus::Pending;
            }
        } else {
            if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
                record.occurrence_status = OccurrenceStatus::Taken { taken_at };
            }
            self.deduct_stock(record_id);
        }
    }
    pub fn mark_as_skipped(&mut self, record_id: &str) {
        let is_taken = match self.records.iter().find(|r| r.id == record_id) {
            Some(r) => matches!(r.occurrence_status, OccurrenceStatus::Taken { .. }),
            None => return,
        };
        if is_taken {
            self.restore_stock(record_id);
        }
        if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
            if matches!(record.occurrence_status, OccurrenceStatus::Skipped { .. }) {
                record.occurrence_status = OccurrenceStatus::Pending;
            } else {
                record.occurrence_status = OccurrenceStatus::Skipped { reason: None };
            }
        }
    }
    pub fn mark_as_missed(&mut self, record_id: &str) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
            record.occurrence_status = OccurrenceStatus::Missed;
        }
    }

    pub fn reschedule_record(&mut self, record_id: &str, new_time: DateTime<Utc>) {
        let is_taken = match self.records.iter().find(|r| r.id == record_id) {
            Some(r) => matches!(r.occurrence_status, OccurrenceStatus::Taken { .. }),
            None => return,
        };
        if is_taken {
            self.restore_stock(record_id);
        }
        if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
            record.time = new_time;
            record.rescheduled = true;
            record.occurrence_status = OccurrenceStatus::Pending;
        }
    }

    pub fn refill_stock(&mut self, medication_id: &str, pills: f32) {
        if let Some(med) = self.medications.iter_mut().find(|m| m.id == medication_id) {
            med.stock += pills;
        }
    }
}
