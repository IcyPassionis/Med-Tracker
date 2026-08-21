use crate::application::message::Message;
use iced::Subscription;
use iced::futures::sink::SinkExt;
use ksni::TrayMethods;

struct WaylandTray {
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl ksni::Tray for WaylandTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "Med-Tracker".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let mut rgba = vec![0x2e_u8, 0xcc, 0x71, 0xff].repeat(24 * 24);
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
        let _ = self.sender.send(Message::TrayLeftClick);
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        let show_sender = self.sender.clone();
        let quit_sender = self.sender.clone();
        vec![
            StandardItem {
                label: "Show Application".into(),
                activate: Box::new(move |_| {
                    let _ = show_sender.send(Message::TrayMenuShow);
                }),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                activate: Box::new(move |_| {
                    let _ = quit_sender.send(Message::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn wayland_tray_subscription() -> Subscription<Message> {
    Subscription::run(wayland_tray_stream)
}

static SHUTDOWN_TX: std::sync::Mutex<Option<tokio::sync::watch::Sender<bool>>> =
    std::sync::Mutex::new(None);

/// Requests the current Wayland (SNI) tray icon to shut down and remove itself.
pub fn request_tray_shutdown() {
    if let Ok(tx) = SHUTDOWN_TX.lock() {
        if let Some(tx) = tx.as_ref() {
            let _ = tx.send(true);
        }
    }
}

fn wayland_tray_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(16, async |mut output| {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Message>();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::watch::channel(false);
        let mut task_shutdown_rx = shutdown_rx.clone();
        {
            let mut guard = SHUTDOWN_TX.lock().expect("shutdown tx lock");
            *guard = Some(shutdown_tx);
        }

        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    let tray = WaylandTray { sender };
                    let mut handle = match tray.spawn().await {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("[tray-wayland] Failed to spawn SNI tray: {e}");
                            return;
                        }
                    };
                    let _ = task_shutdown_rx.changed().await;
                    let _ = handle.shutdown().await;
                });
            }
            Err(e) => {
                eprintln!("[tray-wayland] No Tokio runtime available: {e}");
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
    })
}
