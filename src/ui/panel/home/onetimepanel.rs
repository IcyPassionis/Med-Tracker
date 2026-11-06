use crate::application::medication::dosetype::DoseType;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::scale::text_px;
use crate::ui::style;
use crate::ui::style::medications::container::backdrop;
use crate::ui::style::time::container::overlay_panel_container;
use chrono::{Local, NaiveDate, TimeZone, Utc};
use iced::Length::{Fill, FillPortion};
use iced::widget::{Image, button, column, container, pick_list, row, text, text_input};
use iced::{ContentFit, Element, Padding, Theme, alignment};

#[derive(Debug, Clone, PartialEq)]
pub enum NameChoice {
    New,
    Existing { id: String, label: String },
}

impl std::fmt::Display for NameChoice {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(formatter, "Create New"),
            Self::Existing { label, .. } => write!(formatter, "{label}"),
        }
    }
}

pub struct OneTimeRecordPanel {
    open: bool,
    name: String,
    selected_name: NameChoice,
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
    NameChoiceChange(NameChoice),
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
            selected_name: NameChoice::New,
            dose: String::new(),
            dose_type: DoseType::Mg,
            hour: String::new(),
            minute: String::new(),
            warning: None,
        }
    }

    pub fn view<'a>(
        &'a self,
        tracker: &'a MedicationTracker,
        selected_date: NaiveDate,
    ) -> Option<Element<'a, Message>> {
        if !self.open {
            return None;
        }

        Some(
            container(container(self.form_overlay(tracker, selected_date)).center(Fill))
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
                if matches!(self.selected_name, NameChoice::New) {
                    self.name = value;
                }
                false
            }
            Message::NameChoiceChange(choice) => {
                if matches!(self.selected_name, NameChoice::Existing { .. })
                    && matches!(choice, NameChoice::New)
                {
                    self.name.clear();
                }
                if let NameChoice::Existing { id, .. } = &choice
                    && let Some(medication) = tracker
                        .medications
                        .iter()
                        .find(|medication| medication.id == *id)
                {
                    self.name = medication.name.clone();
                    self.dose_type = medication.dose_type;
                }
                self.selected_name = choice;
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
        self.selected_name = NameChoice::New;
        self.dose.clear();
        self.dose_type = DoseType::Mg;
        self.hour.clear();
        self.minute.clear();
        self.warning = None;
    }

    fn form_overlay<'a>(
        &'a self,
        tracker: &'a MedicationTracker,
        selected_date: NaiveDate,
    ) -> Element<'a, Message> {
        let header = row![
            text("One-Time Record").size(text_px(32.0)).width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .on_press(Message::Close)
                .style(style::time::button::overlay_close_button)
        ]
        .align_y(alignment::Vertical::Center);

        let name_field: Element<'a, Message> = if tracker.medications.is_empty() {
            column![
                text("Name").size(text_px(16.0)),
                text_input("Enter record name...", &self.name)
                    .size(text_px(16.0))
                    .on_input(Message::NameChange),
            ]
            .spacing(8)
            .into()
        } else {
            let mut choices = vec![NameChoice::New];
            choices.extend(
                tracker
                    .medications
                    .iter()
                    .map(|medication| NameChoice::Existing {
                        id: medication.id.clone(),
                        label: if medication.is_archived {
                            format!("{} (archived)", medication.name)
                        } else {
                            medication.name.clone()
                        },
                    }),
            );
            let mut field = column![
                text("Name").size(text_px(16.0)),
                pick_list(
                    choices,
                    Some(self.selected_name.clone()),
                    Message::NameChoiceChange,
                )
                .text_size(text_px(16.0))
                .width(Fill),
            ]
            .spacing(8);
            if matches!(self.selected_name, NameChoice::New) {
                field = field.push(
                    text_input("Enter record name...", &self.name)
                        .size(text_px(16.0))
                        .on_input(Message::NameChange),
                );
            } else {
                field =
                    field.push(text(format!("Recording for {}", self.name)).size(text_px(14.0)));
            }
            field.into()
        };

        let dose_field = column![
            text("Dose").size(text_px(16.0)),
            text_input("1", &self.dose)
                .size(text_px(16.0))
                .on_input(Message::DoseChange),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let dose_type_field = column![
            text("Unit Type").size(text_px(16.0)),
            pick_list(
                vec![DoseType::Mg, DoseType::Mcg, DoseType::Ml, DoseType::Unit],
                Some(self.dose_type),
                Message::DoseTypeChange,
            )
            .text_size(text_px(16.0))
            .width(Fill),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let time_row = row![
            column![
                text("Hour").size(text_px(16.0)),
                text_input("HH", &self.hour)
                    .size(text_px(16.0))
                    .on_input(Message::HourChange)
                    .size(text_px(20.0)),
            ]
            .spacing(8)
            .width(FillPortion(1)),
            text(":").size(text_px(24.0)),
            column![
                text("Minute").size(text_px(16.0)),
                text_input("MM", &self.minute)
                    .size(text_px(16.0))
                    .on_input(Message::MinuteChange)
                    .size(text_px(20.0)),
            ]
            .spacing(8)
            .width(FillPortion(1)),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        let mut form = column![
            name_field,
            row![dose_field, dose_type_field].spacing(20),
            text(format!("Date: {}", selected_date.format("%d-%m-%Y"))).size(text_px(16.0)),
            time_row,
        ]
        .spacing(20);

        if let Some(warning) = &self.warning {
            form = form.push(text(warning).size(text_px(13.0)).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

        let take_now = button(text("Take Now").size(text_px(16.0)))
            .style(style::time::button::add_button)
            .padding([15, 30])
            .on_press_maybe(
                (selected_date == Local::now().date_naive()).then_some(Message::TakeNow),
            );
        let buttons = row![
            button(text("Set Alarm").size(text_px(16.0)))
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
    use crate::application::medication::medication::Medication;
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

    #[test]
    fn selecting_existing_medication_fills_name_without_creating_medication() {
        let mut panel = OneTimeRecordPanel::new();
        let mut tracker = MedicationTracker::new();
        let mut medication = Medication::new("Existing medicine".into(), 10.0);
        medication.dose_type = DoseType::Ml;
        let id = medication.id.clone();
        tracker.medications.push(medication);
        panel.open = true;
        let selected_date = Local::now().date_naive();

        assert!(!panel.update(
            &mut tracker,
            selected_date,
            Message::NameChoiceChange(NameChoice::Existing {
                id,
                label: "Existing medicine".into(),
            }),
        ));
        assert_eq!(panel.name, "Existing medicine");
        assert!(matches!(panel.dose_type, DoseType::Ml));
        assert_eq!(tracker.medications.len(), 1);
    }

    #[test]
    fn new_one_time_name_does_not_create_a_medication() {
        let mut panel = OneTimeRecordPanel::new();
        let mut tracker = MedicationTracker::new();
        let selected_date = Local::now().date_naive();
        panel.open = true;
        panel.update(
            &mut tracker,
            selected_date,
            Message::NameChange("Temporary medicine".into()),
        );
        panel.dose = "1".into();
        panel.hour = "09".into();
        panel.minute = "00".into();

        assert!(panel.update(&mut tracker, selected_date, Message::SetAlarm));
        assert_eq!(tracker.records.len(), 1);
        assert!(tracker.medications.is_empty());
        assert_eq!(
            tracker.records[0].one_time.as_ref().unwrap().name,
            "Temporary medicine"
        );
    }
}
