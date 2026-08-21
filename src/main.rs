mod application;
mod audio;
mod notify;
mod persistence;
mod tray;
mod ui;
mod update;

use application::message::Message;
use iced::{self as ice, Point, Size, Task};
use ui::view;

use crate::application::app::App;
use crate::audio::alarm::{play_alarm, stop_alarm};
use crate::notify::notification::send_alarm_notification;
use crate::tray::subscription::tray_subscription;
use crate::tray::tray::create_tray;
use crate::update::alarm_dismiss::dismiss_expired_alarms;
use crate::update::generate_records::generate_future_records;
use crate::update::loadpanel::load_panel;
use crate::update::time_check::{check_medication_schedule, check_new_day, update_time};
use application::panel::Panel;
use chrono;
use ui::panel::{alarm, home, medications, settings};

fn main() {
    ice::daemon(new, update, view::view)
        .title(view::title)
        .theme(view::theme)
        .subscription(|state| {
            let tray_enabled = state.settings.is_minimize_to_tray;
            if crate::tray::tray::is_wayland() {
                iced::Subscription::batch([
                    update_time(state),
                    iced::window::close_requests().map(Message::CloseRequested),
                    if tray_enabled {
                        crate::tray::wayland::wayland_tray_subscription()
                    } else {
                        iced::Subscription::none()
                    },
                ])
            } else {
                iced::Subscription::batch([
                    update_time(state),
                    iced::window::close_requests().map(Message::CloseRequested),
                    tray_subscription(state.tray_icon.is_some()),
                ])
            }
        })
        .run()
        .expect("a");
}

fn new() -> (App, Task<Message>) {
    let mut app = App::new();
    if let Some(tracker) = persistence::load_tracker() {
        app.medicationtracker = tracker;
    }
    if let Some(settings) = persistence::load_settings() {
        app.settings = settings;
    }
    app.refresh_system_theme();
    let old_date = app.medicationtracker.last_generation_date;
    check_new_day(&mut app.medicationtracker);
    if old_date != app.medicationtracker.last_generation_date {
        save(&app);
    }
    app.tray_icon = if app.settings.is_minimize_to_tray {
        create_tray()
    } else {
        None
    };

    let (main_id, open_task) = iced::window::open(iced::window::Settings {
        size: Size::new(1000.0, 640.0),
        position: iced::window::Position::Centered,
        exit_on_close_request: false,
        ..Default::default()
    });
    app.window_id = Some(main_id);

    let scroll_task = if app.uistate.timeui.needs_scroll_to_today {
        Task::done(Message::Time(home::time::Message::ScrollToToday))
    } else {
        Task::none()
    };

    (app, Task::batch([open_task.map(Message::WindowOpened), scroll_task]))
}

fn save(state: &App) {
    if let Err(e) = persistence::save_tracker(&state.medicationtracker) {
        eprintln!("Save failed: {e}");
    }
}

fn save_settings(state: &App) {
    if let Err(e) = persistence::save_settings(&state.settings) {
        eprintln!("Settings save failed: {e}");
    }
}

fn main_window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: Size::new(1000.0, 640.0),
        position: iced::window::Position::Centered,
        exit_on_close_request: false,
        ..Default::default()
    }
}

/// Shows the main window: focuses it if already open, reopens it if closed.
fn show_main_window(state: &mut App) -> Task<Message> {
    if let Some(id) = state.window_id {
        Task::batch([
            iced::window::set_mode(id, iced::window::Mode::Windowed),
            iced::window::gain_focus(id),
        ])
    } else {
        let (id, task) = iced::window::open(main_window_settings());
        state.window_id = Some(id);
        task.map(Message::WindowOpened)
    }
}

fn update(state: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::TimeCheck => {
            let old_date = state.medicationtracker.last_generation_date;
            check_new_day(&mut state.medicationtracker);
            if old_date != state.medicationtracker.last_generation_date {
                save(state);
            }
            let alarming_records = check_medication_schedule(&state.medicationtracker);
            let mut any_new = false;
            for record_id in alarming_records {
                let is_new = !state.uistate.alarmui.alarming_records.contains(&record_id);
                if is_new {
                    any_new = true;
                    if let Some(record) = state
                        .medicationtracker
                        .records
                        .iter()
                        .find(|r| r.id == record_id)
                    {
                        let med_name = state
                            .medicationtracker
                            .medications
                            .iter()
                            .find(|m| m.id == record.medication_id)
                            .map(|m| m.name.as_str())
                            .unwrap_or("Unknown");
                        let time_str = record
                            .time
                            .with_timezone(&chrono::Local)
                            .format("%H:%M")
                            .to_string();
                        send_alarm_notification(med_name, &time_str);
                    }
                }
                state.uistate.alarmui.add_alarming_record(record_id);
            }
            if any_new && state.settings.sound_enabled {
                play_alarm(
                    &state.settings.alarm_sound_path,
                    state.settings.sound_volume,
                );
            }
            if state.uistate.alarmui.is_active() && state.state.panel != Panel::Alarm {
                state.state.switch_to_alarm();
            }

            let had_expired = dismiss_expired_alarms(
                &mut state.medicationtracker,
                &mut state.uistate.alarmui.alarming_records,
            );
            if had_expired {
                save(state);
            }
            if !state.uistate.alarmui.is_active() && state.state.panel == Panel::Alarm {
                stop_alarm();
                state.state.restore_previous_panel();
            }

            Task::none()
        }
        Message::CloseRequested(id) => {
            if Some(id) == state.popup_window_id {
                state.popup_window_id = None;
                iced::window::close(id)
            } else if state.settings.is_minimize_to_tray {
                state.window_id = None;
                iced::window::close(id)
            } else {
                iced::exit()
            }
        }
        Message::TrayLeftClick => show_main_window(state),
        Message::TrayRightClick { x, y } => {
            let close_existing = if let Some(popup_id) = state.popup_window_id.take() {
                iced::window::close(popup_id)
            } else {
                Task::none()
            };

            let (id, open_task) = iced::window::open(iced::window::Settings {
                size: Size::new(180.0, 80.0),
                position: iced::window::Position::Specific(Point::new(
                    (x as f32 - 90.0).max(0.0),
                    (y as f32 - 80.0).max(0.0),
                )),
                decorations: false,
                resizable: false,
                level: iced::window::Level::AlwaysOnTop,
                ..Default::default()
            });
            state.popup_window_id = Some(id);

            Task::batch([close_existing, open_task.map(Message::WindowOpened)])
        }
        Message::TrayMenuShow => {
            let close_popup = if let Some(popup_id) = state.popup_window_id.take() {
                iced::window::close(popup_id)
            } else {
                Task::none()
            };
            Task::batch([close_popup, show_main_window(state)])
        }
        Message::WindowOpened(_) => Task::none(),
        Message::Quit => iced::exit(),
        Message::OpenTime => {
            load_panel(state, &Panel::Time);
            state.uistate.timeui.set_section_to_main();
            Task::none()
        }
        Message::OpenManageMeds => {
            load_panel(state, &Panel::ManageMeds);
            Task::none()
        }
        Message::OpenCalendar => {
            load_panel(state, &Panel::Calendar);
            Task::none()
        }
        Message::OpenSettings => {
            load_panel(state, &Panel::Settings);
            state.uistate.settingsui.set_section_to_main();
            Task::none()
        }
        Message::Settings(settings::Message::ChooseSoundFile) => Task::perform(
            async {
                rfd::AsyncFileDialog::new()
                    .add_filter("Supported audio", &["wav", "ogg"])
                    .pick_file()
                    .await
                    .map(|file| file.path().to_path_buf())
            },
            |path| Message::Settings(settings::Message::SoundFileSelected(path)),
        ),
        Message::Settings(settings_message) => {
            let refresh_system_theme = matches!(
                &settings_message,
                settings::Message::ApplyGtkTheme
                    | settings::Message::FollowSystemTheme(true)
            );
            let previous_sound_enabled = state.settings.sound_enabled;
            let previous_sound_volume = state.settings.sound_volume;
            let previous_alarm_sound_path = state.settings.alarm_sound_path.clone();
            let previous_minimize_to_tray = state.settings.is_minimize_to_tray;
            let changed = state
                .uistate
                .settingsui
                .update(&mut state.settings, settings_message);
            if refresh_system_theme {
                state.refresh_system_theme();
            }
            if changed {
                save_settings(state);
                if previous_minimize_to_tray != state.settings.is_minimize_to_tray {
                    let close_popup = if let Some(popup_id) = state.popup_window_id.take() {
                        iced::window::close(popup_id)
                    } else {
                        Task::none()
                    };
                    if state.settings.is_minimize_to_tray {
                        state.tray_icon = create_tray();
                    } else {
                        state.tray_icon = None;
                        crate::tray::wayland::request_tray_shutdown();
                    }
                    return close_popup;
                }
                if !state.settings.sound_enabled {
                    stop_alarm();
                } else if state.uistate.alarmui.is_active()
                    && (previous_sound_enabled != state.settings.sound_enabled
                        || previous_sound_volume != state.settings.sound_volume
                        || previous_alarm_sound_path != state.settings.alarm_sound_path)
                {
                    play_alarm(
                        &state.settings.alarm_sound_path,
                        state.settings.sound_volume,
                    );
                }
            }
            Task::none()
        }
        Message::Time(msg) => {
            let should_save = matches!(
                msg,
                home::time::Message::MedicationAdd(_)
                    | home::time::Message::MarkSkipped(_)
                    | home::time::Message::Taken(
                        home::takenpanel::Message::TakeNow
                            | home::takenpanel::Message::Confirm
                    )
                    | home::time::Message::Reschedule(
                        home::reschedulepanel::Message::Confirm
                    )
            );
            let should_generate = matches!(
                msg,
                home::time::Message::MedicationAdd(home::medicationaddpanel::Message::Done)
            );
            let task = state
                .uistate
                .timeui
                .update(&mut state.medicationtracker, msg)
                .map(Message::Time);
            if should_generate {
                generate_future_records(&mut state.medicationtracker);
            }
            if should_save {
                save(state);
            }
            task
        }
        Message::Medications(msg) => {
            use crate::ui::panel::medications::editpanel::Message as EditMsg;
            use medications::medicationsmain::Message as MedMsg;
            let always_save = matches!(
                msg,
                MedMsg::ConfirmDelete
                    | MedMsg::Edit(EditMsg::SaveSchedule)
                    | MedMsg::Edit(EditMsg::DeleteSchedule(_))
            );
            let should_generate = matches!(
                msg,
                MedMsg::Edit(EditMsg::SaveSchedule) | MedMsg::Edit(EditMsg::DeleteSchedule(_))
            );
            state
                .uistate
                .medicationsui
                .update(&mut state.medicationtracker, msg);
            if should_generate {
                generate_future_records(&mut state.medicationtracker);
            }
            let validated_save = state.uistate.medicationsui.edit_panel.pending_save;
            if always_save || validated_save {
                save(state);
            }
            Task::none()
        }
        Message::Calendar(msg) => {
            if let Some(date) = state.uistate.calendarui.update(msg) {
                state.uistate.timeui.select_date(date);
                load_panel(state, &Panel::Time);
            }
            Task::none()
        }
        Message::Alarm(msg) => {
            let should_save = matches!(
                msg,
                alarm::Message::MarkTaken(_)
                    | alarm::Message::MarkSkipped(_)
                    | alarm::Message::Reschedule(
                        crate::ui::panel::home::reschedulepanel::Message::Confirm
                    )
            );
            state
                .uistate
                .alarmui
                .update(&mut state.medicationtracker, msg);
            if !state.uistate.alarmui.is_active() {
                stop_alarm();
                state.state.restore_previous_panel();
            }
            if should_save {
                save(state);
            }
            Task::none()
        }
        Message::HideSidebar => Task::none(),
        Message::OpenSidebar => Task::none(),
    }
}
