use crate::application::medication::dosetype::DoseType;
use crate::application::medication::medication::Medication;
use crate::application::medication::periodtype::PeriodType;
use crate::application::medication::schedule::Schedule;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::style;
use crate::ui::style::time::container::overlay_panel_container;
use chrono::Weekday;
use iced::ContentFit;
use iced::Length::{Fill, FillPortion};
use iced::widget::{
    Image, button, column, container, pick_list, row, scrollable, stack, text, text_input,
};
use iced::{self as ice, Background, Color, Element, Theme, alignment};

#[derive(Debug, Clone)]
pub enum Section {
    Hidden,
    AddMedication,
    ScheduleList,
    AddSchedule,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleMode {
    Interval,
    Weekdays,
}

pub struct MedicationAddPanel {
    pub section: Section,
    medication_name: String,
    medication_stock: String,
    medication_pill_dose: String,
    medication_dose_type: DoseType,
    current_medication_id: Option<String>,
    editing_schedule_id: Option<String>,
    schedule_mode: ScheduleMode,
    schedule_period_type: PeriodType,
    schedule_hour: String,
    schedule_minute: String,
    schedule_period_time: String,
    schedule_dose: String,
    schedule_error: Option<String>,
    selected_weekdays: Vec<Weekday>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    MedicationNameChange(String),
    MedicationStockChange(String),
    MedicationPillDoseChange(String),
    MedicationDoseTypeChange(DoseType),
    AddMedication,
    OpenNewSchedule,
    EditSchedule(String),
    BackToList,
    ScheduleModeChange(ScheduleMode),
    SchedulePeriodTypeChange(PeriodType),
    ToggleWeekday(Weekday),
    ScheduleHourChange(String),
    ScheduleMinuteChange(String),
    SchedulePeriodTimeChange(String),
    ScheduleDoseChange(String),
    SaveSchedule,
    DeleteSchedule(String),
    Done,
}

impl MedicationAddPanel {
    pub fn new() -> Self {
        Self {
            section: Section::Hidden,
            medication_name: String::new(),
            medication_stock: String::new(),
            medication_pill_dose: String::new(),
            medication_dose_type: DoseType::Mg,
            current_medication_id: None,
            editing_schedule_id: None,
            schedule_mode: ScheduleMode::Interval,
            schedule_period_type: PeriodType::Daily,
            schedule_hour: String::new(),
            schedule_minute: String::new(),
            schedule_period_time: String::new(),
            schedule_dose: String::new(),
            schedule_error: None,
            selected_weekdays: vec![],
        }
    }

    pub fn close(&mut self) {
        self.section = Section::Hidden;
        self.reset_all();
    }

    pub fn view<'a>(&self, tracker: &'a MedicationTracker) -> Option<Element<'a, Message>> {
        match self.section {
            Section::Hidden => None,
            Section::AddMedication => Some(
                stack![self.backdrop(), self.medication_overlay()]
                    .width(Fill)
                    .height(Fill)
                    .into(),
            ),
            Section::ScheduleList => Some(
                stack![self.backdrop(), self.schedule_list_overlay(tracker)]
                    .width(Fill)
                    .height(Fill)
                    .into(),
            ),
            Section::AddSchedule => Some(
                stack![self.backdrop(), self.add_schedule_overlay(tracker)]
                    .width(Fill)
                    .height(Fill)
                    .into(),
            ),
        }
    }

    pub fn update(&mut self, state: &mut MedicationTracker, message: Message) {
        match message {
            Message::Open => self.section = Section::AddMedication,
            Message::Close => self.close(),
            Message::MedicationNameChange(v) => self.medication_name = v,
            Message::MedicationStockChange(v) => {
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    let dots = v.chars().filter(|c| *c == '.').count();
                    if dots <= 1 && !v.starts_with('-') {
                        self.medication_stock = v;
                    }
                }
            }
            Message::MedicationPillDoseChange(v) => {
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    let dots = v.chars().filter(|c| *c == '.').count();
                    if dots <= 1 {
                        self.medication_pill_dose = v;
                    }
                }
            }
            Message::MedicationDoseTypeChange(v) => self.medication_dose_type = v,
            Message::AddMedication => {
                let stock: f32 = self.medication_stock.parse().unwrap_or(0.0);
                let pill_dose: f32 = self.medication_pill_dose.parse().unwrap_or(1.0_f32).max(0.0);
                let mut medication = Medication::new(self.medication_name.clone(), stock);
                medication.pill_dose = pill_dose;
                medication.dose_type = self.medication_dose_type;
                self.current_medication_id = Some(medication.id.clone());
                state.medications.push(medication);
                self.medication_name = String::new();
                self.medication_stock = String::new();
                self.medication_pill_dose = String::new();
                self.medication_dose_type = DoseType::Mg;
                self.section = Section::ScheduleList;
            }
            Message::OpenNewSchedule => {
                self.reset_schedule_fields();
                self.section = Section::AddSchedule;
            }
            Message::EditSchedule(id) => {
                if let Some(med_id) = &self.current_medication_id {
                    if let Some(med) = state.medications.iter().find(|m| m.id == *med_id) {
                        if let Some(schedule) = med.schedules.iter().find(|s| s.id == id) {
                            self.schedule_hour = schedule.time[0].to_string();
                            self.schedule_minute = schedule.time[1].to_string();
                            self.schedule_period_time = schedule.period_time.to_string();
                            self.schedule_period_type =
                                schedule.period_type.unwrap_or(PeriodType::Daily);
                            self.schedule_mode = if schedule.week_day.is_some() {
                                ScheduleMode::Weekdays
                            } else {
                                ScheduleMode::Interval
                            };
                            self.selected_weekdays = schedule.week_day.clone().unwrap_or_default();
                            self.schedule_dose = schedule.dose.to_string();
                            self.editing_schedule_id = Some(id);
                        }
                    }
                }
                self.section = Section::AddSchedule;
            }
            Message::BackToList => {
                self.schedule_error = None;
                self.reset_schedule_fields();
                self.section = Section::ScheduleList;
            }
            Message::ScheduleModeChange(mode) => self.schedule_mode = mode,
            Message::SchedulePeriodTypeChange(pt) => self.schedule_period_type = pt,
            Message::ToggleWeekday(day) => {
                if let Some(pos) = self.selected_weekdays.iter().position(|d| *d == day) {
                    self.selected_weekdays.remove(pos);
                } else {
                    self.selected_weekdays.push(day);
                }
            }
            Message::ScheduleHourChange(v) => {
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit()) {
                    self.schedule_hour = v;
                }
            }
            Message::ScheduleMinuteChange(v) => {
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit()) {
                    self.schedule_minute = v;
                }
            }
            Message::SchedulePeriodTimeChange(v) => {
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit()) {
                    self.schedule_period_time = v;
                }
            }
            Message::ScheduleDoseChange(v) => {
                self.schedule_error = None;
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    let dots = v.chars().filter(|c| *c == '.').count();
                    if dots <= 1 && !v.starts_with('-') {
                        self.schedule_dose = v;
                    }
                }
            }
            Message::DeleteSchedule(id) => {
                if let Some(med_id) = &self.current_medication_id {
                    if let Some(med) = state.medications.iter_mut().find(|m| m.id == *med_id) {
                        med.schedules.retain(|s| s.id != id);
                    }
                }
            }
            Message::SaveSchedule => {
                let hour: u8 = self.schedule_hour.parse().unwrap_or(0);
                let minute: u8 = self.schedule_minute.parse().unwrap_or(0);
                let period_time: u8 = self.schedule_period_time.parse().unwrap_or(1);
                let dose: f32 = self.schedule_dose.parse().unwrap_or(0.0);
                if dose <= 0.0 {
                    self.schedule_error = Some("Dose must be greater than zero.".into());
                    return;
                }
                let (period_type, week_day) = match self.schedule_mode {
                    ScheduleMode::Interval => (Some(self.schedule_period_type), None),
                    ScheduleMode::Weekdays => {
                        let days = if self.selected_weekdays.is_empty() {
                            None
                        } else {
                            Some(self.selected_weekdays.clone())
                        };
                        (None, days)
                    }
                };
                if let Some(med_id) = &self.current_medication_id {
                    if let Some(med) = state.medications.iter_mut().find(|m| m.id == *med_id) {
                        if let Some(edit_id) = &self.editing_schedule_id {
                            if let Some(s) = med.schedules.iter_mut().find(|s| s.id == *edit_id) {
                                s.time = [hour, minute];
                                s.period_time = period_time;
                                s.period_type = period_type;
                                s.week_day = week_day;
                            }
                        } else {
                            let mut new_schedule =
                                Schedule::new([hour, minute], period_type, period_time, dose);
                            new_schedule.week_day = week_day;
                            med.schedules.push(new_schedule);
                        }
                    }
                }
                self.reset_schedule_fields();
                self.section = Section::ScheduleList;
            }
            Message::Done => {
                self.current_medication_id = None;
                self.section = Section::Hidden;
            }
        }
    }

    fn reset_all(&mut self) {
        self.medication_name = String::new();
        self.medication_stock = String::new();
        self.medication_pill_dose = String::new();
        self.medication_dose_type = DoseType::Mg;
        self.current_medication_id = None;
        self.reset_schedule_fields();
    }

    fn reset_schedule_fields(&mut self) {
        self.editing_schedule_id = None;
        self.schedule_hour = String::new();
        self.schedule_minute = String::new();
        self.schedule_period_time = String::new();
        self.schedule_dose = String::new();
        self.schedule_error = None;
        self.schedule_period_type = PeriodType::Daily;
        self.schedule_mode = ScheduleMode::Interval;
        self.selected_weekdays = vec![];
    }

    fn backdrop<'a>(&self) -> Element<'a, Message> {
        container(column![])
            .width(Fill)
            .height(Fill)
            .style(|_theme: &Theme| ice::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.3))),
                ..Default::default()
            })
            .into()
    }

    fn medication_overlay<'a>(&self) -> Element<'a, Message> {
        let header = row![
            text("New Medication").size(36).width(Fill).center(),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .on_press(Message::Close)
                .style(style::time::button::overlay_close_button)
        ]
        .align_y(alignment::Vertical::Center);

        let name_field = column![
            text("Medication Name").size(16),
            text_input("Enter medication name...", &self.medication_name)
                .on_input(Message::MedicationNameChange),
        ]
        .spacing(8);

        let stock_field = column![
            text("Stock").size(16),
            text_input("0", &self.medication_stock).on_input(Message::MedicationStockChange),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let unit_field = column![
            text("Unit Type").size(16),
            pick_list(
                vec![DoseType::Mg, DoseType::Mcg, DoseType::Ml, DoseType::Unit],
                Some(self.medication_dose_type),
                Message::MedicationDoseTypeChange,
            )
            .width(Fill),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let pill_dose_field = column![
            text("Pill Dose").size(16),
            text_input("1", &self.medication_pill_dose)
                .on_input(Message::MedicationPillDoseChange),
        ]
        .spacing(8)
        .width(FillPortion(1));

        let form = column![
            name_field,
            row![stock_field, unit_field].spacing(20),
            pill_dose_field,
        ]
        .spacing(20);

        let add_btn = container(
            button("Add")
                .style(style::time::button::add_button)
                .padding([15, 60])
                .on_press(Message::AddMedication),
        )
        .center_x(Fill);

        let panel_content = column![header, form, container(column![]).height(Fill), add_btn]
            .spacing(20)
            .padding(30)
            .height(Fill);

        self.centered_panel(panel_content.into())
    }

    fn schedule_list_overlay<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let med_name = self
            .current_medication_id
            .as_deref()
            .and_then(|id| tracker.medications.iter().find(|m| m.id == id))
            .map(|m| m.name.as_str())
            .unwrap_or("Medication");

        let header = row![
            text(med_name).size(28).width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .on_press(Message::Done)
                .style(style::time::button::overlay_close_button)
        ]
        .align_y(alignment::Vertical::Center);

        let schedules = self
            .current_medication_id
            .as_deref()
            .and_then(|id| tracker.medications.iter().find(|m| m.id == id))
            .map(|m| m.schedules.as_slice())
            .unwrap_or(&[]);

        let dose_type_str = self
            .current_medication_id
            .as_deref()
            .and_then(|id| tracker.medications.iter().find(|m| m.id == id))
            .map(|m| format!("{}", m.dose_type))
            .unwrap_or_default();

        let schedule_list: Element<'a, Message> = if schedules.is_empty() {
            container(text("No schedules yet").size(14).center())
                .width(Fill)
                .padding([20, 0])
                .into()
        } else {
            let mut rows = column![].spacing(8);
            for schedule in schedules {
                let time_str = format!("{:02}:{:02}", schedule.time[0], schedule.time[1]);
                let dose_str = format!("{} {}", schedule.dose, dose_type_str);
                let period_str = match &schedule.period_type {
                    Some(pt) => format!("{}, every {} unit(s)", pt, schedule.period_time),
                    None => String::from("Weekdays"),
                };
                let row_content = row![
                    text(time_str).size(16).width(FillPortion(1)),
                    text(dose_str).size(14).width(FillPortion(1)),
                    text(period_str).size(14).width(Fill),
                ]
                .spacing(10)
                .align_y(alignment::Vertical::Center);

                let schedule_id = schedule.id.clone();
                let schedule_row = row![
                    button(row_content)
                        .style(style::time::button::add_button)
                        .padding([12, 20])
                        .width(Fill)
                        .on_press(Message::EditSchedule(schedule_id.clone())),
                    button(button_with_icon!("icons/cross.png", 20, 6))
                        .style(style::time::button::overlay_close_button)
                        .padding(8)
                        .on_press(Message::DeleteSchedule(schedule_id)),
                ]
                .spacing(8)
                .align_y(alignment::Vertical::Center);

                rows = rows.push(schedule_row);
            }
            scrollable(rows).into()
        };

        let add_schedule_btn = container(
            button("+ Add New Schedule")
                .style(style::time::button::add_button)
                .padding([12, 30])
                .on_press(Message::OpenNewSchedule),
        )
        .center_x(Fill);

        let done_btn = container(
            button("Done")
                .style(style::time::button::add_button)
                .padding([15, 60])
                .on_press(Message::Done),
        )
        .center_x(Fill);

        let panel_content = column![
            header,
            schedule_list,
            container(column![]).height(Fill),
            add_schedule_btn,
            done_btn
        ]
        .spacing(20)
        .padding(30)
        .height(Fill);

        self.centered_panel(panel_content.into())
    }

    fn add_schedule_overlay<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let is_editing = self.editing_schedule_id.is_some();
        let title = if is_editing {
            "Edit Schedule"
        } else {
            "New Schedule"
        };

        let med_name = self
            .current_medication_id
            .as_deref()
            .and_then(|id| tracker.medications.iter().find(|m| m.id == id))
            .map(|m| m.name.as_str())
            .unwrap_or("Medication");

        let header = row![
            column![text(med_name).size(14), text(title).size(28)]
                .spacing(2)
                .width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .on_press(Message::BackToList)
                .style(style::time::button::overlay_close_button)
        ]
        .align_y(alignment::Vertical::Center);

        // Mode toggle
        let interval_active = self.schedule_mode == ScheduleMode::Interval;
        let mode_toggle = row![
            button("Interval")
                .style(style::time::button::calendar_button(interval_active, false))
                .padding([10, 20])
                .on_press(Message::ScheduleModeChange(ScheduleMode::Interval)),
            button("Weekdays")
                .style(style::time::button::calendar_button(!interval_active, false))
                .padding([10, 20])
                .on_press(Message::ScheduleModeChange(ScheduleMode::Weekdays)),
        ]
        .spacing(8);

        // Time inputs (always shown)
        let time_row = row![
            column![
                text("Hour").size(16),
                text_input("00", &self.schedule_hour).on_input(Message::ScheduleHourChange),
            ]
            .spacing(8)
            .width(FillPortion(1)),
            text(":").size(24),
            column![
                text("Minute").size(16),
                text_input("00", &self.schedule_minute).on_input(Message::ScheduleMinuteChange),
            ]
            .spacing(8)
            .width(FillPortion(1)),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        let dose_type_str = self
            .current_medication_id
            .as_deref()
            .and_then(|id| tracker.medications.iter().find(|m| m.id == id))
            .map(|m| format!("{}", m.dose_type))
            .unwrap_or_default();

        let mut dose_field = column![
            text(format!("Dose ({})", dose_type_str)).size(16),
            text_input("1", &self.schedule_dose).on_input(Message::ScheduleDoseChange),
        ]
        .spacing(8);
        if let Some(err) = &self.schedule_error {
            dose_field = dose_field.push(text(err.clone()).size(13).style(|_theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(Color::from_rgb(0.85, 0.2, 0.2)),
                }
            }));
        }

        // Interval-specific fields
        let interval_form: Element<'a, Message> = if interval_active {
            let period_label = match self.schedule_period_type {
                PeriodType::Daily => "Every (days)",
                PeriodType::Weekly => "Every (weeks)",
                PeriodType::Monthly => "Every (months)",
                PeriodType::Hourly => "Every (hours)",
            };
            column![
                column![
                    text("Repeats").size(16),
                    pick_list(
                        vec![PeriodType::Daily, PeriodType::Weekly, PeriodType::Monthly],
                        Some(self.schedule_period_type),
                        Message::SchedulePeriodTypeChange,
                    )
                    .width(Fill),
                ]
                .spacing(8),
                column![
                    text(period_label).size(16),
                    text_input("1", &self.schedule_period_time)
                        .on_input(Message::SchedulePeriodTimeChange),
                ]
                .spacing(8),
            ]
            .spacing(20)
            .into()
        } else {
            let days: [(Weekday, &str); 7] = [
                (Weekday::Mon, "Mo"),
                (Weekday::Tue, "Tu"),
                (Weekday::Wed, "We"),
                (Weekday::Thu, "Th"),
                (Weekday::Fri, "Fr"),
                (Weekday::Sat, "Sa"),
                (Weekday::Sun, "Su"),
            ];
            let mut weekday_row = row![].spacing(6);
            for (day, label) in days {
                let is_selected = self.selected_weekdays.contains(&day);
                weekday_row = weekday_row.push(
                    button(text(label).center())
                        .style(style::time::button::weekday_button(is_selected))
                        .padding([10, 0])
                        .width(FillPortion(1))
                        .on_press(Message::ToggleWeekday(day)),
                );
            }
            weekday_row.into()
        };

        let save_btn = container(
            button(if is_editing {
                "Save Changes"
            } else {
                "Add Schedule"
            })
            .style(style::time::button::add_button)
            .padding([15, 40])
            .on_press(Message::SaveSchedule),
        )
        .center_x(Fill);

        let panel_content = column![
            header,
            mode_toggle,
            time_row,
            dose_field,
            interval_form,
            container(column![]).height(Fill),
            save_btn
        ]
        .spacing(20)
        .padding(30)
        .height(Fill);

        self.centered_panel(panel_content.into())
    }

    fn centered_panel<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        let inner_panel = container(content)
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
