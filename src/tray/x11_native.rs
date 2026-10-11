use crate::application::message::Message;
use crate::tray::backend::{TrayBackend, TrayEvent};
use iced::futures::sink::SinkExt;
use iced::{Subscription, futures::stream::BoxStream};
use std::sync::{Mutex, OnceLock};
use tokio::sync::broadcast;

const EVENT_INITIALIZED: i32 = 0;
const EVENT_EMBEDDED: i32 = 1;
const EVENT_ACTIVATE: i32 = 2;
const EVENT_SHOW: i32 = 3;
const EVENT_EXIT: i32 = 4;
const EVENT_FAILED: i32 = 5;
const EVENT_UNEMBEDDED: i32 = 6;

#[derive(Clone, Debug)]
enum NativeEvent {
    Embedded(u64, bool),
    Activate(u64),
    Show(u64),
    Exit(u64),
    Failed(u64, String),
}

struct DesiredState {
    enabled: bool,
    generation: u64,
    started: bool,
    initialized: bool,
    shutdown_requested: bool,
    failure: Option<String>,
}

struct Controller {
    desired: Mutex<DesiredState>,
    events: broadcast::Sender<NativeEvent>,
}

struct CallbackContext {
    controller: &'static Controller,
}

static CONTROLLER: OnceLock<Controller> = OnceLock::new();

fn controller() -> &'static Controller {
    CONTROLLER.get_or_init(|| {
        let (events, _) = broadcast::channel(32);
        Controller {
            desired: Mutex::new(DesiredState {
                enabled: false,
                generation: 0,
                started: false,
                initialized: false,
                shutdown_requested: false,
                failure: None,
            }),
            events,
        }
    })
}

/// Starts the single GTK owner thread once, or updates its desired visibility.
pub fn set_enabled(enabled: bool, generation: u64) {
    let controller = controller();
    let mut desired = controller.desired.lock().expect("tray state lock");
    desired.enabled = enabled;
    desired.generation = generation;
    if enabled && !desired.started && desired.failure.is_some() {
        // Image loading and thread creation can fail before GTK is touched;
        // allow those failures to be retried on a later enable.
        desired.failure = None;
    }
    if desired.started {
        if desired.initialized {
            #[cfg(target_os = "linux")]
            unsafe {
                med_tracker_x11_tray_set_enabled(enabled.into(), generation);
            }
        }
        return;
    }
    if !enabled {
        return;
    }

    desired.started = true;
    let spawn_result = std::thread::Builder::new()
        .name("med-tracker-gtk-tray".into())
        .spawn(move || run_gtk_thread(controller));
    if let Err(error) = spawn_result {
        desired.started = false;
        let error = error.to_string();
        desired.failure = Some(error.clone());
        let _ = controller
            .events
            .send(NativeEvent::Failed(generation, error));
    }
}

pub fn shutdown() {
    let controller = controller();
    let mut desired = controller.desired.lock().expect("tray state lock");
    desired.enabled = false;
    desired.generation = desired.generation.wrapping_add(1);
    desired.shutdown_requested = true;
    if desired.initialized {
        #[cfg(target_os = "linux")]
        unsafe {
            med_tracker_x11_tray_shutdown();
        }
    }
}

pub fn subscription(generation: u64) -> Subscription<Message> {
    Subscription::run_with(generation, native_stream)
}

fn native_stream(generation: &u64) -> BoxStream<'static, Message> {
    let generation = *generation;
    let controller = controller();
    let mut events = controller.events.subscribe();
    let desired = controller.desired.lock().expect("tray state lock");
    let initial_failure = if desired.generation == generation {
        desired.failure.clone()
    } else {
        None
    };
    if desired.initialized {
        #[cfg(target_os = "linux")]
        unsafe {
            med_tracker_x11_tray_query_state(generation);
        }
    }
    drop(desired);

    Box::pin(iced::stream::channel(16, async move |mut output| {
        if let Some(error) = initial_failure {
            if output
                .send(Message::Tray {
                    backend: TrayBackend::NativeX11,
                    generation,
                    event: TrayEvent::Failed(error),
                })
                .await
                .is_err()
            {
                return;
            }
        }
        loop {
            let event = match events.recv().await {
                Ok(event) => event,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            };
            let message = match event {
                NativeEvent::Embedded(event_generation, embedded)
                    if event_generation == generation =>
                {
                    Some(Message::Tray {
                        backend: TrayBackend::NativeX11,
                        generation,
                        event: TrayEvent::Embedded(embedded),
                    })
                }
                NativeEvent::Activate(event_generation) if event_generation == generation => {
                    Some(Message::Tray {
                        backend: TrayBackend::NativeX11,
                        generation,
                        event: TrayEvent::Activate,
                    })
                }
                NativeEvent::Show(event_generation) if event_generation == generation => {
                    Some(Message::Tray {
                        backend: TrayBackend::NativeX11,
                        generation,
                        event: TrayEvent::Show,
                    })
                }
                NativeEvent::Exit(event_generation) if event_generation == generation => {
                    Some(Message::Tray {
                        backend: TrayBackend::NativeX11,
                        generation,
                        event: TrayEvent::Exit,
                    })
                }
                NativeEvent::Failed(event_generation, error) if event_generation == generation => {
                    Some(Message::Tray {
                        backend: TrayBackend::NativeX11,
                        generation,
                        event: TrayEvent::Failed(error),
                    })
                }
                _ => None,
            };
            if let Some(message) = message {
                if output.send(message).await.is_err() {
                    break;
                }
            }
        }
    }))
}

fn run_gtk_thread(controller: &'static Controller) {
    let Some((rgba, width, height)) = tray_pixels() else {
        if let Ok(mut desired) = controller.desired.lock() {
            desired.started = false;
            let generation = desired.generation;
            let error = "Could not load icons/med-tracker-tray.png".to_string();
            desired.failure = Some(error.clone());
            let _ = controller
                .events
                .send(NativeEvent::Failed(generation, error));
        }
        return;
    };

    // C borrows this context only during its blocking owner-thread loop and
    // clears its callbacks before returning. The RGBA buffer lives for that
    // entire call; C makes its own pixbuf copy before entering GTK's loop.
    let context = Box::new(CallbackContext { controller });
    let context = Box::into_raw(context);
    #[cfg(target_os = "linux")]
    let _result = unsafe {
        med_tracker_x11_tray_run(
            native_event_callback,
            context.cast(),
            rgba.as_ptr(),
            width as i32,
            height as i32,
            (width * 4) as i32,
        )
    };
    #[cfg(not(target_os = "linux"))]
    let _result = 0;
    unsafe {
        drop(Box::from_raw(context));
    }
    if let Ok(mut desired) = controller.desired.lock() {
        desired.initialized = false;
    }
}

fn tray_pixels() -> Option<(Vec<u8>, u32, u32)> {
    let image = image::open("icons/med-tracker-tray.png").ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some((image.into_raw(), width, height))
}

unsafe extern "C" fn native_event_callback(
    event: i32,
    generation: u64,
    user_data: *mut std::ffi::c_void,
) {
    let _ = std::panic::catch_unwind(|| {
        let context = unsafe { &*(user_data.cast::<CallbackContext>()) };
        let controller = context.controller;
        match event {
            EVENT_INITIALIZED => {
                let mut desired = controller.desired.lock().expect("tray state lock");
                desired.initialized = true;
                #[cfg(target_os = "linux")]
                unsafe {
                    if desired.shutdown_requested {
                        med_tracker_x11_tray_shutdown();
                    } else {
                        med_tracker_x11_tray_set_enabled(
                            desired.enabled.into(),
                            desired.generation,
                        );
                    }
                }
            }
            EVENT_EMBEDDED => {
                let _ = controller
                    .events
                    .send(NativeEvent::Embedded(generation, true));
            }
            EVENT_UNEMBEDDED => {
                let _ = controller
                    .events
                    .send(NativeEvent::Embedded(generation, false));
            }
            EVENT_ACTIVATE => {
                let _ = controller.events.send(NativeEvent::Activate(generation));
            }
            EVENT_SHOW => {
                let _ = controller.events.send(NativeEvent::Show(generation));
            }
            EVENT_EXIT => {
                let _ = controller.events.send(NativeEvent::Exit(generation));
            }
            EVENT_FAILED => {
                let current_generation = controller
                    .desired
                    .lock()
                    .map(|state| state.generation)
                    .unwrap_or(generation);
                let error = "GTK3 could not initialize an X11 tray icon".to_string();
                if let Ok(mut desired) = controller.desired.lock() {
                    desired.failure = Some(error.clone());
                }
                let _ = controller
                    .events
                    .send(NativeEvent::Failed(current_generation, error));
            }
            _ => {}
        }
    });
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn med_tracker_x11_tray_run(
        callback: unsafe extern "C" fn(i32, u64, *mut std::ffi::c_void),
        user_data: *mut std::ffi::c_void,
        rgba: *const u8,
        width: i32,
        height: i32,
        stride: i32,
    ) -> i32;
    fn med_tracker_x11_tray_set_enabled(enabled: i32, generation: u64);
    fn med_tracker_x11_tray_query_state(generation: u64);
    fn med_tracker_x11_tray_shutdown();
}
