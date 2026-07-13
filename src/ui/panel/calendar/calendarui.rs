use crate::application::medication::occurrencestatus::OccurrenceStatus;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::style;
use chrono::{Datelike, Duration, Local, NaiveDate};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, alignment};

pub struct CalendarUI {
    pub(super) month: NaiveDate,
}

impl CalendarUI {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            month: first_of_month(today),
        }
    }

    pub fn view<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let calendar = column![
            self.calendar_top_view(tracker),
            self.calendar_bottom_panel_view(),
        ]
        .spacing(16)
        .padding(24)
        .width(Fill)
        .max_width(900);

        container(calendar).center_x(Fill).center_y(Fill).into()
    }

    fn calendar_top_view<'a>(
        &'a self,
        tracker: &'a MedicationTracker,
    ) -> Element<'a, Message> {
        let first_day = first_of_month(self.month);
        let days_in_month = (first_of_month(next_month(first_day)) - Duration::days(1)).day();
        let leading_empty = first_day.weekday().num_days_from_monday() as usize;
        let mut weeks = Vec::new();
        let mut week = row![].spacing(8).width(Fill);
        let mut week_slots = leading_empty;

        for _ in 0..leading_empty {
            week = week.push(Self::empty_day_slot());
        }
        for day in 1..=days_in_month {
            let date = first_day.with_day(day).expect("valid calendar day");
            let percentage = completion_percentage(tracker, date);
            week = week.push(Self::calendar_day_button(date, percentage));
            week_slots += 1;

            if (leading_empty + day as usize) % 7 == 0 {
                weeks.push(week);
                week = row![].spacing(8).width(Fill);
                week_slots = 0;
            }
        }
        if week_slots > 0 {
            for _ in week_slots..7 {
                week = week.push(Self::empty_day_slot());
            }
            weeks.push(week);
        }

        let weekdays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .into_iter()
            .fold(row![].spacing(8).width(Fill), |row, day| {
                row.push(container(text(day).center()).width(Fill))
            });
        let week_column = weeks
            .into_iter()
            .fold(column![].spacing(8), |column, week| column.push(week));

        column![weekdays, week_column].spacing(16).into()
    }

    fn calendar_bottom_panel_view<'a>(&'a self) -> Element<'a, Message> {
        let latest_month = next_month(first_of_month(Local::now().date_naive()));
        let can_go_next = self.month < latest_month;
        let month_label = text(self.month.format("%B %Y").to_string()).size(24);

        row![
            button(text("<").size(22))
                .style(style::calendar::navigation_button)
                .on_press(Message::PreviousMonth),
            container(month_label).center_x(Fill),
            button(text(">").size(22))
                .style(style::calendar::navigation_button)
                .on_press_maybe(can_go_next.then_some(Message::NextMonth)),
        ]
        .spacing(12)
        .align_y(alignment::Vertical::Center)
        .width(Fill)
        .into()
    }

    fn calendar_day_button(date: NaiveDate, percentage: u8) -> Element<'static, Message> {
        let label = column![
            text(date.day().to_string()).size(20),
            text(format!("{}%", percentage)).size(11)
        ]
        .spacing(3)
        .align_x(alignment::Horizontal::Center);

        button(label)
            .style(style::calendar::button(percentage))
            .width(Fill)
            .height(64)
            .padding(6)
            .on_press(Message::SelectDay(date))
            .into()
    }

    fn empty_day_slot() -> Element<'static, Message> {
        container("").width(Fill).height(64).into()
    }

    pub fn update(&mut self, message: Message) -> Option<NaiveDate> {
        match message {
            Message::PreviousMonth => self.month = previous_month(self.month),
            Message::NextMonth => {
                let latest_month = next_month(first_of_month(Local::now().date_naive()));
                if self.month < latest_month {
                    self.month = next_month(self.month);
                }
            }
            Message::SelectDay(date) => return Some(date),
        }
        None
    }
}

fn completion_percentage(tracker: &MedicationTracker, date: NaiveDate) -> u8 {
    let records: Vec<_> = tracker
        .records
        .iter()
        .filter(|record| record.time.with_timezone(&Local).date_naive() == date)
        .collect();
    if records.is_empty() {
        return 0;
    }

    let taken = records
        .iter()
        .filter(|record| matches!(record.occurrence_status, OccurrenceStatus::Taken { .. }))
        .count();
    ((taken * 100) / records.len()) as u8
}

pub(super) fn first_of_month(date: NaiveDate) -> NaiveDate {
    date.with_day(1).expect("valid month")
}

pub(super) fn next_month(date: NaiveDate) -> NaiveDate {
    if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
    }
}

fn previous_month(date: NaiveDate) -> NaiveDate {
    if date.month() == 1 {
        NaiveDate::from_ymd_opt(date.year() - 1, 12, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() - 1, 1).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    PreviousMonth,
    NextMonth,
    SelectDay(NaiveDate),
}
