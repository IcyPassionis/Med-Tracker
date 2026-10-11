#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayBackend {
    NativeX11,
    StatusNotifier,
}

#[derive(Debug, Clone)]
pub enum TrayEvent {
    Embedded(bool),
    Activate,
    Show,
    Exit,
    Failed(String),
}

const WATCHER_NAME: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";

pub fn select_tray_backend(
    is_wayland: bool,
    tray_enabled: bool,
    status_notifier_host_available: bool,
) -> Option<TrayBackend> {
    if !tray_enabled {
        None
    } else if is_wayland || status_notifier_host_available {
        Some(TrayBackend::StatusNotifier)
    } else {
        Some(TrayBackend::NativeX11)
    }
}

pub fn is_wayland() -> bool {
    std::env::var("XDG_SESSION_TYPE")
        .map(|value| value.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false)
}

/// Reports whether an SNI host is registered on the session bus.
///
/// Run zbus's blocking API on a separate thread because this function may be
/// called while Iced's Tokio runtime is active.
pub fn status_notifier_host_available() -> bool {
    std::thread::Builder::new()
        .name("tray-host-probe".into())
        .spawn(query_status_notifier_host)
        .ok()
        .and_then(|thread| thread.join().ok())
        .flatten()
        .unwrap_or(false)
}

fn query_status_notifier_host() -> Option<bool> {
    let connection = zbus::blocking::connection::Builder::session()
        .ok()?
        .method_timeout(std::time::Duration::from_millis(500))
        .build()
        .ok()?;
    let watcher =
        zbus::blocking::Proxy::new(&connection, WATCHER_NAME, WATCHER_PATH, WATCHER_NAME).ok()?;
    watcher
        .get_property::<bool>("IsStatusNotifierHostRegistered")
        .ok()
}

#[cfg(test)]
mod tests {
    use super::{TrayBackend, select_tray_backend};

    #[test]
    fn wayland_uses_status_notifier_without_an_x11_host() {
        assert_eq!(
            select_tray_backend(true, true, false),
            Some(TrayBackend::StatusNotifier)
        );
    }

    #[test]
    fn x11_prefers_sni_only_when_a_host_is_present() {
        assert_eq!(
            select_tray_backend(false, true, true),
            Some(TrayBackend::StatusNotifier)
        );
        assert_eq!(
            select_tray_backend(false, true, false),
            Some(TrayBackend::NativeX11)
        );
    }

    #[test]
    fn disabled_tray_has_no_backend() {
        assert_eq!(select_tray_backend(false, false, true), None);
    }
}
