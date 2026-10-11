use crate::application::message::Message;
use crate::tray::backend::{TrayBackend, TrayEvent};
use iced::Subscription;
use iced::futures::sink::SinkExt;
use iced::futures::stream::BoxStream;
use ksni::TrayMethods;

struct StatusNotifierTray {
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    generation: u64,
}

impl StatusNotifierTray {
    fn send(&self, event: TrayEvent) {
        let _ = self.sender.send(Message::Tray {
            backend: TrayBackend::StatusNotifier,
            generation: self.generation,
            event,
        });
    }
}

impl ksni::Tray for StatusNotifierTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "Med-Tracker".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let Some(mut rgba) = crate::branding::tray_icon_rgba() else {
            eprintln!("[tray-sni] Could not load icons/med-tracker-tray.png");
            return Vec::new();
        };
        for pixel in rgba.chunks_exact_mut(4) {
            pixel.rotate_right(1);
        }
        vec![ksni::Icon {
            width: 24,
            height: 24,
            data: rgba,
        }]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.send(TrayEvent::Activate);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        let show_tray = Self {
            sender: self.sender.clone(),
            generation: self.generation,
        };
        let quit_tray = Self {
            sender: self.sender.clone(),
            generation: self.generation,
        };
        vec![
            StandardItem {
                label: "Show Application".into(),
                activate: Box::new(move |_| {
                    show_tray.send(TrayEvent::Show);
                }),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(move |_| {
                    quit_tray.send(TrayEvent::Exit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn watcher_offline(&self, reason: ksni::OfflineReason) -> bool {
        self.send(TrayEvent::Failed(format!(
            "Status notifier went offline: {reason:?}"
        )));
        false
    }
}

pub fn status_notifier_subscription(generation: u64) -> Subscription<Message> {
    Subscription::run_with(generation, status_notifier_stream)
}

static SHUTDOWN_TX: std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>> =
    std::sync::Mutex::new(None);

// Iced may cancel the stream without reaching the end of its async body.
// Own this subscription's sender so cancellation also retires its SNI task.
struct ShutdownOnDrop(tokio::sync::watch::Sender<bool>);

impl Drop for ShutdownOnDrop {
    fn drop(&mut self) {
        let _ = self.0.send(true);
    }
}

/// Requests the current SNI tray icon to shut down and remove itself.
pub fn request_status_notifier_shutdown() {
    if let Ok(tx) = SHUTDOWN_TX.lock() {
        if let Some(tx) = tx.as_ref() {
            let _ = tx.send(true);
        }
    }
}

fn status_notifier_stream(generation: &u64) -> BoxStream<'static, Message> {
    let generation = *generation;
    Box::pin(iced::stream::channel(16, async move |mut output| {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Message>();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let mut task_shutdown_rx = shutdown_rx.clone();
        let _shutdown_guard = ShutdownOnDrop(shutdown_tx.clone());
        {
            let mut guard = SHUTDOWN_TX.lock().expect("shutdown tx lock");
            *guard = Some(shutdown_tx.clone());
        }

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    let status_sender = sender.clone();
                    let tray = StatusNotifierTray { sender, generation };
                    let handle = match tray.spawn().await {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("[tray-sni] Failed to spawn StatusNotifierItem: {e}");
                            let _ = status_sender.send(Message::Tray {
                                backend: TrayBackend::StatusNotifier,
                                generation,
                                event: TrayEvent::Failed(e.to_string()),
                            });
                            return;
                        }
                    };
                    let _ = status_sender.send(Message::Tray {
                        backend: TrayBackend::StatusNotifier,
                        generation,
                        event: TrayEvent::Embedded(true),
                    });
                    if !*task_shutdown_rx.borrow() {
                        let _ = task_shutdown_rx.changed().await;
                    }
                    let _ = handle.shutdown().await;
                });
            }
            Err(e) => {
                eprintln!("[tray-sni] No Tokio runtime available: {e}");
                let _ = output
                    .send(Message::Tray {
                        backend: TrayBackend::StatusNotifier,
                        generation,
                        event: TrayEvent::Failed(e.to_string()),
                    })
                    .await;
                return;
            }
        }

        loop {
            tokio::select! {
                msg = receiver.recv() => match msg {
                    Some(msg) => {
                        if output.send(msg).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                },
                changed = async { shutdown_rx.changed().await } => {
                    if changed.is_err() || *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
        // Dropping or replacing this subscription must also stop its SNI task.
        let _ = shutdown_tx.send(true);
    }))
}
