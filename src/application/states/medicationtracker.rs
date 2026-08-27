use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::application::medication::{
    dosetype::DoseType, medication::Medication, occurrencestatus::OccurrenceStatus, record::Record,
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

    pub fn insert_one_time_record(
        &mut self,
        name: String,
        dose: f32,
        dose_type: DoseType,
        time: DateTime<Utc>,
    ) -> String {
        let record = Record::new_one_time(name, dose, dose_type, time);
        let record_id = record.id.clone();
        self.records.push(record);
        record_id
    }

    pub fn delete_one_time_record(&mut self, record_id: &str) -> bool {
        let Some(index) = self
            .records
            .iter()
            .position(|record| record.id == record_id && record.is_one_time())
        else {
            return false;
        };

        self.records.remove(index);
        true
    }

    pub fn toggle_muted(&mut self, record_id: &str) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
            record.is_muted = !record.is_muted;
        }
    }

    pub fn clear_mute(&mut self, record_id: &str) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
            record.is_muted = false;
        }
    }

    fn deduct_stock(&mut self, record_id: &str) {
        let record = match self.records.iter().position(|r| r.id == record_id) {
            Some(idx) => idx,
            None => return,
        };
        if self.records[record].is_one_time() {
            return;
        }
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
        if self.records[record].is_one_time() {
            return;
        }
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
                record.is_muted = false;
            }
        } else {
            if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
                record.occurrence_status = OccurrenceStatus::Taken {
                    taken_at: Utc::now(),
                };
                record.is_muted = false;
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
                record.is_muted = false;
            }
        } else {
            if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
                record.occurrence_status = OccurrenceStatus::Taken { taken_at };
                record.is_muted = false;
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
            record.is_muted = false;
        }
    }
    pub fn mark_as_missed(&mut self, record_id: &str) {
        if let Some(record) = self.records.iter_mut().find(|r| r.id == record_id) {
            if record.is_one_time() {
                return;
            }
            record.occurrence_status = OccurrenceStatus::Missed;
            record.is_muted = false;
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
            record.is_muted = false;
        }
    }

    pub fn refill_stock(&mut self, medication_id: &str, pills: f32) {
        if let Some(med) = self.medications.iter_mut().find(|m| m.id == medication_id) {
            med.stock += pills;
        }
    }

    pub fn days_left(&self, medication_id: &str) -> Option<u32> {
        const PROJECTION_DAYS: f32 = 28.0;
        let med = self.medications.iter().find(|m| m.id == medication_id)?;
        if med.stock <= 0.0 || med.pill_dose <= 0.0 {
            return None;
        }
        let now = Utc::now();
        let horizon = now + Duration::days(PROJECTION_DAYS as i64);
        let mut total_pills = 0.0;
        for record in &self.records {
            if record.is_one_time()
                || record.medication_id != medication_id
                || record.time < now
                || record.time > horizon
                || !matches!(record.occurrence_status, OccurrenceStatus::Pending)
            {
                continue;
            }
            let Some(schedule) = med.schedules.iter().find(|s| s.id == record.schedule_id) else {
                continue;
            };
            total_pills += schedule.dose / med.pill_dose;
        }
        let daily_usage = total_pills / PROJECTION_DAYS;
        if daily_usage <= 0.0 {
            return None;
        }
        Some((med.stock / daily_usage).round() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::medication::dosetype::DoseType;
    use chrono::Utc;

    #[test]
    fn one_time_record_does_not_change_stock_or_become_missed() {
        let mut tracker = MedicationTracker::new();
        tracker
            .medications
            .push(Medication::new("Regular".into(), 10.0));
        let record_id =
            tracker.insert_one_time_record("Extra dose".into(), 4.0, DoseType::Unit, Utc::now());

        tracker.mark_as_taken(&record_id);
        let record = tracker.records.iter().find(|r| r.id == record_id).unwrap();
        assert!(matches!(
            record.occurrence_status,
            OccurrenceStatus::Taken { .. }
        ));
        assert_eq!(record.pills_deducted, 0.0);
        assert_eq!(tracker.medications[0].stock, 10.0);

        tracker.mark_as_taken(&record_id);
        tracker.mark_as_missed(&record_id);
        let record = tracker.records.iter().find(|r| r.id == record_id).unwrap();
        assert!(matches!(
            record.occurrence_status,
            OccurrenceStatus::Pending
        ));
    }

    #[test]
    fn deleting_one_time_record_removes_only_that_record() {
        let mut tracker = MedicationTracker::new();
        let first = tracker.insert_one_time_record("First".into(), 1.0, DoseType::Mg, Utc::now());
        let second = tracker.insert_one_time_record("Second".into(), 2.0, DoseType::Mg, Utc::now());

        assert!(tracker.delete_one_time_record(&first));
        assert!(!tracker.records.iter().any(|r| r.id == first));
        assert!(tracker.records.iter().any(|r| r.id == second));
        assert!(!tracker.delete_one_time_record(&first));
    }

    #[test]
    fn one_time_delete_path_cannot_remove_regular_records() {
        let mut tracker = MedicationTracker::new();
        let record = Record::new("medication-id".into(), "schedule-id".into(), Utc::now());
        let record_id = record.id.clone();
        tracker.records.push(record);

        assert!(!tracker.delete_one_time_record(&record_id));
        assert!(tracker.records.iter().any(|r| r.id == record_id));
    }

    #[test]
    fn one_time_record_is_ignored_by_days_left_projection() {
        let mut tracker = MedicationTracker::new();
        let mut medication = Medication::new("Regular".into(), 10.0);
        let mut schedule = crate::application::medication::schedule::Schedule::new(
            [12, 0],
            Some(crate::application::medication::periodtype::PeriodType::Daily),
            1,
            1.0,
        );
        schedule.id = "schedule-id".into();
        medication.schedules.push(schedule);
        let medication_id = medication.id.clone();
        tracker.medications.push(medication);
        tracker.records.push(Record::new(
            medication_id.clone(),
            "schedule-id".into(),
            Utc::now() + Duration::hours(1),
        ));
        tracker.insert_one_time_record(
            "Large one-time dose".into(),
            100.0,
            DoseType::Mg,
            Utc::now() + Duration::hours(1),
        );

        assert_eq!(tracker.days_left(&medication_id), Some(280));
    }

    #[test]
    fn mute_toggle_targets_one_record_and_actions_clear_it() {
        let mut tracker = MedicationTracker::new();
        let first = tracker.insert_one_time_record("First".into(), 1.0, DoseType::Mg, Utc::now());
        let second = tracker.insert_one_time_record("Second".into(), 1.0, DoseType::Mg, Utc::now());

        tracker.toggle_muted(&first);

        assert!(
            tracker
                .records
                .iter()
                .find(|r| r.id == first)
                .unwrap()
                .is_muted
        );
        assert!(
            !tracker
                .records
                .iter()
                .find(|r| r.id == second)
                .unwrap()
                .is_muted
        );

        tracker.mark_as_taken(&first);

        assert!(
            !tracker
                .records
                .iter()
                .find(|r| r.id == first)
                .unwrap()
                .is_muted
        );
    }
}
