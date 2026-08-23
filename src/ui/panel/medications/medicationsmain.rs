use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::button_with_icon;
use crate::ui::panel::medications::{editpanel, refillpanel};
use crate::ui::style;
use crate::ui::style::medications::button as med_button;
use crate::ui::style::medications::container as med_container;
use iced::Length::{Fill, Shrink};
use iced::widget::{Column, Image, button, column, container, row, scrollable, stack, text};
use iced::{ContentFit, Element, alignment};

pub struct Record {
    pending_delete_id: Option<String>,
    pub edit_panel: editpanel::MedicationEditPanel,
    pub refill_panel: refillpanel::RefillPanel,
}

impl Record {
    pub fn new() -> Record {
        Self {
            pending_delete_id: None,
            edit_panel: editpanel::MedicationEditPanel::new(),
            refill_panel: refillpanel::RefillPanel::new(),
        }
    }

    pub fn view<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let list = self.medication_list(tracker);

        let mut layers: Vec<Element<'a, Message>> = vec![list];

        if self.pending_delete_id.is_some() {
            layers.push(self.backdrop());
            layers.push(self.confirm_delete_overlay());
        }

        if let Some(overlay) = self.edit_panel.view(tracker) {
            layers.push(overlay.map(Message::Edit));
        }

        if let Some(overlay) = self.refill_panel.view() {
            layers.push(overlay.map(Message::Refill));
        }

        if layers.len() == 1 {
            layers.remove(0)
        } else {
            stack(layers).width(Fill).height(Fill).into()
        }
    }

    pub fn update(&mut self, tracker: &mut MedicationTracker, message: Message) {
        match message {
            Message::AskDelete(id) => {
                self.pending_delete_id = Some(id);
            }
            Message::ConfirmDelete => {
                if let Some(id) = self.pending_delete_id.take() {
                    tracker.medications.retain(|m| m.id != id);
                    tracker.records.retain(|r| r.medication_id != id);
                }
            }
            Message::CancelDelete => {
                self.pending_delete_id = None;
            }
            Message::OpenEdit(id) => {
                self.edit_panel.open(id, tracker);
            }
            Message::OpenRefill(id) => {
                self.refill_panel.open(id);
            }
            Message::Edit(msg) => {
                self.edit_panel.update(tracker, msg);
            }
            Message::Refill(msg) => {
                self.refill_panel.update(tracker, msg);
            }
        }
    }

    fn medication_list<'a>(&self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let mut list: Column<'a, Message> = column![].spacing(12);

        for med in &tracker.medications {
            let pill_placeholder = container(
                Image::new("icons/pill.png")
                    .content_fit(ContentFit::Cover)
                    .width(32)
                    .height(32),
            )
            .width(52)
            .height(52)
            .center_x(52)
            .center_y(52)
            .style(med_container::pill_icon_container);

            let mut info = column![
                text(&med.name).size(20),
                text(format!("{} {}", med.stock, med.dose_type)).size(14),
            ]
            .spacing(4)
            .width(Fill);

            if let Some(days) = tracker.days_left(&med.id) {
                info = info.push(text(format!("{} days left", days)).size(14));
            }

            let refill_btn = button(button_with_icon!("icons/medicine-syrup.png", 20, 0))
                .style(style::time::button::overlay_close_button)
                .padding(10)
                .on_press(Message::OpenRefill(med.id.clone()));

            let delete_btn = button(button_with_icon!("icons/cross.png", 20, 0))
                .style(style::time::button::overlay_close_button)
                .padding(10)
                .on_press(Message::AskDelete(med.id.clone()));

            let card_row = row![pill_placeholder, info, refill_btn, delete_btn]
                .spacing(16)
                .align_y(alignment::Vertical::Center)
                .padding([14, 20]);

            let card = container(card_row).width(Fill);
            let med_id = med.id.clone();
            let card_btn = button(card)
                .style(med_button::medication_card_button)
                .padding(0)
                .width(Fill)
                .on_press(Message::OpenEdit(med_id));

            list = list.push(card_btn);
        }

        if tracker.medications.is_empty() {
            container(text("No medications added yet.").size(16))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .into()
        } else {
            scrollable(
                container(list.max_width(750))
                    .center_x(Fill)
                    .padding([20, 40]),
            )
            .width(Fill)
            .height(Fill)
            .into()
        }
    }

    fn backdrop<'a>(&self) -> Element<'a, Message> {
        container(text(""))
            .width(Fill)
            .height(Fill)
            .style(med_container::backdrop)
            .into()
    }

    fn confirm_delete_overlay<'a>(&self) -> Element<'a, Message> {
        let panel = container(
            column![
                text("Delete medication?").size(20),
                text("This cannot be undone. Past records will be kept.").size(13),
                row![
                    button("Cancel")
                        .style(style::time::button::add_button)
                        .padding([12, 30])
                        .on_press(Message::CancelDelete),
                    button("Delete")
                        .style(style::time::button::add_button)
                        .padding([12, 30])
                        .on_press(Message::ConfirmDelete),
                ]
                .spacing(16),
            ]
            .spacing(16)
            .padding(30),
        )
        .style(med_container::delete_dialog)
        .width(Shrink)
        .height(Shrink);

        container(panel)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    AskDelete(String),
    ConfirmDelete,
    CancelDelete,
    OpenEdit(String),
    OpenRefill(String),
    Edit(editpanel::Message),
    Refill(refillpanel::Message),
}
