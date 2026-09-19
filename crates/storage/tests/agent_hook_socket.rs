//! A hook firing, end to end: the client Claude Code actually runs, over a real
//! socket, into the sessions the overlay draws. The unit tests call `LiveSessions`
//! directly; this is the only thing that proves the wire between them.
#![cfg(unix)]

use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::Duration;

use adhd_ranch_domain::SessionActivity;
use adhd_ranch_storage::{serve, AgentSessionStore, HookServer, LiveSessions};
use tempfile::TempDir;

const SESSION: &str = "9d93be36-54bb-4eec-bd25-e53b21402288";
const CWD: &str = "/code/adhd-ranch";
const SETTLE: Duration = Duration::from_secs(5);

fn payload(action_note: &str) -> String {
    format!(r#"{{"session_id":"{SESSION}","cwd":"{CWD}","hook_event_name":"{action_note}"}}"#)
}

/// `cargo test --workspace` builds every workspace binary, so the client sits beside
/// this test's own executable's parent directory.
fn client_bin() -> PathBuf {
    let mut dir = std::env::current_exe().expect("test exe");
    dir.pop(); // deps/
    dir.pop(); // debug/
    let bin = dir.join("adhd-ranch-hook");
    assert!(
        bin.is_file(),
        "{} is missing — run the workspace tests so the client is built",
        bin.display()
    );
    bin
}

struct Ranch {
    // Declaration order is drop order: the server goes before the directory its
    // socket lives in.
    _server: HookServer,
    _dir: TempDir,
    sessions: Arc<LiveSessions>,
    socket: PathBuf,
    changes: Receiver<()>,
}

impl Ranch {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let socket = dir.path().join("agent-hooks.sock");
        let sessions = Arc::new(LiveSessions::new());
        let (tx, changes): (Sender<()>, Receiver<()>) = channel();
        let server = serve(
            socket.clone(),
            Arc::clone(&sessions),
            Arc::new(move || {
                let _ = tx.send(());
            }),
        )
        .unwrap();
        Self {
            _server: server,
            _dir: dir,
            sessions,
            socket,
            changes,
        }
    }

    /// Run the client exactly as Claude Code would: payload on stdin, action in argv.
    fn fire(&self, action: &str, payload: &str) {
        let mut client = Command::new(client_bin())
            .arg(&self.socket)
            .arg(action)
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        client
            .stdin
            .take()
            .unwrap()
            .write_all(payload.as_bytes())
            .unwrap();
        assert!(client.wait().unwrap().success());
    }

    fn await_change(&self) -> Result<(), RecvTimeoutError> {
        self.changes.recv_timeout(SETTLE)
    }
}

#[test]
fn a_session_starting_appears_on_the_ranch() {
    let ranch = Ranch::new();

    ranch.fire("start", &payload("SessionStart"));

    ranch.await_change().expect("the ranch should be told");
    let sessions = ranch.sessions.list();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, SESSION);
    assert_eq!(sessions[0].pen.name, "adhd-ranch");
}

#[test]
fn a_turn_beginning_sets_the_session_working() {
    let ranch = Ranch::new();
    ranch.fire("start", &payload("SessionStart"));
    ranch.await_change().unwrap();

    ranch.fire("working", &payload("UserPromptSubmit"));

    ranch.await_change().unwrap();
    assert_eq!(ranch.sessions.list()[0].activity, SessionActivity::Working);
}

#[test]
fn a_turn_ending_sets_the_session_resting() {
    let ranch = Ranch::new();
    ranch.fire("working", &payload("UserPromptSubmit"));
    ranch.await_change().unwrap();

    ranch.fire("idle", &payload("Stop"));

    ranch.await_change().unwrap();
    assert_eq!(ranch.sessions.list()[0].activity, SessionActivity::Idle);
}

#[test]
fn a_session_ending_leaves_the_ranch() {
    let ranch = Ranch::new();
    ranch.fire("start", &payload("SessionStart"));
    ranch.await_change().unwrap();

    ranch.fire("end", &payload("SessionEnd"));

    ranch.await_change().unwrap();
    assert!(ranch.sessions.list().is_empty());
}

/// Nothing the ranch does may surface in someone's turn, so the client succeeds
/// whether or not anyone is listening.
#[test]
fn the_client_succeeds_when_no_ranch_is_running() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("nobody-home.sock");

    let mut client = Command::new(client_bin())
        .arg(&missing)
        .arg("working")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    client
        .stdin
        .take()
        .unwrap()
        .write_all(payload("UserPromptSubmit").as_bytes())
        .unwrap();

    assert!(client.wait().unwrap().success());
}

#[test]
fn a_frame_the_ranch_does_not_understand_is_ignored() {
    let ranch = Ranch::new();

    let mut stream = UnixStream::connect(&ranch.socket).unwrap();
    stream
        .write_all(b"demolish\n{\"session_id\":\"x\",\"cwd\":\"/code/x\"}")
        .unwrap();
    drop(stream);

    assert!(ranch.await_change().is_err());
    assert!(ranch.sessions.list().is_empty());
}

#[test]
fn the_socket_is_removed_when_the_ranch_stops() {
    let dir = TempDir::new().unwrap();
    let socket = dir.path().join("agent-hooks.sock");
    {
        let _server = serve(
            socket.clone(),
            Arc::new(LiveSessions::new()),
            Arc::new(|| {}),
        )
        .unwrap();
        assert!(socket.exists());
    }

    assert!(!socket.exists());
}

/// A socket file outlives a crash. The next launch has to take it over rather than
/// refuse to start.
#[test]
fn a_socket_left_by_a_crash_is_reclaimed() {
    let dir = TempDir::new().unwrap();
    let socket = dir.path().join("agent-hooks.sock");
    std::fs::write(&socket, b"debris").unwrap();

    let server = serve(
        socket.clone(),
        Arc::new(LiveSessions::new()),
        Arc::new(|| {}),
    );

    assert!(server.is_ok());
}
