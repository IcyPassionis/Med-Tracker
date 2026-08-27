use crate::application::medication::record::Record;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::panel::home::reschedulepanel::ReschedulePanel;
use crate::ui::style::alarm::button::{alarm_action_button, alarm_take_button};
use crate::ui::style::alarm::container::{alarm_panel_container, medication_item_container};
use chrono::Local;
use ice::widget::{Space, button, column, container, row, scrollable, stack, text};
use ice::{Element, Length};
use iced as ice;

pub struct AlarmUI {
    pub alarming_records: Vec<String>,
    reschedule_panel: ReschedulePanel,
}
impl AlarmUI {
    pub fn new() -> Self {
        Self {
            alarming_records: Vec::new(),
            reschedule_panel: ReschedulePanel::new(),
        }
    }

    pub fn view<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let records: Vec<&Record> = self
            .alarming_records
            .iter()
            .filter_map(|id| tracker.records.iter().find(|r| &r.id == id))
            .collect();

        let inner = if records.is_empty() {
            column![text("No alarms")].into()
        } else if records.len() == 1 {
            self.single_record_content(tracker, records[0])
        } else {
            self.multiple_records_content(tracker, &records)
        };

        let base = container(
            container(inner)
                .max_width(1000)
                .max_height(640)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(alarm_panel_container)
                .padding(30),
        )
        .center(Length::Fill);

        if let Some(overlay) = self.reschedule_panel.view() {
            stack![base, overlay.map(Message::Reschedule)]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            base.into()
        }
    }

    fn single_record_content<'a>(
        &self,
        tracker: &'a MedicationTracker,
        record: &'a Record,
    ) -> Element<'a, Message> {
        let (med_name, dose_text, record_label) = if let Some(data) = &record.one_time {
            (
                data.name.as_str(),
                format!("{} {}", data.dose, data.dose_type),
                "One-Time Record",
            )
        } else {
            let medication = tracker
                .medications
                .iter()
                .find(|m| m.id == record.medication_id);
            let med_name = medication.map(|m| m.name.as_str()).unwrap_or("Unknown");
            let schedule = medication
                .and_then(|med| med.schedules.iter().find(|s| s.id == record.schedule_id));
            let dose = schedule.map(|s| s.dose).unwrap_or(0.0);
            (med_name, format!("{} mg", dose), "Medication")
        };
        let time = record
            .time
            .with_timezone(&Local)
            .format("%H:%M")
            .to_string();
        let schedule_time_text = format!("{} - {}", time, record_label);
        column![
            container(
                text(schedule_time_text)
                    .size(24)
                    .style(|theme: &ice::Theme| {
                        let palette = theme.extended_palette();
                        ice::widget::text::Style {
                            color: Some(palette.background.strong.text),
                        }
                    })
            )
            .padding(ice::Padding {
                top: 35.0,
                right: 0.0,
                bottom: 25.0,
                left: 0.0
            })
            .center_x(Length::Fill),
            column![text(med_name).size(32), text(dose_text).size(16),]
                .spacing(20)
                .align_x(ice::alignment::Horizontal::Center),
            container("").height(Length::Fill),
            column![
                button(
                    container(text("Take Medication"))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                )
                .style(alarm_take_button)
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .on_press(Message::MarkTaken(record.id.clone())),
                button(
                    container(text("Skipped"))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                )
                .style(alarm_action_button)
                .width(Length::Fill)
                .height(Length::FillPortion(1))
                .on_press(Message::MarkSkipped(record.id.clone())),
                container(
                    button(
                        container(text("Reschedule"))
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                    )
                    .style(alarm_action_button)
                    .width(Length::Fill)
                    .height(Length::FillPortion(1))
                    .on_press(Message::MarkRescheduled(record.id.clone()))
                )
                .center_x(Length::Fill),
            ]
            .max_width(500)
            .spacing(25)
            .width(Length::Fill),
        ]
        .align_x(ice::alignment::Horizontal::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn multiple_records_content<'a>(
        &self,
        tracker: &'a MedicationTracker,
        records: &[&'a Record],
    ) -> Element<'a, Message> {
        let schedule_time_text = if let Some(first_record) = records.first() {
            let time = first_record
                .time
                .with_timezone(&Local)
                .format("%H:%M")
                .to_string();
            format!("{} - Medication", time)
        } else {
            String::from("Medication")
        };
        let count = records.len();
        let header_text = format!("{} Medications", count);
        let record_ids: Vec<String> = records.iter().map(|record| record.id.clone()).collect();
        let mut records_list = column![].spacing(20);
        for record in records {
            let (med_name, dose_text) = if let Some(data) = &record.one_time {
                (
                    data.name.as_str(),
                    format!("{} {}", data.dose, data.dose_type),
                )
            } else {
                let medication = tracker
                    .medications
                    .iter()
                    .find(|m| m.id == record.medication_id);
                let med_name = medication.map(|m| m.name.as_str()).unwrap_or("Unknown");
                let schedule = medication
                    .and_then(|med| med.schedules.iter().find(|s| s.id == record.schedule_id));
                let dose = schedule.map(|s| s.dose).unwrap_or(0.0);
                (med_name, format!("{} mg", dose))
            };
            let medication_container = container(
                row![
                    column![text(med_name).size(22), text(dose_text).size(16),].spacing(10),
                    Space::new().width(Length::Fill),
                    row![
                        button(
                            container(text("Take Now"))
                                .center_x(Length::Fill)
                                .center_y(Length::Fill)
                                .padding(10)
                        )
                        .style(alarm_take_button)
                        .width(Length::Shrink)
                        .on_press(Message::MarkTaken(record.id.clone())),
                        button(
                            container(text("Skip"))
                                .center_x(Length::Fill)
                                .center_y(Length::Fill)
                                .padding(10)
                        )
                        .style(alarm_action_button)
                        .width(Length::Shrink)
                        .on_press(Message::MarkSkipped(record.id.clone())),
                        button(
                            container(text("Reschedule"))
                                .center_x(Length::Fill)
                                .center_y(Length::Fill)
                                .padding(10)
                        )
                        .style(alarm_action_button)
                        .width(Length::Shrink)
                        .on_press(Message::MarkRescheduled(record.id.clone())),
                    ]
                    .spacing(10),
                ]
                .spacing(30)
                .align_y(ice::alignment::Vertical::Center)
                .padding(40),
            )
            .style(medication_item_container)
            .width(Length::Fill);

            records_list = records_list.push(medication_container);
        }
        column![
            container(
                text(schedule_time_text)
                    .size(24)
                    .style(|theme: &ice::Theme| {
                        let palette = theme.extended_palette();
                        ice::widget::text::Style {
                            color: Some(palette.background.strong.text),
                        }
                    })
            )
            .padding(ice::Padding {
                top: 35.0,
                right: 0.0,
                bottom: 25.0,
                left: 0.0
            })
            .center_x(Length::Fill),
            container(text(header_text).size(32))
                .center_x(Length::Fill)
                .padding(ice::Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 20.0,
                    left: 0.0
                }),
            scrollable(records_list).height(Length::Fill),
            row![
                button(container(text("Take all")).center_x(Length::Fill))
                    .style(alarm_take_button)
                    .width(Length::Fill)
                    .padding(12)
                    .on_press(Message::MarkAllTaken(record_ids.clone())),
                button(container(text("Skip all")).center_x(Length::Fill))
                    .style(alarm_action_button)
                    .width(Length::Fill)
                    .padding(12)
                    .on_press(Message::MarkAllSkipped(record_ids.clone())),
                button(container(text("Reschedule all")).center_x(Length::Fill))
                    .style(alarm_action_button)
                    .width(Length::Fill)
                    .padding(12)
                    .on_press(Message::MarkAllRescheduled(record_ids)),
            ]
            .spacing(15)
            .width(Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn update(&mut self, tracker: &mut MedicationTracker, message: Message) {
        match message {
            Message::MarkTaken(record_id) => {
                tracker.mark_as_taken(&record_id);
                self.remove_record(&record_id);
            }
            Message::MarkSkipped(record_id) => {
                tracker.mark_as_skipped(&record_id);
                self.remove_record(&record_id);
            }
            Message::MarkAllTaken(record_ids) => {
                for record_id in &record_ids {
                    tracker.mark_as_taken(record_id);
                }
                self.remove_records(&record_ids);
            }
            Message::MarkAllSkipped(record_ids) => {
                for record_id in &record_ids {
                    tracker.mark_as_skipped(record_id);
                }
                self.remove_records(&record_ids);
            }
            Message::MarkRescheduled(record_id) => {
                if let Some(record) = tracker.records.iter().find(|r| r.id == record_id) {
                    self.reschedule_panel.open(record_id, record.time);
                }
            }
            Message::MarkAllRescheduled(record_ids) => {
                if let Some(record) = record_ids
                    .iter()
                    .find_map(|id| tracker.records.iter().find(|record| record.id == *id))
                {
                    self.reschedule_panel.open_many(record_ids, record.time);
                }
            }
            Message::Reschedule(msg) => {
                if let Some(rescheduled_ids) = self.reschedule_panel.update(tracker, msg) {
                    self.remove_records(&rescheduled_ids);
                }
            }
        }
    }

    fn remove_record(&mut self, record_id: &str) {
        self.alarming_records.retain(|id| id != record_id);
    }

    fn remove_records(&mut self, record_ids: &[String]) {
        self.alarming_records
            .retain(|id| !record_ids.iter().any(|record_id| record_id == id));
    }

    pub fn is_active(&self) -> bool {
        !self.alarming_records.is_empty()
    }

    pub fn add_alarming_record(&mut self, record_id: String) {
        if !self.alarming_records.contains(&record_id) {
            self.alarming_records.push(record_id);
        }
    }

    pub fn set_section_to_main(&mut self) {}
}

#[derive(Debug, Clone)]
pub enum Message {
    MarkTaken(String),
    MarkSkipped(String),
    MarkAllTaken(Vec<String>),
    MarkAllSkipped(Vec<String>),
    MarkRescheduled(String),
    MarkAllRescheduled(Vec<String>),
    Reschedule(crate::ui::panel::home::reschedulepanel::Message),
}
