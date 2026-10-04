use tray::{Icon, TrayIcon, TrayIconBuilder};
use x11rb::protocol::xproto::ConnectionExt;

fn tray_icon() -> Option<Icon> {
    let rgba = crate::branding::tray_icon_rgba()?;
    Icon::from_rgba(rgba, 24, 24).ok()
}

pub fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|v| v.to_lowercase() == "wayland")
        .unwrap_or(false)
}

pub fn system_tray_owner() -> Result<Option<u32>, String> {
    let (connection, _) = x11rb::connect(None).map_err(|error| error.to_string())?;
    let atom = connection
        .intern_atom(false, b"_NET_SYSTEM_TRAY_S0")
        .map_err(|error| error.to_string())?
        .reply()
        .map_err(|error| error.to_string())?
        .atom;
    let owner = connection
        .get_selection_owner(atom)
        .map_err(|error| error.to_string())?
        .reply()
        .map_err(|error| error.to_string())?
        .owner;
    Ok((owner != x11rb::NONE).then_some(owner))
}

pub fn create_tray() -> Option<TrayIcon> {
    if is_wayland() {
        return None;
    }

    let Some(icon) = tray_icon() else {
        eprintln!("[tray] Could not load icons/med-tracker-tray.png");
        return None;
    };

    match TrayIconBuilder::new()
        .with_tooltip("Med-Tracker")
        .with_icon(icon)
        .build()
    {
        Ok(tray) => Some(tray),
        Err(e) => {
            eprintln!("[tray] Failed to create tray icon: {e}");
            None
        }
    }
}
