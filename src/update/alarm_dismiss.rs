use chrono::Utc;

use crate::application::states::medicationtracker::MedicationTracker;

pub fn dismiss_expired_alarms(
    tracker: &mut MedicationTracker,
    alarming_records: &mut Vec<String>,
) -> bool {
    let now = Utc::now();
    let expired: Vec<String> = alarming_records
        .iter()
        .filter(|id| {
            tracker
                .records
                .iter()
                .find(|r| &r.id == *id)
                .map(|r| now.signed_duration_since(r.time).num_minutes() > 15)
                .unwrap_or(true)
        })
        .cloned()
        .collect();
    for id in &expired {
        if tracker
            .records
            .iter()
            .find(|record| &record.id == id)
            .is_some_and(|record| !record.is_one_time())
        {
            tracker.mark_as_missed(id);
        }
        alarming_records.retain(|r| r != id);
    }
    !expired.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::medication::{
        dosetype::DoseType, occurrencestatus::OccurrenceStatus, record::Record,
    };
    use chrono::{Duration, Utc};

    #[test]
    fn expired_one_time_alarm_is_removed_without_marking_record_missed() {
        let mut tracker = MedicationTracker::new();
        let record_id = tracker.insert_one_time_record(
            "One-time alarm".into(),
            1.0,
            DoseType::Mg,
            Utc::now() - Duration::minutes(16),
        );
        let mut alarming_records = vec![record_id.clone()];

        assert!(dismiss_expired_alarms(&mut tracker, &mut alarming_records));
        assert!(alarming_records.is_empty());
        assert!(matches!(
            tracker.records[0].occurrence_status,
            OccurrenceStatus::Pending
        ));
    }

    #[test]
    fn expired_regular_alarm_is_still_marked_missed() {
        let mut tracker = MedicationTracker::new();
        let record = Record::new(
            "medication-id".into(),
            "schedule-id".into(),
            Utc::now() - Duration::minutes(16),
        );
        let record_id = record.id.clone();
        tracker.records.push(record);
        let mut alarming_records = vec![record_id];

        dismiss_expired_alarms(&mut tracker, &mut alarming_records);

        assert!(matches!(
            tracker.records[0].occurrence_status,
            OccurrenceStatus::Missed
        ));
    }
}
