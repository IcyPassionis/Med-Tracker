use crate::application::message::Message;
use iced::futures::sink::SinkExt;
use iced::{Subscription, futures::stream::BoxStream};

pub fn x11_tray_status_subscription(tray_active: bool) -> Subscription<Message> {
    Subscription::run_with(tray_active, x11_tray_status_stream)
}

fn x11_tray_status_stream(tray_active: &bool) -> BoxStream<'static, Message> {
    let tray_active = *tray_active;
    let poll_interval = if tray_active { 250 } else { 1_000 };
    Box::pin(iced::stream::channel(1, async move |mut output| {
        let mut observed_owner = match crate::tray::tray::system_tray_owner() {
            Ok(owner) => owner,
            Err(error) => {
                eprintln!("[tray] Could not read X11 system tray owner: {error}");
                None
            }
        };

        if !tray_active && observed_owner.is_some() {
            let _ = output.send(Message::TrayHostChanged(observed_owner)).await;
        }

        let mut last_error = None;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(poll_interval)).await;
            match crate::tray::tray::system_tray_owner() {
                Ok(owner) => {
                    last_error = None;
                    if owner != observed_owner || (owner.is_some() && !tray_active) {
                        observed_owner = owner;
                        if output.send(Message::TrayHostChanged(owner)).await.is_err() {
                            break;
                        }
                    }
                }
                Err(error) => {
                    if last_error.as_deref() != Some(error.as_str()) {
                        eprintln!("[tray] Could not read X11 system tray owner: {error}");
                        last_error = Some(error);
                    }
                }
            }
        }
    }))
}
