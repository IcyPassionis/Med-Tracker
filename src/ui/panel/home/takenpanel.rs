use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::scale::text_px;
use crate::ui::style;
use crate::ui::style::alarm::button::{alarm_action_button, alarm_take_button};
use crate::ui::style::medications::container::backdrop;
use crate::ui::style::time::container::overlay_panel_container;
use chrono::{DateTime, TimeZone, Timelike, Utc};
use iced::Length::Fill;
use iced::widget::{Image, button, column, container, row, text, text_input};
use iced::{ContentFit, Element, Padding, alignment};

#[derive(Debug, Clone, PartialEq)]
enum Section {
    Choice,
    ExactTime,
}

pub struct TakenPanel {
    record_id: Option<String>,
    section: Section,
    hour: String,
    minute: String,
    warning: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(String),
    TakeNow,
    PickExactTime,
    HourChange(String),
    MinuteChange(String),
    Confirm,
    Cancel,
}

impl TakenPanel {
    pub fn new() -> Self {
        Self {
            record_id: None,
            section: Section::Choice,
            hour: String::new(),
            minute: String::new(),
            warning: None,
        }
    }

    pub fn open(&mut self, record_id: String) {
        self.record_id = Some(record_id);
        self.section = Section::Choice;
        self.hour.clear();
        self.minute.clear();
        self.warning = None;
    }

    pub fn view(&self) -> Option<Element<'_, Message>> {
        self.record_id.as_ref()?;

        let content = match self.section {
            Section::Choice => self.choice_view(),
            Section::ExactTime => self.exact_time_view(),
        };

        let overlay = container(container(content).center(Fill))
            .style(backdrop)
            .width(Fill)
            .height(Fill);

        Some(overlay.into())
    }

    fn choice_view(&self) -> Element<'_, Message> {
        let header = row![
            text("When did you take your medication?")
                .size(text_px(18.0))
                .width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .style(style::time::button::overlay_close_button)
                .padding(5)
                .on_press(Message::Cancel),
        ]
        .align_y(alignment::Vertical::Center);

        let buttons = column![
            button(
                container(text("Now").size(text_px(16.0)))
                    .center_x(Fill)
                    .center_y(Fill)
            )
            .style(alarm_take_button)
            .width(Fill)
            .height(48)
            .padding(12)
            .on_press(Message::TakeNow),
            button(
                container(text("Pick Exact Time").size(text_px(16.0)))
                    .center_x(Fill)
                    .center_y(Fill)
            )
            .style(alarm_action_button)
            .width(Fill)
            .height(48)
            .padding(12)
            .on_press(Message::PickExactTime),
        ]
        .spacing(12);

        container(
            column![header, buttons]
                .padding(Padding::new(30.0))
                .spacing(20),
        )
        .style(overlay_panel_container)
        .width(320)
        .into()
    }

    fn exact_time_view(&self) -> Element<'_, Message> {
        let header = row![
            text("Select time").size(text_px(22.0)).width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .style(style::time::button::overlay_close_button)
                .padding(5)
                .on_press(Message::Cancel),
        ]
        .align_y(alignment::Vertical::Center);

        let time_row = row![
            text_input("HH", &self.hour)
                .size(text_px(16.0))
                .on_input(Message::HourChange)
                .width(70)
                .size(text_px(24.0)),
            text(":").size(text_px(24.0)),
            text_input("MM", &self.minute)
                .size(text_px(16.0))
                .on_input(Message::MinuteChange)
                .width(70)
                .size(text_px(24.0)),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

        let done_button = button(
            container(text("Done").size(text_px(16.0)))
                .center_x(Fill)
                .center_y(Fill),
        )
        .style(style::time::button::add_button)
        .width(Fill)
        .height(48)
        .padding(12)
        .on_press(Message::Confirm);

        let mut content = column![header, time_row].spacing(20);

        if let Some(warning) = &self.warning {
            let warning_text = text(warning)
                .size(text_px(14.0))
                .style(|theme: &iced::Theme| iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                });
            content = content.push(warning_text);
        }

        content = content.push(done_button);

        container(content.padding(Padding::new(30.0)))
            .style(overlay_panel_container)
            .width(280)
            .into()
    }

    pub fn update(&mut self, tracker: &mut MedicationTracker, msg: Message) -> Option<String> {
        match msg {
            Message::Open(id) => self.handle_open(id),
            Message::TakeNow => self.handle_take_now(tracker),
            Message::PickExactTime => self.handle_pick_exact_time(),
            Message::HourChange(v) => self.handle_hour_change(v),
            Message::MinuteChange(v) => self.handle_minute_change(v),
            Message::Confirm => self.handle_confirm(tracker),
            Message::Cancel => self.handle_cancel(),
        }
    }

    fn handle_open(&mut self, id: String) -> Option<String> {
        self.open(id);
        None
    }

    fn handle_take_now(&mut self, tracker: &mut MedicationTracker) -> Option<String> {
        let id = self.record_id.take()?;
        tracker.mark_as_taken_at(&id, Utc::now());
        self.section = Section::Choice;
        Some(id)
    }

    fn handle_pick_exact_time(&mut self) -> Option<String> {
        self.section = Section::ExactTime;
        None
    }

    fn handle_hour_change(&mut self, v: String) -> Option<String> {
        if v.len() <= 2 && v.chars().all(|c| c.is_ascii_digit()) {
            self.hour = v;
            self.warning = None;
        }
        None
    }

    fn handle_minute_change(&mut self, v: String) -> Option<String> {
        if v.len() <= 2 && v.chars().all(|c| c.is_ascii_digit()) {
            self.minute = v;
            self.warning = None;
        }
        None
    }

    fn handle_confirm(&mut self, tracker: &mut MedicationTracker) -> Option<String> {
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
        let id = self.record_id.take()?;
        let taken_at = chrono::Local::now()
            .date_naive()
            .and_hms_opt(hour, minute, 0)
            .and_then(|dt| chrono::Local.from_local_datetime(&dt).single())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        tracker.mark_as_taken_at(&id, taken_at);
        self.section = Section::Choice;
        self.clear_inputs();
        Some(id)
    }

    fn handle_cancel(&mut self) -> Option<String> {
        self.record_id = None;
        self.section = Section::Choice;
        self.warning = None;
        self.clear_inputs();
        None
    }

    fn clear_inputs(&mut self) {
        self.hour.clear();
        self.minute.clear();
    }
}
