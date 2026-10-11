//! Single instance guard.
//!
//! Med-Tracker keeps one history file and raises alarms from a background tick,
//! so a second copy only duplicates notifications and races writes. The first
//! launch binds a private unix socket and holds it for its lifetime. A later
//! launch writes to that socket, asks the running copy to raise its window, and
//! quits before Iced is even asked for a window.
//!
//! The guard never blocks the app: when the socket cannot be prepared, the
//! launch proceeds instead of the app disappearing.

use std::io::{self, Read, Write};
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use iced::Subscription;
use iced::futures::sink::SinkExt;

use crate::application::message::Message;

/// How often the running instance looks for activation requests.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Keeps a wedged peer from stalling either side of the request.
const REQUEST_TIMEOUT: Duration = Duration::from_millis(500);

/// What a second launch sends to the instance that owns the socket.
const ACTIVATE_REQUEST: &[u8] = b"show";

/// Guard of the process that claimed the socket path first.
static INSTANCE: OnceLock<Instance> = OnceLock::new();

struct Instance {
    listener: UnixListener,
    socket: PathBuf,
}

/// Outcome of trying to claim the guard socket.
enum Claim {
    /// This process owns the socket and holds it until exit.
    Owned(UnixListener),
    /// Another instance is running and was asked to show its window.
    Taken,
    /// The guard could not be prepared, so this launch proceeds without one.
    Unavailable,
}

/// Returns `true` when another instance is running, after asking it to show its
/// main window; such a launch should return from `main` right away.
///
/// Returns `false` when this process may start its UI: either it owns the guard
/// now, or the guard is unavailable and starting wins over staying hidden.
pub fn already_running() -> bool {
    let Some(socket) = socket_path() else {
        return false;
    };

    match claim(&socket) {
        Claim::Owned(listener) => {
            INSTANCE.get_or_init(|| Instance { listener, socket });
            false
        }
        Claim::Taken => true,
        Claim::Unavailable => false,
    }
}

/// Streams activation requests from later launches as [`Message::ShowRequested`].
pub fn subscription() -> Subscription<Message> {
    if INSTANCE.get().is_some() {
        Subscription::run(accept_requests)
    } else {
        Subscription::none()
    }
}

/// Removes the socket path on a normal shutdown so the next launch does not
/// have to clean it up. A killed process leaves it behind, which
/// [`already_running`] recovers from.
pub fn release() {
    if let Some(instance) = INSTANCE.get() {
        let _ = std::fs::remove_file(&instance.socket);
    }
}

/// Private, per-user location of the guard socket.
fn socket_path() -> Option<PathBuf> {
    let dir = dirs::runtime_dir().or_else(dirs::config_dir)?;
    Some(dir.join("med-tracker").join("single-instance.sock"))
}

fn claim(socket: &Path) -> Claim {
    match bind(socket) {
        Ok(listener) => Claim::Owned(listener),
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
            // The path is busy. An owner that answers is really running; a
            // socket left behind by a killed process is cleared and reclaimed.
            if notify_running(socket) {
                return Claim::Taken;
            }
            let _ = std::fs::remove_file(socket);
            match bind(socket) {
                Ok(listener) => Claim::Owned(listener),
                Err(error) if error.kind() == io::ErrorKind::AddrInUse => Claim::Taken,
                Err(_) => Claim::Unavailable,
            }
        }
        Err(_) => Claim::Unavailable,
    }
}

fn bind(socket: &Path) -> io::Result<UnixListener> {
    if let Some(dir) = socket.parent() {
        // 0700, so no other user can claim the path and stall this launch.
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)?;
    }
    let listener = UnixListener::bind(socket)?;
    // Accepting must never wait for a request that is not coming.
    listener.set_nonblocking(true)?;
    Ok(listener)
}

/// Asks the instance that owns the socket to show itself. A refused connection
/// means the file is a leftover rather than busy.
fn notify_running(socket: &Path) -> bool {
    let Ok(mut stream) = UnixStream::connect(socket) else {
        return false;
    };
    let _ = stream.set_write_timeout(Some(REQUEST_TIMEOUT));
    stream.write_all(ACTIVATE_REQUEST).is_ok()
}

fn accept_requests() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(1, async |mut output| {
        // `subscription` builds this stream only once this process owns the
        // listener, which lives as long as the process.
        let Some(instance) = INSTANCE.get() else {
            eprintln!("DBG stream: no guard"); // TEMP
            return;
        };
        eprintln!("DBG accept_requests stream started"); // TEMP
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            // Nonblocking: having nothing to accept is the normal case.
            while let Ok((mut request, _)) = instance.listener.accept() {
                let _ = request.set_read_timeout(Some(REQUEST_TIMEOUT));
                // Drain the request before closing, so the sender learns that
                // this instance really is the live owner.
                let mut request_text = String::new();
                let _ = request.read_to_string(&mut request_text);
                eprintln!("DBG accepted request={request_text:?}"); // TEMP
                output.send(Message::ShowRequested).await.ok();
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A socket path in this test process' own temporary directory.
    fn socket(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "med-tracker-guard-{}-{name}.sock",
            std::process::id()
        ))
    }

    struct Guard(PathBuf);

    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    #[test]
    fn the_first_launch_owns_the_socket() {
        let path = socket("first");
        let _cleanup = Guard(path.clone());

        assert!(matches!(claim(&path), Claim::Owned(_)));
    }

    #[test]
    fn a_second_launch_reaches_the_running_instance() {
        let path = socket("second");
        let _cleanup = Guard(path.clone());
        let owner = bind(&path).expect("owner binds the socket");

        assert!(matches!(claim(&path), Claim::Taken));

        // The running instance sees the request itself, not just a port in use.
        let (mut request, _) = owner
            .accept()
            .expect("the request is queued in the backlog");
        let mut text = String::new();
        request
            .read_to_string(&mut text)
            .expect("the request is readable");
        assert_eq!(text, "show");
    }

    #[test]
    fn a_socket_left_by_a_dead_instance_is_reclaimed() {
        let path = socket("stale");
        let _cleanup = Guard(path.clone());
        // Dropping the listener closes it but keeps the file, like a kill would.
        drop(bind(&path).expect("the dead instance bound the socket"));

        assert!(matches!(claim(&path), Claim::Owned(_)));
        assert!(path.exists());
    }

    #[test]
    fn an_unusable_socket_path_does_not_stop_a_launch() {
        let blocker = socket("blocker-file");
        let _cleanup = Guard(blocker.clone());
        std::fs::write(&blocker, b"not a directory").expect("a plain file");
        // Under a file there is no directory to create, so binding cannot work.
        let path = blocker.join("nested").join("single-instance.sock");

        assert!(matches!(claim(&path), Claim::Unavailable));
    }
}
