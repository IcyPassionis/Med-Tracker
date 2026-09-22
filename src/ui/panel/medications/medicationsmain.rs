use crate::application::states::medicationtracker::MedicationTracker;
use crate::ui::macros::{button_with_icon, button_with_icon_text};
use crate::ui::panel::medications::{editpanel, refillpanel};
use crate::ui::style;
use crate::ui::style::medications::button as med_button;
use crate::ui::style::medications::container as med_container;
use iced::Length::{Fill, Shrink};
use iced::widget::{Column, Image, button, column, container, row, scrollable, stack, text};
use iced::{ContentFit, Element, alignment};

pub struct Record {
    pending_delete_id: Option<String>,
    pending_archive_id: Option<String>,
    pending_unarchive_id: Option<String>,
    show_archived: bool,
    pub edit_panel: editpanel::MedicationEditPanel,
    pub refill_panel: refillpanel::RefillPanel,
}

impl Record {
    pub fn new() -> Record {
        Self {
            pending_delete_id: None,
            pending_archive_id: None,
            pending_unarchive_id: None,
            show_archived: false,
            edit_panel: editpanel::MedicationEditPanel::new(),
            refill_panel: refillpanel::RefillPanel::new(),
        }
    }

    pub fn view<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let panel = self.medication_panel(tracker);

        let mut layers: Vec<Element<'a, Message>> = vec![panel];

        if self.pending_delete_id.is_some() {
            layers.push(self.backdrop());
            layers.push(self.confirm_delete_overlay());
        }

        if self.pending_archive_id.is_some() {
            layers.push(self.backdrop());
            layers.push(self.confirm_archive_overlay());
        }

        if self.pending_unarchive_id.is_some() {
            layers.push(self.backdrop());
            layers.push(self.confirm_unarchive_overlay());
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
            Message::AskArchive(id) => {
                self.pending_archive_id = Some(id);
            }
            Message::ConfirmArchive => {
                if let Some(id) = self.pending_archive_id.take() {
                    archive_medication(tracker, &id);
                }
            }
            Message::CancelArchive => {
                self.pending_archive_id = None;
            }
            Message::Unarchive(id) => {
                self.pending_unarchive_id = Some(id);
            }
            Message::ConfirmUnarchive => {
                if let Some(id) = self.pending_unarchive_id.take()
                    && let Some(medication) = tracker.medications.iter_mut().find(|m| m.id == id)
                {
                    medication.is_archived = false;
                }
            }
            Message::CancelUnarchive => {
                self.pending_unarchive_id = None;
            }
            Message::ToggleArchivedView => {
                self.show_archived = !self.show_archived;
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

    fn medication_panel<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let archived_count = tracker.medications.iter().filter(|m| m.is_archived).count();

        let mut content: Column<'a, Message> = column![].width(Fill).height(Fill);

        if self.show_archived || archived_count > 0 {
            content = content.push(self.archived_toggle(archived_count));
        }

        content.push(self.medication_list(tracker)).into()
    }

    fn archived_toggle<'a>(&self, archived_count: usize) -> Element<'a, Message> {
        let (label, icon) = if self.show_archived {
            ("Back to Medications".to_string(), "icons/arrow-back-up.png")
        } else {
            (
                format!("Archived Medications ({archived_count})"),
                "icons/archive.png",
            )
        };

        container(
            button(button_with_icon_text!(label, icon))
                .style(style::time::button::add_button)
                .padding([12, 30])
                .on_press(Message::ToggleArchivedView),
        )
        .width(Fill)
        .center_x(Fill)
        .padding([12, 40])
        .into()
    }

    fn medication_list<'a>(&'a self, tracker: &'a MedicationTracker) -> Element<'a, Message> {
        let mut list: Column<'a, Message> = column![].spacing(12);

        for med in tracker
            .medications
            .iter()
            .filter(|med| med.is_archived == self.show_archived)
        {
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

            if !med.is_archived
                && let Some(days) = tracker.days_left(&med.id)
            {
                info = info.push(text(format!("{} days left", days)).size(14));
            }

            if med.is_archived {
                info = info.push(text("Archived").size(14));
            }

            let archive_btn = button(button_with_icon!("icons/archive.png", 20, 0))
                .style(style::time::button::overlay_close_button)
                .padding(10)
                .on_press(if self.show_archived {
                    Message::Unarchive(med.id.clone())
                } else {
                    Message::AskArchive(med.id.clone())
                });

            let delete_btn = button(button_with_icon!("icons/cross.png", 20, 0))
                .style(style::time::button::overlay_close_button)
                .padding(10)
                .on_press(Message::AskDelete(med.id.clone()));

            let card_row = if self.show_archived {
                row![pill_placeholder, info, archive_btn, delete_btn]
            } else {
                let refill_btn = button(button_with_icon!("icons/medicine-syrup.png", 20, 0))
                    .style(style::time::button::overlay_close_button)
                    .padding(10)
                    .on_press(Message::OpenRefill(med.id.clone()));

                row![pill_placeholder, info, refill_btn, archive_btn, delete_btn]
            }
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

        let is_empty = !tracker
            .medications
            .iter()
            .any(|med| med.is_archived == self.show_archived);

        if is_empty {
            let message = if self.show_archived {
                "No archived medications."
            } else {
                "No medications added yet."
            };
            container(text(message).size(16))
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
                text("This will also delete this medication's history. Use Archive instead if you want to keep it.")
                    .size(13)
                    .width(360),
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

    fn confirm_archive_overlay<'a>(&self) -> Element<'a, Message> {
        let panel = container(
            column![
                text("Archive medication?").size(20),
                text("Future records will be removed. Today's and past records will be kept.")
                    .size(13)
                    .width(360),
                row![
                    button("Cancel")
                        .style(style::time::button::add_button)
                        .padding([12, 30])
                        .on_press(Message::CancelArchive),
                    button("Archive")
                        .style(style::time::button::add_button)
                        .padding([12, 30])
                        .on_press(Message::ConfirmArchive),
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

    fn confirm_unarchive_overlay<'a>(&self) -> Element<'a, Message> {
        let panel = container(
            column![
                text("Restore medication?").size(20),
                text("Future records will be generated again. Today's and past records are not changed.")
                    .size(13)
                    .width(360),
                row![
                    button("Cancel")
                        .style(style::time::button::add_button)
                        .padding([12, 30])
                        .on_press(Message::CancelUnarchive),
                    button("Restore")
                        .style(style::time::button::add_button)
                        .padding([12, 30])
                        .on_press(Message::ConfirmUnarchive),
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

fn archive_medication(tracker: &mut MedicationTracker, id: &str) {
    if let Some(medication) = tracker.medications.iter_mut().find(|m| m.id == id) {
        medication.is_archived = true;
    }
    tracker.remove_future_empty_records(id);
}

#[derive(Debug, Clone)]
pub enum Message {
    AskDelete(String),
    ConfirmDelete,
    CancelDelete,
    AskArchive(String),
    ConfirmArchive,
    CancelArchive,
    Unarchive(String),
    ConfirmUnarchive,
    CancelUnarchive,
    ToggleArchivedView,
    OpenEdit(String),
    OpenRefill(String),
    Edit(editpanel::Message),
    Refill(refillpanel::Message),
}
