use chrono::{Datelike, Days, Local, Months, NaiveDate, NaiveTime, TimeZone};

use crate::application::medication::{
    medication::Medication, periodtype::PeriodType, record::Record, schedule::Schedule,
};
use crate::application::states::medicationtracker::MedicationTracker;

const GENERATION_WINDOW_MONTHS: u32 = 2;

pub fn generate_future_records(tracker: &mut MedicationTracker) {
    let today = Local::now().date_naive();
    let end_date = today
        .checked_add_months(Months::new(GENERATION_WINDOW_MONTHS))
        .expect("Date overflow adding generation window");
    for medication in &tracker.medications {
        if medication.is_archived {
            continue;
        }
        for schedule in &medication.schedules {
            let new_records = if schedule.period_type.is_some() {
                generate_interval_records(medication, schedule, today, end_date, &tracker.records)
            } else {
                generate_weekday_records(medication, schedule, today, end_date, &tracker.records)
            };
            for record in &new_records {
                println!(
                    "  Record: {} at {}",
                    record.medication_id,
                    record.time.with_timezone(&Local).format("%Y-%m-%d %H:%M")
                );
            }
            tracker.records.extend(new_records);
        }
    }
}

pub fn generate_records_for_medication(tracker: &mut MedicationTracker, medication_id: &str) {
    let today = Local::now().date_naive();
    let end_date = today
        .checked_add_months(Months::new(GENERATION_WINDOW_MONTHS))
        .expect("Date overflow adding generation window");
    let medication = tracker.medications.iter().find(|m| m.id == medication_id);
    if let Some(medication) = medication {
        if medication.is_archived {
            return;
        }
        let mut new_records = Vec::new();
        for schedule in &medication.schedules {
            let records = if schedule.period_type.is_some() {
                generate_interval_records(medication, schedule, today, end_date, &tracker.records)
            } else {
                generate_weekday_records(medication, schedule, today, end_date, &tracker.records)
            };
            new_records.extend(records);
        }
        for record in &new_records {
            println!(
                "  Record: {} at {}",
                record.medication_id,
                record.time.with_timezone(&Local).format("%Y-%m-%d %H:%M")
            );
        }
        tracker.records.extend(new_records);
    }
    println!("Record-Size: {}", tracker.records.len());
}

fn generate_weekday_records(
    medication: &Medication,
    schedule: &Schedule,
    start_date: NaiveDate,
    end_date: NaiveDate,
    existing_records: &[Record],
) -> Vec<Record> {
    println!("Entered Generate weedkay records");
    println!("Start-Date: {start_date} End-Date: {end_date}");
    let mut records = Vec::new();
    let mut date = start_date;
    while date <= end_date {
        let day_matches = match &schedule.week_day {
            Some(days) => days.is_empty() || days.contains(&date.weekday()),
            None => true,
        };
        if day_matches {
            let record = create_record(medication, schedule, date);
            if !record_exists(existing_records, &record) {
                records.push(record);
            }
        }
        date = date
            .checked_add_days(Days::new(1))
            .expect("Date overflow in weekday generation");
    }

    records
}

fn generate_interval_records(
    medication: &Medication,
    schedule: &Schedule,
    start_date: NaiveDate,
    end_date: NaiveDate,
    existing_records: &[Record],
) -> Vec<Record> {
    let period_type = schedule.period_type.as_ref().unwrap();
    let interval = schedule.period_time;
    println!("Entered Generate interval records");
    println!("Start-Date: {start_date} End-Date: {end_date}");
    let anchor_date =
        find_last_record_date(existing_records, &medication.id, &schedule.id).unwrap_or(start_date);
    let mut records = Vec::new();
    let mut date = anchor_date;
    while date < start_date {
        date = advance_by_period(date, period_type, interval);
    }
    if date <= end_date {
        let record = create_record(medication, schedule, date);
        if !record_exists(existing_records, &record) {
            records.push(record);
        }
    }
    loop {
        date = advance_by_period(date, period_type, interval);
        if date > end_date {
            break;
        }
        let record = create_record(medication, schedule, date);
        if !record_exists(existing_records, &record) {
            records.push(record);
        }
    }

    records
}

fn find_last_record_date(
    records: &[Record],
    medication_id: &str,
    schedule_id: &str,
) -> Option<NaiveDate> {
    records
        .iter()
        .filter(|r| r.medication_id == medication_id && r.schedule_id == schedule_id)
        .map(|r| r.time.with_timezone(&Local).date_naive())
        .max()
}

fn advance_by_period(date: NaiveDate, period_type: &PeriodType, interval: u8) -> NaiveDate {
    match period_type {
        PeriodType::Hourly => {
            // Hourly doesn't make sense at date level — treat as daily
            date.checked_add_days(Days::new(1))
                .expect("Date overflow in hourly advance")
        }
        PeriodType::Daily => date
            .checked_add_days(Days::new(interval as u64))
            .expect("Date overflow in daily advance"),
        PeriodType::Weekly => date
            .checked_add_days(Days::new(interval as u64 * 7))
            .expect("Date overflow in weekly advance"),
        PeriodType::Monthly => date
            .checked_add_months(Months::new(interval as u32))
            .expect("Date overflow in monthly advance"),
    }
}

fn record_exists(existing_records: &[Record], new_record: &Record) -> bool {
    let new_date = new_record.time.with_timezone(&Local).date_naive();
    existing_records.iter().any(|r| {
        r.medication_id == new_record.medication_id
            && r.schedule_id == new_record.schedule_id
            && r.time.with_timezone(&Local).date_naive() == new_date
    })
}

fn create_record(medication: &Medication, schedule: &Schedule, date: NaiveDate) -> Record {
    let naive_time = NaiveTime::from_hms_opt(schedule.time[0] as u32, schedule.time[1] as u32, 0)
        .expect("Invalid schedule time");
    let naive_datetime = date.and_time(naive_time);
    let local_datetime = Local
        .from_local_datetime(&naive_datetime)
        .single()
        .expect("Ambiguous or invalid local datetime");
    let utc_datetime = local_datetime.to_utc();
    Record::new(medication.id.clone(), schedule.id.clone(), utc_datetime)
}
