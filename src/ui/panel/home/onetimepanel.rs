use crate::application::medication::dosetype::DoseType;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::style;
use crate::ui::style::medications::container::backdrop;
use crate::ui::style::time::container::overlay_panel_container;
use chrono::{Local, NaiveDate, TimeZone, Utc};
use iced::Length::{Fill, FillPortion};
use iced::widget::{Image, button, column, container, pick_list, row, text, text_input};
use iced::{ContentFit, Element, Padding, Theme, alignment};

pub struct OneTimeRecordPanel {
    open: bool,
    name: String,
    dose: String,
    dose_type: DoseType,
    hour: String,
    minute: String,
    warning: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    NameChange(String),
    DoseChange(String),
    DoseTypeChange(DoseType),
    HourChange(String),
    MinuteChange(String),
    SetAlarm,
    TakeNow,
}

impl OneTimeRecordPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            name: String::new(),
            dose: String::new(),
            dose_type: DoseType::Mg,
            hour: String::new(),
            minute: String::new(),
            warning: None,
        }
    }

    pub fn view(&self, selected_date: NaiveDate) -> Option<Element<'_, Message>> {
        if !self.open {
            return None;
        }

        Some(
            container(container(self.form_overlay(selected_date)).center(Fill))
                .style(backdrop)
                .width(Fill)
                .height(Fill)
                .into(),
        )
    }

    pub fn update(
        &mut self,
        tracker: &mut MedicationTracker,
        selected_date: NaiveDate,
        message: Message,
    ) -> bool {
        match message {
            Message::Open => {
                self.open = true;
                self.clear_fields();
                false
            }
            Message::Close => {
                self.close();
                false
            }
            Message::NameChange(value) => {
                self.name = value;
                false
            }
            Message::DoseChange(value) => {
                if value.is_empty() || value.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    let dots = value.chars().filter(|c| *c == '.').count();
                    if dots <= 1 && !value.starts_with('-') {
                        self.dose = value;
                    }
                }
                false
            }
            Message::DoseTypeChange(value) => {
                self.dose_type = value;
                false
            }
            Message::HourChange(value) => {
                if value.len() <= 2 && value.chars().all(|c| c.is_ascii_digit()) {
                    self.hour = value;
                    self.warning = None;
                }
                false
            }
            Message::MinuteChange(value) => {
                if value.len() <= 2 && value.chars().all(|c| c.is_ascii_digit()) {
                    self.minute = value;
                    self.warning = None;
                }
                false
            }
            Message::SetAlarm => self.create_record(tracker, selected_date, false),
            Message::TakeNow => {
                if selected_date != Local::now().date_naive() {
                    self.warning = Some("Take Now is available only for today.".into());
                    false
                } else {
                    self.create_record(tracker, selected_date, true)
                }
            }
        }
    }

    fn create_record(
        &mut self,
        tracker: &mut MedicationTracker,
        selected_date: NaiveDate,
        taken: bool,
    ) -> bool {
        let name = self.name.trim();
        if name.is_empty() {
            self.warning = Some("Name cannot be empty.".into());
            return false;
        }

        let dose = match self.dose.parse::<f32>() {
            Ok(dose) if dose.is_finite() && dose > 0.0 => dose,
            _ => {
                self.warning = Some("Dose must be greater than zero.".into());
                return false;
            }
        };

        let hour = match self.hour.parse::<u32>() {
            Ok(hour) if hour <= 23 => hour,
            _ => {
                self.warning = Some("Hour must be between 0 and 23.".into());
                return false;
            }
        };
        let minute = match self.minute.parse::<u32>() {
            Ok(minute) if minute <= 59 => minute,
            _ => {
                self.warning = Some("Minute must be between 0 and 59.".into());
                return false;
            }
        };

        let Some(local_datetime) = selected_date
            .and_hms_opt(hour, minute, 0)
            .and_then(|datetime| Local.from_local_datetime(&datetime).single())
        else {
            self.warning = Some("That local time is not available.".into());
            return false;
        };
        let record_time = local_datetime.with_timezone(&Utc);
        let record_id =
            tracker.insert_one_time_record(name.to_owned(), dose, self.dose_type, record_time);
        if taken {
            tracker.mark_as_taken_at(&record_id, record_time);
        }
        self.close();
        true
    }

    pub fn close(&mut self) {
        self.open = false;
        self.clear_fields();
    }

    fn clear_fields(&mut self) {
        self.name.clear();
        self.dose.clear();
        self.dose_type = DoseType::Mg;
        self.hour.clear();
        self.minute.clear();
        self.warning = None;
    }

    fn form_overlay(&self, selected_date: NaiveDate) -> Element<'_, Message> {
        let header = row![
            text("One-Time Record").size(32).width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .on_press(Message::Close)
                .style(style::time::button::overlay_close_button)
        ]
        .align_y(alignment::Vertical::Center);

        let name_field = column![
            text("Name").size(16),
            text_input("Enter record name...", &self.name).on_input(Message::NameChange),
        ]
        .spacing(8);

        let dose_field = column![
            text("Dose").size(16),
            text_input("1", &self.dose).on_input(Message::DoseChange),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let dose_type_field = column![
            text("Unit Type").size(16),
            pick_list(
                vec![DoseType::Mg, DoseType::Mcg, DoseType::Ml, DoseType::Unit],
                Some(self.dose_type),
                Message::DoseTypeChange,
            )
            .width(Fill),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let time_row = row![
            column![
                text("Hour").size(16),
                text_input("HH", &self.hour)
                    .on_input(Message::HourChange)
                    .size(20),
            ]
            .spacing(8)
            .width(FillPortion(1)),
            text(":").size(24),
            column![
                text("Minute").size(16),
                text_input("MM", &self.minute)
                    .on_input(Message::MinuteChange)
                    .size(20),
            ]
            .spacing(8)
            .width(FillPortion(1)),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        let mut form = column![
            name_field,
            row![dose_field, dose_type_field].spacing(20),
            text(format!("Date: {}", selected_date.format("%d-%m-%Y"))).size(16),
            time_row,
        ]
        .spacing(20);

        if let Some(warning) = &self.warning {
            form = form.push(text(warning).size(13).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

        let take_now = button("Take Now")
            .style(style::time::button::add_button)
            .padding([15, 30])
            .on_press_maybe(
                (selected_date == Local::now().date_naive()).then_some(Message::TakeNow),
            );
        let buttons = row![
            button("Set Alarm")
                .style(style::time::button::add_button)
                .padding([15, 30])
                .on_press(Message::SetAlarm),
            take_now,
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);

        let panel_content = column![header, form, container(column![]).height(Fill), buttons]
            .spacing(20)
            .padding(Padding::new(30.0))
            .height(Fill);

        let inner_panel = container(panel_content)
            .style(overlay_panel_container)
            .width(FillPortion(6))
            .height(FillPortion(6));

        column![
            container(column![]).height(FillPortion(1)),
            row![
                container(row![]).width(FillPortion(1)),
                inner_panel,
                container(row![]).width(FillPortion(1)),
            ]
            .height(FillPortion(5)),
            container(column![]).height(FillPortion(1)),
        ]
        .width(Fill)
        .height(Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::medication::occurrencestatus::OccurrenceStatus;

    #[test]
    fn invalid_one_time_form_does_not_insert_a_record() {
        let mut panel = OneTimeRecordPanel::new();
        let mut tracker = MedicationTracker::new();
        panel.open = true;
        panel.name = "As-needed medicine".into();
        panel.dose = "0".into();
        panel.hour = "12".into();
        panel.minute = "30".into();

        assert!(!panel.update(&mut tracker, Local::now().date_naive(), Message::SetAlarm));
        assert!(tracker.records.is_empty());
        assert!(panel.warning.is_some());
    }

    #[test]
    fn take_now_inserts_taken_record_at_entered_local_time() {
        let mut panel = OneTimeRecordPanel::new();
        let mut tracker = MedicationTracker::new();
        panel.open = true;
        panel.name = "As-needed medicine".into();
        panel.dose = "2.5".into();
        panel.dose_type = DoseType::Ml;
        panel.hour = "08".into();
        panel.minute = "05".into();
        let selected_date = Local::now().date_naive();

        assert!(panel.update(&mut tracker, selected_date, Message::TakeNow));
        assert!(matches!(
            tracker.records[0].occurrence_status,
            OccurrenceStatus::Taken { .. }
        ));
        assert_eq!(
            tracker.records[0].time.with_timezone(&Local).date_naive(),
            selected_date
        );
    }
}
