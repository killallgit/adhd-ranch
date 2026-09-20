//! The socket agents' hooks deliver to.
//!
//! One connection per firing, one frame per connection: the action on the first
//! line, the agent's payload after it. The ranch applies it in memory and tells
//! whoever is listening; nothing touches a disk, so there is no state to go stale
//! and nothing to clean up when the hooks are removed.

use std::io::Read;
use std::net::Shutdown;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{fs, io};

use adhd_ranch_domain::agents::hooks::HookAction;

use super::OnChange;
use crate::agent_session_store::HookEventSink;

/// A client that connects and then says nothing must not hold up the ones behind it.
const READ_TIMEOUT: Duration = Duration::from_millis(250);

/// A hook payload is a few hundred bytes; anything vastly larger is not a hook.
const MAX_FRAME: u64 = 1 << 20;

/// How long an idle listener waits before looking again.
///
/// Accept is polled rather than blocked on so that shutdown cannot depend on the
/// socket file still being there to connect to — a removed socket would otherwise
/// leave the ranch unable to quit. The cost is one syscall every interval and at
/// most this much delay on a firing, neither of which anyone can perceive.
const ACCEPT_POLL: Duration = Duration::from_millis(25);

/// Only this user may report sessions. The socket is the whole authorisation story,
/// so it is the file mode that does the work.
const SOCKET_MODE: u32 = 0o600;

pub struct HookServer {
    socket_path: PathBuf,
    running: Arc<AtomicBool>,
    accepting: Option<JoinHandle<()>>,
}

pub fn serve(
    socket_path: PathBuf,
    sessions: Arc<dyn HookEventSink>,
    on_change: OnChange,
) -> io::Result<HookServer> {
    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let listener = bind(&socket_path)?;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(SOCKET_MODE))?;

    listener.set_nonblocking(true)?;
    log::info!("agent hooks: listening on {}", socket_path.display());

    let running = Arc::new(AtomicBool::new(true));
    let accepting = std::thread::spawn({
        let running = Arc::clone(&running);
        move || {
            while running.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if handle(stream, sessions.as_ref()) {
                            on_change();
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        std::thread::sleep(ACCEPT_POLL)
                    }
                    Err(error) => log::warn!("agent hooks: accept failed: {error}"),
                }
            }
        }
    });

    Ok(HookServer {
        socket_path,
        running,
        accepting: Some(accepting),
    })
}

/// A socket file outlives a crash, and the leftover makes the address look taken.
/// Nobody answering on it means it is debris, not a second ranch.
fn bind(socket_path: &Path) -> io::Result<UnixListener> {
    match UnixListener::bind(socket_path) {
        Ok(listener) => Ok(listener),
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
            if UnixStream::connect(socket_path).is_ok() {
                return Err(error);
            }
            fs::remove_file(socket_path)?;
            UnixListener::bind(socket_path)
        }
        Err(error) => Err(error),
    }
}

fn handle(mut stream: UnixStream, sessions: &dyn HookEventSink) -> bool {
    // Accepted from a non-blocking listener, so it may have inherited that; the read
    // below wants to wait for the client, bounded by its own timeout.
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let mut frame = String::new();
    if (&mut stream)
        .take(MAX_FRAME)
        .read_to_string(&mut frame)
        .is_err()
    {
        return false;
    }
    let _ = stream.shutdown(Shutdown::Both);

    let Some((verb, payload)) = frame.split_once('\n') else {
        return false;
    };
    let Some(action) = HookAction::from_verb(verb.trim()) else {
        return false;
    };
    sessions.apply(action, payload)
}

impl Drop for HookServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(accepting) = self.accepting.take() {
            let _ = accepting.join();
        }
        let _ = fs::remove_file(&self.socket_path);
    }
}
