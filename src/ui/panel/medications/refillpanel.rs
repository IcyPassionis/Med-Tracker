use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::style;
use crate::ui::style::medications::container::backdrop;
use crate::ui::style::time::container::overlay_panel_container;
use iced::Length::Fill;
use iced::widget::{Image, button, column, container, row, text, text_input};
use iced::{ContentFit, Element, Padding, alignment};

pub struct RefillPanel {
    medication_id: Option<String>,
    pills_input: String,
    warning: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Open(String),
    PillsChange(String),
    Confirm,
    Cancel,
}

impl RefillPanel {
    pub fn new() -> Self {
        Self {
            medication_id: None,
            pills_input: String::new(),
            warning: None,
        }
    }

    pub fn open(&mut self, medication_id: String) {
        self.medication_id = Some(medication_id);
        self.pills_input.clear();
        self.warning = None;
    }

    fn close(&mut self) {
        self.medication_id = None;
        self.pills_input.clear();
        self.warning = None;
    }

    pub fn view(&self) -> Option<Element<'_, Message>> {
        self.medication_id.as_ref()?;

        let header = row![
            text("How many pills you want to add?")
                .size(18)
                .width(Fill),
            button(button_with_icon!("icons/cross.png", 30, 10))
                .style(style::time::button::overlay_close_button)
                .padding(5)
                .on_press(Message::Cancel),
        ]
        .align_y(alignment::Vertical::Center);

        let pills_input = text_input("0", &self.pills_input)
            .on_input(Message::PillsChange)
            .width(Fill)
            .size(24);

        let add_button = button(container(text("Add")).center_x(Fill).center_y(Fill))
            .style(style::time::button::add_button)
            .width(Fill)
            .height(48)
            .padding(12)
            .on_press(Message::Confirm);

        let mut content = column![header, pills_input]
            .spacing(20)
            .padding(Padding::new(30.0));

        if let Some(warning) = &self.warning {
            let warning_text =
                text(warning)
                    .size(14)
                    .style(|theme: &iced::Theme| iced::widget::text::Style {
                        color: Some(theme.extended_palette().danger.base.color),
                    });
            content = content.push(warning_text);
        }

        content = content.push(add_button);

        let panel = container(content)
            .style(overlay_panel_container)
            .width(320);

        let overlay = container(container(panel).center(Fill))
            .style(backdrop)
            .width(Fill)
            .height(Fill);

        Some(overlay.into())
    }

    pub fn update(&mut self, tracker: &mut MedicationTracker, msg: Message) -> Option<String> {
        match msg {
            Message::Open(id) => {
                self.open(id);
                None
            }
            Message::PillsChange(v) => {
                self.pills_input = v;
                self.warning = None;
                None
            }
            Message::Confirm => {
                let pills: f32 = match self.pills_input.trim().parse() {
                    Ok(v) => v,
                    Err(_) => {
                        self.warning = Some("Enter a valid number.".into());
                        return None;
                    }
                };
                if pills <= 0.0 {
                    self.warning = Some("Must be greater than zero.".into());
                    return None;
                }
                let id = self.medication_id.take()?;
                tracker.refill_stock(&id, pills);
                self.close();
                Some(id)
            }
            Message::Cancel => {
                self.close();
                None
            }
        }
    }
}