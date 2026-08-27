use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::style;
use crate::ui::style::medications::container::backdrop;
use crate::ui::style::time::container::overlay_panel_container;
use chrono::{DateTime, TimeZone, Timelike, Utc};
use iced::Length::Fill;
use iced::widget::{Image, button, column, container, row, text, text_input};
use iced::{ContentFit, Element, Padding, alignment};

pub struct ReschedulePanel {
    record_ids: Vec<String>,
    hour: String,
    minute: String,
    warning: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(String),
    HourChange(String),
    MinuteChange(String),
    Confirm,
    Cancel,
}

impl ReschedulePanel {
    pub fn new() -> Self {
        Self {
            record_ids: Vec::new(),
            hour: String::new(),
            minute: String::new(),
            warning: None,
        }
    }

    pub fn open(&mut self, record_id: String, current_time: DateTime<Utc>) {
        self.open_many(vec![record_id], current_time);
    }

    pub fn open_many(&mut self, record_ids: Vec<String>, current_time: DateTime<Utc>) {
        let local = current_time.with_timezone(&chrono::Local);
        self.hour = format!("{:02}", local.hour());
        self.minute = format!("{:02}", local.minute());
        self.record_ids = record_ids;
        self.warning = None;
    }

    pub fn is_open(&self) -> bool {
        !self.record_ids.is_empty()
    }

    pub fn view(&self) -> Option<Element<'_, Message>> {
        if self.record_ids.is_empty() {
            return None;
        }

        let time_row = row![
            text_input("HH", &self.hour)
                .on_input(Message::HourChange)
                .width(70)
                .size(24),
            text(":").size(24),
            text_input("MM", &self.minute)
                .on_input(Message::MinuteChange)
                .width(70)
                .size(24),
        ]
        .spacing(8)
        .align_y(iced::alignment::Vertical::Center);

        let header = row![
            text("Reschedule").size(22).width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .style(style::time::button::overlay_close_button)
                .padding(5)
                .on_press(Message::Cancel),
        ]
        .align_y(iced::alignment::Vertical::Center);

        let done_button = button(container(text("Done")).center_x(Fill).center_y(Fill))
            .style(style::time::button::add_button)
            .width(Fill)
            .height(48)
            .padding(12)
            .on_press(Message::Confirm);

        let mut content = column![header, time_row].spacing(20);

        if let Some(warning) = &self.warning {
            let warning_text =
                text(warning)
                    .size(14)
                    .style(|theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().danger.base.color),
                    });
            content = content.push(warning_text);
        }

        content = content.push(done_button);

        let panel = container(content.padding(Padding::new(30.0)))
            .style(overlay_panel_container)
            .width(280);

        let overlay = container(container(panel).center(Fill))
            .style(backdrop)
            .width(Fill)
            .height(Fill);

        Some(overlay.into())
    }

    pub fn update(&mut self, tracker: &mut MedicationTracker, msg: Message) -> Option<Vec<String>> {
        match msg {
            Message::Open(id) => self.handle_open(tracker, id),
            Message::HourChange(v) => self.handle_hour_change(v),
            Message::MinuteChange(v) => self.handle_minute_change(v),
            Message::Confirm => self.handle_confirm(tracker),
            Message::Cancel => self.handle_cancel(),
        }
    }

    fn handle_open(&mut self, tracker: &MedicationTracker, id: String) -> Option<Vec<String>> {
        if let Some(record) = tracker.records.iter().find(|r| r.id == id) {
            self.open(id, record.time);
        }
        None
    }

    fn handle_hour_change(&mut self, v: String) -> Option<Vec<String>> {
        if v.len() <= 2 && v.chars().all(|c| c.is_ascii_digit()) {
            self.hour = v;
            self.warning = None;
        }
        None
    }

    fn handle_minute_change(&mut self, v: String) -> Option<Vec<String>> {
        if v.len() <= 2 && v.chars().all(|c| c.is_ascii_digit()) {
            self.minute = v;
            self.warning = None;
        }
        None
    }

    fn handle_confirm(&mut self, tracker: &mut MedicationTracker) -> Option<Vec<String>> {
        let hour: i32 = self.hour.parse().unwrap_or(-1);
        let minute: i32 = self.minute.parse().unwrap_or(-1);
        if hour < 0 || hour > 23 {
            self.warning = Some("Hour must be between 0 and 23.".into());
            return None;
        }
        if minute < 0 || minute > 59 {
            self.warning = Some("Minute must be between 0 and 59.".into());
            return None;
        }
        let hour = hour as u32;
        let minute = minute as u32;
        let ids = std::mem::take(&mut self.record_ids);
        for id in &ids {
            if let Some(record) = tracker.records.iter().find(|r| r.id == *id) {
                let local = record.time.with_timezone(&chrono::Local);
                let new_time = local
                    .date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .and_then(|dt| chrono::Local.from_local_datetime(&dt).single())
                    .map(|dt| dt.with_timezone(&Utc));
                if let Some(new_time) = new_time {
                    tracker.reschedule_record(id, new_time);
                }
            }
        }
        self.clear_inputs();
        Some(ids)
    }

    fn handle_cancel(&mut self) -> Option<Vec<String>> {
        self.record_ids.clear();
        self.warning = None;
        self.clear_inputs();
        None
    }

    fn clear_inputs(&mut self) {
        self.hour.clear();
        self.minute.clear();
    }
}
