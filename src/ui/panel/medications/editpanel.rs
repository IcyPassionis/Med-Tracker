use crate::application::medication::dosetype::DoseType;
use crate::application::medication::occurrencestatus::OccurrenceStatus;
use crate::application::medication::periodtype::PeriodType;
use crate::application::medication::schedule::Schedule;
use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::style;
use crate::ui::style::time::container::overlay_panel_container;
use chrono::Weekday;
use iced::Length::{Fill, FillPortion};
use iced::widget::{
    Image, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{self as ice, Background, Color, ContentFit, Element, Theme, alignment};

#[derive(Debug, Clone)]
pub enum Section {
    Options,
    MedicationEdit,
    ScheduleList,
    ScheduleEdit { schedule_id: Option<String> },
    Stock,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleMode {
    Interval,
    Weekdays,
}

pub struct MedicationEditPanel {
    current_medication_id: Option<String>,
    pub section: Section,
    name_input: String,
    stock_input: String,
    pill_dose_input: String,
    dose_type: DoseType,
    editing_schedule_id: Option<String>,
    schedule_mode: ScheduleMode,
    schedule_period_type: PeriodType,
    schedule_hour: String,
    schedule_minute: String,
    schedule_period_time: String,
    schedule_dose: String,
    schedule_error: Option<String>,
    selected_weekdays: Vec<Weekday>,
    stock_edit_input: String,
    name_error: Option<String>,
    stock_error: Option<String>,
    pill_dose_error: Option<String>,
    stock_edit_error: Option<String>,
    pub pending_save: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    GoToMedicationEdit,
    GoToScheduleList,
    GoToStock,
    NameChange(String),
    StockChange(String),
    PillDoseChange(String),
    DoseTypeChange(DoseType),
    SaveMedicationEdit,
    BackToOptions,
    OpenNewSchedule,
    EditSchedule(String),
    DeleteSchedule(String),
    BackToOptionsFromSchedules,
    ScheduleModeChange(ScheduleMode),
    SchedulePeriodTypeChange(PeriodType),
    ToggleWeekday(Weekday),
    ScheduleHourChange(String),
    ScheduleMinuteChange(String),
    SchedulePeriodTimeChange(String),
    ScheduleDoseChange(String),
    SaveSchedule,
    BackToScheduleList,
    StockEditChange(String),
    SaveStock,
    BackToOptionsFromStock,
}

impl MedicationEditPanel {
    pub fn new() -> Self {
        Self {
            current_medication_id: None,
            section: Section::Options,
            name_input: String::new(),
            stock_input: String::new(),
            pill_dose_input: String::new(),
            dose_type: DoseType::Mg,
            editing_schedule_id: None,
            schedule_mode: ScheduleMode::Interval,
            schedule_period_type: PeriodType::Daily,
            schedule_hour: String::new(),
            schedule_minute: String::new(),
            schedule_period_time: String::new(),
            schedule_dose: String::new(),
            schedule_error: None,
            selected_weekdays: vec![],
            stock_edit_input: String::new(),
            name_error: None,
            stock_error: None,
            pill_dose_error: None,
            stock_edit_error: None,
            pending_save: false,
        }
    }

    pub fn open(&mut self, id: String, tracker: &MedicationTracker) {
        if let Some(med) = tracker.medications.iter().find(|m| m.id == id) {
            self.name_input = med.name.clone();
            self.stock_input = med.stock.to_string();
            self.pill_dose_input = med.pill_dose.to_string();
            self.dose_type = med.dose_type;
            self.stock_edit_input = med.stock.to_string();
        }
        self.current_medication_id = Some(id);
        self.section = Section::Options;
        self.reset_schedule_fields();
    }

    pub fn close(&mut self) {
        self.current_medication_id = None;
        self.name_input = String::new();
        self.stock_input = String::new();
        self.pill_dose_input = String::new();
        self.dose_type = DoseType::Mg;
        self.stock_edit_input = String::new();
        self.name_error = None;
        self.stock_error = None;
        self.pill_dose_error = None;
        self.stock_edit_error = None;
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

    pub fn view<'a>(&self, tracker: &'a MedicationTracker) -> Option<Element<'a, Message>> {
        if self.current_medication_id.is_none() {
            return None;
        }
        let section_view = match &self.section {
            Section::Options => self.options_overlay(tracker),
            Section::MedicationEdit => self.medication_edit_overlay(),
            Section::ScheduleList => self.schedule_list_overlay(tracker),
            Section::ScheduleEdit { .. } => self.schedule_edit_overlay(tracker),
            Section::Stock => self.stock_overlay(),
        };
        Some(
            iced::widget::stack![self.backdrop(), section_view]
                .width(Fill)
                .height(Fill)
                .into(),
        )
    }

    pub fn update(&mut self, tracker: &mut MedicationTracker, message: Message) {
        self.pending_save = false;
        match message {
            Message::Close => self.close(),

            Message::GoToMedicationEdit => self.section = Section::MedicationEdit,
            Message::GoToScheduleList => self.section = Section::ScheduleList,
            Message::GoToStock => self.section = Section::Stock,

            Message::NameChange(v) => {
                self.name_error = None;
                self.name_input = v;
            }
            Message::StockChange(v) => {
                self.stock_error = None;
                self.stock_input = v;
            }
            Message::PillDoseChange(v) => {
                self.pill_dose_error = None;
                if v.is_empty() || v.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    let dots = v.chars().filter(|c| *c == '.').count();
                    if dots <= 1 {
                        self.pill_dose_input = v;
                    }
                }
            }
            Message::DoseTypeChange(v) => self.dose_type = v,
            Message::SaveMedicationEdit => {
                let name = self.name_input.trim();
                if name.len() < 3 {
                    self.name_error = Some("Name must be at least 3 characters.".into());
                    return;
                }
                let parsed_stock: Option<f32> = self.stock_input.parse().ok();
                match parsed_stock {
                    None => {
                        self.stock_error = Some("Enter a valid number.".into());
                        return;
                    }
                    Some(v) if v < 0.0 => {
                        self.stock_error = Some("Stock cannot be negative.".into());
                        return;
                    }
                    Some(_) => {}
                }
                let parsed_pill_dose: Option<f32> = self.pill_dose_input.parse().ok();
                match parsed_pill_dose {
                    None => {
                        self.pill_dose_error = Some("Enter a valid number.".into());
                        return;
                    }
                    Some(v) if v <= 0.0 => {
                        self.pill_dose_error = Some("Pill dose must be greater than zero.".into());
                        return;
                    }
                    Some(_) => {}
                }
                if let Some(id) = self.current_medication_id.clone() {
                    if let Some(med) = tracker.medications.iter_mut().find(|m| m.id == id) {
                        med.name = name.to_string();
                        med.stock = parsed_stock.unwrap();
                        med.pill_dose = parsed_pill_dose.unwrap();
                        med.dose_type = self.dose_type;
                        self.pending_save = true;
                    }
                }
                self.section = Section::Options;
            }
            Message::BackToOptions => {
                self.name_error = None;
                self.stock_error = None;
                self.pill_dose_error = None;
                self.section = Section::Options;
            }

            Message::OpenNewSchedule => {
                self.reset_schedule_fields();
                self.section = Section::ScheduleEdit { schedule_id: None };
            }
            Message::EditSchedule(id) => {
                if let Some(med_id) = self.current_medication_id.clone() {
                    if let Some(med) = tracker.medications.iter().find(|m| m.id == med_id) {
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
                            self.editing_schedule_id = Some(id.clone());
                        }
                    }
                }
                self.section = Section::ScheduleEdit {
                    schedule_id: Some(id),
                };
            }
            Message::DeleteSchedule(id) => {
                let med_id = self.current_medication_id.clone();
                if let Some(med_id) = med_id {
                    if let Some(med) = tracker.medications.iter_mut().find(|m| m.id == med_id) {
                        med.schedules.retain(|s| s.id != id);
                    }
                    let today = chrono::Local::now().date_naive();
                    tracker.records.retain(|r| {
                        !(r.schedule_id == id
                            && r.time.with_timezone(&chrono::Local).date_naive() >= today
                            && matches!(r.occurrence_status, OccurrenceStatus::Pending)
                            && !r.rescheduled)
                    });
                }
            }
            Message::BackToOptionsFromSchedules => self.section = Section::Options,

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

                let med_id = self.current_medication_id.clone();
                let edit_id = self.editing_schedule_id.clone();

                if let Some(med_id) = med_id {
                    if let Some(ref eid) = edit_id {
                        let eid = eid.clone();
                        let today = chrono::Local::now().date_naive();
                        tracker.records.retain(|r| {
                            !(r.schedule_id == eid
                                && r.time.with_timezone(&chrono::Local).date_naive() >= today
                                && matches!(r.occurrence_status, OccurrenceStatus::Pending)
                                && !r.rescheduled)
                        });
                        if let Some(med) = tracker.medications.iter_mut().find(|m| m.id == med_id) {
                            if let Some(s) = med.schedules.iter_mut().find(|s| s.id == eid) {
                                s.time = [hour, minute];
                                s.period_time = period_time;
                                s.period_type = period_type;
                                s.week_day = week_day;
                                s.dose = dose;
                            }
                        }
                    } else {
                        if let Some(med) = tracker.medications.iter_mut().find(|m| m.id == med_id) {
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
            Message::BackToScheduleList => {
                self.schedule_error = None;
                self.reset_schedule_fields();
                self.section = Section::ScheduleList;
            }

            Message::StockEditChange(v) => {
                self.stock_edit_error = None;
                self.stock_edit_input = v;
            }
            Message::SaveStock => {
                let parsed_stock: Option<f32> = self.stock_edit_input.parse().ok();
                match parsed_stock {
                    None => {
                        self.stock_edit_error = Some("Enter a valid number.".into());
                        return;
                    }
                    Some(v) if v < 0.0 => {
                        self.stock_edit_error = Some("Stock cannot be negative.".into());
                        return;
                    }
                    Some(v) => {
                        if let Some(id) = self.current_medication_id.clone() {
                            if let Some(med) = tracker.medications.iter_mut().find(|m| m.id == id) {
                                med.stock = v;
                                self.pending_save = true;
                            }
                        }
                    }
                }
                self.section = Section::Options;
            }
            Message::BackToOptionsFromStock => {
                self.stock_edit_error = None;
                self.section = Section::Options;
            }
        }
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

    fn options_overlay<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let med = self
            .current_medication_id
            .as_deref()
            .and_then(|id| tracker.medications.iter().find(|m| m.id == id));

        let name = med.map(|m| m.name.as_str()).unwrap_or("Medication");
        let stock = med.map(|m| m.stock).unwrap_or(0.0);
        let dose_type = med.map(|m| m.dose_type).unwrap_or_default();
        let pill_dose = med.map(|m| m.pill_dose).unwrap_or(1.0);
        let schedule_count = med.map(|m| m.schedules.len()).unwrap_or(0);

        let header = row![
            text(name).size(32).width(Fill).center(),
            button(button_with_icon!("icons/cross.png", 50, 10))
                .on_press(Message::Close)
                .style(style::time::button::overlay_close_button),
        ]
        .align_y(alignment::Vertical::Center);

        let info = column![
            text(format!("Stock: {} {}", stock, dose_type)).size(16),
            text(format!("Pill Dose: {} {}", pill_dose, dose_type)).size(16),
            text(format!("Schedules: {}", schedule_count)).size(16),
        ]
        .spacing(8);

        let edit_btn = button(text("Edit Info").width(Fill).center())
            .style(style::time::button::add_button)
            .padding([12, 30])
            .width(Fill)
            .on_press(Message::GoToMedicationEdit);

        let schedules_btn = button(text("Schedules").width(Fill).center())
            .style(style::time::button::add_button)
            .padding([12, 30])
            .width(Fill)
            .on_press(Message::GoToScheduleList);

        let stock_btn = button(text("Update Stock").width(Fill).center())
            .style(style::time::button::add_button)
            .padding([12, 30])
            .width(Fill)
            .on_press(Message::GoToStock);

        let content = column![
            header,
            info,
            container(column![]).height(Fill),
            edit_btn,
            schedules_btn,
            stock_btn,
        ]
        .spacing(16)
        .padding(30)
        .height(Fill);

        self.centered_panel(content.into())
    }

    fn medication_edit_overlay<'a>(&self) -> Element<'a, Message> {
        let header = row![
            text("Edit Medication").size(32).width(Fill).center(),
            button(button_with_icon!("icons/cross.png", 50, 10))
                .on_press(Message::BackToOptions)
                .style(style::time::button::overlay_close_button),
        ]
        .align_y(alignment::Vertical::Center);

        let mut name_field = column![
            text("Medication Name").size(16),
            text_input("Name...", &self.name_input).on_input(Message::NameChange),
        ]
        .spacing(8);
        if let Some(err) = &self.name_error {
            name_field = name_field.push(text(err.clone()).size(13).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

        let mut stock_field = column![
            text("Stock").size(16),
            text_input("0", &self.stock_input).on_input(Message::StockChange),
        ]
        .spacing(8)
        .width(FillPortion(1));
        if let Some(err) = &self.stock_error {
            stock_field = stock_field.push(text(err.clone()).size(13).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

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

        let mut pill_dose_field = column![
            text("Pill Dose").size(16),
            text_input("1", &self.pill_dose_input).on_input(Message::PillDoseChange),
        ]
        .spacing(8);
        if let Some(err) = &self.pill_dose_error {
            pill_dose_field = pill_dose_field.push(text(err.clone()).size(13).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

        let save_btn = container(
            button("Save")
                .style(style::time::button::add_button)
                .padding([15, 60])
                .on_press(Message::SaveMedicationEdit),
        )
        .center_x(Fill);

        let content = column![
            header,
            name_field,
            row![stock_field, dose_type_field].spacing(20),
            pill_dose_field,
            container(column![]).height(Fill),
            save_btn,
        ]
        .spacing(20)
        .padding(30)
        .height(Fill);

        self.centered_panel(content.into())
    }

    fn schedule_list_overlay<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let header = row![
            text("Schedules").size(32).width(Fill).center(),
            button(button_with_icon!("icons/cross.png", 50, 10))
                .on_press(Message::BackToOptionsFromSchedules)
                .style(style::time::button::overlay_close_button),
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

        let add_btn = container(
            button("+ Add New Schedule")
                .style(style::time::button::add_button)
                .padding([12, 30])
                .on_press(Message::OpenNewSchedule),
        )
        .center_x(Fill);

        let content = column![
            header,
            schedule_list,
            container(column![]).height(Fill),
            add_btn,
        ]
        .spacing(20)
        .padding(30)
        .height(Fill);

        self.centered_panel(content.into())
    }

    fn schedule_edit_overlay<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let is_editing = self.editing_schedule_id.is_some();
        let title = if is_editing {
            "Edit Schedule"
        } else {
            "New Schedule"
        };

        let header = row![
            text(title).size(32).width(Fill).center(),
            button(button_with_icon!("icons/cross.png", 50, 10))
                .on_press(Message::BackToScheduleList)
                .style(style::time::button::overlay_close_button),
        ]
        .align_y(alignment::Vertical::Center);

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
            dose_field = dose_field.push(text(err.clone()).size(13).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

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

        let content = column![
            header,
            mode_toggle,
            time_row,
            dose_field,
            interval_form,
            container(column![]).height(Fill),
            save_btn,
        ]
        .spacing(20)
        .padding(30)
        .height(Fill);

        self.centered_panel(content.into())
    }

    fn stock_overlay<'a>(&self) -> Element<'a, Message> {
        let header = row![
            text("Update Stock").size(32).width(Fill).center(),
            button(button_with_icon!("icons/cross.png", 50, 10))
                .on_press(Message::BackToOptionsFromStock)
                .style(style::time::button::overlay_close_button),
        ]
        .align_y(alignment::Vertical::Center);

        let mut stock_field = column![
            text("Current Stock").size(16),
            text_input("0", &self.stock_edit_input).on_input(Message::StockEditChange),
        ]
        .spacing(8);
        if let Some(err) = &self.stock_edit_error {
            stock_field = stock_field.push(text(err.clone()).size(13).style(|theme: &Theme| {
                iced::widget::text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                }
            }));
        }

        let save_btn = container(
            button("Save")
                .style(style::time::button::add_button)
                .padding([15, 60])
                .on_press(Message::SaveStock),
        )
        .center_x(Fill);

        let content = column![
            header,
            stock_field,
            container(column![]).height(Fill),
            save_btn,
        ]
        .spacing(20)
        .padding(30)
        .height(Fill);

        self.centered_panel(content.into())
    }
}
