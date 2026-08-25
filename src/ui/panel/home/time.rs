use crate::application::medication::dosetype::DoseType;
use crate::application::medication::occurrencestatus::OccurrenceStatus;
use crate::application::medication::record::Record;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::{self, button_with_icon};
use crate::ui::style;
use crate::ui::style::medications::container::backdrop;
use crate::ui::style::time::container::{record_status_container, schedule_container};
use chrono::{Datelike, Duration, Local, NaiveDate, Timelike};
use iced::Length::{Fill, FillPortion, Shrink};
use iced::widget::operation;
use iced::widget::{Id, Image, button, column, container, row, scrollable, stack, text};
use iced::{ContentFit, Element, Padding, Task, alignment};

pub struct TimeUI {
    pub selected_date: NaiveDate,
    medication_panel: super::medicationaddpanel::MedicationAddPanel,
    one_time_panel: super::onetimepanel::OneTimeRecordPanel,
    reschedule_panel: super::reschedulepanel::ReschedulePanel,
    taken_panel: super::takenpanel::TakenPanel,
    calendar_scroll_id: Id,
    pub needs_scroll_to_today: bool,
    add_choice_open: bool,
}

impl TimeUI {
    pub fn new() -> TimeUI {
        Self {
            selected_date: Local::now().date_naive(),
            medication_panel: super::medicationaddpanel::MedicationAddPanel::new(),
            one_time_panel: super::onetimepanel::OneTimeRecordPanel::new(),
            reschedule_panel: super::reschedulepanel::ReschedulePanel::new(),
            taken_panel: super::takenpanel::TakenPanel::new(),
            calendar_scroll_id: Id::unique(),
            needs_scroll_to_today: true,
            add_choice_open: false,
        }
    }

    pub fn view<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let main = column![
            self.calendar_part(),
            container(self.main_part(tracker)).height(Fill),
            self.add_panel()
        ];

        let base: Element<Message> = if let Some(overlay) = self.medication_panel.view(tracker) {
            stack![main, overlay.map(Message::MedicationAdd)]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            main.into()
        };

        let base: Element<Message> =
            if let Some(overlay) = self.one_time_panel.view(self.selected_date) {
                stack![base, overlay.map(Message::OneTimeRecord)]
                    .width(Fill)
                    .height(Fill)
                    .into()
            } else {
                base
            };

        let base: Element<Message> = if self.add_choice_open {
            stack![base, self.add_choice_overlay()]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            base
        };

        let base: Element<Message> = if let Some(overlay) = self.reschedule_panel.view() {
            stack![base, overlay.map(Message::Reschedule)]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            base
        };

        if let Some(overlay) = self.taken_panel.view() {
            stack![base, overlay.map(Message::Taken)]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            base
        }
    }

    pub fn update(
        &mut self,
        state: &mut MedicationTracker,
        message: Message,
    ) -> (Task<Message>, bool) {
        let mut one_time_created = false;
        let task = match message {
            Message::SelectDay(date) => {
                self.selected_date = date;
                println!("Current TimeUI Selected Date: {}", self.selected_date);
                Task::none()
            }
            Message::MedicationAdd(msg) => {
                self.medication_panel.update(state, msg);
                Task::none()
            }
            Message::OpenAddChoice => {
                self.add_choice_open = true;
                Task::none()
            }
            Message::CloseAddChoice => {
                self.add_choice_open = false;
                Task::none()
            }
            Message::OpenRegularMedication => {
                self.add_choice_open = false;
                self.medication_panel
                    .update(state, super::medicationaddpanel::Message::Open);
                Task::none()
            }
            Message::OpenOneTimeRecord => {
                self.add_choice_open = false;
                self.one_time_panel.update(
                    state,
                    self.selected_date,
                    super::onetimepanel::Message::Open,
                );
                Task::none()
            }
            Message::OneTimeRecord(msg) => {
                one_time_created = self.one_time_panel.update(state, self.selected_date, msg);
                Task::none()
            }
            Message::DeleteRecord(id) => {
                state.delete_one_time_record(&id);
                Task::none()
            }
            Message::MarkSkipped(id) => {
                state.mark_as_skipped(&id);
                Task::none()
            }
            Message::MarkTakenToggle(id) => {
                state.mark_as_taken(&id);
                Task::none()
            }
            Message::Taken(msg) => {
                self.taken_panel.update(state, msg);
                Task::none()
            }
            Message::Reschedule(msg) => {
                self.reschedule_panel.update(state, msg);
                Task::none()
            }
            Message::ToggleSound(_hour, _minute) => Task::none(),
            Message::ScrollToToday => {
                self.needs_scroll_to_today = false;
                let relative_x = 30.0 / 61.0;
                operation::snap_to(
                    self.calendar_scroll_id.clone(),
                    scrollable::RelativeOffset {
                        x: Some(relative_x),
                        y: Some(0.0),
                    },
                )
            }
            Message::ScrollCalendarLeft => operation::scroll_by(
                self.calendar_scroll_id.clone(),
                scrollable::AbsoluteOffset { x: -400.0, y: 0.0 },
            ),
            Message::ScrollCalendarRight => operation::scroll_by(
                self.calendar_scroll_id.clone(),
                scrollable::AbsoluteOffset { x: 400.0, y: 0.0 },
            ),
        };
        (task, one_time_created)
    }

    pub fn set_section_to_main(&mut self) {
        self.medication_panel.close();
        self.one_time_panel.close();
        self.add_choice_open = false;
    }

    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
    }

    fn main_part<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let mut grouped: std::collections::BTreeMap<(u32, u32), Vec<&Record>> =
            std::collections::BTreeMap::new();
        for record in &tracker.records {
            if record.time.with_timezone(&Local).date_naive() != self.selected_date {
                continue;
            }
            grouped
                .entry((
                    record.time.with_timezone(&Local).hour(),
                    record.time.with_timezone(&Local).minute(),
                ))
                .or_default()
                .push(record);
        }

        let mut medications_container_list = column![].spacing(20);
        for ((hour, minute), records) in &grouped {
            let mut schedule_container_column = column![].padding([20, 40]).spacing(20);
            let hour_minute = format!("{:02}:{:02}", hour, minute);
            let schedule_label = text(hour_minute).size(32).width(Fill);
            let sound_button = button(button_with_icon!("icons/soundon.png", 32, 10))
                .style(style::time::button::record_action_button)
                .padding(10)
                .on_press(Message::ToggleSound(*hour, *minute));
            let schedule_header =
                row![schedule_label, sound_button].align_y(alignment::Vertical::Center);
            schedule_container_column = schedule_container_column.push(schedule_header);

            let mut medications_list = column![].spacing(10).padding([0, 20]);
            for record in records {
                let Some((name, dose, dose_type)) = self.record_details(tracker, record) else {
                    continue;
                };
                let is_one_time = record.is_one_time();
                let status_icon: Element<'a, Message> = match &record.occurrence_status {
                    OccurrenceStatus::Taken { .. } => Image::new("icons/check.png")
                        .content_fit(ContentFit::Cover)
                        .width(42)
                        .height(42)
                        .into(),
                    OccurrenceStatus::Skipped { .. } | OccurrenceStatus::Missed => {
                        Image::new("icons/cross.png")
                            .content_fit(ContentFit::Cover)
                            .width(42)
                            .height(42)
                            .into()
                    }
                    OccurrenceStatus::Pending => column![].width(42).height(42).into(),
                };
                let status_container = container(status_icon).style(record_status_container);
                let status_text = match &record.occurrence_status {
                    OccurrenceStatus::Taken { taken_at } => {
                        let local_time = taken_at.with_timezone(&Local);
                        let record_date = record.time.with_timezone(&Local).date_naive();
                        if local_time.date_naive() == record_date {
                            format!(
                                "Taken at {:02}:{:02}",
                                local_time.hour(),
                                local_time.minute()
                            )
                        } else {
                            format!("Taken at {}", local_time.format("%d-%m-%Y %H:%M"))
                        }
                    }
                    OccurrenceStatus::Skipped { reason: None } => String::from("Skipped"),
                    OccurrenceStatus::Skipped { reason: Some(r) } => r.clone(),
                    OccurrenceStatus::Pending | OccurrenceStatus::Missed => {
                        if is_one_time {
                            format!("{} {}", dose, dose_type)
                        } else {
                            match tracker.days_left(&record.medication_id) {
                                Some(days) => {
                                    format!("{} {} · {} days left", dose, dose_type, days)
                                }
                                None => format!("{} {}", dose, dose_type),
                            }
                        }
                    }
                };
                let medication_info = column![text(name).size(22), text(status_text).size(16)]
                    .spacing(5)
                    .width(Fill);
                let second_action: Element<'a, Message> = if is_one_time {
                    button(button_with_icon!("icons/trash.png", 32, 10))
                        .style(style::time::button::record_action_button)
                        .padding(10)
                        .on_press(Message::DeleteRecord(record.id.clone()))
                        .into()
                } else {
                    button(button_with_icon!("icons/cross.png", 32, 10))
                        .style(style::time::button::record_action_button)
                        .padding(10)
                        .on_press(Message::MarkSkipped(record.id.clone()))
                        .into()
                };
                let action_buttons = row![
                    button(button_with_icon!("icons/check.png", 32, 10))
                        .style(style::time::button::record_action_button)
                        .padding(10)
                        .on_press(
                            if matches!(record.occurrence_status, OccurrenceStatus::Taken { .. }) {
                                Message::MarkTakenToggle(record.id.clone())
                            } else {
                                Message::Taken(super::takenpanel::Message::Open(record.id.clone()))
                            }
                        ),
                    second_action,
                    button(button_with_icon!("icons/clock.png", 32, 10))
                        .style(style::time::button::record_action_button)
                        .padding(10)
                        .on_press(Message::Reschedule(super::reschedulepanel::Message::Open(
                            record.id.clone()
                        ))),
                ]
                .spacing(30)
                .align_y(alignment::Vertical::Center);

                let medication_row = row![status_container, medication_info, action_buttons]
                    .align_y(alignment::Vertical::Center)
                    .spacing(20);

                medications_list = medications_list.push(medication_row);
            }
            schedule_container_column = schedule_container_column
                .push(medications_list)
                .width(FillPortion(1));
            let schedule_container = container(schedule_container_column).style(schedule_container);
            medications_container_list = medications_container_list.push(schedule_container);
        }
        scrollable(container(medications_container_list.max_width(750)).center_x(Fill)).into()
    }

    fn add_panel<'a>(&self) -> Element<'a, Message> {
        container(
            button(macros::button_with_icon_text!("Add Med", "icons/plus.png"))
                .style(style::time::button::add_button)
                .padding([20, 100])
                .on_press(Message::OpenAddChoice),
        )
        .center_x(Fill)
        .height(Shrink)
        .padding(Padding::new(0.0).bottom(20))
        .into()
    }

    fn record_details<'a>(
        &self,
        tracker: &'a MedicationTracker,
        record: &'a Record,
    ) -> Option<(&'a str, f32, DoseType)> {
        if let Some(one_time) = &record.one_time {
            return Some((&one_time.name, one_time.dose, one_time.dose_type));
        }

        let medication = tracker
            .medications
            .iter()
            .find(|medication| medication.id == record.medication_id)?;
        let dose = medication
            .schedules
            .iter()
            .find(|schedule| schedule.id == record.schedule_id)
            .map(|schedule| schedule.dose)
            .unwrap_or(0.0);
        Some((&medication.name, dose, medication.dose_type))
    }

    fn add_choice_overlay(&self) -> Element<'_, Message> {
        let header = row![
            text("Add Record").size(28).width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .style(style::time::button::overlay_close_button)
                .padding(5)
                .on_press(Message::CloseAddChoice),
        ]
        .align_y(alignment::Vertical::Center);

        let options = column![
            button(container(text("Regular Medication")).center_x(Fill))
                .style(style::time::button::add_button)
                .padding([15, 30])
                .width(Fill)
                .on_press(Message::OpenRegularMedication),
            button(container(text("One-Time Record")).center_x(Fill))
                .style(style::time::button::add_button)
                .padding([15, 30])
                .width(Fill)
                .on_press(Message::OpenOneTimeRecord),
        ]
        .spacing(15);

        let panel = container(column![header, options].spacing(20).padding(30))
            .style(crate::ui::style::time::container::overlay_panel_container)
            .width(380);

        container(container(panel).center(Fill))
            .style(backdrop)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn calendar_part<'a>(&self) -> Element<'a, Message> {
        let today = Local::now().date_naive();
        let mut days = row![].spacing(8);
        let days_back = 30i64;
        let days_forward = 30i64;
        for i in -days_back..=days_forward {
            let date = today + Duration::days(i);
            let weekday = match date.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };
            let day_month = format!("{}/{}", date.day(), date.month());
            let label = column![text(weekday).center(), text(day_month).center()].spacing(20);
            let is_selected = date == self.selected_date;
            let is_today = date == today;
            days = days.push(
                button(label)
                    .style(style::time::button::calendar_button(is_selected, is_today))
                    .padding([20, 30])
                    .width(130)
                    .on_press(Message::SelectDay(date)),
            );
        }
        let cal_scrollable = scrollable(days)
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::hidden(),
            ))
            .id(self.calendar_scroll_id.clone());

        let left_arrow = button(text("<").size(20))
            .style(style::time::button::record_action_button)
            .padding([10, 14])
            .on_press(Message::ScrollCalendarLeft);
        let right_arrow = button(text(">").size(20))
            .style(style::time::button::record_action_button)
            .padding([10, 14])
            .on_press(Message::ScrollCalendarRight);

        row![
            left_arrow,
            container(cal_scrollable.width(Fill).height(Shrink)).width(Fill),
            right_arrow
        ]
        .align_y(iced::alignment::Vertical::Center)
        .spacing(8)
        .width(Fill)
        .padding(Padding::new(0.0).bottom(20))
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectDay(NaiveDate),
    MedicationAdd(super::medicationaddpanel::Message),
    OneTimeRecord(super::onetimepanel::Message),
    OpenAddChoice,
    CloseAddChoice,
    OpenRegularMedication,
    OpenOneTimeRecord,
    DeleteRecord(String),
    Taken(super::takenpanel::Message),
    MarkSkipped(String),
    MarkTakenToggle(String),
    Reschedule(super::reschedulepanel::Message),
    ToggleSound(u32, u32),
    ScrollToToday,
    ScrollCalendarLeft,
    ScrollCalendarRight,
}
