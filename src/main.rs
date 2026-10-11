mod application;
mod audio;
mod branding;
mod instance;
mod notify;
mod persistence;
mod startup;
mod tray;
mod ui;
mod update;

use application::message::Message;
use iced::{self as ice, Size, Task};
use ui::view;

use crate::application::app::App;
use crate::audio::alarm::{play_alarm, stop_alarm};
use crate::notify::notification::send_alarm_notification;
use crate::update::alarm_dismiss::dismiss_expired_alarms;
use crate::update::generate_records::generate_future_records;
use crate::update::loadpanel::load_panel;
use crate::update::time_check::{check_medication_schedule, check_new_day, update_time};
use application::panel::Panel;
use chrono;
use std::sync::OnceLock;
use ui::panel::{alarm, home, medications, settings};

static RESOURCE_STARTUP_ERROR: OnceLock<Option<String>> = OnceLock::new();

fn main() {
    let resource_error = crate::startup::use_resource_root()
        .err()
        .map(|error| format!("Application resources could not be located: {error}"));
    if let Some(error) = &resource_error {
        eprintln!("{error}");
    }
    let _ = RESOURCE_STARTUP_ERROR.set(resource_error);

    // A second launch only wakes the window that is already open. It must never
    // build a UI of its own, so it quits here, before Iced starts.
    if instance::already_running() {
        eprintln!("Med-Tracker is already running; asked that instance to show its window.");
        return;
    }

    let result = ice::daemon(new, update, view::view)
        .title(view::title)
        .theme(view::theme)
        .subscription(|state| {
            let tray_enabled = state.settings.is_minimize_to_tray;
            let native_active = tray_enabled
                && state.tray_backend == Some(crate::tray::backend::TrayBackend::NativeX11);
            let status_notifier_active = tray_enabled
                && state.tray_backend == Some(crate::tray::backend::TrayBackend::StatusNotifier);
            iced::Subscription::batch([
                update_time(state),
                iced::window::close_requests().map(Message::CloseRequested),
                instance::subscription(),
                if native_active {
                    crate::tray::x11_native::subscription(state.tray_generation)
                } else {
                    iced::Subscription::none()
                },
                if status_notifier_active {
                    crate::tray::status_notifier::status_notifier_subscription(
                        state.tray_generation,
                    )
                } else {
                    iced::Subscription::none()
                },
            ])
        })
        .run();

    crate::tray::x11_native::shutdown();
    instance::release();
    result.expect("a");
}

fn new() -> (App, Task<Message>) {
    let mut app = App::new();
    if let Some(tracker) = persistence::load_tracker() {
        app.medicationtracker = tracker;
    }
    if let Some(settings) = persistence::load_settings() {
        app.settings = settings;
    }
    app.resource_startup_error = RESOURCE_STARTUP_ERROR.get().and_then(Clone::clone);
    app.refresh_system_theme();
    let old_date = app.medicationtracker.last_generation_date;
    check_new_day(&mut app.medicationtracker);
    if old_date != app.medicationtracker.last_generation_date {
        save(&app);
    }
    let wayland = crate::tray::backend::is_wayland();
    let sni_host_available = !wayland && crate::tray::backend::status_notifier_host_available();
    app.tray_backend = crate::tray::backend::select_tray_backend(
        wayland,
        app.settings.is_minimize_to_tray,
        sni_host_available,
    );
    app.tray_generation = u64::from(app.tray_backend.is_some());
    if app.tray_backend == Some(crate::tray::backend::TrayBackend::NativeX11) {
        crate::tray::x11_native::set_enabled(true, app.tray_generation);
    }

    if let Err(error) = crate::startup::set_autostart(app.settings.is_auto_startup) {
        eprintln!("Startup setting could not be reconciled: {error}");
        app.startup_registration_error = Some(error.to_string());
        app.uistate
            .settingsui
            .set_auto_startup_error(Some(error.to_string()));
    }

    let resources_ready = RESOURCE_STARTUP_ERROR.get().is_some_and(Option::is_none);
    let startup_window = crate::startup::startup_window_decision(
        crate::startup::is_started_minimized(),
        app.settings.is_minimize_to_tray,
        app.tray_backend.is_some(),
        app.tray_ready,
        resources_ready,
    );
    let (open_task, tray_timeout_task) = match startup_window {
        crate::startup::StartupWindow::Hidden => (Task::none(), Task::none()),
        crate::startup::StartupWindow::WaitForTray => {
            app.startup_waiting_for_tray = true;
            let timeout = Task::perform(
                async {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                },
                move |_| Message::TrayStartupTimeout {
                    generation: app.tray_generation,
                },
            );
            (Task::none(), timeout)
        }
        crate::startup::StartupWindow::Open => {
            let (main_id, task) = iced::window::open(main_window_settings());
            app.window_id = Some(main_id);
            (task.map(Message::WindowOpened), Task::none())
        }
    };

    let scroll_task = if app.uistate.timeui.needs_scroll_to_today {
        Task::done(Message::Time(home::time::Message::ScrollToToday))
    } else {
        Task::none()
    };

    (
        app,
        Task::batch([open_task, tray_timeout_task, scroll_task]),
    )
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
        icon: crate::branding::app_window_icon(),
        exit_on_close_request: false,
        ..Default::default()
    }
}

/// Shows the main window: focuses it if already open, reopens it if closed.
fn show_main_window(state: &mut App) -> Task<Message> {
    state.startup_waiting_for_tray = false;
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

fn handle_tray_event(
    state: &mut App,
    backend: crate::tray::backend::TrayBackend,
    generation: u64,
    event: crate::tray::backend::TrayEvent,
) -> Task<Message> {
    use crate::tray::backend::{TrayBackend, TrayEvent};

    if !state.settings.is_minimize_to_tray
        || state.tray_backend != Some(backend)
        || state.tray_generation != generation
    {
        return Task::none();
    }

    match event {
        TrayEvent::Embedded(embedded) => {
            let was_ready = state.tray_ready;
            state.tray_ready = embedded;
            if embedded {
                state.tray_loss_generation = state.tray_loss_generation.wrapping_add(1);
                let transition = crate::startup::tray_startup_transition(
                    state.startup_waiting_for_tray,
                    false,
                    state.window_id.is_some(),
                    crate::startup::TrayStartupEvent::Ready,
                );
                state.startup_waiting_for_tray = transition.waiting_for_tray;
                Task::none()
            } else if was_ready {
                state.tray_loss_generation = state.tray_loss_generation.wrapping_add(1);
                let loss_generation = state.tray_loss_generation;
                Task::perform(
                    async {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    },
                    move |_| Message::TrayLossTimeout {
                        generation,
                        loss_generation,
                    },
                )
            } else {
                Task::none()
            }
        }
        TrayEvent::Activate | TrayEvent::Show => show_main_window(state),
        TrayEvent::Exit => {
            if backend == TrayBackend::NativeX11 {
                crate::tray::x11_native::shutdown();
            } else {
                crate::tray::status_notifier::request_status_notifier_shutdown();
            }
            iced::exit()
        }
        TrayEvent::Failed(error) => {
            eprintln!("System tray could not start: {error}");
            state.tray_ready = false;
            state.tray_generation = state.tray_generation.wrapping_add(1);
            state.tray_loss_generation = state.tray_loss_generation.wrapping_add(1);
            if backend == TrayBackend::StatusNotifier {
                crate::tray::status_notifier::request_status_notifier_shutdown();
                if !crate::tray::backend::is_wayland() {
                    state.tray_backend = Some(TrayBackend::NativeX11);
                    crate::tray::x11_native::set_enabled(true, state.tray_generation);
                    if state.startup_waiting_for_tray {
                        let generation = state.tray_generation;
                        return Task::perform(
                            async {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            },
                            move |_| Message::TrayStartupTimeout { generation },
                        );
                    }
                    if state.window_id.is_none() {
                        let generation = state.tray_generation;
                        let loss_generation = state.tray_loss_generation;
                        return Task::perform(
                            async {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            },
                            move |_| Message::TrayLossTimeout {
                                generation,
                                loss_generation,
                            },
                        );
                    }
                    return Task::none();
                }
            }
            state.tray_backend = None;
            state.startup_waiting_for_tray = false;
            if state.window_id.is_none() {
                show_main_window(state)
            } else {
                Task::none()
            }
        }
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
            let alarming_records = check_medication_schedule(
                &state.medicationtracker,
                state.settings.alarm_time_minutes,
            );
            let mut any_new_audible = false;
            for record_id in alarming_records {
                let is_new = !state.uistate.alarmui.alarming_records.contains(&record_id);
                if is_new {
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
                            .or_else(|| record.one_time.as_ref().map(|data| data.name.as_str()))
                            .unwrap_or("Unknown");
                        let time_str = record
                            .time
                            .with_timezone(&chrono::Local)
                            .format("%H:%M")
                            .to_string();
                        send_alarm_notification(med_name, &time_str);
                        if !record.is_muted {
                            any_new_audible = true;
                        }
                    }
                }
                state.uistate.alarmui.add_alarming_record(record_id);
            }
            if any_new_audible && state.settings.sound_enabled {
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
                state.settings.alarm_time_minutes,
            );
            if had_expired {
                save(state);
            }
            if !state
                .uistate
                .alarmui
                .has_audible_alarm(&state.medicationtracker)
            {
                stop_alarm();
            }
            if !state.uistate.alarmui.is_active() && state.state.panel == Panel::Alarm {
                state.state.restore_previous_panel();
            }

            Task::none()
        }
        Message::CloseRequested(id) => {
            if state.settings.is_minimize_to_tray && state.tray_ready {
                state.window_id = None;
                iced::window::close(id)
            } else {
                iced::exit()
            }
        }
        Message::ShowRequested => {
            let transition = crate::startup::tray_startup_transition(
                state.startup_waiting_for_tray,
                state.tray_ready,
                state.window_id.is_some(),
                crate::startup::TrayStartupEvent::UserActivated,
            );
            state.startup_waiting_for_tray = transition.waiting_for_tray;
            show_main_window(state)
        }
        Message::Tray {
            backend,
            generation,
            event,
        } => handle_tray_event(state, backend, generation, event),
        Message::TrayStartupTimeout { generation } => {
            if generation != state.tray_generation {
                return Task::none();
            }
            let transition = crate::startup::tray_startup_transition(
                state.startup_waiting_for_tray,
                state.tray_ready,
                state.window_id.is_some(),
                crate::startup::TrayStartupEvent::Timeout,
            );
            state.startup_waiting_for_tray = transition.waiting_for_tray;
            if transition.show_window {
                show_main_window(state)
            } else {
                Task::none()
            }
        }
        Message::TrayLossTimeout {
            generation,
            loss_generation,
        } => {
            if state.tray_generation == generation
                && crate::startup::tray_loss_fallback_due(
                    state.settings.is_minimize_to_tray,
                    state.tray_ready,
                    state.window_id.is_none(),
                    loss_generation,
                    state.tray_loss_generation,
                )
            {
                show_main_window(state)
            } else {
                Task::none()
            }
        }
        Message::WindowOpened(_) => Task::none(),
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
        Message::Settings(settings::Message::ToggleAutoStartup(enabled)) => {
            let previous = state.settings.is_auto_startup;
            if previous == enabled {
                return Task::none();
            }
            state.uistate.settingsui.set_auto_startup_error(None);
            let result = crate::startup::apply_startup_preference(
                previous,
                enabled,
                |requested| {
                    crate::startup::set_autostart(requested).map_err(|error| error.to_string())
                },
                |requested| {
                    persistence::save_settings_with_auto_startup(&state.settings, requested)
                        .map_err(|error| error.to_string())
                },
            );
            match result {
                Ok(()) => state.settings.is_auto_startup = enabled,
                Err(error) => {
                    eprintln!("Start-with-system setting failed: {error}");
                    state.uistate.settingsui.set_auto_startup_error(Some(error));
                }
            }
            Task::none()
        }
        Message::Settings(settings_message) => {
            let refresh_system_theme = matches!(
                &settings_message,
                settings::Message::ApplyGtkTheme | settings::Message::FollowSystemTheme(true)
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
                    state.tray_generation = state.tray_generation.wrapping_add(1);
                    state.tray_loss_generation = state.tray_loss_generation.wrapping_add(1);
                    state.tray_ready = false;
                    state.startup_waiting_for_tray = false;
                    if state.settings.is_minimize_to_tray {
                        let wayland = crate::tray::backend::is_wayland();
                        let sni_host_available =
                            !wayland && crate::tray::backend::status_notifier_host_available();
                        state.tray_backend = crate::tray::backend::select_tray_backend(
                            wayland,
                            true,
                            sni_host_available,
                        );
                        if state.tray_backend == Some(crate::tray::backend::TrayBackend::NativeX11)
                        {
                            crate::tray::x11_native::set_enabled(true, state.tray_generation);
                        }
                    } else {
                        state.tray_backend = None;
                        crate::tray::x11_native::set_enabled(false, state.tray_generation);
                        crate::tray::status_notifier::request_status_notifier_shutdown();
                    }
                    return Task::none();
                }
                if !state.settings.sound_enabled {
                    stop_alarm();
                } else if state
                    .uistate
                    .alarmui
                    .has_audible_alarm(&state.medicationtracker)
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
            let deleted_record_id = match &msg {
                home::time::Message::ConfirmDelete(id) => Some(id.clone()),
                _ => None,
            };
            let should_save = matches!(
                msg,
                home::time::Message::MedicationAdd(_)
                    | home::time::Message::MarkSkipped(_)
                    | home::time::Message::MarkTakenToggle(_)
                    | home::time::Message::ConfirmDelete(_)
                    | home::time::Message::Taken(
                        home::takenpanel::Message::TakeNow | home::takenpanel::Message::Confirm
                    )
                    | home::time::Message::Reschedule(home::reschedulepanel::Message::Confirm)
            );
            let should_generate = matches!(
                msg,
                home::time::Message::MedicationAdd(home::medicationaddpanel::Message::Done)
            );
            let (task, one_time_created) = state
                .uistate
                .timeui
                .update(&mut state.medicationtracker, msg);
            let task = task.map(Message::Time);
            if let Some(record_id) = deleted_record_id {
                state
                    .uistate
                    .alarmui
                    .alarming_records
                    .retain(|id| id != &record_id);
            }
            if should_generate {
                generate_future_records(&mut state.medicationtracker);
            }
            if should_save || one_time_created {
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
                    | MedMsg::ConfirmArchive
                    | MedMsg::ConfirmUnarchive
                    | MedMsg::Edit(EditMsg::SaveSchedule)
                    | MedMsg::Edit(EditMsg::DeleteSchedule(_))
            );
            let should_generate = matches!(
                msg,
                MedMsg::ConfirmArchive
                    | MedMsg::ConfirmUnarchive
                    | MedMsg::Edit(EditMsg::SaveSchedule)
                    | MedMsg::Edit(EditMsg::DeleteSchedule(_))
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
            let was_audible = state
                .uistate
                .alarmui
                .has_audible_alarm(&state.medicationtracker);
            let toggled_mute = matches!(&msg, alarm::Message::ToggleMuted(_));
            let should_save = matches!(
                &msg,
                alarm::Message::MarkTaken(_)
                    | alarm::Message::MarkSkipped(_)
                    | alarm::Message::MarkAllTaken(_)
                    | alarm::Message::MarkAllSkipped(_)
                    | alarm::Message::Reschedule(
                        crate::ui::panel::home::reschedulepanel::Message::Confirm
                    )
            );
            state
                .uistate
                .alarmui
                .update(&mut state.medicationtracker, msg);
            let is_audible = state
                .uistate
                .alarmui
                .has_audible_alarm(&state.medicationtracker);
            if !state.settings.sound_enabled || !is_audible {
                stop_alarm();
            } else if toggled_mute && !was_audible {
                play_alarm(
                    &state.settings.alarm_sound_path,
                    state.settings.sound_volume,
                );
            }
            if !state.uistate.alarmui.is_active() {
                state.state.restore_previous_panel();
            }
            if should_save {
                save(state);
            }
            Task::none()
        }
        Message::HideSidebar => {
            state.sidebar_visible = false;
            Task::none()
        }
        Message::OpenSidebar => {
            state.sidebar_visible = true;
            Task::none()
        }
    }
}
