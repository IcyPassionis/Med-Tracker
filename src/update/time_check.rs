use iced::{Subscription, time};

use crate::application::{
    app::App, medication::occurrencestatus::OccurrenceStatus, message::Message,
    states::medicationtracker::MedicationTracker,
};
use crate::update::generate_records::generate_future_records;
use chrono::{Datelike, Local, Utc};

pub fn update_time(state: &App) -> Subscription<Message> {
    time::every(time::Duration::from_secs(30)).map(|_| Message::TimeCheck)
}

pub fn check_new_day(tracker: &mut MedicationTracker) {
    let today = Local::now().date_naive();
    let is_new_day = match tracker.last_generation_date {
        Some(last_date) => today > last_date,
        None => true,
    };
    if is_new_day {
        tracker.last_generation_date = Some(today);
        generate_future_records(tracker);
        println!("New day detected: {today}. Generated future records.");
    }
}

pub fn check_medication_schedule(
    tracker: &MedicationTracker,
    alarm_time_minutes: u32,
) -> Vec<String> {
    let now = Utc::now();
    let today = Local::now().date_naive();
    let mut alarming_records = Vec::new();
    for record in &tracker.records {
        let is_pending = matches!(record.occurrence_status, OccurrenceStatus::Pending);
        if !is_pending {
            continue;
        }
        let record_date = record.time.with_timezone(&Local).date_naive();
        let is_today = record_date.year() == today.year()
            && record_date.month() == today.month()
            && record_date.day() == today.day();
        if !is_today {
            continue;
        }
        let diff = now.signed_duration_since(record.time);
        let elapsed = diff.num_minutes();
        if elapsed >= 0 && (alarm_time_minutes == 0 || elapsed <= alarm_time_minutes as i64) {
            alarming_records.push(record.id.clone());
        }
    }

    alarming_records
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::medication::dosetype::DoseType;
    use chrono::Utc;

    #[test]
    fn one_time_record_created_at_local_now_is_found_by_alarm_scan() {
        let mut tracker = MedicationTracker::new();
        let record_id =
            tracker.insert_one_time_record("Local alarm".into(), 1.0, DoseType::Mg, Utc::now());

        assert!(check_medication_schedule(&tracker, 15).contains(&record_id));
    }

    #[test]
    fn alarm_scan_uses_configured_window_and_zero_is_infinite() {
        let mut tracker = MedicationTracker::new();
        let record_id = tracker.insert_one_time_record(
            "Long alarm".into(),
            1.0,
            DoseType::Mg,
            Utc::now() - chrono::Duration::minutes(20),
        );

        assert!(!check_medication_schedule(&tracker, 15).contains(&record_id));
        assert!(check_medication_schedule(&tracker, 30).contains(&record_id));
        assert!(check_medication_schedule(&tracker, 0).contains(&record_id));
    }
}
